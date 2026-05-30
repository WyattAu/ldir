use std::collections::HashMap;
use std::fmt;

use crate::solver::matrix::DenseMatrix;

#[allow(dead_code)]
type SoftConstraint = (Vec<(usize, f64)>, f64, f64, bool);

#[allow(dead_code)]
/// Solver variable identified by index.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Variable(pub u32);

/// Constraint strength (higher priority wins).
///
/// Declaration order matches priority: REQUIRED is highest.
#[allow(clippy::upper_case_acronyms)]
#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum Strength {
    /// Mild preference (weight 1).
    WEAK,
    /// Moderate preference (weight 10^3).
    MEDIUM,
    /// Strong preference (weight 10^6).
    STRONG,
    /// Must-satisfy (infeasibility is an error).
    REQUIRED,
}

impl Strength {
    #[allow(dead_code)]
    fn weight(&self) -> f64 {
        match self {
            Strength::REQUIRED => f64::INFINITY,
            Strength::STRONG => 1e6,
            Strength::MEDIUM => 1e3,
            Strength::WEAK => 1.0,
        }
    }
}

#[allow(dead_code)]
/// Linear expression: `sum(terms) + constant`.
#[derive(Clone, Debug, Default)]
pub struct Expression {
    /// Coefficient pairs: (variable, coefficient).
    pub terms: Vec<(Variable, f64)>,
    /// Constant offset.
    pub constant: f64,
}

impl Expression {
    /// Create an empty expression.
    pub fn new() -> Self {
        Self {
            terms: Vec::new(),
            constant: 0.0,
        }
    }

    /// Create an expression from a constant only.
    pub fn from_constant(c: f64) -> Self {
        Self {
            terms: Vec::new(),
            constant: c,
        }
    }

    /// Create a single-variable expression.
    pub fn from_var(v: Variable, coeff: f64) -> Self {
        Self {
            terms: vec![(v, coeff)],
            constant: 0.0,
        }
    }

    /// Add a variable term (builder pattern).
    pub fn add_var(mut self, v: Variable, coeff: f64) -> Self {
        self.terms.push((v, coeff));
        self
    }

    /// Add a constant offset (builder pattern).
    pub fn add_const(mut self, c: f64) -> Self {
        self.constant += c;
        self
    }
}

/// Constraint relation operator.
#[allow(clippy::upper_case_acronyms)]
#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Relation {
    /// Equality: expr == 0.
    EQ,
    /// Less-than-or-equal: expr <= 0.
    LEQ,
    /// Greater-than-or-equal: expr >= 0.
    GEQ,
}

#[allow(dead_code)]
/// Linear constraint: `expr OP 0` at given strength.
#[derive(Clone, Debug)]
pub struct Constraint {
    /// The linear expression.
    pub expr: Expression,
    /// Constraint strength.
    pub strength: Strength,
    /// Relation operator.
    pub operator: Relation,
}

#[allow(dead_code)]
/// Solver error variants.
#[derive(Debug, Clone)]
pub enum SolverError {
    /// Required constraints cannot all be satisfied.
    Infeasible(String),
    /// Iteration limit reached without convergence.
    MaxIterationsExceeded,
    /// Constraint ID not found.
    InvalidConstraint(u64),
}

impl fmt::Display for SolverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SolverError::Infeasible(msg) => write!(f, "infeasible: {}", msg),
            SolverError::MaxIterationsExceeded => write!(f, "max iterations exceeded"),
            SolverError::InvalidConstraint(id) => write!(f, "invalid constraint id: {}", id),
        }
    }
}

impl std::error::Error for SolverError {}

#[allow(dead_code)]
/// Cassowary-inspired constraint solver.
///
/// Uses Gaussian elimination for required equalities and iterative
/// relaxation for soft constraints and inequality projection.
#[derive(Debug, Clone)]
pub struct Solver {
    next_var_id: u32,
    next_constraint_id: u64,
    constraints: HashMap<u64, Constraint>,
    edit_constraints: HashMap<Variable, (u64, f64)>,
    variables: Vec<Variable>,
}

