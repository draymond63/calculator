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
        if unit.is_empty() {
            return UnitVal::scalar(value);
        }
        let (exp, base_unit) = UnitVal::from_unit(unit).unwrap();
        let scale_factor = 10.0_f32.powf(exp as f32);
        let value = value * scale_factor;
        base_unit.into_unit_val(value)
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
        let base_unit = self.get_unit().unwrap();
        let val = base_unit.from_si(self.value);
        let exp = val.log10().floor() as i32 / 3 * 3;
        let val = val / 10.0_f32.powf(exp as f32);
        if exp == 0 {
            format!("{} {}", val, base_unit.name)
        } else {
            let prefix = prefix_map().get_by_right(&exp).expect("Invalid unit prefix");
            format!("{} {}{}", val, prefix, base_unit.name)
        }
    }

    fn get_unit(&self) -> CResult<Unit> {
        if self.quantity == UnitVal::unitless() {
            return Ok(Unit::unit_scalar())
        }
        Unit::from_quantity(&self.quantity)
    }

    pub fn is_scalar(&self) -> bool {
        self.quantity == UnitVal::unitless()
    }

    /// Parsing the Unit and exponential from the unit's shortand and it's prefix e.g. "kN" -> (3, NewtonUnit)
    fn from_unit(unit: &str) -> Result<(i32, Unit), Error> {
        if unit.is_empty() {
            return Err(Error::UnitError("No units given. Value is scalar".to_string()));
        }
        let possible_prefix = unit.chars().next().expect("Unit was empty but not caught by is_empty check");
        if unit.len() > 1 && prefix_map().contains_left(&possible_prefix) {
            let prefix = possible_prefix;
            let unit_shorthand = &unit[1..];
            let exp = prefix_map().get_by_left(&prefix);
            let base_unit = unit_map().get(unit_shorthand);
            match (exp, base_unit) {
                (Some(e), Some(q)) => Ok((e.clone(), q.clone())),
                (None, _) => Err(Error::UnitError(format!("Invalid unit prefix '{prefix}'"))),
                (_, None) => Err(Error::UnitError(format!("Invalid base unit '{unit_shorthand}'")))
            }
        } else {
            let base_unit = unit_map().get(unit);
            match base_unit {
                Some(q) => Ok((0, q.clone())),
                None => Err(Error::UnitError(format!("Invalid base unit '{unit}'")))
            }
        }
    }

    fn compose_quantities(q1: &Quantity, q2: &Quantity, func: impl Fn(i32, i32) -> i32) -> Quantity {
        (0..q1.len()).map(|i| func(q1[i], q2[i])).collect()
    }

    pub fn scalar(value: f32) -> UnitVal { UnitVal::new(value, UnitVal::unitless()) }
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


#[derive(Debug, Clone, PartialEq)]
struct Unit {
    pub name: String,
    pub si_offset: f32, // TODO: Offsets aren't used properly
    pub si_scale: f32,
    pub quantity: Quantity,
}

impl Unit {
    pub fn new(name: &str, si_offset: f32, si_scale: f32, quantity: Quantity) -> Self {
        Unit { name: name.to_string(), si_offset, si_scale, quantity }
    }

    pub fn from_quantity(quantity: &Quantity) -> CResult<Self> {
        let used_units = Unit::compile_used_units(quantity)?;
        let unit_vec = used_units.iter().map(|(name, exp)| (unit_map().get(name).unwrap(), *exp)).collect_vec();
        let si_scale  = unit_vec.iter().fold(1.0, |acc, (unit, exp)| acc * (unit.si_scale.powi(*exp)));
        // TODO: Is this right?
        let si_offset = unit_vec.iter().fold(0.0, |acc, (unit, exp)| acc + (unit.si_offset * (*exp as f32)));
        let name = Unit::get_unit_str(used_units);
        Ok(Unit::new(name.as_str(), si_offset, si_scale, quantity.clone()))
    }

