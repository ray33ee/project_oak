/*

Contains functions to assist installation, but do not change target computer state (i.e. do not have inverses
and do not contribute to the uninstaller)

Registry
    - expand: expands environment variables

Misc
    - get: Get some input from the user

*/

use std::collections::HashMap;
use std::path::{Path};
use std::time::SystemTime;
use registry::Security;
use rlua::{Context, Result, Value};
use crate::mlc::registry_ex::{RootKey};

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

pub fn get_registry_data<'l>(c: Context<'l>, root: &RootKey, key: String) -> Result<rlua::Table<'l>> {

    let reg = registry::Hive::from(root).open(key, Security::Read).unwrap();

    let table = c.create_table().unwrap();

    let mut subkeys = vec![];
    let mut kv_pairs = HashMap::new();


    for key in reg.keys() {

        if let Ok(r) = key {
            subkeys.push(r.to_string());
        }
    }

    for value in reg.values() {
        if let Ok(v) = value {
            kv_pairs.insert(v.name().to_string_lossy(), crate::mlc::registry_ex::Data::from(v.data().clone()));
        }
    }

    table.set(Value::String(c.create_string("subkeys")?), c.create_sequence_from(subkeys)?)?;
    table.set(Value::String(c.create_string("kv_pairs")?), c.create_table_from(kv_pairs)?)?;


    Ok(table)
}

pub fn get_attributes(path: &Path) -> std::result::Result<u32, u32> {
    use winapi::um::fileapi::GetFileAttributesA;
    use winapi::um::errhandlingapi::GetLastError;

    let abs_str = path.to_str().unwrap().as_bytes();

    unsafe {
        let pointer = abs_str.as_ptr() as *const i8;

        let res = GetFileAttributesA(pointer);

        if res == 4294967295 {


            Err(GetLastError())
        } else {
            Ok(res)
        }
    }

}
