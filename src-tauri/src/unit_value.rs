use crate::error::Error;
use crate::types::{BaseField, CResult};
use crate::units::*;

use serde::Serialize;


#[derive(Debug, Clone, PartialEq)]
pub struct UnitVal {
    pub value: f64,
    pub quantity: Quantity,
}


impl UnitVal {
    pub fn new(value: f64, quantity: Quantity) -> Self {
        UnitVal { value, quantity }
    }

    pub fn new_value(value: f64, unit: &str) -> Self {
        if unit.is_empty() {
            return UnitVal::scalar(value);
        }
        let (exp, base_unit) = UnitVal::from_unit_str(unit).unwrap();
        let scale_factor = 10.0_f64.powf(exp as f64);
        let value = value * scale_factor;
        UnitVal::new(base_unit.to_si(value), base_unit.quantity.clone())
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

        if let Some((_, numerator_unit_exp)) = numerator_units.get(0) {
            let val_exp = val.log10().floor() as i32;
            // Reduce the exponent to the nearest multiple of 3
            let val_exp = val_exp / 3 * 3;
            let reduced_val = val / 10.0_f64.powf(val_exp as f64);
            // Account for the exponent of the unit it's being applied to
            let val_exp = val_exp / *numerator_unit_exp;
            if let Some(prefix) = prefix_map().get_by_right(&val_exp) {
                format!("{} {}{}", reduced_val, prefix, base_unit.name)
            } else {
                format!("{} {}", val, base_unit.name)
            }
        } else {
            // TODO: Allow prefixes when there is a single denominator units
            format!("{} {}", val, base_unit.name)
        }
    }

    pub fn is_scalar(&self) -> bool {
        self.quantity == Quantity::unitless()
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
                (_, None) => Err(Error::DefinitionNotFoundError(unit_shorthand.to_string()))
            }
        } else {
            let base_unit = unit_map().get(unit);
            match base_unit {
                Some(q) => Ok((0, q.clone())),
                None => Err(Error::DefinitionNotFoundError(base_unit.unwrap().name.clone()))
            }
        }
    }

    fn powi(&self, n: i32) -> Self {
        UnitVal::new(self.value.powi(n), self.quantity.powi(n))
    }

    pub fn scalar(value: f64) -> UnitVal { UnitVal::new(value, Quantity::unitless()) }
}


impl std::fmt::Display for UnitVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl<'a> BaseField<'a> for UnitVal {
    fn as_scalar(&self) -> Result<f64, Error> {
        if self.quantity != Quantity::unitless() {
            Err(Error::UnitError(format!("Cannot convert unit to scalar: {}", self)))
        } else {
            Ok(self.value)
        }
    }

    fn powf(&self, exp: UnitVal) -> CResult<Self> {
        let exp: f64 = exp.as_scalar()?;
        if exp.fract() == 0.0 {
            let exp = exp as i32;
            Ok(self.powi(exp))
        } else if exp.abs() < 1.0 && (1.0 / exp).fract() == 0.0 {
            let n  = 1.0 / exp;
            self.root(UnitVal::scalar(n))
        } else {
            let value = self.as_scalar()?.powf(exp);
            Ok(UnitVal::scalar(value))
        }
    }

    fn root(&self, n: Self) -> CResult<Self> {
        let n = n.as_scalar()?;
        if n.fract() != 0.0 {
            Err(Error::UnitError(format!("Cannot take the {n}th root of {self}")))
        } else if let Ok(new_quantity) = self.quantity.clone().root(n as i32) {
            let exp = 1.0 / n;
            Ok(UnitVal::new(self.value.powf(exp), new_quantity))
        } else {
            Err(Error::UnitError(format!("Cannot take the {n}th root of {self}")))
        }
    }

    fn fract(&self) -> Result<f64, Error> {
        println!("Var: {:?}", self.as_scalar());
        let value = self.as_scalar()? % 1.0;
        Ok(value)
    }

    fn sin(&self) -> CResult<Self> {
        Ok(UnitVal::scalar(self.as_scalar()?.sin()))
    }
    fn cos(&self) -> CResult<Self> {
        Ok(UnitVal::scalar(self.as_scalar()?.cos()))
    }
    fn tan(&self) -> CResult<Self> {
        Ok(UnitVal::scalar(self.as_scalar()?.tan()))
    }
}


impl std::ops::Mul for UnitVal {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let value = self.value * rhs.value;
        UnitVal::new(value, self.quantity + rhs.quantity)
    }
}

impl std::ops::Div for UnitVal {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        let value = self.value / rhs.value;
        UnitVal::new(value, self.quantity - rhs.quantity)
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

impl<'a> std::convert::TryFrom<&'a str> for UnitVal {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        if UnitVal::is_valid_unit(value) {
            Ok(UnitVal::new_identity(value))
        } else {
            Err(Box::new(Error::UnitError(format!("Invalid unit: {value}"))))
        }
    }
}

impl std::convert::From<f64> for UnitVal {
    fn from(value: f64) -> Self {
        UnitVal::scalar(value)
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


#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluator::Evaluator;
    use crate::types::Span;
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
        assert!(UnitVal::is_valid_unit("ft"));
        assert!(UnitVal::is_valid_unit("lb"));
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
            for (prefix, _) in prefix_map() {
                let unit_str = format!("{}{}", prefix, base_unit);
                let val = UnitVal::new_value(1.0, &unit_str);
                let expected = format!("1 {}", unit_str);
                println!("(Expected) {} = {:?} (Received)", expected, val);
                assert_eq!(expected, val.to_string());
            }
        }
    }

    #[test]
    fn test_unit_conversions() {
        let tests = vec![
            ("1 N/kg", "1 m/s^2"),
            ("1 kPa/N", "1000 /m^2"),
            ("0.01 km^2", "10000 m^2"),
        ];
        for (input, expected) in tests {
            let input = Span::new(input);
            let response = evaluate_line(input,  &mut Evaluator::<UnitVal>::new()).unwrap().unwrap();
            assert_eq!(response.to_string(), expected);
        }
    }

    #[test]
    fn test_system_conversions() {
        let tests = vec![
            ("1 ft", "0.3048 m"),
            ("1 ft^2", "0.09290304 m^2"),
            ("100 ft^2", "9.290304 m^2"),
        ];
        for (input, expected) in tests {
            let input = Span::new(input);
            let response = evaluate_line(input, &mut Evaluator::<UnitVal>::new()).unwrap().unwrap();
            assert_eq!(response.to_string(), expected);
        }
    }
}