    fn get_unit_str(units: HashMap<&str, i32>) -> String {
        let mut unit = String::new();
        let mut seen_negatives = false;
        for (base_unit, power) in units.iter().sorted() {
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
        unit
    }

    /// Returns a map of string units and their exponents that summarize the given quantity
    fn compile_used_units(quantity: &Quantity) -> CResult<HashMap<&str, i32>> {
        let max_iter = 5;
        let avail_units = unit_map(); // TODO: (optimization) Make mutable so that repeated iterations are on a subset
        let mut used_units = HashMap::new();
        let mut iterations = 0;
        let mut current_quantity = quantity.clone();

        while Unit::dimensionality(&current_quantity) != 0 && iterations < max_iter {
            let mut best_unit = "";
            let mut best_cost = Unit::dimensionality(&current_quantity);
            let mut remaining_quantity = current_quantity.clone();
            for (name, unit) in avail_units.iter() {
                // TODO: Can't handle divisions
                let next_quantity = UnitVal::compose_quantities(&current_quantity, &unit.quantity, |x, y| x - y);
                let unit_cost = Unit::dimensionality(&next_quantity);
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
            // println!("Cost at end of round: {} ({current_quantity:?})", Unit::dimensionality(&current_quantity));
        }
        if iterations >= max_iter {
            Err(Error::UnitError(format!("Unable to compile units to string given {quantity:?}. {current_quantity:?} still remains")))
        } else {
            Ok(used_units)
        }
    }

    fn dimensionality(quantity: &Quantity) -> i32 {
        quantity.iter().map(|x| x.abs()).sum()
    }


    pub fn into_unit_val(&self, value: f32) -> UnitVal {
        UnitVal::new(self.to_si(value), self.quantity.clone())
    }

    pub fn to_si(&self, value: f32) -> f32 {
        (value + self.si_offset) * self.si_scale
    }

    pub fn from_si(&self, value: f32) -> f32 {
        (value - self.si_offset) / self.si_scale
    }

    pub fn unit_scalar() -> Self {
        Unit::new("", 0.0, 1.0, Unit::scalar())
    }

    pub fn scalar() -> Quantity { vec![0, 0, 0, 0, 0, 0, 0] }
    pub fn length() -> Quantity { vec![1, 0, 0, 0, 0, 0, 0] }
    pub fn time() -> Quantity { vec![0, 1, 0, 0, 0, 0, 0] }
    pub fn frequency() -> Quantity { vec![0, -1, 0, 0, 0, 0, 0] }
    pub fn mass() -> Quantity { vec![0, 0, 1, 0, 0, 0, 0] }
    pub fn current() -> Quantity { vec![0, 0, 0, 1, 0, 0, 0] }
    pub fn temp() -> Quantity { vec![0, 0, 0, 0, 1, 0, 0] }
    pub fn amount() -> Quantity { vec![0, 0, 0, 0, 0, 1, 0] }
    pub fn lumenous() -> Quantity { vec![0, 0, 0, 0, 0, 0, 1] }
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

fn unit_map() -> &'static HashMap<&'static str, Unit> {
    static HASHMAP: OnceLock<HashMap<&str, Unit>> = OnceLock::new();
    HASHMAP.get_or_init(|| {
        let mut m = HashMap::new();
        // m.insert("C", Unit::new("C", 272.15, 1.0, Unit::temp())); // Celsius
        // m.insert("F", Unit::new("F", 255.3722, 2.2, Unit::temp())); // Fahrenheit
        m.insert("N", Unit::new("N", 0.0, 1000.0, vec![1, -2, 1, 0, 0, 0, 0])); // Newton
        m.insert("m", Unit::new("m", 0.0, 1.0, Unit::length())); // Meter
        m.insert("s", Unit::new("s", 0.0, 1.0, Unit::time())); // Second
        m.insert("g", Unit::new("g", 0.0, 1.0, Unit::mass())); // Gram
        m.insert("Hz", Unit::new("Hz",  0.0, 1.0, Unit::frequency())); // Hertz
        m.insert("A", Unit::new("A", 0.0, 1.0, Unit::current())); // Ampere
        m.insert("K", Unit::new("K", 0.0, 1.0, Unit::temp())); // Kelvin
        m.insert("mol", Unit::new("mol", 0.0, 1.0, Unit::amount())); // Mole
        m.insert("cd", Unit::new("cd", 0.0, 1.0, Unit::lumenous())); // Candela
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