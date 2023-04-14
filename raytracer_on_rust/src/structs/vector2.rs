use crate::traits::{Dotable, Length, Normalizable, Zero};
use std::ops::{Add, Mul, Neg, Sub};

#[derive(Default, Copy, Clone)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Zero for Vector2 {
    fn zero() -> Self {
        Vector2 {
            x: 0.0_f32,
            y: 0.0_f32,
        }
    }
}

impl Length for Vector2 {
    fn length(&self) -> f32 {
        let length: f32 = (self.x * self.x + self.y * self.y).sqrt();
        length
    }
}

impl Normalizable for Vector2 {
    fn normalize(&self) -> Self {
        let length: f32 = self.length();
        assert!(length != 0.0_f32);
        Vector2 {
            x: self.x / length,
            y: self.y / length,
        }
    }
}

impl Dotable for Vector2 {
    type Operand = Vector2;
    fn dot(&self, other: &Self::Operand) -> f32 {
        self.x * other.x + self.y * other.y
    }
}

impl Sub for Vector2 {
    type Output = Vector2;
    fn sub(self, rhs: Self) -> Self::Output {
        Vector2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Add for Vector2 {
    type Output = Vector2;
    fn add(self, rhs: Self) -> Self::Output {
        Vector2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Neg for Vector2 {
    type Output = Vector2;
    fn neg(self) -> Self::Output {
        Vector2 {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl Mul<Vector2> for Vector2 {
    type Output = Vector2;
    fn mul(self, rhs: Vector2) -> Self::Output {
        Vector2 {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl Mul<f32> for Vector2 {
    type Output = Vector2;
    fn mul(self, rhs: f32) -> Self::Output {
        Vector2 {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Vector2 {
    #[allow(unused)]
    pub fn new(x: f32, y: f32) -> Vector2 {
        Vector2 { x, y }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_vector2() {
        let vec_src = Vector2::new(10.0_f32, 15.0_f32);
        assert_eq!(vec_src.x, 10.0_f32);
        assert_eq!(vec_src.y, 15.0_f32);

        let vec_add = vec_src + Vector2::new(2.0_f32, 5.0_f32);
        assert_eq!(vec_add.x, 12.0_f32);
        assert_eq!(vec_add.y, 20.0_f32);

        let vec_sub = vec_add - Vector2::new(10.0_f32, 15.0_f32);
        assert_eq!(vec_sub.x, 2.0_f32);
        assert_eq!(vec_sub.y, 5.0_f32);

        let vec_neg = -vec_sub;
        assert_eq!(vec_neg.x, -2.0_f32);
        assert_eq!(vec_neg.y, -5.0_f32);

        let vec_mul_1 = vec_neg * Vector2::new(-2.0_f32, -3.0_f32);
        assert_eq!(vec_mul_1.x, 4.0_f32);
        assert_eq!(vec_mul_1.y, 15.0_f32);

        let vec_mul_2 = vec_mul_1 * 2.0_f32;
        assert_eq!(vec_mul_2.x, 8.0_f32);
        assert_eq!(vec_mul_2.y, 30.0_f32);

        let vec_zero = Vector2::zero();
        assert_eq!(vec_zero.x, 0.0_f32);
        assert_eq!(vec_zero.y, 0.0_f32);

        let vec_length = Vector2::new(10.0_f32, 8.0_f32).length();
        assert_eq!(vec_length, 12.806249);

        let vec_normalized = Vector2::new(5.0, 5.0).normalize();
        assert_eq!(vec_normalized.x, 0.70710677);
        assert_eq!(vec_normalized.y, 0.70710677);

        let vec_dot_1 = Vector2::new(0.75, 0.75).dot(&Vector2::new(0.75, 0.75));
        assert_eq!(vec_dot_1, 1.125);

        let vec_dot_2 = Vector2::new(0.75, 0.75).dot(&Vector2::new(-0.75, -0.75));
        assert_eq!(vec_dot_2, -1.125);

        let vec_dot_3 = Vector2::new(0.75, 0.75).dot(&Vector2::new(0.75, -0.75));
        assert_eq!(vec_dot_3, 0.0);
    }
}
