use crate::error::Error;
use crate::types::CResult;

use bimap::BiMap;
use itertools::Itertools;
use std::{collections::HashMap, sync::OnceLock, vec};
use serde::Serialize;


#[derive(Debug, Clone, PartialEq)]
pub struct UnitVal {
    pub value: f32,
    pub quantity: Quantity,
}

type Quantity = vec::Vec<i32>;


impl UnitVal {
    pub fn new(value: f32, quantity: Quantity) -> Self {
        if quantity.len() != 7 {
            panic!("Invalid quantity. Should have a length of 7: {:?}", quantity)
        }
        UnitVal { value, quantity }
    }

    pub fn new_value(value: f32, unit: &str) -> Self {
        let (exp, quantity) = UnitVal::from_unit(unit).unwrap();
        let scale_factor = 10.0_f32.powf(exp as f32);
        let value = value * scale_factor;
        UnitVal::new(value, quantity)
    }

    pub fn scalar(value: f32) -> Self {
        UnitVal { value, quantity: UnitVal::unitless() }
    }

    pub fn is_valid_unit(unit: &str) -> bool {
        UnitVal::from_unit(unit).is_ok()
    }

    pub fn new_identity(unit: &str) -> Self {
        UnitVal::new_value(1.0, unit)
    }

    pub fn to_string(&self) -> String {
        if self.is_scalar() {
            return self.value.to_string()
        } 
        let exp = self.value.log10().floor() as i32 / 3 * 3;
        let val: f32 = self.value / 10.0_f32.powf(exp as f32);
        let base_unit = self.unit_str().unwrap();
        if exp == 0 {
            format!("{} {}", val, base_unit)
        } else {
            let prefix = prefix_map().get_by_right(&exp).expect("Invalid unit prefix");
            format!("{} {}{}", val, prefix, base_unit)
        }
    }

    pub fn unit_str(&self) -> CResult<String> {
        let mut used_units = HashMap::new();
        for (index, power) in self.quantity.iter().enumerate() {
            if *power != 0 {
                let mut identity_quantity = vec![0; 7];
                identity_quantity[index] = 1;
                let base_unit = match unit_map().get_by_right(&identity_quantity) {
                    Some(unit) => *unit,
                    None => return Err(Error::UnitError(format!("Invalid unit quantity: {:?}", identity_quantity)))
                };
                used_units.insert(base_unit, *power);
            }
        }
        let mut unit = String::new();
        let mut seen_negatives = false;
        for (base_unit, power) in used_units.iter().sorted() {
            if *power < 0 && !seen_negatives {
                seen_negatives = true;
                unit.push_str("/");
            }
            let power = power.abs();
            if power == 1 {
                unit.push_str(base_unit);
            } else {
                unit.push_str(&format!("{}^{}", base_unit, power));
            }
        }
        Ok(unit.to_string())
    }

    pub fn is_scalar(&self) -> bool {
        self.quantity == UnitVal::unitless()
    }

    fn from_unit(unit: &str) -> Result<(i32, Quantity), Error> {
        if unit.is_empty() {
            return Ok((0, UnitVal::unitless()));
        }
        let possible_prefix = unit.chars().next().expect("Unit was empty but not caught by is_empty check");
        if unit.len() > 1 && prefix_map().contains_left(&possible_prefix) {
            let prefix = possible_prefix;
            let base_unit = &unit[1..];
            let exp = prefix_map().get_by_left(&prefix);
            let quantity = unit_map().get_by_left(base_unit);
            match (exp, quantity) {
                (Some(e), Some(q)) => Ok((e.clone(), q.clone())),
                (None, _) => Err(Error::UnitError(format!("Invalid unit prefix '{prefix}'"))),
                (_, None) => Err(Error::UnitError(format!("Invalid base unit '{base_unit}'")))
            }
        } else {
            let quantity = unit_map().get_by_left(unit);
            match quantity {
                Some(q) => Ok((0, q.clone())),
                None => Err(Error::UnitError(format!("Invalid base unit '{unit}'")))
            }
        }
    }

    fn meter() -> Quantity { vec![1, 0, 0, 0, 0, 0, 0] }
    fn second() -> Quantity { vec![0, 1, 0, 0, 0, 0, 0] }
    fn hertz() -> Quantity { vec![0, -1, 0, 0, 0, 0, 0] }
    fn gram() -> Quantity { vec![0, 0, 1, 0, 0, 0, 0] }
    fn ampere() -> Quantity { vec![0, 0, 0, 1, 0, 0, 0] }
    fn kelvin() -> Quantity { vec![0, 0, 0, 0, 1, 0, 0] }
    fn mole() -> Quantity { vec![0, 0, 0, 0, 0, 1, 0] }
    fn candela() -> Quantity { vec![0, 0, 0, 0, 0, 0, 1] }
    fn unitless() -> Quantity { vec![0, 0, 0, 0, 0, 0, 0] }
}


