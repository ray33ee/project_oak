use std::path::{Path, PathBuf};
use std::result::Result;
use crate::{OakRead, OakWrite, PathType};

use std::cell::UnsafeCell;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use hlua::{AnyLuaValue, AsMutLua, Lua, LuaRead};
use crate::path_type::Inverse;
use crate::registry_ex::Data;


//Take the oak code and run it
pub fn run(code: & str, install: & OakRead, mut uninstall: Option<& OakWrite>, inverses: Option<& Inverse>, temp: &tempfile::TempDir) -> Result<(), ()> {

    let mut lua = Lua::new();

    lua.openlibs();

    //Delete the file and io functions
    lua.set("io", hlua::AnyLuaValue::LuaNil);

    //Add the other oak functions
    {
        //Add our owm oan functions
        // - oak.zip
        // - oak.unzip
        // - oak.delete_registry_entry
        // - oak.write_registry_value (one for each value type)
        // - oak.write_registry_key
        // - oak.download
        // - oak.edit

        let oak_functions = lua.empty_array("oak");
    }

    //Add the __* functions
    lua.set("__test", hlua::function1(move |path: PathType| -> () { println!("{:?}", path); }));
    lua.set("__reg_test", hlua::function1(move |data: Data| -> () { println!("{:?}", data); }));

    lua.set("__delete", hlua::function1(move |path: PathType| -> () { crate::functions::delete(uninstall, inverses, &path, temp); }));

    lua.set("__deletse", hlua::function1(move |path: PathType| -> () { crate::functions::delete(uninstall, inverses, &path, temp); }));



    let code = format!("
-- Delete the tmpname and execute functions
os.tmpname = null
os.execute = null

-- Redefine remove and rename
os.remove = __remove
os.rename = __rename

-- Add the following functions
os.move = __move
os.copy = __copy
os.create_dir = __create_dir
os.create_shortcut = __create_shortcut

-- Clean up
__move = null
__copy = null
__create_dir = null
__create_shortcut = null

-- PathType functions

function _temp (path)
    local res = {{}}
    res.ident = \"t\"
    res.path = path
    return res
end

function _absolute (path)
    local res = {{}}
    res.ident = \"a\"
    res.path = path
    return res
end

pathtype = {{}}
pathtype.temp = _temp
pathtype.absolute = _absolute

_temp = null
_absolute = null

function _expanded (s)
    local res = {{}}
    res.ident = \"expanded\"
    res.value = s
    return res
end

function _dword (n)
    local res = {{}}
    res.ident = \"dword\"
    res.value = n
    return res
end

registry = {{}}
registry.expanded = _expanded
registry.dword = _dword

_expanded = null
_dword = null


    {}", code);

    lua.execute::<()>(code.as_str()).unwrap();

    Ok(())
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


impl<'l, L: AsMutLua<'l>> LuaRead<L> for Data {
    fn lua_read_at_position(lua: L, index: i32) -> Result<Self, L> {
        let val = AnyLuaValue::lua_read_at_position(lua, index)?;

        match val {
            AnyLuaValue::LuaString(s) => { Ok(Data::String(s)) }
            AnyLuaValue::LuaAnyString(_) => {panic!("Cannot convert lua AnyString to registry data")}
            AnyLuaValue::LuaNumber(n) => { Ok(Data::U32(n as u32)) }
            AnyLuaValue::LuaBoolean(_) => {panic!("Cannot convert lua boolean to registry data")}
            AnyLuaValue::LuaArray(v) => {

                let ident = v.iter().find_map(|(key, value)| {
                    if let AnyLuaValue::LuaString(s) = key {
                        if s == "ident" {
                            if let AnyLuaValue::LuaString(v) = value {
                                return Some(v)
                            }
                        }
                    }

                    None
                });

                let value = v.iter().find_map(|(key, value)| {
                    if let AnyLuaValue::LuaString(s) = key {
                        if s == "value" {
                            return Some(value)
                        }
                    }

                    None
                });

                match ident {
                    Some(i) => {
                        match i.as_str() {
                            "expanded" => {
                                if let AnyLuaValue::LuaString(s) = value.unwrap() {
                                    Ok(Data::ExpandString(s.clone()))
                                } else {
                                    panic!("If ident is 'expanded', value must be a string")
                                }
                            }
                            "dword" => {
                                if let AnyLuaValue::LuaNumber(n) = value.unwrap() {
                                    Ok(Data::U64(*n as u64))
                                } else {
                                    panic!("If ident is 'dword', value must be a number")
                                }
                            }
                            _ => {panic!("Invalid ident")}
                        }
                    }
                    None => {
                        let is_string_array = v.iter().all(|(key, value)| {
                            if let AnyLuaValue::LuaNumber(_) = key {
                                if let AnyLuaValue::LuaString(_) = value {
                                    return true;
                                }
                            }
                            false
                        });

                        if is_string_array {

                            let mut multiline: Vec<_> = v.iter().map(|_| None).collect();

                            for (key, value) in v {
                                if let AnyLuaValue::LuaNumber(n) = key {
                                    if let AnyLuaValue::LuaString(s) = value {
                                        multiline[n as usize] = Some(s.clone())
                                    }
                                }
                            }

                            Ok(Data::MultiString(multiline.iter().map(|x| x.as_ref().unwrap().clone()).collect()))

                        } else {
                            panic!("Could not convert value to registry data")
                        }
                    }
                }


            }
            AnyLuaValue::LuaNil => {Ok(Data::None)}
            AnyLuaValue::LuaOther => {panic!("Cannot convert lua other to registry data")}
        }
    }
}