impl Default for Solver {
    fn default() -> Self {
        Self::new()
    }
}

impl Solver {
    /// Create a new empty solver.
    pub fn new() -> Self {
        Self {
            next_var_id: 0,
            next_constraint_id: 0,
            constraints: HashMap::new(),
            edit_constraints: HashMap::new(),
            variables: Vec::new(),
        }
    }

    /// Add a new decision variable, returning its handle.
    pub fn add_variable(&mut self) -> Variable {
        let v = Variable(self.next_var_id);
        self.next_var_id += 1;
        self.variables.push(v);
        v
    }

    /// Add a constraint, returning its ID.
    pub fn add_constraint(
        &mut self,
        expr: Expression,
        strength: Strength,
        operator: Relation,
    ) -> u64 {
        let id = self.next_constraint_id;
        self.next_constraint_id += 1;
        self.constraints.insert(
            id,
            Constraint {
                expr,
                strength,
                operator,
            },
        );
        id
    }

    /// Remove a constraint by ID.
    pub fn remove_constraint(&mut self, id: u64) -> Result<(), SolverError> {
        if self.constraints.remove(&id).is_none() {
            return Err(SolverError::InvalidConstraint(id));
        }
        self.edit_constraints.retain(|_, (cid, _)| *cid != id);
        Ok(())
    }

    /// Add or replace an edit constraint (strong-strength equality).
    pub fn suggest_value(&mut self, var: Variable, value: f64) {
        if let Some((old_id, _)) = self.edit_constraints.remove(&var) {
            self.constraints.remove(&old_id);
        }
        let expr = Expression::from_var(var, 1.0).add_const(-value);
        let id = self.add_constraint(expr, Strength::STRONG, Relation::EQ);
        self.edit_constraints.insert(var, (id, value));
    }