impl std::fmt::Display for UnitVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl UnitVal {
    pub fn as_scalar(&self) -> Result<f32, Error> {
        if self.quantity != UnitVal::unitless() {
            Err(Error::UnitError(format!("Cannot convert unit to scalar: {}", self)))
        } else {
            Ok(self.value)
        }
    }

    pub fn powf(&self, exp: UnitVal) -> Result<Self, Error> {
        let exp: f32 = exp.as_scalar()?;
        if exp.fract() == 0.0 {
            let exp = exp as i32;
            Ok(self.powi(exp))
        } else {
            let value = self.as_scalar()?.powf(exp);
            Ok(UnitVal::new(value, UnitVal::unitless()))
        }
    }

    fn powi(&self, exp: i32) -> Self {
        let a = &self.quantity;
        UnitVal {
            value: self.value.powi(exp),
            quantity: (0..a.len()).map(|i| a[i] * exp).collect()
        }
    }

    pub fn sqrt(&self) -> Result<Self, Error> {
        let value = self.as_scalar()?.sqrt();
        Ok(UnitVal::new(value, UnitVal::unitless()))
    }

    pub fn fract(&self) -> Result<f32, Error> {
        let value = 1.0 / self.as_scalar()?;
        Ok(value)
    }
}


impl std::ops::Mul for UnitVal {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let value = self.value * rhs.value;
        let a = self.quantity;
        let b = rhs.quantity;
        let quantity = (0..a.len()).map(|i| a[i] + b[i]).collect();
        UnitVal { value, quantity }
    }
}

impl std::ops::Div for UnitVal {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        let value = self.value / rhs.value;
        let a = self.quantity;
        let b = rhs.quantity;
        let quantity = (0..a.len()).map(|i| a[i] - b[i]).collect();
        UnitVal { value, quantity }
    }
}

impl std::ops::Add for UnitVal {
    type Output = CResult<Self>;

    fn add(self, rhs: Self) -> Self::Output {
        if self.quantity != rhs.quantity {
            return Err(Error::UnitError(format!("Cannot add units with different quantities: {:?} and {:?}", self.to_string(), rhs.to_string())));
        }
        let value = self.value + rhs.value;
        let quantity = self.quantity;
        Ok(UnitVal { value, quantity })
    }
}

impl std::ops::AddAssign for UnitVal {
    fn add_assign(&mut self, rhs: Self) {
        if self.quantity != rhs.quantity {
            panic!("Cannot add units with different quantities: {:?} and {:?}", self.to_string(), rhs.to_string())
        }
        self.value += rhs.value;
    }
}

impl std::ops::Sub for UnitVal {
    type Output = CResult<Self>;

    fn sub(self, rhs: Self) -> Self::Output {
        if self.quantity != rhs.quantity {
            return Err(Error::UnitError(format!("Cannot subtract units with different quantities: {:?} and {:?}", self.to_string(), rhs.to_string())));
        }
        let value = self.value - rhs.value;
        let quantity = self.quantity;
        Ok(UnitVal { value, quantity })
    }
}

impl Serialize for UnitVal {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

// Implementation rom https://crates.io/crates/lazy_static
fn prefix_map() -> &'static BiMap<char, i32> {
    static HASHMAP: OnceLock<BiMap<char, i32>> = OnceLock::new();
    HASHMAP.get_or_init(|| {
        let mut m = BiMap::new();
        m.insert('p', -12);
        m.insert('n', -9);
        m.insert('u', -6);
        m.insert('m', -3);
        m.insert('k', 3);
        m.insert('M', 6);
        m.insert('G', 9);
        m.insert('T', 12);
        m
    })
}

fn unit_map() -> &'static BiMap<&'static str, Quantity> {
    static HASHMAP: OnceLock<BiMap<&str, Quantity>> = OnceLock::new();
    HASHMAP.get_or_init(|| {
        let mut m = BiMap::new();
        m.insert("m", UnitVal::meter());
        m.insert("s", UnitVal::second());
        m.insert("g", UnitVal::gram());
        m.insert("Hz", UnitVal::hertz());
        m.insert("A", UnitVal::ampere());
        m.insert("K", UnitVal::kelvin());
        m.insert("mol", UnitVal::mole());
        m.insert("cd", UnitVal::candela());
        m
    })
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_unit() {
        assert!(UnitVal::is_valid_unit("m"));
        assert!(UnitVal::is_valid_unit("s"));
        assert!(UnitVal::is_valid_unit("Hz"));
        assert!(UnitVal::is_valid_unit("km"));
        assert!(UnitVal::is_valid_unit("mm"));
        assert!(UnitVal::is_valid_unit("mA"));
    }
}