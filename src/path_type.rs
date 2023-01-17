use std::path::PathBuf;
use std::sync::Mutex;
use tempfile::TempDir;


#[derive(PartialEq, Eq, Clone, Debug)]
pub enum PathType {
    Special(PathBuf, PathBuf),
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
            PathType::Special(special, path) => {
                let special = PathBuf::from(std::env::var(special.to_str().unwrap()).unwrap());
                special.join(&path)
            }
        }


    }

    pub fn is_temp(&self) -> bool {
        match &self {
            PathType::Special(_, _) => {false}
            PathType::Absolute(_) => {false}
            PathType::Temporary(_) => {true}
        }
    }
}


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
        let guard = self.0.lock().unwrap();

        guard.iter().fold(String::new(), |mut source, line| { source.push_str(line); source.push_str("\n"); source })
    }

}


