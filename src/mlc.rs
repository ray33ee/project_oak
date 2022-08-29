use std::path::{Path, PathBuf};
use std::result::Result;
use crate::{OakRead, OakWrite};

use crate::path_type::{Inverse, PathType};

use rlua::{Context, FromLua, Lua, Table, Value};
use crate::registry_ex::Data;

pub fn thing(path: PathType) {
    println!("path: {:?}", path);

}

//Take the oak code and run it
pub fn run(code: & str, install: & OakRead, mut uninstall: Option<& OakWrite>, inverses: Option<& Inverse>, temp: &tempfile::TempDir) -> Result<(), ()> {

    let lua = Lua::new();


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

    lua.context(|ctx| {
        ctx.scope(|scope| {

            ctx.globals().set("__delete",
                              scope.create_function(|_, path: PathType| {
                                  crate::functions::delete(uninstall, inverses, &path, temp);
                                  Ok(())
                              }).unwrap()
            ).unwrap();

            ctx.globals().set("__reg",
                              scope.create_function(|_, data: Data| {
                                  println!("{:?}", data);
                                  Ok(())
                              }).unwrap()
            ).unwrap();



            ctx.load(code.as_str()).exec().unwrap();

        });
    });

    //Delete the file and io functions

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

    }

    /*lua.context(|ctx| {
        let f= ctx.create_function(|_, path: PathType| {
            crate::functions::delete(uninstall, inverses, &path, temp);
            Ok(())
        }).unwrap();

        ctx.globals().set("__test", f).unwrap();
    });*/

    Ok(())
}

impl<'l> FromLua<'l> for PathType {
    fn from_lua(lua_value: Value<'l>, lua: Context<'l>) -> rlua::Result<Self> {
        let table = Table::from_lua(lua_value, lua)?;

        let ident: String = table.get("ident")?;

        let path: String = table.get("path")?;

        match ident.as_str() {
            "t" => { Ok(PathType::Temporary(PathBuf::from(path))) },
            "a" => { Ok(PathType::Absolute(PathBuf::from(path))) },
            _ => {
                Err(rlua::Error::FromLuaConversionError {
                    from: "Lua Table",
                    to: "PathType",
                    message: Some(format!("Invalid PathType value. Please create a pathtype via the pathtype.temp or pathtype.absolute functions"))
                })
            }
        }


    }
}

impl<'lua> FromLua<'lua> for Data {
    fn from_lua(lua_value: Value<'lua>, lua: Context<'lua>) -> rlua::Result<Self> {
        match lua_value {
            Value::Nil => {Ok(Data::None)}
            Value::Boolean(_) => {
                Err(rlua::Error::FromLuaConversionError {
                    from: "Boolean",
                    to: "Data",
                    message: Some("Could not convert lua boolean to registry Data".to_string()),
                })
            }
            Value::LightUserData(_) => {
                Err(rlua::Error::FromLuaConversionError {
                    from: "LightUserData",
                    to: "Data",
                    message: Some("Could not convert lua LightUserData to registry Data".to_string()),
                })
            }
            Value::Integer(i) => {Ok(Data::U32(i as u32))}
            Value::Number(n) => {Ok(Data::U32(n as u32))}
            Value::String(s) => {Ok(Data::String(s.to_str().unwrap().to_string()))}
            Value::Table(table) => {
                match table.get::<& str, String>("ident") {
                    Ok(ident) => {

                        match ident.as_str() {
                            "dword" => {
                                let val = table.get::<& str, u64>("value")?;
                                Ok(Data::U64(val))
                            },
                            "expanded" => {
                                let val = table.get::<& str, String>("value")?;
                                Ok(Data::ExpandString(val))
                            },
                            _ => {
                                Err(rlua::Error::FromLuaConversionError {
                                    from: "Table",
                                    to: "Data",
                                    message: Some("Invalid ident in table. Please use the registry functions, string, list of strings, or number for registry data".to_string()),
                                })
                            }
                        }

                    }
                    Err(_) => {

                        let pairs: Vec<_> = table.pairs::<Value, Value>().into_iter().map(|x| x.unwrap()).collect();

                        let is_multiline = move || {
                            let mut multi = vec![None; pairs.len()];

                            for (k, v) in pairs {

                                if let Value::String(s) = v {
                                    let ind = if let Value::Number(n) = k {
                                        n as usize
                                    } else if let Value::Integer(n) = k {
                                        n as usize
                                    } else {
                                        return Ok(None)
                                    };

                                    *(multi.get_mut(ind).ok_or(rlua::Error::FromLuaConversionError {
                                        from: "Vector of Strings",
                                        to: "Data::MultiString",
                                        message: Some("Bad index in multi string lua table".to_string()),
                                    })?) = Some(s.to_str().map_err(|x| x)?.to_string());

                                } else {
                                    return Ok(None);
                                }

                            }

                            Ok(Some(multi))
                        };

                        match is_multiline()? {
                            Some(v) => {
                                Ok(Data::MultiString(v.into_iter().map(|x| x.unwrap()).collect()))
                            }
                            None => {

                                Err(rlua::Error::FromLuaConversionError {
                                    from: "Table",
                                    to: "Data",
                                    message: Some("Could not convert lua table to registry Data".to_string()),
                                })
                            }
                        }



                    }
                }

            }
            Value::Function(_) => {
                Err(rlua::Error::FromLuaConversionError {
                    from: "Function",
                    to: "Data",
                    message: Some("Could not convert lua function to registry Data".to_string()),
                })
            }
            Value::Thread(_) => {
                Err(rlua::Error::FromLuaConversionError {
                    from: "Thread",
                    to: "Data",
                    message: Some("Could not convert lua thread to registry Data".to_string()),
                })
            }
            Value::UserData(_) => {
                Err(rlua::Error::FromLuaConversionError {
                    from: "UserData",
                    to: "Data",
                    message: Some("Could not convert lua userdata to registry Data".to_string()),
                })
            }
            Value::Error(_) => {
                Err(rlua::Error::FromLuaConversionError {
                    from: "Error",
                    to: "Data",
                    message: Some("Could not convert lua error to registry Data".to_string()),
                })
            }
        }
    }
}
