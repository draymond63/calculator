use crate::types::{CResult, BaseField};
use std::ops::{Add, Sub, Mul, Div};
use std::convert::{TryFrom, From};
use serde::Serialize;


#[derive(Debug, Clone, PartialEq)]
pub struct Float {
    value: f64,
}


impl<'a> BaseField<'a> for Float {
    fn as_scalar(&self) -> CResult<f64> {
        Ok(self.value)
    }

    fn powf(&self, exp: Self) -> CResult<Self> {
        Ok(Float { value: self.value.powf(exp.value) })
    }

    fn root(&self, n: Self) -> CResult<Self> {
        Ok(Float { value: self.value.powf(1.0 / n.value) })
    }

    fn fract(&self) -> CResult<f64> {
        Ok(self.value.fract())
    }

    fn sin(&self) -> CResult<Self> {
        Ok(Float { value: self.value.sin() })
    }

    fn cos(&self) -> CResult<Self> {
        Ok(Float { value: self.value.cos() })
    }

    fn tan(&self) -> CResult<Self> {
        Ok(Float { value: self.value.tan() })
    }
}

impl TryFrom<&str> for Float {
    type Error = Box<dyn std::error::Error>;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let value = s.parse::<f64>()?;
        Ok(Float { value })
    }
}

impl From<f64> for Float {
    fn from(value: f64) -> Self {
        Float { value }
    }
}

impl Add for Float {
    type Output = CResult<Self>;

    fn add(self, other: Self) -> CResult<Self> {
        Ok(Float { value: self.value + other.value })
    }
}

impl Sub for Float {
    type Output = CResult<Self>;

    fn sub(self, other: Self) -> CResult<Self> {
        Ok(Float { value: self.value - other.value })
    }
}

impl Mul for Float {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Float { value: self.value * other.value }
    }
}

impl Div for Float {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        Float { value: self.value / other.value }
    }
}

impl std::fmt::Display for Float {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Serialize for Float {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        serializer.serialize_f64(self.value)
    }
}
