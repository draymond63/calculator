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

// TODO: Convert to struct with methods
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
        let (exp, base_unit) = UnitVal::from_unit_str(unit).unwrap();
        let scale_factor = 10.0_f32.powf(exp as f32);
        let value = value * scale_factor;
        base_unit.into_unit_val(value)
    }

    pub fn is_valid_unit(unit: &str) -> bool {
        UnitVal::from_unit_str(unit).is_ok()
    }

    pub fn new_identity(unit: &str) -> Self {
        UnitVal::new_value(1.0, unit)
    }

    pub fn to_string(&self) -> String {
        if self.is_scalar() {
            return self.value.to_string()
        }
        let used_units = Unit::compile_used_units(&self.quantity, "SI").unwrap();
        let base_unit = Unit::compose(&used_units, &self.quantity);
        let val = base_unit.from_si(self.value);

        let numerator_units: Vec<(&&str, &i32)> = used_units.iter().filter(|(_, exp)| **exp > 0).collect();
        let has_units_multiplied: bool = numerator_units.len() > 1;
        if has_units_multiplied || base_unit.name == "kg" { // TODO: Make working with grams more ergonomic
            return format!("{} {}", self.value, base_unit.name)
        }

        let val_exp = val.log10().floor() as i32;
        if val_exp == 0 {
            format!("{} {}", val, base_unit.name)
        } else if let Some((_, numerator_unit_exp)) = numerator_units.get(0) {
            let val = val / 10.0_f32.powf(val_exp as f32);
            // Account for the exponent of the unit it's being applied to
            let val_exp = val_exp / *numerator_unit_exp;
            // Reduce the exponent to the nearest multiple of 3
            let val_exp = val_exp / 3 * 3;
            let prefix = prefix_map().get_by_right(&val_exp).expect("Invalid unit prefix");
            format!("{} {}{}", val, prefix, base_unit.name)
        } else {
            // TODO: Allow prefixes when there is a single denominator units
            format!("{} {}", val, base_unit.name)
        }
    }

    pub fn is_scalar(&self) -> bool {
        self.quantity == UnitVal::unitless()
    }

    /// Parsing the Unit and exponential from a base unit's shortand and it's prefix e.g. "kN" -> (3, NewtonUnit).
    /// 
    /// Cannot handle composed units like "kg*m/s^2"
    /// 
    /// TODO: Doesn't work with mols or psi since they start with prefix letters m & p
    fn from_unit_str(unit: &str) -> Result<(i32, Unit), Error> {
        if unit.is_empty() {
            return Err(Error::UnitError("No units given. Value is scalar".to_string()));
        }
        let possible_prefix = unit.chars().next().expect("Unit was empty but not caught by is_empty check");
        if unit == "kg" {
            Ok((0, unit_map().get("kg").unwrap().clone()))
        } else if unit.len() > 1 && prefix_map().contains_left(&possible_prefix) {
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
    pub si_scale: f32,
    pub quantity: Quantity,
    // TODO: Add optional max and min prefixes (e.g. can't have megametres)
}

impl Unit {
    pub fn new(name: &str, si_scale: f32, quantity: Quantity) -> Self {
        Unit { name: name.to_string(), si_scale, quantity }
    }

    pub fn compose(units: &HashMap<&str, i32>, quantity: &Quantity) -> Self {
        // TODO: Doesn't check that the unit map matches the quantity
        let unit_vec = units.iter().map(|(name, exp)| (unit_map().get(name).unwrap(), *exp)).collect_vec();
        let si_scale  = unit_vec.iter().fold(1.0, |acc, (unit, exp)| acc * (unit.si_scale.powi(*exp)));
        let name = Unit::get_unit_str(units);
        Unit::new(&name, si_scale, quantity.clone())
    }

    fn get_unit_str(units: &HashMap<&str, i32>) -> String {
        let mut unit = String::new();
        let mut seen_negatives = false;
        for (base_unit, power) in units.iter().sorted() {
            if *power < 0 && !seen_negatives {
                seen_negatives = true;
                unit.push_str("/");
            }
            // TODO: If there are only negative exponents, show them as negatives instead of /x
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
    pub fn compile_used_units(quantity: &Quantity, system: &str) -> CResult<HashMap<&'static str, i32>> {
        let max_iter = 5;
        let system = &unit_system()[system];
        let mut avail_units = unit_map().clone();
        let mut used_units = HashMap::new();
        let mut iterations = 0;
        let mut current_quantity = quantity.clone();

        while Unit::dimensionality(&current_quantity) != 0 && iterations < max_iter {
            let mut best_unit = ""; // TODO: Allow multiple potential matches (e.g. s and Hz)
            let mut best_match = 0;
            let mut remaining_quantity = current_quantity.clone();
            let mut exp = 0;
            for (name, unit) in avail_units.clone().iter() {
                if system.contains(name) && unit.is_subset_of_quantity(&current_quantity) {
                    let match_score = unit.quantity.iter().map(|x| x.abs()).sum();
                    if match_score > best_match {
                        best_unit = name;
                        best_match = match_score;
                        exp = unit.find_best_exp(&current_quantity).expect("Exponent not found");
                        remaining_quantity = unit.zip_quantity(&current_quantity).map(|(a, b)| b - a * exp).collect();
                    }
                } else {
                    avail_units.remove(name); // No need to check it ever again
                }
            }
            used_units.insert(best_unit,  exp);
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

    fn is_subset_of_quantity(&self, q2: &Quantity) -> bool {
        let sign = match self.find_best_exp(q2) {
            Some(e) => e.signum(),
            None => return false
        };
        self.zip_quantity(q2).all(|(sub_unit, set_unit)| {
            let sub_unit = *sub_unit;
            let set_unit = *set_unit;
            // If the signs are consistently the same (or consistently opposite) and the sub_unit is less than the set_unit
            let consistent_sign = (sub_unit * sign).signum() == set_unit.signum();
            let not_too_big = sub_unit.abs() <= set_unit.abs();
            (sub_unit == 0) || (consistent_sign && not_too_big)
        })
    }

    fn find_best_exp(&self, q2: &Quantity) -> Option<i32> {
        let mut possible_exps = vec![];
        for (a, b) in self.zip_quantity(q2) {
            if *a != 0 && *b != 0 {
                possible_exps.push(b / a);
                break;
            }
        }
        possible_exps.iter().min().copied()
    }

    fn zip_quantity<'a>(&'a self, q2: &'a Quantity) -> impl Iterator<Item = (&'a i32, &'a i32)> {
        self.quantity.iter().zip(q2.iter())
    }


    pub fn into_unit_val(&self, value: f32) -> UnitVal {
        UnitVal::new(self.to_si(value), self.quantity.clone())
    }

    pub fn to_si(&self, value: f32) -> f32 {
        value * self.si_scale
    }

    pub fn from_si(&self, value: f32) -> f32 {
        value / self.si_scale
    }

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
        m.insert("Pa", Unit::new("Pa", 1.0, vec![-1, -2, 1, 0, 0, 0, 0])); // Newton
        m.insert("psi", Unit::new("psi", 6894.757, vec![-1, -2, 1, 0, 0, 0, 0])); // Newton
        m.insert("bar", Unit::new("bar", 100000.0, vec![-1, -2, 1, 0, 0, 0, 0])); // Newton
        m.insert("N", Unit::new("N", 1.0, vec![1, -2, 1, 0, 0, 0, 0])); // Newton
        m.insert("lbf", Unit::new("lbf", 4.448222, vec![1, -2, 1, 0, 0, 0, 0])); // Newton
        m.insert("m", Unit::new("m", 1.0, Unit::length())); // Meter
        m.insert("ft", Unit::new("ft", 0.3048, Unit::length())); // Meter
        m.insert("in", Unit::new("in", 0.0254, Unit::length())); // Meter
        m.insert("s", Unit::new("s", 1.0, Unit::time())); // Second
        m.insert("lb", Unit::new("lb", 0.4535924, Unit::mass())); // Gram
        m.insert("kg", Unit::new("kg", 1.0, Unit::mass())); // Gram
        m.insert("g", Unit::new("g", 0.001, Unit::mass())); // Gram
        m.insert("Hz", Unit::new("Hz", 1.0, Unit::frequency())); // Hertz
        m.insert("A", Unit::new("A", 1.0, Unit::current())); // Ampere
        m.insert("K", Unit::new("K", 1.0, Unit::temp())); // Kelvin
        m.insert("mol", Unit::new("mol", 1.0, Unit::amount())); // Mole
        m.insert("cd", Unit::new("cd", 1.0, Unit::lumenous())); // Candela
        m
    })
}

/// Used to determine which units are shown in the UI
fn unit_system() -> &'static HashMap<&'static str, Vec<&'static str>> {
    static HASHMAP: OnceLock<HashMap<&'static str, Vec<&'static str>>> = OnceLock::new();
    HASHMAP.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert("SI", vec!["m",  "s", "kg", "A", "K", "cd", "N", "Pa"]);
        m.insert("US", vec!["ft", "s", "lb", "A", "K", "cd", "lbf"]);
        m
    })
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Span, Context};
    use crate::evaluate_line;

    #[test]
    fn test_valid_unit() {
        for system in unit_system() {
            for unit in system.1 {
                println!("Testing unit: {}", unit);
                assert!(UnitVal::is_valid_unit(unit));
            }
        }
        assert!(UnitVal::is_valid_unit("Hz"));
        assert!(UnitVal::is_valid_unit("km"));
        assert!(UnitVal::is_valid_unit("mm"));
        assert!(UnitVal::is_valid_unit("mA"));
    }

    #[test]
    fn test_unit_identities() {
        let tests = unit_system().get("SI").unwrap();
        for base_unit in tests {
            // Special case for kg, it handles prefixes differently
            if *base_unit == "kg" {
                assert_eq!(UnitVal::new_value(1.0, base_unit).to_string(), "1 kg");
                continue;
            }
            for prefix in prefix_map() {
                let unit_str = format!("{}{}", prefix.0, base_unit);
                let val = UnitVal::new_value(1.0, &unit_str);
                let expected = format!("1 {}", unit_str);
                println!("{:?} = {}", val, expected);
                assert_eq!(val.to_string(), expected);
            }
        }
    }

    #[test]
    fn test_unit_conversions() {
        let tests = vec![
            ("1 N/kg", "1 m/s^2"),
            ("1 kPa/N", "1000 /m^2"),
        ];
        for (input, expected) in tests {
            let input = Span::new(input);
            let response = evaluate_line(input, &mut Context::new()).unwrap().unwrap();
            assert_eq!(response.to_string(), expected);
        }
    }
}