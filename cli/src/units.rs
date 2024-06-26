use itertools::Itertools;
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
        println!("val: {}, exp: {}, quantity: {:?}", val, exp, self.quantity);
        let base_unit = unit_map().get_by_right(&self.quantity).expect("Invalid unit quantity");
        if exp == 0 {
            format!("{} {}", val, base_unit)
        } else {
            let prefix = prefix_map().get_by_left(&exp).expect("Invalid unit prefix");
            format!("{} {}{}", val, prefix, base_unit)
        }
    }

    pub fn is_scalar(&self) -> bool {
        self.quantity == UnitVal::unitless()
    }

    fn from_unit(unit: &str) -> Result<(i32, Quantity), String> {
        if unit.len() == 2 {
            let (prefix, base_unit) = unit.chars().into_iter().collect_tuple().expect("Expected 2 chars");
            let exp = *prefix_map().get_by_right(&prefix).expect("Invalid unit prefix"); // TODO: Propogate error
            let quantity = UnitVal::from_base_unit(base_unit)?;
            Ok((exp, quantity))
        } else if unit.len() == 1 {
            let base_unit = unit.chars().next().expect("Expected 1 char");
            let quantity = UnitVal::from_base_unit(base_unit)?;
            Ok((0, quantity))
        } else if unit.len() == 0 {
            Err("Empty unit received".to_string())
        } else {
            Err(format!("Invalid unit '{unit}'"))
        }
    }

    fn from_base_unit(unit: char) -> Result<Quantity, String> {
        let quantity = unit_map().get_by_left(&unit);
        match quantity {
            Some(q) => Ok(q.clone()),
            None => Err(format!("Invalid base unit '{unit}'"))
        }
    }

    fn meter() -> Quantity {
        (1, 0, 0, 0, 0, 0, 0)
    }

    fn second() -> Quantity {
        (0, 1, 0, 0, 0, 0, 0)
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


fn prefix_map() -> &'static BiMap<i32, char> {
    static HASHMAP: OnceLock<BiMap<i32, char>> = OnceLock::new();
    HASHMAP.get_or_init(|| {
        let mut m = BiMap::new();
        m.insert(-12, 'p');
        m.insert(-9, 'n');
        m.insert(-6, 'u');
        m.insert(-3, 'm');
        m.insert(3, 'k');
        m.insert(6, 'M');
        m.insert(9, 'G');
        m.insert(12, 'T');
        m
    })
}

fn unit_map() -> &'static BiMap<char, Quantity> {
    static HASHMAP: OnceLock<BiMap<char, Quantity>> = OnceLock::new();
    HASHMAP.get_or_init(|| {
        let mut m = BiMap::new();
        m.insert('m', UnitVal::meter());
        m.insert('s', UnitVal::second());
        m.insert('g', UnitVal::gram());
        m.insert('A', UnitVal::ampere());
        m
    })
}