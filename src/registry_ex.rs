use std::path::Path;
use registry::{RegKey, Security};
use serde::{Serialize, Deserialize};
use crate::error::Result;

//Creating our own Data and Rootkey implementations is required for serde
#[derive(Debug, Serialize, Deserialize)]
pub enum Data {
    None,
    String(String),
    ExpandString(String),
    Binary(Vec<u8>),
    U32(u32),
    U32BE(u32),
    Link,
    MultiString(Vec<String>),
    ResourceList,
    FullResourceDescriptor,
    ResourceRequirementsList,
    U64(u64),
}

impl From<&str> for Data {
    fn from(s: &str) -> Self {
        match s.parse::<u32>() {
            Ok(v) => {Data::U32(v)}
            Err(_) => {Data::String(String::from(s))}
        }
    }
}

impl From<registry::Data> for Data {
    fn from(d: registry::Data) -> Self {
        match d {
            registry::Data::None => {Data::None}
            registry::Data::String(z) => {Data::String(z.to_string_lossy())}
            registry::Data::ExpandString(z) => {Data::ExpandString(z.to_string_lossy())}
            registry::Data::Binary(z) => {Data::Binary(z)}
            registry::Data::U32(z) => {Data::U32(z)}
            registry::Data::U32BE(z) => {Data::U32BE(z)}
            registry::Data::Link => {Data::Link}
            registry::Data::MultiString(z) => {Data::MultiString(z.iter().map(|x| x.to_string_lossy()).collect())}
            registry::Data::ResourceList => {Data::ResourceList}
            registry::Data::FullResourceDescriptor => {Data::FullResourceDescriptor}
            registry::Data::ResourceRequirementsList => {Data::ResourceRequirementsList}
            registry::Data::U64(z) => {Data::U64(z)}
        }
    }
}

impl From<&Data> for registry::Data {
    fn from(d: &Data) -> Self {
        match d {
            Data::None => {registry::Data::None}
            Data::String(z) => {registry::Data::String(utfx::U16CString::try_from(z).unwrap())}
            Data::ExpandString(z) => {registry::Data::ExpandString(utfx::U16CString::try_from(z).unwrap())}
            Data::Binary(z) => {registry::Data::Binary(z.clone())}
            Data::U32(z) => {registry::Data::U32(z.clone())}
            Data::U32BE(z) => {registry::Data::U32BE(z.clone())}
            Data::Link => {registry::Data::Link}
            Data::MultiString(z) => {registry::Data::MultiString(z.iter().map(|s| utfx::U16CString::try_from(s).unwrap()).collect())}
            Data::ResourceList => {registry::Data::ResourceList}
            Data::FullResourceDescriptor => {registry::Data::FullResourceDescriptor}
            Data::ResourceRequirementsList => {registry::Data::ResourceRequirementsList}
            Data::U64(z) => {registry::Data::U64(z.clone()) }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RootKey {
    HKLM,
    HKCC,
    HKCR,
    HKCU,
    HKU
}

impl From<&str> for RootKey {
    fn from(s: &str) -> Self {
        match s {
            "hklm" => RootKey::HKLM,
            "hkcc" => RootKey::HKCC,
            "hkcr" => RootKey::HKCR,
            "hkcu" => RootKey::HKCU,
            "hku" => RootKey::HKU,
            _ => panic!("Invalid registry root key")
        }
    }
}

impl From<&RootKey> for registry::Hive {
    fn from(rk: &RootKey) -> Self {
        match rk {
            RootKey::HKLM => {registry::Hive::LocalMachine}
            RootKey::HKCC => {registry::Hive::CurrentConfig}
            RootKey::HKCR => {registry::Hive::ClassesRoot}
            RootKey::HKCU => {registry::Hive::CurrentUser}
            RootKey::HKU => {registry::Hive::Users}
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tree {
    name: String,
    keys: Vec<Tree>,
    values: Vec<(String, Data)>
}

impl Tree {
    //Take the registry tree and restore it to the given root and key
    pub fn restore(&self, root: &RootKey, key: &str) -> Result<()> {


        let reg = registry::Hive::from(root).open(key, Security::AllAccess)?;

        let path = Path::new(key).join(&self.name);
        let sub = path.to_str().unwrap();

        reg.create(&self.name, Security::AllAccess)?;


        for key in self.keys.iter() {
            key.restore(root, sub)?;
        }

        let reg = registry::Hive::from(root).open(sub, Security::AllAccess)?;

        for (name, data) in self.values.iter() {
            reg.set_value(name, &registry::Data::from(data))?;
        }

        Ok(())
    }
}

impl From<&RegKey> for Tree {
    //Recursively save registry key, subkeys and values as Tree
    fn from(k: &RegKey) -> Self {
        let path = k.to_string();

        let name = Path::new(&path).components().last().unwrap().as_os_str().to_str().unwrap();
        let name = String::from(name);

        let values = k
            .values()
            .map(|value| {
                let unwrap = value.unwrap();
                (unwrap.name().to_string_lossy(), Data::from(unwrap.data().clone()))
            })
            .collect();

        let keys = k
            .keys()
            .map(|keyref| {
                let unwrapped = keyref.unwrap();
                let reg = unwrapped.open(Security::AllAccess).unwrap();
                Tree::from(&reg)

            }).collect();

        Self {
            name,
            keys,
            values,
        }
    }
}