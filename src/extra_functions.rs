/*

Contains functions to assist installation, but do not change target computer state (i.e. do not have inverses
and do not contribute to the uninstaller)

Registry
    - load: Loads the subkeys and values of the specified registry entry into a lua table
    - expand: expands environment variables

Misc
    - get: Get some input from the user

*/

use std::collections::HashMap;
use std::path::{Path};
use std::time::SystemTime;
use rlua::Result;

pub fn directory_contents(path: &Path) -> Result<HashMap<String, Vec<String>>> {
    let mut map = HashMap::new();

    let mut folders = Vec::new();
    let mut files = Vec::new();
    let mut other = Vec::new();

    for path in std::fs::read_dir(path).unwrap().map(|x| x.unwrap().path()) {
        if path.is_file() {
            files.push(path.to_str().unwrap().to_string());
        } else if path.is_dir() {
            folders.push(path.to_str().unwrap().to_string());
        } else {
            other.push(path.to_str().unwrap().to_string());
        }
    }

    map.insert(String::from("folders"), folders);
    map.insert(String::from("files"), files);
    map.insert(String::from("other"), other);

    Ok(map)
}

pub fn file_type(path: &Path) -> Result<String> {
    Ok(String::from(if path.is_file() {
        "file"
    } else if path.is_dir() {
        "directory"
    } else if path.is_symlink() {
        "symlink"
    } else {
        "unknown"
    }))
}

pub fn exists(path: &Path) -> Result<bool> {
    Ok(path.exists())
}

///Take a system time and convert it into a hashmap lua style
pub fn system_time_to_epoch(time: std::io::Result<SystemTime>) -> Option<u64> {
    match time {
        Ok(t) => {t.duration_since(SystemTime::UNIX_EPOCH).ok().map(|d| d.as_secs())}
        Err(_) => {None}
    }
}

pub fn file_timestamps(path: &Path) -> Result<HashMap<String, Option<u64>>> {

    let mut stamps = HashMap::new();

    let m = path.metadata().unwrap();

    stamps.insert("modified".to_string(), system_time_to_epoch(m.modified()));
    stamps.insert("created".to_string(), system_time_to_epoch(m.created()));
    stamps.insert("accessed".to_string(), system_time_to_epoch(m.accessed()));

    Ok(stamps)
}
