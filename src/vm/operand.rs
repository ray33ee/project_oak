use std::fmt::{Formatter};
use std::path::{PathBuf};
use registry::Data;
use tempfile::TempDir;

#[derive(PartialEq, Eq, Clone)]
pub enum PathType {
    Absolute(PathBuf),
    Temporary(PathBuf),
}

impl PathType {
    pub fn path(&self, temp: Option<&TempDir>) -> PathBuf {
        match self {
            PathType::Absolute(path) => {
                path.clone()
            }
            PathType::Temporary(path) => {
                temp.unwrap().path().join(path)
            }
        }


    }

    pub fn is_temp(&self) -> bool {
        if let PathType::Absolute(_) = self {
            false
        } else {
            true
        }
    }
}

impl std::fmt::Debug for PathType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PathType::Absolute(p) => {write!(f, "p\"{}\"", p.to_str().unwrap())}
            PathType::Temporary(p) => {write!(f, "t\"{}\"", p.to_str().unwrap())}
        }
    }
}

impl From<&str>  for PathType {
    fn from(string: &str) -> Self {
        let key = "temp";

        if string.starts_with(key) {
            PathType::Temporary(PathBuf::from(&string[key.len()..]).strip_prefix("\\").unwrap().to_path_buf())
        } else {
            PathType::Absolute(PathBuf::from(string))
        }
    }
}


#[derive(PartialEq, Eq, Clone)]
pub enum Operand {
    Null, //From string: literal 'null'
    I64(i64), //From String: literal number
    String(String), //From String: quote-enclosed string
    Bool(bool), //From String: literal 'true' or 'false'
    Path(PathType), //From String: Quote enclosed string prefixed with 'p' for absolute path or 't' for temporary
    //Array(Vec<Operand>), //From String: List of Operands separated by commas and enclosed in '[' and ']'

    //Registry data types
    //Binary(Vec<u8>), // From String: quote enclosed string prefixed with 'b'
    //MultiString(Vec<String>),
    //Expanded(String), // From String: quote enclosed string prefixed with 'e'
}

/*impl Default for Operand {
    fn default() -> Self {
        Operand::Null
    }
}*/

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

impl std::fmt::Debug for Operand {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::Null => {write!(f, "null")}
            Operand::I64(n) => {write!(f, "{}", n)}
            Operand::String(str) => {write!(f, "\"{}\"", str)}
            Operand::Bool(b) => {write!(f, "{}", b)}
            Operand::Path(path) => {write!(f, "{:?}", path)}
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

impl TryFrom<Operand> for registry::Data {
    type Error = ();

    fn try_from(value: Operand) -> Result<Self, Self::Error> {
        match value {
            Operand::Null => {Ok(registry::Data::None)}
            Operand::I64(n) => {Ok(registry::Data::U32(n as u32))}
            Operand::String(s) => {Ok(registry::Data::String(s.parse().unwrap()))}
            Operand::Bool(_) => {Err(())}
            Operand::Path(_) => {Err(())}
        }
    }
}

impl TryFrom<registry::Data> for Operand {
    type Error = ();

    fn try_from(value: Data) -> Result<Self, Self::Error> {
        match value {
            Data::None => {Ok(Operand::Null)}
            Data::String(s) => {Ok(Operand::String(s.to_string_lossy()))}
            Data::ExpandString(s) => {Ok(Operand::String(s.to_string_lossy()))}
            Data::U32(n) => {Ok(Operand::I64(n as i64))}
            Data::U64(n) => {Ok(Operand::I64(n as i64))}
            _ => Err(())
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
            Operand::Path(_) => {
                if let Operand::Path(_) = rhs {
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
            Operand::Path(_) => {

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
