use polynomial::Polynomial;
use rand::{Rng, thread_rng};

const P: u32 = 487;
pub fn lagrange_interpolation(xs: &[u32], ys: &[u32]) -> Polynomial<f64> {
    let n = xs.len();
    assert_eq!(n, ys.len(), "xs and ys must have the same length");

    let mut result = vec![0.0; n];

    for i in 0..n {
        let mut li = vec![1.0];

        for j in 0..n {
            if i != j {
                let xj = xs[j] as f64;
                let xi = xs[i] as f64;

                li = multiply_polynomials(&li, &[-xj / (xi - xj), 1.0 / (xi - xj)]);
            }
        }

        for (coeff_idx, coeff) in li.iter().enumerate() {
            result[coeff_idx] += ys[i] as f64 * coeff;
        }
    }

    Polynomial::new(result)
}

pub fn gen_random_polynomial(degree: usize) -> Polynomial<i32> {
    let mut rng = thread_rng();
    let mut coefficients = Vec::with_capacity(degree + 1);

    for _ in 0..=degree {
      coefficients.push(rng.gen_range(0..P as i32));
    };

    Polynomial::new(coefficients)
}

fn multiply_polynomials(p1: &[f64], p2: &[f64]) -> Vec<f64> {
    let mut result = vec![0.0; p1.len() + p2.len() - 1];

    for (i, &coeff1) in p1.iter().enumerate() {
        for (j, &coeff2) in p2.iter().enumerate() {
            result[i + j] += coeff1 * coeff2;
        }
    }

    result
}
