use std::fmt;

pub struct DenseMatrix {
    data: Vec<f64>,
    rows: usize,
    cols: usize,
}

impl DenseMatrix {
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            data: vec![0.0; rows * cols],
            rows,
            cols,
        }
    }

    #[allow(dead_code)]
    pub fn rows(&self) -> usize {
        self.rows
    }

    #[allow(dead_code)]
    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn get(&self, row: usize, col: usize) -> f64 {
        self.data[row * self.cols + col]
    }

    pub fn set(&mut self, row: usize, col: usize, value: f64) {
        self.data[row * self.cols + col] = value;
    }

    pub fn swap_rows(&mut self, r1: usize, r2: usize) {
        if r1 == r2 {
            return;
        }
        let cols = self.cols;
        let base1 = r1 * cols;
        let base2 = r2 * cols;
        for c in 0..cols {
            self.data.swap(base1 + c, base2 + c);
        }
    }

    pub fn scale_row(&mut self, row: usize, factor: f64) {
        let base = row * self.cols;
        for i in base..base + self.cols {
            self.data[i] *= factor;
        }
    }

    pub fn add_scaled_row(&mut self, source: usize, target: usize, factor: f64) {
        let src_base = source * self.cols;
        let tgt_base = target * self.cols;
        for c in 0..self.cols {
            self.data[tgt_base + c] += factor * self.data[src_base + c];
        }
    }
}

impl fmt::Debug for DenseMatrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for r in 0..self.rows {
            for c in 0..self.cols {
                write!(f, "{:>10.4}", self.get(r, c))?;
                if c + 1 < self.cols {
                    write!(f, " ")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_new_zero_size() {
        let m = DenseMatrix::new(0, 0);
        assert_eq!(m.rows(), 0);
        assert_eq!(m.cols(), 0);
    }

    #[test]
    fn test_matrix_new() {
        let m = DenseMatrix::new(3, 4);
        assert_eq!(m.rows(), 3);
        assert_eq!(m.cols(), 4);
        for r in 0..3 {
            for c in 0..4 {
                assert_eq!(m.get(r, c), 0.0);
            }
        }
    }

    #[test]
    fn test_matrix_get_set() {
        let mut m = DenseMatrix::new(2, 3);
        m.set(0, 1, 5.0);
        m.set(1, 2, -3.0);
        assert_eq!(m.get(0, 1), 5.0);
        assert_eq!(m.get(1, 2), -3.0);
        assert_eq!(m.get(0, 0), 0.0);
    }

    #[test]
    fn test_matrix_swap_rows() {
        let mut m = DenseMatrix::new(2, 3);
        m.set(0, 0, 1.0);
        m.set(0, 1, 2.0);
        m.set(1, 0, 3.0);
        m.set(1, 1, 4.0);
        m.swap_rows(0, 1);
        assert_eq!(m.get(0, 0), 3.0);
        assert_eq!(m.get(0, 1), 4.0);
        assert_eq!(m.get(1, 0), 1.0);
        assert_eq!(m.get(1, 1), 2.0);
    }

    #[test]
    fn test_matrix_swap_rows_same() {
        let mut m = DenseMatrix::new(2, 2);
        m.set(0, 0, 1.0);
        m.swap_rows(0, 0);
        assert_eq!(m.get(0, 0), 1.0);
    }

    #[test]
    fn test_matrix_scale_row() {
        let mut m = DenseMatrix::new(1, 3);
        m.set(0, 0, 2.0);
        m.set(0, 1, -4.0);
        m.set(0, 2, 1.0);
        m.scale_row(0, 0.5);
        assert!((m.get(0, 0) - 1.0).abs() < 1e-10);
        assert!((m.get(0, 1) - (-2.0)).abs() < 1e-10);
        assert!((m.get(0, 2) - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_matrix_scale_row_zero() {
        let mut m = DenseMatrix::new(1, 2);
        m.set(0, 0, 7.0);
        m.set(0, 1, 3.0);
        m.scale_row(0, 0.0);
        assert_eq!(m.get(0, 0), 0.0);
        assert_eq!(m.get(0, 1), 0.0);
    }

    #[test]
    fn test_matrix_add_scaled_row() {
        let mut m = DenseMatrix::new(2, 3);
        m.set(0, 0, 1.0);
        m.set(0, 1, 2.0);
        m.set(0, 2, 3.0);
        m.set(1, 0, 4.0);
        m.set(1, 1, 5.0);
        m.set(1, 2, 6.0);
        m.add_scaled_row(0, 1, -3.0);
        assert!((m.get(1, 0) - (4.0 - 3.0 * 1.0)).abs() < 1e-10);
        assert!((m.get(1, 1) - (5.0 - 3.0 * 2.0)).abs() < 1e-10);
        assert!((m.get(1, 2) - (6.0 - 3.0 * 3.0)).abs() < 1e-10);
    }

    #[test]
    fn test_matrix_add_scaled_row_zero_factor() {
        let mut m = DenseMatrix::new(2, 2);
        m.set(0, 0, 1.0);
        m.set(1, 0, 2.0);
        m.add_scaled_row(0, 1, 0.0);
        assert_eq!(m.get(1, 0), 2.0);
    }

    #[test]
    fn test_gauss_elimination_identity() {
        let mut m = DenseMatrix::new(3, 4);
        for i in 0..3 {
            m.set(i, i, 1.0);
            m.set(i, 3, (i + 1) as f64);
        }
        let mut pivot_row = 0usize;
        for col in 0..3 {
            if m.get(pivot_row, col).abs() < 1e-10 {
                continue;
            }
            let p = m.get(pivot_row, col);
            m.scale_row(pivot_row, 1.0 / p);
            for row in 0..3 {
                if row == pivot_row {
                    continue;
                }
                let f = m.get(row, col);
                if f.abs() < 1e-15 {
                    continue;
                }
                m.add_scaled_row(pivot_row, row, -f);
            }
            pivot_row += 1;
        }
        assert!((m.get(0, 3) - 1.0).abs() < 1e-10);
        assert!((m.get(1, 3) - 2.0).abs() < 1e-10);
        assert!((m.get(2, 3) - 3.0).abs() < 1e-10);
    }
}
