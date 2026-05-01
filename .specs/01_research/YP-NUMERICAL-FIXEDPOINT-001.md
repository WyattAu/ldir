---
document_id: YP-NUMERICAL-FIXEDPOINT-001
version: 0.1.0
status: DRAFT
domain: Numerical Methods
subdomains: [Fixed-Point Arithmetic, Computer Typography]
applicable_standards: [IEEE 754, ISO 9899]
created: 2026-04-23
author: DeepThought
confidence_level: 0.90
tqa_level: 3
---

# YP-NUMERICAL-FIXEDPOINT-001: 26.6 Fixed-Point Arithmetic for Deterministic Geometric Computation

**Document ID:** YP-NUMERICAL-FIXEDPOINT-001
**Version:** 0.1.0
**Status:** DRAFT
**Domain:** Numerical Methods
**Subdomains:** Fixed-Point Arithmetic, Computer Typography
**Applicable Standards:** IEEE 754-2019, ISO/IEC 9899:2018
**Created:** 2026-04-23
**Author:** DeepThought
**Confidence Level:** 0.90
**TQA Level:** 3

---

## YP-2: Executive Summary

### Problem Statement

LDIR requires all geometric computations to produce bit-identical results across x86-64 and AArch64 architectures, across Linux, macOS, and Windows, and regardless of thread count (1, 4, or 16 cores). IEEE 754 floating-point arithmetic is unsuitable for this purpose because:

1. **Transcendental function divergence:** Different math libraries (libm, libc++, Win32) produce non-identical results for `sqrt`, `sin`, `cos`, etc.
2. **Fused multiply-add (FMA) inconsistency:** FMA instructions (`vfmadd`, `fmla`) produce a single rounded result, while separate multiply-add produces a double-rounded result. Compiler flags (`-ffp-contract`) control this, but the choice is platform-dependent.
3. **Extended precision registers:** x87 FPU uses 80-bit registers, introducing hidden precision that varies across compilers and optimization levels.
4. **Non-deterministic SIMD:** Vectorized floating-point reductions (sum, dot product) may change associativity based on lane width.

The central question this paper addresses is:

> **Can a 26.6 fixed-point arithmetic system replace IEEE 754 floating-point for all LDIR geometric computations while maintaining sufficient precision and guaranteeing cross-platform determinism?**

### Objective Function

$$\forall a, b \in \mathbb{Q}_{26.6},\; \forall \mathrm{op} \in \{+, -, \times, \div\},\; \forall \text{platform } p_1, p_2:\; \text{op}_{p_1}(a, b) = \text{op}_{p_2}(a, b)$$

### Scope

| Aspect | In-Scope | Out-of-Scope |
|--------|----------|--------------|
| Format | 26.6 signed fixed-point (DEF-FP266) | 16.16, 24.8, or other fixed-point formats |
| Operations | Add, subtract, multiply, divide, compare | Trigonometric functions, logarithms |
| Rounding | Round-to-nearest-even | Other rounding modes (truncation, ceiling, floor) |
| Overflow | Saturation arithmetic (DEF-FP-SAT) | Modular wrapping arithmetic |
| Error analysis | Per-operation and accumulated error bounds | Error-free transformations (Dekker, Knuth) |
| Algorithms | ALG-FP-MUL, ALG-FP-DIV, ALG-FP-SQRT | General transcendental function approximation |
| Testing | Test vectors for all operations | Fuzzing harness design |

### Dependencies

This document depends on:
- **REQ-3.2.4:** Fixed-point requirement for all geometric calculations
- **REQ-3.2.5:** 26.6 format specification (matches FreeType)
- **REQ-3.2.6:** Range and encoding definition
- **REQ-3.2.7:** Quantization error bound (±1/128)
- **REQ-4.3.4.2:** Cassowary solver fixed-point requirement
- **REQ-11.3.1–3:** Cross-platform determinism requirements
- **YP-IR-SEMANTICS-001:** G-IR well-formedness and fixed-point closure (AX-005)

---

## YP-3: Nomenclature and Notation

### 3.1 Symbol Table

| Symbol | Description | Units | Domain | Source |
|--------|-------------|-------|--------|--------|
| $x_{26.6}$ | A value in 26.6 fixed-point format | scaled points (sp) | $\mathbb{Q} \cap [-2^{25}, 2^{25} - 2^{-6}]$ | This paper |
| $m$ | Integer mantissa (the raw stored bits) | — | $\mathbb{Z} \cap [-2^{31}, 2^{31} - 1]$ | DEF-FP266 |
| $F$ | Encoding function: real → mantissa | — | $\mathbb{R} \to \mathbb{Z}$ | DEF-FP266 |
| $F^{-1}$ | Decoding function: mantissa → real | scaled points | $\mathbb{Z} \to \mathbb{Q}$ | DEF-FP266 |
| $\epsilon_{26.6}$ | Unit in the last place (ULP) of 26.6 | scaled points | $\mathbb{R}^+$ | DEF-FP266 |
| $\text{round}(x)$ | Round-to-nearest-even | — | $\mathbb{R} \to \mathbb{Z}$ | AX-FP-002 |
| $\text{trunc}(x)$ | Truncation toward zero | — | $\mathbb{R} \to \mathbb{Z}$ | Convention |
| $\oplus$ | Fixed-point addition (with saturation) | — | $\mathbb{Z} \times \mathbb{Z} \to \mathbb{Z}$ | DEF-FP-ADD |
| $\ominus$ | Fixed-point subtraction (with saturation) | — | $\mathbb{Z} \times \mathbb{Z} \to \mathbb{Z}$ | DEF-FP-ADD |
| $\otimes$ | Fixed-point multiplication | — | $\mathbb{Z} \times \mathbb{Z} \to \mathbb{Z}$ | DEF-FP-MUL |
| $\oslash$ | Fixed-point division | — | $\mathbb{Z} \times \mathbb{Z} \to \mathbb{Z}$ | DEF-FP-DIV |
| $\text{sat}_{32}$ | 32-bit signed saturation clamp | — | $\mathbb{Z} \to \mathbb{Z}$ | DEF-FP-SAT |
| $N$ | Number of sequential operations | — | $\mathbb{N}$ | THM-FP-ACCUMULATION |
| $\delta_{\max}$ | Maximum accumulated error | scaled points | $\mathbb{R}^+$ | THM-FP-ACCUMULATION |
| $M_{\min}, M_{\max}$ | Saturation bounds for mantissa | — | $\{-2^{31}, 2^{31} - 1\}$ | DEF-FP-SAT |
| $\text{ulp}(x)$ | ULP of value $x$ in 26.6 | scaled points | $\mathbb{R}^+$ | This paper |
| $q$ | Quotient bits in division | bits | $\mathbb{N}$ | ALG-FP-DIV |

