use std::path::PathBuf;
use std::sync::Mutex;
use tempfile::TempDir;

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