    /// Solve the constraint system and return variable values.
    pub fn resolve(&self) -> Result<HashMap<Variable, f64>, SolverError> {
        let n = self.variables.len();
        if n == 0 {
            return Ok(HashMap::new());
        }

        let var_index: HashMap<Variable, usize> = self
            .variables
            .iter()
            .enumerate()
            .map(|(i, &v)| (v, i))
            .collect();

        let mut required_eq: Vec<(Vec<(usize, f64)>, f64)> = Vec::new();
        let mut required_ineq: Vec<(Vec<(usize, f64)>, f64)> = Vec::new();
        let mut soft: Vec<SoftConstraint> = Vec::new();

        for constraint in self.constraints.values() {
            let mut terms: Vec<(usize, f64)> = constraint
                .expr
                .terms
                .iter()
                .filter_map(|&(v, c)| var_index.get(&v).map(|&i| (i, c)))
                .collect();

            match constraint.strength {
                Strength::REQUIRED => match constraint.operator {
                    Relation::EQ => {
                        required_eq.push((terms, -constraint.expr.constant));
                    }
                    Relation::LEQ => {
                        required_ineq.push((terms, -constraint.expr.constant));
                    }
                    Relation::GEQ => {
                        for c in terms.iter_mut() {
                            c.1 = -c.1;
                        }
                        required_ineq.push((terms, constraint.expr.constant));
                    }
                },
                _ => {
                    let weight = constraint.strength.weight();
                    let is_eq = constraint.operator == Relation::EQ;
                    let rhs = match constraint.operator {
                        Relation::EQ | Relation::LEQ => -constraint.expr.constant,
                        Relation::GEQ => {
                            for c in terms.iter_mut() {
                                c.1 = -c.1;
                            }
                            constraint.expr.constant
                        }
                    };
                    soft.push((terms, rhs, weight, is_eq));
                }
            }
        }

        // Gaussian elimination on required equalities
        let m = required_eq.len();
        let mut is_pivot = vec![false; n];
        let mut pivot_info: Vec<(usize, usize)> = Vec::new();
        let mut aug = DenseMatrix::new(m, n + 1);

        for (i, (terms, rhs)) in required_eq.iter().enumerate() {
            for &(vi, coeff) in terms {
                aug.set(i, vi, aug.get(i, vi) + coeff);
            }
            aug.set(i, n, aug.get(i, n) + rhs);
        }

        let mut pivot_row = 0usize;
        for (col, is_piv) in is_pivot.iter_mut().enumerate().take(n) {
            let mut max_val = 0.0f64;
            let mut max_row = pivot_row;
            for row in pivot_row..m {
                let val = aug.get(row, col).abs();
                if val > max_val {
                    max_val = val;
                    max_row = row;
                }
            }
            if max_val < 1e-10 {
                continue;
            }
            aug.swap_rows(pivot_row, max_row);
            let pivot = aug.get(pivot_row, col);
            aug.scale_row(pivot_row, 1.0 / pivot);
            for row in 0..m {
                if row == pivot_row {
                    continue;
                }
                let factor = aug.get(row, col);
                if factor.abs() < 1e-15 {
                    continue;
                }
                aug.add_scaled_row(pivot_row, row, -factor);
            }
            *is_piv = true;
            pivot_info.push((pivot_row, col));
            pivot_row += 1;
        }

        // Check for infeasible rows (0 = nonzero)
        for row in pivot_row..m {
            if aug.get(row, n).abs() > 1e-6 {
                return Err(SolverError::Infeasible(
                    "Infeasible required constraints".to_string(),
                ));
            }
        }

        // Initialize free variables from soft constraints
        let mut values = vec![0.0f64; n];
        for (terms, rhs, _weight, _is_eq) in &soft {
            if terms.len() == 1 {
                let (vi, coeff) = terms[0];
                if !is_pivot[vi] && coeff.abs() > 1e-10 {
                    values[vi] = rhs / coeff;
                }
            }
        }

        // Compute pivot variables
        for &(row, col) in &pivot_info {
            let mut val = aug.get(row, n);
            for (j, &val_j) in values.iter().enumerate().take(n) {
                if j != col {
                    val -= aug.get(row, j) * val_j;
                }
            }
            values[col] = val;
        }

        // Iterative relaxation
        for _ in 0..1000 {
            let old_values = values.clone();

            // Soft corrections on free variables
            for (terms, rhs, weight, is_eq) in &soft {
                if terms.is_empty() {
                    continue;
                }
                let current: f64 = terms.iter().map(|&(vi, c)| c * values[vi]).sum();
                if *is_eq {
                    let residual = rhs - current;
                    if residual.abs() < 1e-12 {
                        continue;
                    }
                    let total_coeff_sq: f64 = terms
                        .iter()
                        .filter(|&&(vi, _)| !is_pivot[vi])
                        .map(|&(_, c)| c * c)
                        .sum();
                    if total_coeff_sq < 1e-20 {
                        continue;
                    }
                    let damping = weight / (weight + 1e6) * 0.5;
                    let step = residual / total_coeff_sq * damping;
                    for &(vi, c) in terms {
                        if !is_pivot[vi] {
                            values[vi] += c * step;
                        }
                    }
                } else {
                    // Soft inequality (stored as LEQ): only correct if violated
                    if current <= rhs + 1e-10 {
                        continue;
                    }
                    let violation = current - rhs;
                    let total_coeff_sq: f64 = terms
                        .iter()
                        .filter(|&&(vi, _)| !is_pivot[vi])
                        .map(|&(_, c)| c * c)
                        .sum();
                    if total_coeff_sq < 1e-20 {
                        continue;
                    }
                    let damping = weight / (weight + 1e6) * 0.5;
                    let step = violation / total_coeff_sq * damping;
                    for &(vi, c) in terms {
                        if !is_pivot[vi] {
                            values[vi] -= c * step;
                        }
                    }
                }
            }

            // Project onto required inequalities
            for (terms, rhs) in &required_ineq {
                let current: f64 = terms.iter().map(|&(vi, c)| c * values[vi]).sum();
                if current <= rhs + 1e-10 {
                    continue;
                }
                let violation = current - rhs;
                let total_coeff_sq: f64 = terms
                    .iter()
                    .filter(|&&(vi, _)| !is_pivot[vi])
                    .map(|&(_, c)| c * c)
                    .sum();
                if total_coeff_sq < 1e-20 {
                    continue;
                }
                let correction = violation / total_coeff_sq;
                for &(vi, c) in terms {
                    if !is_pivot[vi] {
                        values[vi] -= c * correction;
                    }
                }
            }

            // Recompute pivot variables
            for &(row, col) in &pivot_info {
                let mut val = aug.get(row, n);
                for (j, &val_j) in values.iter().enumerate().take(n) {
                    if j != col {
                        val -= aug.get(row, j) * val_j;
                    }
                }
                values[col] = val;
            }

            let max_change = values
                .iter()
                .zip(old_values.iter())
                .fold(0.0f64, |acc, (&a, &b)| acc.max((a - b).abs()));

            if max_change < 1e-6 {
                break;
            }
        }

        // Final feasibility check for inequalities
        for (terms, rhs) in &required_ineq {
            let current: f64 = terms.iter().map(|&(vi, c)| c * values[vi]).sum();
            if current > rhs + 1e-4 {
                return Err(SolverError::Infeasible(
                    "Cannot satisfy all required inequality constraints".to_string(),
                ));
            }
        }

        let mut result = HashMap::new();
        for &var in &self.variables {
            if let Some(&idx) = var_index.get(&var) {
                result.insert(var, values[idx]);
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-4
    }

    // --- Nominal tests ---

    #[test]
    fn test_single_variable_equality() {
        let mut s = Solver::new();
        let x = s.add_variable();
        s.add_constraint(
            Expression::from_var(x, 1.0).add_const(-5.0),
            Strength::REQUIRED,
            Relation::EQ,
        );
        let r = s.resolve().unwrap();
        assert!(approx_eq(r[&x], 5.0));
    }

    #[test]
    fn test_two_variables_equality() {
        let mut s = Solver::new();
        let x = s.add_variable();
        let y = s.add_variable();
        s.add_constraint(
            Expression::from_var(x, 1.0)
                .add_var(y, 1.0)
                .add_const(-10.0),
            Strength::REQUIRED,
            Relation::EQ,
        );
        s.add_constraint(
            Expression::from_var(x, 1.0).add_const(-3.0),
            Strength::REQUIRED,
            Relation::EQ,
        );
        let r = s.resolve().unwrap();
        assert!(approx_eq(r[&x], 3.0));
        assert!(approx_eq(r[&y], 7.0));
    }

    #[test]
    fn test_inequality_leq_clamp() {
        let mut s = Solver::new();
        let x = s.add_variable();
        s.add_constraint(
            Expression::from_var(x, 1.0).add_const(-5.0),
            Strength::REQUIRED,
            Relation::LEQ,
        );
        s.suggest_value(x, 10.0);
        let r = s.resolve().unwrap();
        assert!(approx_eq(r[&x], 5.0));
    }

    #[test]
    fn test_inequality_geq_clamp() {
        let mut s = Solver::new();
        let x = s.add_variable();
        s.add_constraint(
            Expression::from_var(x, 1.0).add_const(-5.0),
            Strength::REQUIRED,
            Relation::GEQ,
        );
        s.suggest_value(x, 2.0);
        let r = s.resolve().unwrap();
        assert!(approx_eq(r[&x], 5.0));
    }

    #[test]
    fn test_multiple_constraints_same_variable() {
        let mut s = Solver::new();
        let x = s.add_variable();
        let y = s.add_variable();
        s.add_constraint(
            Expression::from_var(x, 1.0).add_const(-3.0),
            Strength::REQUIRED,
            Relation::EQ,
        );
        s.add_constraint(
            Expression::from_var(x, 1.0)
                .add_var(y, 1.0)
                .add_const(-10.0),
            Strength::REQUIRED,
            Relation::EQ,
        );
        let r = s.resolve().unwrap();
        assert!(approx_eq(r[&x], 3.0));
        assert!(approx_eq(r[&y], 7.0));
    }

    #[test]
    fn test_stay_constraint_override() {
        let mut s = Solver::new();
        let x = s.add_variable();
        s.suggest_value(x, 3.0);
        s.suggest_value(x, 5.0);
        let r = s.resolve().unwrap();
        assert!(approx_eq(r[&x], 5.0));
    }

    #[test]
    fn test_remove_constraint() {
        let mut s = Solver::new();
        let x = s.add_variable();
        let id = s.add_constraint(
            Expression::from_var(x, 1.0).add_const(-5.0),
            Strength::REQUIRED,
            Relation::EQ,
        );
        s.remove_constraint(id).unwrap();
        let r = s.resolve().unwrap();
        assert!(approx_eq(r[&x], 0.0));
    }

    #[test]
    fn test_infeasible_conflicting_required() {
        let mut s = Solver::new();
        let x = s.add_variable();
        s.add_constraint(
            Expression::from_var(x, 1.0).add_const(-1.0),
            Strength::REQUIRED,
            Relation::EQ,
        );
        s.add_constraint(
            Expression::from_var(x, 1.0).add_const(-2.0),
            Strength::REQUIRED,
            Relation::EQ,
        );
        let err = s.resolve().unwrap_err();
        assert!(matches!(err, SolverError::Infeasible(_)));
    }

    #[test]
    fn test_empty_solver() {
        let s = Solver::new();
        let r = s.resolve().unwrap();
        assert!(r.is_empty());
    }

    #[test]
    fn test_ten_variables_stress() {
        let mut s = Solver::new();
        let mut vars = Vec::new();
        for _ in 0..10 {
            vars.push(s.add_variable());
        }
        for (i, &v) in vars.iter().enumerate() {
            let val = (i + 1) as f64;
            s.add_constraint(
                Expression::from_var(v, 1.0).add_const(-val),
                Strength::REQUIRED,
                Relation::EQ,
            );
        }
        // Sum constraint: x0 + x1 + ... + x9 = 55
        let mut sum_expr = Expression::new();
        for &v in &vars {
            sum_expr = sum_expr.add_var(v, 1.0);
        }
        sum_expr = sum_expr.add_const(-55.0);
        s.add_constraint(sum_expr, Strength::REQUIRED, Relation::EQ);

        let r = s.resolve().unwrap();
        for (i, &v) in vars.iter().enumerate() {
            assert!(approx_eq(r[&v], (i + 1) as f64));
        }
    }

    #[test]
    fn test_determinism() {
        let build = || {
            let mut s = Solver::new();
            let x = s.add_variable();
            let y = s.add_variable();
            s.add_constraint(
                Expression::from_var(x, 2.0).add_var(y, 1.0).add_const(-7.0),
                Strength::REQUIRED,
                Relation::EQ,
            );
            s.add_constraint(
                Expression::from_var(x, 1.0)
                    .add_var(y, -1.0)
                    .add_const(-2.0),
                Strength::REQUIRED,
                Relation::EQ,
            );
            (s, x, y)
        };

        let (s1, x1, y1) = build();
        let (s2, x2, y2) = build();
        let r1 = s1.resolve().unwrap();
        let r2 = s2.resolve().unwrap();
        assert_eq!(r1[&x1], r2[&x2]);
        assert_eq!(r1[&y1], r2[&y2]);
    }

    #[test]
    fn test_edit_resolve_cycle() {
        let mut s = Solver::new();
        let x = s.add_variable();
        s.suggest_value(x, 3.0);
        let r1 = s.resolve().unwrap();
        assert!(approx_eq(r1[&x], 3.0));

        s.suggest_value(x, 7.0);
        let r2 = s.resolve().unwrap();
        assert!(approx_eq(r2[&x], 7.0));

        s.suggest_value(x, -2.5);
        let r3 = s.resolve().unwrap();
        assert!(approx_eq(r3[&x], -2.5));
    }

    #[test]
    fn test_required_beats_soft() {
        let mut s = Solver::new();
        let x = s.add_variable();
        s.add_constraint(
            Expression::from_var(x, 1.0).add_const(-5.0),
            Strength::REQUIRED,
            Relation::EQ,
        );
        s.suggest_value(x, 10.0);
        let r = s.resolve().unwrap();
        assert!(approx_eq(r[&x], 5.0));
    }

    #[test]
    fn test_add_variable_unique_ids() {
        let mut s = Solver::new();
        let v0 = s.add_variable();
        let v1 = s.add_variable();
        let v2 = s.add_variable();
        assert_eq!(v0, Variable(0));
        assert_eq!(v1, Variable(1));
        assert_eq!(v2, Variable(2));
        assert_ne!(v0, v1);
    }

    #[test]
    fn test_remove_nonexistent_constraint() {
        let mut s = Solver::new();
        let err = s.remove_constraint(999).unwrap_err();
        assert!(matches!(err, SolverError::InvalidConstraint(999)));
    }

    #[test]
    fn test_three_variable_system() {
        let mut s = Solver::new();
        let x = s.add_variable();
        let y = s.add_variable();
        let z = s.add_variable();
        s.add_constraint(
            Expression::from_var(x, 1.0)
                .add_var(y, 1.0)
                .add_var(z, 1.0)
                .add_const(-6.0),
            Strength::REQUIRED,
            Relation::EQ,
        );
        s.add_constraint(
            Expression::from_var(x, 2.0)
                .add_var(y, 1.0)
                .add_var(z, -1.0)
                .add_const(-1.0),
            Strength::REQUIRED,
            Relation::EQ,
        );
        s.add_constraint(
            Expression::from_var(x, 1.0)
                .add_var(y, -1.0)
                .add_var(z, 3.0)
                .add_const(-8.0),
            Strength::REQUIRED,
            Relation::EQ,
        );
        let r = s.resolve().unwrap();
        assert!(approx_eq(r[&x], 1.0), "x = {}", r[&x]);
        assert!(approx_eq(r[&y], 2.0), "y = {}", r[&y]);
        assert!(approx_eq(r[&z], 3.0), "z = {}", r[&z]);
    }

    #[test]
    fn test_complex_linear_system() {
        let mut s = Solver::new();
        let x = s.add_variable();
        let y = s.add_variable();
        s.add_constraint(
            Expression::from_var(x, 2.0).add_var(y, 1.0).add_const(-7.0),
            Strength::REQUIRED,
            Relation::EQ,
        );
        s.add_constraint(
            Expression::from_var(x, 1.0)
                .add_var(y, -1.0)
                .add_const(-2.0),
            Strength::REQUIRED,
            Relation::EQ,
        );
        let r = s.resolve().unwrap();
        assert!(approx_eq(r[&x], 3.0), "x = {}", r[&x]);
        assert!(approx_eq(r[&y], 1.0), "y = {}", r[&y]);
    }

    #[test]
    fn test_soft_equality_no_required() {
        let mut s = Solver::new();
        let x = s.add_variable();
        s.suggest_value(x, 42.0);
        let r = s.resolve().unwrap();
        assert!(approx_eq(r[&x], 42.0));
    }

    // --- Expression tests ---

    #[test]
    fn test_expression_from_var() {
        let v = Variable(0);
        let e = Expression::from_var(v, 3.0);
        assert_eq!(e.terms.len(), 1);
        assert_eq!(e.terms[0], (v, 3.0));
        assert_eq!(e.constant, 0.0);
    }

    #[test]
    fn test_expression_chaining() {
        let x = Variable(0);
        let y = Variable(1);
        let e = Expression::from_var(x, 2.0).add_var(y, -1.0).add_const(5.0);
        assert_eq!(e.terms.len(), 2);
        assert_eq!(e.terms[0], (x, 2.0));
        assert_eq!(e.terms[1], (y, -1.0));
        assert_eq!(e.constant, 5.0);
    }

    #[test]
    fn test_expression_from_constant() {
        let e = Expression::from_constant(7.5);
        assert!(e.terms.is_empty());
        assert_eq!(e.constant, 7.5);
    }

    #[test]
    fn test_expression_default() {
        let e = Expression::default();
        assert!(e.terms.is_empty());
        assert_eq!(e.constant, 0.0);
    }

    // --- Strength tests ---

    #[test]
    fn test_strength_ordering() {
        assert!(Strength::REQUIRED > Strength::STRONG);
        assert!(Strength::STRONG > Strength::MEDIUM);
        assert!(Strength::MEDIUM > Strength::WEAK);
    }

    // --- Inequality tests ---

    #[test]
    fn test_multiple_inequalities() {
        let mut s = Solver::new();
        let x = s.add_variable();
        s.add_constraint(
            Expression::from_var(x, 1.0).add_const(-10.0),
            Strength::REQUIRED,
            Relation::LEQ,
        );
        s.add_constraint(
            Expression::from_var(x, 1.0).add_const(-3.0),
            Strength::REQUIRED,
            Relation::GEQ,
        );
        s.suggest_value(x, 1.0);
        let r = s.resolve().unwrap();
        assert!(approx_eq(r[&x], 3.0));
    }

    #[test]
    fn test_soft_inequality_respected_when_possible() {
        let mut s = Solver::new();
        let x = s.add_variable();
        s.add_constraint(
            Expression::from_var(x, 1.0).add_const(-100.0),
            Strength::WEAK,
            Relation::LEQ,
        );
        s.suggest_value(x, 50.0);
        let r = s.resolve().unwrap();
        assert!(approx_eq(r[&x], 50.0));
    }

    // --- Error tests ---

    #[test]
    fn test_solver_error_display() {
        let e = SolverError::Infeasible("test msg".to_string());
        assert_eq!(format!("{}", e), "infeasible: test msg");

        let e = SolverError::MaxIterationsExceeded;
        assert_eq!(format!("{}", e), "max iterations exceeded");

        let e = SolverError::InvalidConstraint(42);
        assert_eq!(format!("{}", e), "invalid constraint id: 42");
    }

    // --- Edge cases ---

    #[test]
    fn test_variable_with_no_constraints() {
        let mut s = Solver::new();
        let x = s.add_variable();
        let y = s.add_variable();
        s.add_constraint(
            Expression::from_var(x, 1.0).add_const(-5.0),
            Strength::REQUIRED,
            Relation::EQ,
        );
        let r = s.resolve().unwrap();
        assert!(approx_eq(r[&x], 5.0));
        assert!(approx_eq(r[&y], 0.0));
    }

    #[test]
    fn test_infeasible_equality_inequality_combo() {
        let mut s = Solver::new();
        let x = s.add_variable();
        s.add_constraint(
            Expression::from_var(x, 1.0).add_const(-10.0),
            Strength::REQUIRED,
            Relation::EQ,
        );
        s.add_constraint(
            Expression::from_var(x, 1.0).add_const(-5.0),
            Strength::REQUIRED,
            Relation::LEQ,
        );
        let err = s.resolve().unwrap_err();
        assert!(matches!(err, SolverError::Infeasible(_)));
    }

    #[test]
    fn test_medium_strength_soft_constraint() {
        let mut s = Solver::new();
        let x = s.add_variable();
        s.add_constraint(
            Expression::from_var(x, 1.0).add_const(-8.0),
            Strength::MEDIUM,
            Relation::EQ,
        );
        let r = s.resolve().unwrap();
        assert!(approx_eq(r[&x], 8.0));
    }

    #[test]
    fn test_zero_coefficient_constraint() {
        let mut s = Solver::new();
        let x = s.add_variable();
        s.add_constraint(
            Expression::from_constant(0.0),
            Strength::REQUIRED,
            Relation::EQ,
        );
        s.add_constraint(
            Expression::from_var(x, 1.0).add_const(-3.0),
            Strength::REQUIRED,
            Relation::EQ,
        );
        let r = s.resolve().unwrap();
        assert!(approx_eq(r[&x], 3.0));
    }
}
