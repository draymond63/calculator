use crate::error::Error;
use crate::types::CResult;

use bimap::BiMap;
use itertools::Itertools;
use std::{collections::HashMap, sync::OnceLock, vec};


#[derive(Debug, Clone, PartialEq)]
pub struct Unit {
    pub name: String,
    pub si_scale: f64,
    pub quantity: Quantity,
    // TODO: Add optional max and min prefixes (e.g. can't have megametres)
}

impl Unit {
    pub fn new(name: &str, si_scale: f64, quantity: Quantity) -> Self {
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

        while current_quantity.dimensionality() != 0 && iterations < max_iter {
            let mut best_unit = ""; // TODO: Allow multiple potential matches (e.g. s and Hz)
            let mut best_match = 0;
            let mut remaining_quantity = current_quantity.clone();
            let mut exp = 0;
            for (name, unit) in avail_units.clone().iter() {
                if system.contains(name) && unit.quantity.is_subset(&current_quantity) {
                    let match_score = unit.quantity.quantity.iter().map(|x| x.abs()).sum();
                    if match_score > best_match {
                        best_unit = name;
                        best_match = match_score;
                        exp = unit.quantity.find_subset_power(&current_quantity).expect("Exponent not found");
                        remaining_quantity = Quantity::new(unit.quantity.zip(&current_quantity).map(|(a, b)| b - a * exp).collect());
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

    pub fn to_si(&self, value: f64) -> f64 {
        value * self.si_scale
    }

    pub fn from_si(&self, value: f64) -> f64 {
        value / self.si_scale
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct Quantity {
    quantity: vec::Vec<i32>,
}

impl Quantity {
    pub fn new(quantity: vec::Vec<i32>) -> Self {
        if quantity.len() != 7 {
            panic!("Invalid quantity. Should have a length of 7: {:?}", quantity)
        }
        Quantity { quantity }
    }

    pub fn zip<'a>(&'a self, q2: &'a Self) -> impl Iterator<Item = (&'a i32, &'a i32)> {
        self.quantity.iter().zip(q2.quantity.iter())
    }

    pub fn map(&self, f: impl Fn(i32) -> i32) -> Self {
        Quantity::new(self.quantity.iter().map(|x| f(*x)).collect())
    }

    pub fn dimensionality(&self) -> i32 {
        self.quantity.iter().map(|x| x.abs()).sum()
    }

    pub fn is_subset(&self, q2: &Quantity) -> bool {
        let sign = match self.find_subset_power(q2) {
            Some(e) => e.signum(),
            None => return false
        };
        self.zip(q2).all(|(sub_unit, set_unit)| {
            let sub_unit = *sub_unit;
            let set_unit = *set_unit;
            // If the signs are consistently the same (or consistently opposite) and the sub_unit is less than the set_unit
            let consistent_sign = (sub_unit * sign).signum() == set_unit.signum();
            let not_too_big = sub_unit.abs() <= set_unit.abs();
            (sub_unit == 0) || (consistent_sign && not_too_big)
        })
    }

    pub fn find_subset_power(&self, larger_quantity: &Quantity) -> Option<i32> {
        let mut possible_exps = vec![];
        for (a, b) in self.zip(larger_quantity) {
            if *a != 0 && *b != 0 {
                possible_exps.push(b / a);
            }
        }
        possible_exps.iter().min().copied()
    }

    pub fn powi(&self, n: i32) -> Self {
        self.map(|a| a * n)
    }

    pub fn root(&self, n: i32) -> Result<Self, &str> {
        for magnitude in self.quantity.iter() {
            if magnitude % n != 0 {
                return Err("Failed to take the root of a quantity");
            }
        }
        Ok(self.map(|a| a / n))
    }

    pub fn length() -> Self { Quantity::new(vec![1, 0, 0, 0, 0, 0, 0]) }
    pub fn time() -> Self { Quantity::new(vec![0, 1, 0, 0, 0, 0, 0]) }
    pub fn frequency() -> Self { Quantity::new(vec![0, -1, 0, 0, 0, 0, 0]) }
    pub fn mass() -> Self { Quantity::new(vec![0, 0, 1, 0, 0, 0, 0]) }
    pub fn current() -> Self { Quantity::new(vec![0, 0, 0, 1, 0, 0, 0]) }
    pub fn temp() -> Self { Quantity::new(vec![0, 0, 0, 0, 1, 0, 0]) }
    pub fn amount() -> Self { Quantity::new(vec![0, 0, 0, 0, 0, 1, 0]) }
    pub fn lumenous() -> Self { Quantity::new(vec![0, 0, 0, 0, 0, 0, 1]) }
    pub fn force() -> Self { Quantity::new(vec![1, -2, 1, 0, 0, 0, 0]) }
    pub fn pressure() -> Self { Quantity::new(vec![-1, -2, 1, 0, 0, 0, 0]) }
    pub fn unitless() -> Quantity { Quantity::new(vec![0, 0, 0, 0, 0, 0, 0]) }
}

impl std::ops::Add for Quantity {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let quantity = self.quantity.iter().zip(rhs.quantity.iter()).map(|(a, b)| a + b).collect();
        Quantity::new(quantity)
    }
}
impl std::ops::Sub for Quantity {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let quantity = self.quantity.iter().zip(rhs.quantity.iter()).map(|(a, b)| a - b).collect();
        Quantity::new(quantity)
    }
}

// Implementation rom https://crates.io/crates/lazy_static
pub fn prefix_map() -> &'static BiMap<char, i32> {
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

pub fn unit_map() -> &'static HashMap<&'static str, Unit> {
    static HASHMAP: OnceLock<HashMap<&str, Unit>> = OnceLock::new();
    HASHMAP.get_or_init(|| {
        let mut m = HashMap::new();
        // m.insert("C", Unit::new("C", 272.15, 1.0, Unit::temp())); // Celsius
        // m.insert("F", Unit::new("F", 255.3722, 2.2, Unit::temp())); // Fahrenheit
        m.insert("Pa", Unit::new("Pa", 1.0, Quantity::pressure())); // Pascals
        m.insert("psi", Unit::new("psi", 6894.757, Quantity::pressure())); // PSI
        m.insert("bar", Unit::new("bar", 100000.0, Quantity::pressure())); // Bars
        m.insert("N", Unit::new("N", 1.0, Quantity::force())); // Newton
        m.insert("lbf", Unit::new("lbf", 4.448222, Quantity::force())); // Newton
        m.insert("m", Unit::new("m", 1.0, Quantity::length())); // Meter
        m.insert("ft", Unit::new("ft", 0.3048, Quantity::length())); // Meter
        m.insert("in", Unit::new("in", 0.0254, Quantity::length())); // Meter
        m.insert("s", Unit::new("s", 1.0, Quantity::time())); // Second
        m.insert("lb", Unit::new("lb", 0.4535924, Quantity::mass())); // Gram
        m.insert("kg", Unit::new("kg", 1.0, Quantity::mass())); // Gram
        m.insert("g", Unit::new("g", 0.001, Quantity::mass())); // Gram
        m.insert("Hz", Unit::new("Hz", 1.0, Quantity::frequency())); // Hertz
        m.insert("A", Unit::new("A", 1.0, Quantity::current())); // Ampere
        m.insert("K", Unit::new("K", 1.0, Quantity::temp())); // Kelvin
        m.insert("mol", Unit::new("mol", 1.0, Quantity::amount())); // Mole
        m.insert("cd", Unit::new("cd", 1.0, Quantity::lumenous())); // Candela
        m
    })
}

/// Used to determine which units are shown in the UI
/// 
/// TODO: Use systems on the frontend. Have the background return all possible units and the frontend filters them.
/// TODO: User should be able to specify a custom system (on frontend)
pub fn unit_system() -> &'static HashMap<&'static str, Vec<&'static str>> {
    static HASHMAP: OnceLock<HashMap<&'static str, Vec<&'static str>>> = OnceLock::new();
    HASHMAP.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert("SI", vec!["m",  "s", "kg", "A", "K", "cd", "N", "Pa"]);
        m.insert("US", vec!["ft", "s", "lb", "A", "K", "cd", "lbf"]);
        m
    })
}