### 3.2 Conventions

- **Mantissa notation:** When we write $m = 768$, the decoded value is $768 / 64 = 12.0$ scaled points. We use $x_{26.6}$ to denote the decoded real value and $m$ for the raw integer.
- **Precision notation:** "26.6" means 26 integer bits (including sign) and 6 fractional bits, stored in a 32-bit signed integer. The total is 32 bits.
- **Signed range:** The sign bit is part of the 26 "integer" bits, so the whole-number range is $[-2^{25}, 2^{25} - 1] = [-33554432, 33554431]$.
- **Scaling factor:** $s = 2^6 = 64$. To encode: $m = \text{round}(x \cdot s)$. To decode: $x = m / s$.
- **Rounding convention:** Round-to-nearest-even (banker's rounding) per AX-FP-002. Ties ($0.5$) round to the nearest even integer.
- **Overflow convention:** Saturation to $M_{\min}$ or $M_{\max}$ per DEF-FP-SAT. No wrapping.

### 3.3 Relationship to IEEE 754

This paper references IEEE 754-2019 for comparison only. LDIR does not use IEEE 754 arithmetic for geometric computation. The 26.6 fixed-point format can be compared to binary32 (single precision) as follows:

| Property | 26.6 Fixed-Point | binary32 (IEEE 754) |
|----------|-------------------|---------------------|
| Storage | 32 bits | 32 bits |
| Precision | Uniform (6 fractional bits) | Non-uniform (23 mantissa bits + implicit 1) |
| Range | $[-2^{25}, 2^{25}]$ | $\approx [\pm 1.2 \times 10^{-38}, \pm 3.4 \times 10^{38}]$ |
| Max error | $2^{-7} = 1/128 \approx 0.0078$ sp | Varies: $2^{-24} \cdot |x|$ (relative) |
| Determinism | Exact (integer arithmetic) | Platform-dependent (FMA, extended precision) |
| Subnormals | N/A | Yes (denormalized numbers) |
| Special values | Saturation only | NaN, ±Infinity |
| Performance | Integer multiply/add | May use FPU, but non-deterministic |

---

## YP-4: Theoretical Foundation

### 4.1 Axioms

**AX-FP-001: Fixed-Point Representability.**
All geometric values in LDIR G-IR are representable in 26.6 fixed-point format.

$$\forall g \in \mathcal{G},\; \forall c \in g,\; \forall x \in \text{coords}(c):\; x \in \mathbb{Q}_{26.6}$$

where $\mathbb{Q}_{26.6} = \{m / 64 \mid m \in \mathbb{Z},\; -2^{31} \leq m \leq 2^{31} - 1\}$.

*Intuition:* No geometric computation may produce a value outside the representable range. This is the same as AX-005 in YP-IR-SEMANTICS-001, restated for the numerical domain.

**AX-FP-002: Round-to-Nearest-Even.**
All rounding operations use round-to-nearest-even (IEEE 754 "roundTiesToEven" mode, applied to integer arithmetic).

$$\text{round}(x) = \begin{cases} \lfloor x \rfloor & \text{if } x - \lfloor x \rfloor < 0.5 \\ \lceil x \rceil & \text{if } x - \lfloor x \rfloor > 0.5 \\ \lfloor x \rfloor & \text{if } x - \lfloor x \rfloor = 0.5 \text{ and } \lfloor x \rfloor \text{ is even} \\ \lceil x \rceil & \text{if } x - \lfloor x \rfloor = 0.5 \text{ and } \lfloor x \rfloor \text{ is odd} \end{cases}$$

*Intuition:* This is the only rounding mode used in LDIR. It minimizes expected error and eliminates directional bias (unlike round-half-up, which systematically rounds up on ties).

**AX-FP-003: Integer Arithmetic Determinism.**
All underlying operations are performed using signed 32-bit and 64-bit integer arithmetic with defined overflow behavior (saturation, not wrapping).

$$\forall a, b \in \mathbb{Z}_{32},\; a \oplus b = \text{sat}_{32}(a + b)$$

*Intuition:* Rust's `i32` wrapping semantics are replaced by explicit saturation. Two's-complement overflow is never permitted to propagate silently.

**AX-FP-004: No Floating-Point in Hot Path.**
No IEEE 754 floating-point operation is used in the geometric computation path.

$$\forall \text{op} \in \text{geometric\_ops}:\; \text{type}(\text{op}) \in \{\text{i32}, \text{i64}\}$$

*Intuition:* Even a single floating-point operation in the hot path could introduce platform-dependent results. This axiom ensures the entire pipeline is integer-only.

### 4.2 Definitions

**DEF-FP266: 26.6 Fixed-Point Encoding.**
A 26.6 fixed-point value encodes a rational number $x$ as a 32-bit signed integer $m$:

$$x_{26.6} = m \times 2^{-6} = \frac{m}{64}$$

where:
- $m \in \mathbb{Z}$, $-2^{31} \leq m \leq 2^{31} - 1$
- The decoded value $x$ lies in $[-2^{25}, 2^{25} - 2^{-6}] = [-33554432.0,\; 33554431.984375]$

The encoding and decoding functions are:

$$F(x) = \text{round}(x \cdot 64)$$

$$F^{-1}(m) = \frac{m}{64}$$

The unit in the last place (ULP) is:

$$\epsilon_{26.6} = 2^{-6} = \frac{1}{64} \approx 0.015625 \text{ scaled points}$$

The maximum quantization error from encoding is:

$$|F(x) / 64 - x| \leq \frac{1}{2} \cdot \epsilon_{26.6} = \frac{1}{128} \approx 0.0078125 \text{ scaled points}$$

*Example:* $x = 12.3 \text{ sp}$ encodes as $m = \text{round}(12.3 \times 64) = \text{round}(787.2) = 787$. Decoded: $787 / 64 = 12.296875 \text{ sp}$. Error: $|12.296875 - 12.3| = 0.003125 \leq 1/128$.

**DEF-FP-ADD: Fixed-Point Addition and Subtraction (with Saturation).**
Given two 26.6 mantissas $a, b \in \mathbb{Z}$, $-2^{31} \leq a, b \leq 2^{31} - 1$:

$$a \oplus b = \text{sat}_{32}(a + b)$$

$$a \ominus b = \text{sat}_{32}(a - b)$$

where the saturation function is:

$$\text{sat}_{32}(v) = \begin{cases} -2^{31} & \text{if } v < -2^{31} \\ v & \text{if } -2^{31} \leq v \leq 2^{31} - 1 \\ 2^{31} - 1 & \text{if } v > 2^{31} - 1 \end{cases}$$

The intermediate sum/difference is computed in 64-bit arithmetic to detect overflow before saturation.

*Example:* $a = 2^{31} - 1 = 2147483647$, $b = 1$. Intermediate: $a + b = 2147483648 > 2^{31} - 1$. Result: $\text{sat}_{32}(2147483648) = 2147483647$.

**DEF-FP-MUL: Fixed-Point Multiplication.**
Given two 26.6 mantissas $a, b$, the multiplication is:

$$a \otimes b = \text{round}\left(\frac{a \cdot b}{64}\right)$$

computed as:

1. Promote both operands to 64-bit: $a_{64} = \text{i64}(a)$, $b_{64} = \text{i64}(b)$
2. Multiply: $p = a_{64} \cdot b_{64}$ (64-bit signed product)
3. Apply fixed-point correction: $q = (p + 32) \gg 6$ (equivalent to dividing by 64 with round-to-nearest for non-negative $p$)
4. Apply rounding: $r = \text{round}(p / 64.0)$ — see ALG-FP-MUL for exact integer implementation
5. Saturate: $a \otimes b = \text{sat}_{32}(r)$

*Example:* $a = 128$ (representing 2.0 sp), $b = 192$ (representing 3.0 sp). Product: $128 \times 192 = 24576$. Corrected: $24576 / 64 = 384$. Decoded: $384 / 64 = 6.0$ sp. Exact.

**DEF-FP-DIV: Fixed-Point Division.**
Given two 26.6 mantissas $a$ (dividend) and $b$ (divisor), with $b \neq 0$:

$$a \oslash b = \text{round}\left(\frac{a \cdot 64}{b}\right)$$

computed using 64-bit intermediate arithmetic.

*Example:* $a = 384$ (6.0 sp), $b = 128$ (2.0 sp). Result: $\text{round}(384 \times 64 / 128) = \text{round}(192) = 192$. Decoded: $192 / 64 = 3.0$ sp. Exact.

**DEF-FP-SAT: Saturation Arithmetic.**
Saturation clamps a value to the representable range of the target type:

$$\text{sat}_{32}(v) = \text{clamp}(v,\; -2^{31},\; 2^{31} - 1)$$

This is the overflow policy for all 26.6 arithmetic operations. Saturation is preferred over wrapping because:

1. Wrapping produces mathematically meaningless results (e.g., $2^{31} - 1 + 1 = -2^{31}$).
2. Saturation produces the closest representable value, which is geometrically meaningful (a very large coordinate is clamped to the maximum, rather than wrapping to a negative value).
3. Saturation is monotonic: $a \leq b \implies \text{sat}(a) \leq \text{sat}(b)$. Wrapping is not monotonic.

### 4.3 Lemmas

**LEM-FP-001: Addition Exactness.**
*Statement:* For any two 26.6 mantissas $a, b$ where the true sum does not overflow, addition is exact.

$$|a + b| \leq 2^{31} - 1 \implies a \oplus b = a + b$$

*Proof:* When $|a + b| \leq 2^{31} - 1$, the saturation function $\text{sat}_{32}$ is the identity. Integer addition is exact by definition (no rounding). Therefore $a \oplus b = a + b$. $\square$

**LEM-FP-002: Subtraction Exactness.**
*Statement:* For any two 26.6 mantissas $a, b$ where the true difference does not overflow, subtraction is exact.

$$|a - b| \leq 2^{31} - 1 \implies a \ominus b = a - b$$

*Proof:* Identical to LEM-FP-001, replacing addition with subtraction. $\square$

**LEM-FP-003: Multiplication ULP Bound.**
*Statement:* The rounding error of fixed-point multiplication is at most 0.5 ULP of the result.

$$\left|\frac{a \otimes b}{64} - \frac{a \cdot b}{64^2}\right| \leq \frac{1}{2} \cdot \frac{1}{64} = \frac{1}{128}$$

*Proof:* The product $a \cdot b$ is an exact integer. Dividing by 64 may produce a non-integer. The rounding operation $\text{round}(a \cdot b / 64)$ introduces an error of at most $1/2$ in the mantissa, which corresponds to $1/2 \times 1/64 = 1/128$ in the decoded value. $\square$

**LEM-FP-004: Division Error Bound.**
*Statement:* The rounding error of fixed-point division is at most 0.5 ULP of the result.

$$\left|\frac{a \oslash b}{64} - \frac{a}{b}\right| \leq \frac{1}{128}$$

*Proof:* The division $(a \cdot 64) / b$ may produce a non-integer. Rounding introduces at most $1/2$ error in the mantissa, corresponding to $1/128$ in the decoded value. $\square$

**LEM-FP-005: Saturation Monotonicity.**
*Statement:* The saturation function is monotonically non-decreasing.

$$a \leq b \implies \text{sat}_{32}(a) \leq \text{sat}_{32}(b)$$

*Proof:* Case analysis on the regions:
- If both $a, b < -2^{31}$: $\text{sat}(a) = \text{sat}(b) = -2^{31}$. Equal.
- If $a < -2^{31} \leq b \leq 2^{31} - 1$: $\text{sat}(a) = -2^{31} \leq b = \text{sat}(b)$.
- If $a \leq b$ and both in range: $\text{sat}(a) = a \leq b = \text{sat}(b)$.
- If $a \leq 2^{31} - 1 < b$: $\text{sat}(a) = a \leq 2^{31} - 1 = \text{sat}(b)$.
- If both $a, b > 2^{31} - 1$: $\text{sat}(a) = \text{sat}(b) = 2^{31} - 1$. Equal.
In all cases, $\text{sat}(a) \leq \text{sat}(b)$. $\square$

### 4.4 Theorems

**THM-FP-ADD-EXACT: Addition is Exact (No Rounding Error).**
*Statement:* Fixed-point addition of two 26.6 values within the representable range produces an exact result with zero rounding error.

$$\forall a, b \in \mathbb{Z}_{26.6}:\; |a + b| \leq 2^{31} - 1 \implies |(a \oplus b) / 64 - a/64 - b/64| = 0$$

*Proof:* By LEM-FP-001, when the sum does not overflow, $a \oplus b = a + b$ (integer addition). The decoded values satisfy $(a + b)/64 = a/64 + b/64$ exactly, because integer division by a power of two distributes over addition. Therefore the decoded result equals the true sum. $\square$

*Corollary:* The same holds for subtraction by LEM-FP-002.

**THM-FP-MUL-ROUND: Multiplication Error is at Most 0.5 ULP.**
*Statement:* The decoded result of fixed-point multiplication differs from the true product by at most $1/128$ scaled points.

$$\forall a, b \in \mathbb{Z}_{26.6}:\; \left|\frac{a \otimes b}{64} - \frac{a}{64} \cdot \frac{b}{64}\right| \leq \frac{1}{128}$$

*Proof:* The true product is $(a/64) \cdot (b/64) = (a \cdot b) / 4096$. The computed product is $(a \otimes b) / 64$. We have $a \otimes b = \text{round}(a \cdot b / 64)$. Therefore:

$$\left|\frac{a \otimes b}{64} - \frac{a \cdot b}{4096}\right| = \left|\frac{\text{round}(a \cdot b / 64)}{64} - \frac{a \cdot b}{4096}\right| = \frac{1}{64} \cdot \left|\text{round}(a \cdot b / 64) - \frac{a \cdot b}{64}\right|$$

By the property of rounding, $|\text{round}(x) - x| \leq 0.5$. Therefore:

$$\frac{1}{64} \cdot 0.5 = \frac{1}{128} \quad \square$$

**THM-FP-SATURATION: Saturation Prevents Overflow.**
*Statement:* Saturation arithmetic guarantees that all results remain within the representable range of the 26.6 format.

$$\forall a, b \in \mathbb{Z}_{26.6},\; \forall \text{op} \in \{\oplus, \ominus, \otimes, \oslash\}:\; a \text{ op } b \in [-2^{31}, 2^{31} - 1]$$

*Proof:* Each operation applies $\text{sat}_{32}$ to its result. By definition of $\text{sat}_{32}$, the output is always in $[-2^{31}, 2^{31} - 1]$. $\square$

*Note:* Saturation silently clamps values. Callers must detect overflow separately if they need to distinguish between "clamped" and "naturally at the boundary." This is handled by overflow flag inspection in the implementation.

**THM-FP-COMPARISON: Comparison is Exact for Equal Values.**
*Statement:* Two 26.6 mantissas represent the same real value if and only if they are bit-identical.

$$\forall a, b \in \mathbb{Z}_{26.6}:\; a/64 = b/64 \iff a = b$$

*Proof:*
- ($\Rightarrow$) If $a/64 = b/64$, then $a = b$ by multiplying both sides by 64.
- ($\Leftarrow$) If $a = b$, then $a/64 = b/64$ trivially.
$\square$

*Corollary:* There is no NaN, no negative zero, and no denormalized numbers in the 26.6 format. Every bit pattern represents exactly one real value, and comparison reduces to integer comparison.

**THM-FP-ACCUMULATION: Accumulated Error Over N Operations.**
*Statement:* After $N$ sequential non-additive operations (multiplication, division), the accumulated rounding error is at most $N / 128$ scaled points.

$$\delta_{\max}(N) = N \cdot \epsilon_{26.6} / 2 = N / 128$$

*Proof:* Each multiplication or division introduces at most $1/128$ error (THM-FP-MUL-ROUND, LEM-FP-004). Addition and subtraction are exact (THM-FP-ADD-EXACT) and contribute zero error. In the worst case, all errors accumulate in the same direction. By the triangle inequality:

$$\left|\sum_{i=1}^{N} \delta_i\right| \leq \sum_{i=1}^{N} |\delta_i| \leq N \cdot \frac{1}{128} \quad \square$$

*Practical note:* For typical typesetting operations (line breaking, box positioning), $N$ is bounded by the number of glyphs in a paragraph (typically 50–500). Even at $N = 500$, the worst-case accumulated error is $500/128 \approx 3.9$ scaled points, which is below the visual acuity threshold of $\approx 0.5$ printer's points ($\approx 32768$ scaled points). In practice, errors tend to cancel (both positive and negative), so the actual accumulated error is typically $O(\sqrt{N}) \cdot 1/128$ by a random walk argument.

**THM-FP-DETERMINISM: Cross-Platform Determinism.**
*Statement:* All 26.6 arithmetic operations produce bit-identical results across all platforms supported by LDIR.

$$\forall a, b \in \mathbb{Z}_{26.6},\; \forall \text{op} \in \{\oplus, \ominus, \otimes, \oslash\},\; \forall p_1, p_2 \in \text{Platforms}:\; \text{op}_{p_1}(a, b) = \text{op}_{p_2}(a, b)$$

*Proof:*
- All operations are defined as sequences of integer additions, subtractions, multiplications, and shifts on signed 32-bit and 64-bit integers.
- The C/Rust abstract machine guarantees that signed integer overflow wraps in two's complement, but LDIR uses saturation (DEF-FP-SAT) with explicit overflow checks on 64-bit intermediates, so the wrapping behavior is never triggered.
- Bit shifts ($\gg 6$) on signed integers are implementation-defined in C, but LDIR uses arithmetic right shift (which is the behavior of both GCC and Clang on all target platforms, and is guaranteed in Rust).
- The rounding function $\text{round}(x)$ is implemented using integer-only arithmetic (see ALG-FP-MUL), avoiding any floating-point dependency.
- Therefore, all platforms produce identical bit patterns. $\square$

---

## YP-5: Algorithm Specification

### ALG-FP-MUL: Fixed-Point Multiplication

Multiply two 26.6 values using 64-bit intermediate arithmetic with round-to-nearest-even.

```
Algorithm: fp266_mul
Input:  a: i32  (26.6 mantissa)
        b: i32  (26.6 mantissa)
Output: c: i32  (26.6 mantissa, saturated)

 1:  function FP266_MUL(a: i32, b: i32) → i32
 2:    // Promote to 64-bit to avoid overflow
 3:    a_wide ← i64(a)
 4:    b_wide ← i64(b)
 5:
 6:    // Compute full 64-bit product
 7:    product ← a_wide × b_wide
 8:
 9:    // Round-to-nearest-even: add 32 (half of 64) before shift
10:    // This is equivalent to round(product / 64) for non-negative product
11:    // For negative product, we need to adjust for truncation toward zero
12:    if product ≥ 0 then
13:      rounded ← (product + 32) >> 6
14:    else
15:      // For negative values: round(product/64) with ties to even
16:      // Adjustment: add 32 for rounding, then shift
17:      // But (product + 32) >> 6 with negative product truncates toward -∞
18:      // which is correct for round-to-nearest (not banker's rounding)
19:      // For true round-to-nearest-even on negative, we use:
20:      offset ← (-product) mod 64   // distance to next multiple of 64
21:      if offset = 32 then
22:        // Exact tie: round to even
23:        base ← (product - 32) >> 6  // round toward -∞
24:        if base is odd then
25:          rounded ← base + 1
26:        else
27:          rounded ← base
28:        end if
29:      else if offset > 32 then
30:        rounded ← (product + 32) >> 6  // round away from zero
31:      else
32:        rounded ← (product - 32) >> 6  // round toward zero
33:      end if
34:    end if
35:
36:    // Saturate to 32-bit range
37:    if rounded > 2147483647 then
38:      return 2147483647
39:    else if rounded < -2147483648 then
40:      return -2147483648
41:    else
42:      return i32(rounded)
43:    end if
44:  end function
```

**Complexity Analysis:**

| Metric | Value | Derivation |
|--------|-------|------------|
| Time | $O(1)$ | Constant number of integer operations |
| Space | $O(1)$ | 3 × i64 temporaries |
| Throughput | ~1 cycle on modern CPUs | 64-bit multiply + shift is single micro-op |

**Preconditions:**

| ID | Condition | Rationale |
|----|-----------|-----------|
| PRE-MUL-001 | $a, b \in [-2^{31}, 2^{31} - 1]$ | Guaranteed by i32 type |

**Postconditions:**

| ID | Condition | Rationale |
|----|-----------|-----------|
| POST-MUL-001 | Result $\in [-2^{31}, 2^{31} - 1]$ | Saturation (THM-FP-SATURATION) |
| POST-MUL-002 | $|\text{result}/64 - a/64 \cdot b/64| \leq 1/128$ | THM-FP-MUL-ROUND |
| POST-MUL-003 | Ties resolved to even | AX-FP-002 |

### ALG-FP-DIV: Fixed-Point Division

Divide two 26.6 values using 64-bit intermediate arithmetic with round-to-nearest-even.

```
Algorithm: fp266_div
Input:  a: i32  (26.6 mantissa, dividend)
        b: i32  (26.6 mantissa, divisor)
Output: c: i32  (26.6 mantissa, saturated)
Precondition: b ≠ 0

 1:  function FP266_DIV(a: i32, b: i32) → i32
 2:    assert b ≠ 0                    // PRE-DIV-001
 3:
 4:    // Promote to 64-bit
 5:    a_wide ← i64(a)
 6:    b_wide ← i64(b)
 7:
 8:    // Scale numerator by 64 to maintain 26.6 format
 9:    // We want: round((a * 64) / b)
10:    // Use 128-bit intermediate to avoid overflow:
11:    // a is at most 2^31 - 1, so a * 64 is at most ~2^37
12:    // This fits in i64, so we can compute directly
13:    scaled_num ← a_wide × 64
14:
15:    // Perform division with rounding
16:    // quotient = floor(scaled_num / b_wide)
17:    // remainder = scaled_num - quotient * b_wide
18:    quotient ← scaled_num / b_wide    // truncation toward zero
19:    remainder ← scaled_num % b_wide   // sign of dividend
20:
21:    // Round-to-nearest-even
22:    abs_b ← if b_wide < 0 then -b_wide else b_wide
23:    half_b ← abs_b / 2
24:    // True half: abs_b is even → half_b is exact; abs_b is odd → 0.5
25:    is_tie ← (abs_b mod 2 = 0) ∧ (|remainder| = half_b)
26:
27:    if remainder = 0 then
28:      rounded ← quotient
29:    else if is_tie then
30:      // Round to even
31:      if quotient is odd then
32:        rounded ← quotient + (if b_wide > 0 then 1 else -1)
33:      else
34:        rounded ← quotient
35:      end if
36:    else if |remainder| > half_b then
37:      rounded ← quotient + (if b_wide > 0 then 1 else -1)
38:    else
39:      rounded ← quotient
40:    end if
41:
42:    // Saturate to 32-bit range
43:    if rounded > 2147483647 then
44:      return 2147483647
45:    else if rounded < -2147483648 then
46:      return -2147483648
47:    else
48:      return i32(rounded)
49:    end if
50:  end function
```

**Complexity Analysis:**

| Metric | Value | Derivation |
|--------|-------|------------|
| Time | $O(1)$ | Constant number of integer operations |
| Space | $O(1)$ | 4 × i64 temporaries |
| Throughput | ~5–10 cycles on modern CPUs | 64-bit division is slower than multiply |

**Preconditions:**

| ID | Condition | Rationale |
|----|-----------|-----------|
| PRE-DIV-001 | $b \neq 0$ | Division by zero is undefined |

**Postconditions:**

| ID | Condition | Rationale |
|----|-----------|-----------|
| POST-DIV-001 | Result $\in [-2^{31}, 2^{31} - 1]$ | Saturation (THM-FP-SATURATION) |
| POST-DIV-002 | $|\text{result}/64 - a/64 \div b/64| \leq 1/128$ | LEM-FP-004 |
| POST-DIV-003 | Ties resolved to even | AX-FP-002 |

### ALG-FP-SQRT: Fixed-Point Square Root

Compute the integer square root of a 26.6 value using Newton's method in fixed-point.

```
Algorithm: fp266_sqrt
Input:  a: i32  (26.6 mantissa, a ≥ 0)
Output: c: i32  (26.6 mantissa, saturated)
Precondition: a ≥ 0

 1:  function FP266_SQRT(a: i32) → i32
 2:    assert a ≥ 0                    // PRE-SQRT-001
 3:
 4:    if a = 0 then
 5:      return 0
 6:    end if
 7:
 8:    // We want sqrt(a / 64) in 26.6 format
 9:    // = sqrt(a) / 8  (since sqrt(a/64) = sqrt(a)/sqrt(64) = sqrt(a)/8)
10:    // But a is an integer mantissa, so we compute sqrt(a) using Newton's
11:    // method on integers, then shift right by 3 (divide by 8)
12:    //
13:    // Actually: sqrt(mantissa) in 26.6 means:
14:    //   result_mantissa / 64 = sqrt(input_mantissa / 64)
15:    //   result_mantissa = 64 * sqrt(input_mantissa / 64)
16:    //                    = 64 * sqrt(input_mantissa) / sqrt(64)
17:    //                    = 8 * sqrt(input_mantissa)
18:    //
19:    // So we need: round(8 * sqrt(a))
20:    // Compute sqrt(a) to sufficient precision using Newton's method,
21:    // then multiply by 8.
22:
23:    // Scale input up by 6 bits for extra precision in Newton's method
24:    // We compute sqrt(a << 6) then shift back
25:    target ← i64(a) << 6              // a * 64, in i64
26:
27:    // Initial guess: use bit length
28:    // sqrt(2^n) ≈ 2^(n/2)
29:    bit_len ← floor_log2(target)
30:    x ← i64(1) << ((bit_len + 1) / 2) // initial guess
31:
32:    // Newton's iteration: x_{n+1} = (x_n + target / x_n) / 2
33:    // Converges quadratically; 32 iterations is vast overkill for i64
34:    for i ← 1 to 32 do
35:      x_new ← (x + target / x) / 2
36:      if x_new ≥ x then              // converged (Newton's for sqrt)
37:        break
38:      end if
39:      x ← x_new
40:    end for
41:
42:    // x ≈ sqrt(target) = sqrt(a * 64) = 8 * sqrt(a)
43:    // We need result_mantissa = 8 * sqrt(a) = x (already correct!)
44:    // But we want the result in 26.6, so result/64 = sqrt(a/64)
45:    // x/64 = 8*sqrt(a)/64 = sqrt(a)/8 = sqrt(a/64). Correct.
46:
47:    // Apply rounding-to-nearest-even
48:    // x is currently floor(sqrt(target)). Check if we should round up.
49:    lower_sq ← x × x
50:    upper_sq ← (x + 1) × (x + 1)
51:    midpoint ← lower_sq + (upper_sq - lower_sq) / 2
52:
53:    if target > midpoint then
54:      x ← x + 1
55:    else if target = midpoint then
56:      // Tie: round to even
57:      if x is odd then
58:        x ← x + 1
59:      end if
60:    end if
61:
62:    // Saturate to 32-bit range
63:    if x > 2147483647 then
64:      return 2147483647
65:    else
66:      return i32(x)
67:    end if
68:  end function
```

**Complexity Analysis:**

| Metric | Value | Derivation |
|--------|-------|------------|
| Time | $O(\log \log M)$ | Newton's method converges quadratically; ~5–6 iterations for i64 |
| Space | $O(1)$ | 4 × i64 temporaries |
| Iterations | ≤ 32 (typically 5–6) | Bit-length-based initial guess gives ~1 correct bit; each iteration doubles |

**Preconditions:**

| ID | Condition | Rationale |
|----|-----------|-----------|
| PRE-SQRT-001 | $a \geq 0$ | Square root is undefined for negative values in real arithmetic |

**Postconditions:**

| ID | Condition | Rationale |
|----|-----------|-----------|
| POST-SQRT-001 | Result $\in [0, 2^{31} - 1]$ | Saturation; non-negative by construction |
| POST-SQRT-002 | $|\text{result}/64 - \sqrt{a/64}| \leq 1/128$ | Rounding to nearest |
| POST-SQRT-003 | Ties resolved to even | AX-FP-002 |

---

## YP-6: Test Vector Specification

**Reference file:** `.specs/01_research/test_vectors/test_vectors_numerical.toml`

### 6.1 Test Vector Categories

| Category | Description | Coverage Target | Count (Minimum) |
|----------|-------------|-----------------|-----------------|
| **Nominal** | Standard typographic values: font sizes (10pt, 12pt, 14pt), line spacing (1.2×, 1.5×, 2.0×), glyph advances, page margins, paragraph indentation | 40% | 40 |
| **Boundary** | Minimum/maximum representable values, values near overflow/underflow, zero, ±1 ULP, values at exact halves, values requiring all 6 fractional bits | 20% | 20 |
| **Adversarial** | Maximum × maximum, minimum × minimum, division by 1, division by −1, large quotient near overflow, values producing exact ties in rounding | 15% | 15 |
| **Regression** | Known FreeType coordinate values, TeX dimension conversions, PDF coordinate transformations, METAFONT-style rasterization coordinates | 10% | 10 |
| **Random** | Property-based generated pairs (QuickCheck / proptest) with uniform and biased distributions, stress-testing accumulation bounds | 15% | Continuous (fuzzing) |

### 6.2 Property-Based Invariants

For random testing, the following invariants must hold for all generated test inputs $(a, b)$:

$$a \oplus b = b \oplus a \quad \text{(commutativity of addition)}$$

$$a \otimes b = b \otimes a \quad \text{(commutativity of multiplication)}$$

$$(a \oplus b) \oplus c = a \oplus (b \oplus c) \quad \text{(associativity of addition, when no overflow)}$$

$$a \otimes (b \oplus c) = (a \otimes b) \oplus (a \otimes c) + \delta, \quad |\delta| \leq 2/128 \quad \text{(approximate distributivity)}$$

$$\text{sat}_{32}(\text{sat}_{32}(a)) = \text{sat}_{32}(a) \quad \text{(idempotence of saturation)}$$

$$a \oslash 1 = a \quad \text{(division identity)}$$

$$a \oslash a = 64 \quad \text{(self-division yields 1.0 in 26.6)}$$

$$\text{FP266\_SQRT}(a \otimes a) \approx a, \quad |\text{result} - a| \leq 1 \quad \text{(sqrt of square)}$$

### 6.3 Cross-Platform Verification Protocol

Each test vector must produce identical results on:
1. x86-64 (GCC, Clang) with and without `-march=native`
2. AArch64 (GCC, Clang) with and without `-mcpu=native`
3. WebAssembly (wasm32, wasm64)
4. All three with thread counts 1, 4, and 16

Verification command:

```
cargo test --package ldir-core --test numerical_fixedpoint -- --nocapture
```

---

## YP-7: Domain Constraints

**Reference file:** `.specs/01_research/domain_constraints/domain_constraints_typesetting.toml`

### 7.1 Numerical Constraints

| ID | Constraint | Value | Unit | Source |
|----|------------|-------|------|--------|
| NC-FP-001 | Mantissa range (minimum) | $-2^{31} = -2147483648$ | — | DEF-FP266 |
| NC-FP-002 | Mantissa range (maximum) | $2^{31} - 1 = 2147483647$ | — | DEF-FP266 |
| NC-FP-003 | Decoded value range (minimum) | $-33554432.0$ | sp | REQ-3.2.6 |
| NC-FP-004 | Decoded value range (maximum) | $33554431.984375$ | sp | REQ-3.2.6 |
| NC-FP-005 | ULP (unit in last place) | $2^{-6} = 1/64 \approx 0.015625$ | sp | DEF-FP266 |
| NC-FP-006 | Maximum quantization error | $1/128 \approx 0.0078125$ | sp | REQ-3.2.7 |
| NC-FP-007 | Accumulated error (N operations) | $N / 128$ | sp | THM-FP-ACCUMULATION |
| NC-FP-008 | Scaling factor | $2^6 = 64$ | — | DEF-FP266 |
| NC-FP-009 | Fractional bits | 6 | bits | REQ-3.2.5 |
| NC-FP-010 | Integer bits (including sign) | 26 | bits | REQ-3.2.5 |
| NC-FP-011 | Storage size | 32 | bits | REQ-3.2.4 |

### 7.2 Structural Constraints

| ID | Constraint | Rationale |
|----|------------|-----------|
| NC-FP-012 | No NaN, no ±Infinity, no denormals | Integer representation has no special values |
| NC-FP-013 | Negative zero does not exist | $-0 = 0$ in two's complement |
| NC-FP-014 | Comparison is integer comparison | THM-FP-COMPARISON |
| NC-FP-015 | All operations use integer-only arithmetic | AX-FP-004 |
| NC-FP-016 | Overflow policy is saturation (not wrapping) | DEF-FP-SAT |
| NC-FP-017 | Rounding mode is round-to-nearest-even | AX-FP-002 |

### 7.3 Derived Constraints

The following constraints are consequences of the axioms and definitions:

$$\text{NC-FP-003} \land \text{NC-FP-004} \implies \text{page dimensions} \leq 33554431 \text{ sp per axis}$$

$$\text{NC-FP-006} \land N = 500 \implies \delta_{\max} \leq 3.9 \text{ sp} \ll 32768 \text{ sp (visual threshold)}$$

$$\text{NC-FP-012} \land \text{NC-FP-013} \implies \text{no special-case handling needed in comparison}$$

$$\text{NC-FP-011} \land \text{NC-FP-008} \implies \text{max mantissa for 1pt} = 64, \text{max mantissa for 1000pt} = 64000$$

### 7.4 Constraint Conflicts

| ID | Conflict | Impact | Resolution |
|----|----------|--------|------------|
| CONF-FP-001 | NC-FP-003 (range limit) vs. very large page sizes (e.g., billboard printing) | Coordinates may exceed representable range | Define maximum page dimensions; clamp during compilation (per REQ-3.2.4) |
| CONF-FP-002 | NC-FP-006 (precision limit) vs. sub-pixel hinting requirements | 1/128 sp ≈ 0.000015 pt may be insufficient for fine hinting | Sub-pixel hinting is deferred to rasterization (post G-IR), where floating-point is acceptable per REQ-2.8 |

---

## YP-8: Bibliography

| ID | Citation | Relevance | TQA Level | Confidence |
|----|----------|-----------|-----------|------------|
| [1] IEEE Computer Society (2019). *IEEE 754-2019: Standard for Floating-Point Arithmetic.* DOI: 10.1109/IEEESTD.2019.8766229 | Reference for comparison; LDIR deliberately avoids this standard for geometric computation | 5 | 0.99 |
| [2] Knuth, D.E. (1997). *The Art of Computer Programming, Volume 2: Seminumerical Algorithms*, 3rd ed. Addison-Wesley. ISBN: 0-201-89684-2 | Fixed-point arithmetic (§4.2.1–4.2.4), multiple-precision arithmetic, rounding analysis | 5 | 0.99 |
| [3] FreeType Project (2024). "FreeType API Reference: Fixed-Point Types." https://freetype.org/freetype2/docs/reference/ft2-base_interface.html#FT_Pos | Primary source for 26.6 format; LDIR matches FreeType's internal coordinate representation | 4 | 0.95 |
| [4] Knuth, D.E. (1986). *The Metafontbook.* Addison-Wesley. ISBN: 0-201-13445-4 | METAFONT's use of fixed-point arithmetic for rasterization; scaled points; grid fitting | 4 | 0.95 |
| [5] ISO/IEC 9899:2018. *Programming Languages — C.* §6.3.1.4 (real floating-integer conversions), §5.2.4.2.2 (characteristics of integer types) | C standard for integer arithmetic guarantees; two's complement requirements (C23 mandatory) | 5 | 0.99 |
| [6] Muller, J.-M., Brunie, N., de Dinechin, F., Jeannerod, C.-P., Joldes, M., Lefèvre, V., Melquiond, G., Revol, N., Torres, S. (2018). *Handbook of Floating-Point Arithmetic*, 2nd ed. Birkhäuser. DOI: 10.1007/978-3-319-76526-6 | Error analysis techniques adapted for fixed-point; ULP-based error bounds | 5 | 0.95 |
| [7] Higham, N.J. (2002). *Accuracy and Stability of Numerical Algorithms*, 2nd ed. SIAM. ISBN: 0-89871-521-0 | Backward error analysis; accumulation of rounding errors; condition number theory | 5 | 0.99 |
| [8] Rust Language Team (2024). *The Rust Reference: Type Layout and ABI.* https://doc.rust-lang.org/reference/type-layout.html | Rust's guarantees for i32/i64 arithmetic, wrapping behavior, and alignment | 4 | 0.95 |
| [9] Adobe Systems (2023). "CFF (Compact Font Format) Specification." Adobe Technical Note #5176. | Fixed-point usage in font programs; hinting bytecode arithmetic | 3 | 0.90 |

---

## YP-9: Knowledge Graph Concepts

| ID | Concept | Language | Source | Confidence | Relationships |
|----|---------|----------|--------|------------|---------------|
| CON-FP-001 | 26.6 fixed-point arithmetic | EN | This paper | 0.95 | used-in → G-IR coordinates; matches → FreeType internal format |
| CON-FP-001a | 26.6 定点算术 | ZH | This paper | 0.90 | zh-translation-of → CON-FP-001 |
| CON-FP-002 | Round-to-nearest-even | EN | IEEE 754 [1] | 0.99 | rounding-mode → all 26.6 operations; also-known-as → banker's rounding |
| CON-FP-002a | 四舍六入五成双 | ZH | This paper | 0.90 | zh-translation-of → CON-FP-002 |
| CON-FP-003 | Saturation arithmetic | EN | This paper | 0.95 | overflow-policy → 26.6 operations; prevents → wrapping |
| CON-FP-003a | 饱和算术 | ZH | This paper | 0.90 | zh-translation-of → CON-FP-003 |
| CON-FP-004 | Unit in the Last Place (ULP) | EN | IEEE 754 [1] | 0.99 | measures → quantization error; value → 1/64 sp in 26.6 |
| CON-FP-004a | 最低有效位单位 | ZH | This paper | 0.85 | zh-translation-of → CON-FP-004 |
| CON-FP-005 | Scaled point (sp) | EN | Knuth [4] | 0.99 | unit-of → G-IR coordinates; defined-as → 1/65536 pt |
| CON-FP-005a | 比例点 (sp) | ZH | This paper | 0.85 | zh-translation-of → CON-FP-005 |
| CON-FP-006 | Quantization error | EN | This paper | 0.95 | bounded-by → NC-FP-006; accumulates-as → THM-FP-ACCUMULATION |
| CON-FP-006a | 量化误差 | ZH | This paper | 0.90 | zh-translation-of → CON-FP-006 |
| CON-FP-007 | Cross-platform determinism | EN | REQ-11.3.x | 0.95 | guaranteed-by → THM-FP-DETERMINISM; requires → AX-FP-004 |
| CON-FP-007a | 跨平台确定性 | ZH | This paper | 0.90 | zh-translation-of → CON-FP-007 |
| CON-FP-008 | Newton's method (integer) | EN | Knuth [2] | 0.99 | used-in → ALG-FP-SQRT; converges → quadratically |
| CON-FP-008a | 牛顿法（整数） | ZH | This paper | 0.85 | zh-translation-of → CON-FP-008 |
| CON-FP-009 | Grid fitting | EN | FreeType [3] | 0.90 | uses → 26.6 arithmetic; snaps → glyph outlines to pixel grid |
| CON-FP-009a | 网格对齐 | ZH | This paper | 0.85 | zh-translation-of → CON-FP-009 |
| CON-FP-010 | Mantissa (fixed-point) | EN | This paper | 0.95 | represents → 26.6 value; stored-as → i32 |
| CON-FP-010a | 尾数（定点） | ZH | This paper | 0.90 | zh-translation-of → CON-FP-010 |

---

## YP-10: Quality Checklist

- [x] **Document header complete** — YAML frontmatter with all required fields (YP-1)
- [x] **Executive summary with objective function** — Problem statement, scope, objective, IEEE 754 comparison table (YP-2)
- [x] **Nomenclature table with all symbols defined** — 18 symbols with domain and source (YP-3)
- [x] **Axioms (4) formally stated** — AX-FP-001 through AX-FP-004 with formal notation and intuition (YP-4.1)
- [x] **Definitions (6) formally stated with examples** — DEF-FP266, DEF-FP-ADD, DEF-FP-MUL, DEF-FP-DIV, DEF-FP-SAT (YP-4.2)
- [x] **Lemmas (5) with proof sketches** — LEM-FP-001 through LEM-FP-005 (YP-4.3)
- [x] **Theorems (6) with proof sketches** — THM-FP-ADD-EXACT, THM-FP-MUL-ROUND, THM-FP-SATURATION, THM-FP-COMPARISON, THM-FP-ACCUMULATION, THM-FP-DETERMINISM (YP-4.4)
- [x] **Algorithm specifications (3) with complexity analysis** — ALG-FP-MUL, ALG-FP-DIV, ALG-FP-SQRT (YP-5)
- [x] **Pre/postconditions defined** — 3 preconditions, 7 postconditions across all algorithms (YP-5)
- [x] **Test vector categories specified** — 5 categories with coverage targets and property-based invariants (YP-6)
- [x] **Domain constraints referenced** — 17 constraints with derivations and 2 conflict resolutions (YP-7)
- [x] **Bibliography with DOIs/URLs** — 9 references with TQA levels (YP-8)
- [x] **Knowledge graph concepts extracted** — 20 concepts (10 EN + 10 ZH) with relationships (YP-9)
- [x] **Quality checklist complete** — This section (YP-10)

---

*End of YP-NUMERICAL-FIXEDPOINT-001 v0.1.0*
