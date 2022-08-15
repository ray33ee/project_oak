use std::fmt::{Display, Formatter};
use std::ops::Add;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use crate::steps::PathType;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Operand {
    Null,
    I64(i64),
    String(String),
    Bool(bool),
    Path(PathType),
    //Array(Vec<Operand>),
}

impl Default for Operand {
    fn default() -> Self {
        Operand::Null
    }
}

impl From<&str> for Operand {
    fn from(str: &str) -> Self {
        if str == "true" {
            Operand::Bool(true)
        } else if str == "false" {
            Operand::Bool(false)
        } else if str == "null" {
            Operand::Null
        } else if str.as_bytes()[0] == '\"' as u8 && str.as_bytes()[str.len()-1] == '\"' as u8 {
            Operand::String(String::from(&str[1..str.len()-1]))
        } else if let Ok(i) = str.parse::<i64>() {
            Operand::I64(i)
        } else if &str[0..2] == "p\"" {
            Operand::Path(PathType::Absolute(PathBuf::from(&str[2..str.len()-1])))
        } else if &str[0..2] == "t\"" {
            Operand::Path(PathType::Temporary(PathBuf::from(&str[2..str.len()-1])))
        } else {
            panic!("Oak Script Error: Could not convert string to operand {}", str);
        }
    }
}

impl std::fmt::Display for Operand {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::Null => {write!(f, "null")}
            Operand::I64(n) => {write!(f, "{}", n)}
            Operand::String(str) => {write!(f, "{}", str)}
            Operand::Bool(b) => {write!(f, "{}", b)}
            Operand::Path(path) => {write!(f, "{}", path)}
        }
    }
}

impl TryFrom<Operand> for PathType {
    type Error = ();

    fn try_from(value: Operand) -> Result<Self, Self::Error> {
        if let Operand::Path(special) = value {
            Ok(special.clone())
        } else {
            Err(())
        }
    }
}

impl TryFrom<Operand> for String {
    type Error = ();

    fn try_from(value: Operand) -> Result<Self, Self::Error> {
        if let Operand::String(str) = value {
            Ok(str.clone())
        } else {
            Err(())
        }
    }
}

impl Operand {

    pub fn same_type(&self, rhs: &Operand) -> bool {
        use std::mem::discriminant;

        discriminant(self) == discriminant(&rhs)
    }

    pub fn try_add(&self, rhs: &Self) -> Result<Self, ()> {

        match self {
            Operand::Null => {
                panic!("Oak Script Error: Cannot add 'null' to values");
            }
            Operand::I64(n) => {
                if let Operand::I64(r) = rhs {
                    return Ok(Operand::I64(n+r))
                }
            }
            Operand::String(s) => {
                if let Operand::String(r) = rhs {
                    let mut s = s.clone();
                    s.push_str(&r);
                    return Ok(Operand::String(s))
                }
            }
            Operand::Bool(_) => {
                panic!("Oak Script Error: Cannot add 'bool' to values");
            }
            Operand::Path(p) => {
                if let Operand::Path(r) = rhs {
                    todo!()

                }
            }
        }

        Err(())
    }

    pub fn bool(&self) -> bool {
        match self {
            Operand::Null => {false}
            Operand::I64(n) => { *n != 0 }
            Operand::String(str) => { !str.is_empty() }
            Operand::Bool(b) => {*b}
            Operand::Path(p) => {

                panic!("Oak Script Error: Cannot convert path to boolean");
            }
        }
    }

    pub fn absolute_path(&self, temp: Option<&TempDir>) -> PathBuf {
        match self {
            Operand::Path(p) => {
                p.path(temp)
            }
            _ => {
                panic!("Can only get absolute path for a Path enum")
            }
        }
    }

}
