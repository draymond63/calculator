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
        let (exp, base_unit) = UnitVal::from_unit(unit).unwrap();
        let scale_factor = 10.0_f32.powf(exp as f32);
        let value = value * scale_factor;
        let value = UnitVal::new(value - 1.0, base_unit.quantity.clone());
        (base_unit + value).unwrap()
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
        if self.quantity == UnitVal::unitless() {
            return Ok(String::from(""))
        }
        let used_units = UnitVal::compile_used_units(&self.quantity)?;
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

    /// Returns a map of string units and their exponents that summarize the given quantity
    fn compile_used_units(quantity: &Quantity) -> CResult<HashMap<&str, i32>> {
        let max_iter = 5;
        let avail_units = unit_map();
        let mut used_units = HashMap::new();
        let mut iterations = 0;
        let mut current_quantity = quantity.clone();

        while UnitVal::quantity_cost(&current_quantity) != 0 && iterations < max_iter {
            let mut best_unit = "";
            let mut best_cost = UnitVal::quantity_cost(&current_quantity);
            let mut remaining_quantity = current_quantity.clone();
            for (name, unit) in avail_units.iter() {
                let next_quantity = UnitVal::compose_quantities(&current_quantity, &unit.quantity, |x, y| x - y);
                let unit_cost = UnitVal::quantity_cost(&next_quantity);
                // println!("Dropping {name} from {current_quantity:?} results in {next_quantity:?} with cost of {unit_cost}");
                if unit_cost < best_cost {
                    best_cost = unit_cost;
                    best_unit = *name;
                    remaining_quantity = next_quantity;
                }
            }
            let current_unit_power = used_units.get(best_unit).unwrap_or(&0);
            used_units.insert(best_unit, current_unit_power + 1);
            iterations += 1;
            current_quantity = remaining_quantity;
            // println!("Cost at end of round: {} ({current_quantity:?})", UnitVal::quantity_cost(&current_quantity));
        }
        if iterations >= max_iter {
            Err(Error::UnitError(format!("Unable to compile units to string given {quantity:?}. {current_quantity:?} still remains")))
        } else {
            Ok(used_units)
        }
    }

    fn quantity_cost(quantity: &Quantity) -> i32 {
        quantity.iter().map(|x| x.abs()).sum()
    }

    pub fn is_scalar(&self) -> bool {
        self.quantity == UnitVal::unitless()
    }

    fn from_unit(unit: &str) -> Result<(i32, UnitVal), Error> {
        if unit.is_empty() {
            return Ok((0, UnitVal::scalar(1.0)));
        }
        let possible_prefix = unit.chars().next().expect("Unit was empty but not caught by is_empty check");
        if unit.len() > 1 && prefix_map().contains_left(&possible_prefix) {
            let prefix = possible_prefix;
            let base_unit = &unit[1..];
            let exp = prefix_map().get_by_left(&prefix);
            let quantity = unit_map().get(base_unit);
            match (exp, quantity) {
                (Some(e), Some(q)) => Ok((e.clone(), q.clone())),
                (None, _) => Err(Error::UnitError(format!("Invalid unit prefix '{prefix}'"))),
                (_, None) => Err(Error::UnitError(format!("Invalid base unit '{base_unit}'")))
            }
        } else {
            let quantity = unit_map().get(unit);
            match quantity {
                Some(q) => Ok((0, q.clone())),
                None => Err(Error::UnitError(format!("Invalid base unit '{unit}'")))
            }
        }
    }

    fn compose_quantities(q1: &Quantity, q2: &Quantity, func: impl Fn(i32, i32) -> i32) -> Quantity {
        (0..q1.len()).map(|i| func(q1[i], q2[i])).collect()
    }

    pub fn scalar(value: f32) -> UnitVal { UnitVal::new(value, UnitVal::unitless()) }
    fn meter(value: f32) -> UnitVal { UnitVal::new(value, vec![1, 0, 0, 0, 0, 0, 0]) }
    fn second(value: f32) -> UnitVal { UnitVal::new(value, vec![0, 1, 0, 0, 0, 0, 0]) }
    fn hertz(value: f32) -> UnitVal { UnitVal::new(value, vec![0, -1, 0, 0, 0, 0, 0]) }
    fn gram(value: f32) -> UnitVal { UnitVal::new(value, vec![0, 0, 1, 0, 0, 0, 0]) }
    fn ampere(value: f32) -> UnitVal { UnitVal::new(value, vec![0, 0, 0, 1, 0, 0, 0]) }
    fn kelvin(value: f32) -> UnitVal { UnitVal::new(value, vec![0, 0, 0, 0, 1, 0, 0]) }
    fn mole(value: f32) -> UnitVal { UnitVal::new(value, vec![0, 0, 0, 0, 0, 1, 0]) }
    fn candela(value: f32) -> UnitVal { UnitVal::new(value, vec![0, 0, 0, 0, 0, 0, 1]) }
    fn unitless() -> Quantity { vec![0, 0, 0, 0, 0, 0, 0] }
}


impl std::fmt::Display for UnitVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl UnitVal {
    pub fn as_scalar(&self) -> Result<f32, Error> {
        if self.quantity != vec![0, 0, 0, 0, 0, 0, 0] {
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
            Ok(UnitVal::scalar(value))
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
        Ok(UnitVal::scalar(value))
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

fn unit_map() -> &'static HashMap<&'static str, UnitVal> {
    static HASHMAP: OnceLock<HashMap<&str, UnitVal>> = OnceLock::new();
    HASHMAP.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert("C", UnitVal::kelvin(272.15));
        m.insert("N", UnitVal::new(1000.0, vec![1, -2, 1, 0, 0, 0, 0]));
        m.insert("m", UnitVal::meter(1.0));
        m.insert("s", UnitVal::second(1.0));
        m.insert("g", UnitVal::gram(1.0));
        m.insert("Hz", UnitVal::hertz(1.0));
        m.insert("A", UnitVal::ampere(1.0));
        m.insert("K", UnitVal::kelvin(1.0));
        m.insert("mol", UnitVal::mole(1.0));
        m.insert("cd", UnitVal::candela(1.0));
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