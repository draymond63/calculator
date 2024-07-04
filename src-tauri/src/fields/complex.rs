use crate::error;
use crate::types::{CResult, BaseField};
use std::ops::{Add, Sub, Mul, Div};
use std::convert::{TryFrom, From};
use serde::Serialize;
use num_complex::Complex as NumComplex;


#[derive(Debug, Clone, PartialEq)]
pub struct Complex {
    value: NumComplex<f64>,
}

impl<'a> BaseField<'a> for Complex {
    fn as_scalar(&self) -> CResult<f64> {
        if self.value.im == 0.0 {
            Ok(self.value.re)
        } else {
            Err(error::Error::EvalError("Complex number is not a scalar".to_string()))
        }
    }

    fn powf(&self, exp: Self) -> CResult<Self> {
        Ok(Complex { value: self.value.powc(exp.value) })
    }

    fn root(&self, n: Self) -> CResult<Self> {
        Ok(Complex { value: self.value.powc(1.0 / n.value) })
    }

    fn fract(&self) -> CResult<f64> {
        let val = self.as_scalar()?;
        Ok(val.fract())
    }

    fn sin(&self) -> CResult<Self> {
        Ok(Complex { value: self.value.sin() })
    }

    fn cos(&self) -> CResult<Self> {
        Ok(Complex { value: self.value.cos() })
    }

    fn tan(&self) -> CResult<Self> {
        Ok(Complex { value: self.value.tan() })
    }
}

impl std::fmt::Display for Complex {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl TryFrom<&str> for Complex {
    type Error = Box<dyn std::error::Error>;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        if s == "i" {
            Ok(Complex { value: NumComplex::new(0.0, 1.0) })
        } else {
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid complex number")))
        }
    }
}

impl From<f64> for Complex {
    fn from(value: f64) -> Self {
        Complex { value: NumComplex::new(value, 0.0) }
    }
}

impl Add for Complex {
    type Output = CResult<Self>;

    fn add(self, other: Self) -> CResult<Self> {
        Ok(Complex { value: self.value + other.value })
    }
}

impl Sub for Complex {
    type Output = CResult<Self>;

    fn sub(self, other: Self) -> CResult<Self> {
        Ok(Complex { value: self.value - other.value })
    }
}

impl Mul for Complex {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Complex { value: self.value * other.value }
    }
}

impl Div for Complex {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        Complex { value: self.value / other.value }
    }
}

impl Serialize for Complex {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        serializer.serialize_str(&self.value.to_string())
    }
}
