use bimap::BiMap;
use std::sync::OnceLock;

#[derive(Debug, Clone, PartialEq)]
pub struct UnitVal {
    pub value: f32,
    pub quantity: Quantity,
}

type Quantity = (i32, i32, i32, i32, i32, i32, i32);


impl UnitVal {
    pub fn new(value: f32, unit: &str) -> Self {
        let (exp, quantity) = UnitVal::from_unit(unit).unwrap();
        let scale_factor = 10.0_f32.powf(exp as f32);
        let value = value * scale_factor;
        UnitVal { value, quantity }
    }

    pub fn scalar(value: f32) -> Self {
        UnitVal { value, quantity: UnitVal::unitless() }
    }

    pub fn is_valid_unit(unit: &str) -> bool {
        UnitVal::from_unit(unit).is_ok()
    }

    pub fn new_base(unit: &str) -> Self {
        UnitVal::new(1.0, unit)
    }

    pub fn to_string(&self) -> String {
        if self.is_scalar() {
            return self.value.to_string()
        } 
        let exp = self.value.log10().floor() as i32 / 3 * 3;
        let val = self.value / 10.0_f32.powf(exp as f32);
        let base_unit = unit_map().get_by_right(&self.quantity).expect("Invalid unit quantity");
        if exp == 0 {
            format!("{} {}", val, base_unit)
        } else {
            let prefix = prefix_map().get_by_right(&exp).expect("Invalid unit prefix");
            format!("{} {}{}", val, prefix, base_unit)
        }
    }

    pub fn is_scalar(&self) -> bool {
        self.quantity == UnitVal::unitless()
    }

    fn from_unit(unit: &str) -> Result<(i32, Quantity), String> {
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
                (None, _) => Err(format!("Invalid unit prefix '{prefix}'")),
                (_, None) => Err(format!("Invalid base unit '{base_unit}'"))
            }
        } else {
            let quantity = unit_map().get_by_left(unit);
            match quantity {
                Some(q) => Ok((0, q.clone())),
                None => Err(format!("Invalid base unit '{unit}'"))
            }
        }
    }

    fn meter() -> Quantity {
        (1, 0, 0, 0, 0, 0, 0)
    }

    fn second() -> Quantity {
        (0, 1, 0, 0, 0, 0, 0)
    }

    fn hertz() -> Quantity {
        (0, -1, 0, 0, 0, 0, 0)
    }

    fn gram() -> Quantity {
        (0, 0, 1, 0, 0, 0, 0)
    }

    fn ampere() -> Quantity {
        (0, 0, 0, 1, 0, 0, 0)
    }

    fn unitless() -> Quantity {
        (0, 0, 0, 0, 0, 0, 0)
    }
}


impl UnitVal {
    pub fn as_scalar(&self) -> f32 {
        if self.quantity != UnitVal::unitless() {
            panic!("Cannot convert unit to scalar: {:?}", self.to_string())
        }
        self.value
    }

    pub fn powf(&self, exp: UnitVal) -> Self {
        let value = self.as_scalar().powf(exp.as_scalar());
        UnitVal { value, quantity: self.quantity }
    }

    pub fn sqrt(&self) -> Self {
        let value = self.as_scalar().sqrt();
        UnitVal { value, quantity: self.quantity }
    }

    pub fn fract(&self) -> f32 {
        let value = 1.0 / self.as_scalar();
        value
    }
}


impl std::ops::Mul for UnitVal {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let value = self.value * rhs.value;
        let quantity = (
            self.quantity.0 + rhs.quantity.0,
            self.quantity.1 + rhs.quantity.1,
            self.quantity.2 + rhs.quantity.2,
            self.quantity.3 + rhs.quantity.3,
            self.quantity.4 + rhs.quantity.4,
            self.quantity.5 + rhs.quantity.5,
            self.quantity.6 + rhs.quantity.6,
        );
        UnitVal { value, quantity }
    }
}

impl std::ops::Div for UnitVal {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        let value = self.value / rhs.value;
        let quantity = (
            self.quantity.0 - rhs.quantity.0,
            self.quantity.1 - rhs.quantity.1,
            self.quantity.2 - rhs.quantity.2,
            self.quantity.3 - rhs.quantity.3,
            self.quantity.4 - rhs.quantity.4,
            self.quantity.5 - rhs.quantity.5,
            self.quantity.6 - rhs.quantity.6,
        );
        UnitVal { value, quantity }
    }
}

impl std::ops::Add for UnitVal {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        if self.quantity != rhs.quantity {
            panic!("Cannot add units with different quantities: {:?} and {:?}", self.to_string(), rhs.to_string())
        }
        let value = self.value + rhs.value;
        let quantity = self.quantity;
        UnitVal { value, quantity }
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
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        if self.quantity != rhs.quantity {
            panic!("Cannot subtract units with different quantities: {:?} and {:?}", self.to_string(), rhs.to_string())
        }
        let value = self.value - rhs.value;
        let quantity = self.quantity;
        UnitVal { value, quantity }
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