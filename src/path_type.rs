use std::path::PathBuf;
use std::sync::Mutex;
use hlua::{AsMutLua, PushGuard};
use tempfile::TempDir;

use hlua::implement_lua_read;

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum PathType {
    Absolute(PathBuf),
    Temporary(PathBuf),
}

impl PathType {
    pub fn to_absolute_path(&self, temp: &TempDir) -> PathBuf {
        match self {
            PathType::Absolute(path) => {
                path.clone()
            }
            PathType::Temporary(path) => {
                temp.path().join(path)
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

impl<'l, L: AsMutLua<'l>> hlua::LuaRead<L> for PathType {
    fn lua_read_at_position(lua: L, index: i32) -> Result<Self, L> {
        let mut table = hlua::LuaTable::lua_read_at_position(lua, index)?;

        let identifier: String = table.get("ident").unwrap();

        let path: String = table.get("path").unwrap();

        match identifier.as_str() {
            "t" => Ok(PathType::Temporary(PathBuf::from(path))),
            "a" => Ok(PathType::Absolute(PathBuf::from(path))),
            _ => todo!(),
        }
    }
}

/*impl hlua::Push<L> for PathType {
    type Err = ();

    fn push_to_lua(self, lua: L) -> Result<PushGuard<L>, (Self::Err, L)> {



        todo!()
    }
}*/

pub struct Inverse(Mutex<Vec<String>>);

impl Inverse {

    pub fn new() -> Self {
        Self(Mutex::new(Vec::new()))
    }

    pub fn insert(&self, index: usize, line: String) {
        let mut guard = self.0.lock().unwrap();
        guard.insert(index, line);
    }

    pub fn combine(&self) -> String {
        let mut guard = self.0.lock().unwrap();

        guard.iter().fold(String::new(), |mut source, line| { source.push_str(line); source })
    }

}


