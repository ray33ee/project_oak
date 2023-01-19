mod functions;
mod extra_functions;
mod higher_functions;
mod registry_ex;

use std::path::{PathBuf};
use std::sync::Arc;
use crate::{OakRead, OakWrite};

use crate::path_type::{Inverse, PathType};

use rlua::{Context, FromLua, Lua, Table, ToLua, Value};
use rlua::prelude::{LuaError};
use crate::mlc::registry_ex::{Data, RootKey};

use crate::error::{Error};

use rlua::Result;

//Take the oak code and run it
pub fn run(code: & str, install: & OakRead, uninstall: Option<& OakWrite>, inverses: Option<& Inverse>, temp: &tempfile::TempDir) -> Result<()> {

    let lua = Lua::new();


    let code = format!("
-- Delete the tmpname and execute functions
os.tmpname = null
io.tmpfile = null
os.execute = null

-- Redefine remove and rename
os.remove = __remove
os.rename = __rename

-- Add the following functions
os.move = __move
os.copy = __copy

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

function _special (special, path)
    local res = {{}}
    res.special = special
    res.ident = \"s\"
    res.path = path
    return res
end

pathtype = {{}}
pathtype.temp = _temp
pathtype.absolute = _absolute
pathtype.special = _special

pathtype.AppData = \"APPDATA\"
pathtype.ProgramFiles = \"PROGRAMFILES\"
pathtype.HomePath = \"HOMEPATH\"

_temp = null
_absolute = null

function _expanded (s)
    local res = {{}}
    res.ident = \"expanded\"
    res.value = s
    return res
end

function _qword (n)
    local res = {{}}
    res.ident = \"qword\"
    res.value = n
    return res
end

registry = {{}}
registry.expanded = _expanded
registry.qword = _qword

HKLM =\"HKLM\"
HKCC =\"HKCC\"
HKCR =\"HKCR\"
HKCU =\"HKCU\"
HKU  = \"HKU\"

_expanded = null
_qword = null

____io_open = io.open

function _open (filename, mode)
    __file_open(filename, mode)
    path = __get_abs_path(filename)
    return ____io_open(path, mode)
end

io.open = _open

--_open = null

oak = {{}}
oak.delete = __delete
oak.move = __move
oak.rename = __rename
oak.data = __data
oak.mkdir = __mkdir
oak.copy = __copy
oak.zip = __zip
oak.unzip = __unzip
oak.download = __download
oak.edit = _edit
oak.reg_write_key = __reg_write_key
oak.reg_delete_key = __reg_delete_key
oak.reg_write_value = __reg_write_value
oak.reg_delete_value = __reg_delete_value

oak.directory_contents = __directory_contents
oak.file_type = __file_type
oak.exists = __exists
oak.file_timestamps = __file_timestamps
oak.get_registry_data = __get_registry_data
oak.set_attributes = __set_attributes


{}


    ", code);

    lua.context(|ctx| {
        ctx.scope(|scope| {



            let globals = ctx.globals();

            globals.set("__delete",
                        scope.create_function(|_, path: PathType| {
                            functions::delete( uninstall, inverses.clone(), &path, temp)?;
                            Ok(())
                        }).unwrap()
            ).unwrap();

            globals.set("__move",
                        scope.create_function(|_, (source, destination): (PathType, PathType)| {
                            functions::_move(inverses, &source, &destination, temp)?;
                            Ok(())
                        }).unwrap()
            ).unwrap();

            globals.set("__rename",
                        scope.create_function(|_, (source, destination): (PathType, PathType)| {
                            functions::_move(inverses, &source, &destination, temp)?;
                            Ok(())
                        }).unwrap()
            ).unwrap();

            globals.set("__data",
                        scope.create_function(|_, (name, destination): (String, PathType)| {
                            functions::data(install, inverses, &name, &destination, temp)?;
                            Ok(())
                        }).unwrap()
            ).unwrap();

            /*globals.set("__create",
                        scope.create_function(|_, path: PathType| {
                            crate::functions::create( inverses, path, temp)?;
                            Ok(())
                        }).unwrap()
            ).unwrap();*/

            globals.set("__mkdir",
                        scope.create_function(|_, path: PathType| {
                            functions::mkdir( inverses, path, temp)?;
                            Ok(())
                        }).unwrap()
            ).unwrap();

            globals.set("__copy",
                        scope.create_function(|_, (source, destination): (PathType, PathType)| {
                            functions::copy(inverses, &source, &destination, temp)?;
                            Ok(())
                        }).unwrap()
            ).unwrap();

            globals.set("__zip",
                        scope.create_function(|_, (archive, folder): (PathType, PathType)| {
                            functions::zip(inverses, &archive, &folder, temp)?;
                            Ok(())
                        }).unwrap()
            ).unwrap();

            globals.set("__unzip",
                        scope.create_function(|_, (archive, folder): (PathType, PathType)| {
                            functions::unzip(inverses, &archive, &folder, temp)?;
                            Ok(())
                        }).unwrap()
            ).unwrap();

            globals.set("__download",
                        scope.create_function(|_, (url, destination): (String, PathType)| -> rlua::Result<String> {
                            let f = functions::download(inverses, &url, &destination, temp)?;
                            Ok(f)
                        }).unwrap()
            ).unwrap();

            globals.set("__edit",
                        scope.create_function(|_, (path, reg): (PathType, String)| {
                            functions::edit(uninstall, inverses, &path, &reg, temp)?;
                            Ok(())
                        }).unwrap()
            ).unwrap();

            globals.set("__reg_write_key",
                        scope.create_function(|_, (root, key): (RootKey, String)| {
                            functions::write_reg_key( inverses, &root, &key)?;
                            Ok(())
                        }).unwrap()
            ).unwrap();

            globals.set("__reg_delete_key",
                        scope.create_function(|_, (root, key): (RootKey, String)| {
                            functions::delete_reg_key( inverses, &root, &key)?;
                            Ok(())
                        }).unwrap()
            ).unwrap();

            globals.set("__reg_write_value",
                        scope.create_function(|_, (root, key, value, data): (RootKey, String, String, Data)| {
                            functions::write_reg_value( inverses, &root, &key, &value, &registry::Data::from(&data))?;
                            Ok(())
                        }).unwrap()
            ).unwrap();

            globals.set("__reg_delete_value",
                        scope.create_function(|_, (root, key, value): (RootKey, String, String)| {
                            functions::delete_reg_value( inverses, &root, &key, &value)?;
                            Ok(())
                        }).unwrap()
            ).unwrap();

            globals.set("__directory_contents",
                        scope.create_function(|_, path: String| {
                            extra_functions::directory_contents(&PathBuf::from(path))
                        }).unwrap()
            ).unwrap();

            globals.set("__file_type",
                        scope.create_function(|_, path: String| {
                            extra_functions::file_type(&PathBuf::from(path))
                        }).unwrap()
            ).unwrap();

            globals.set("__exists",
                        scope.create_function(|_, path: String| {
                            extra_functions::exists(&PathBuf::from(path))
                        }).unwrap()
            ).unwrap();

            globals.set("__file_timestamps",
                        scope.create_function(|_, path: String| {
                            extra_functions::file_timestamps(&PathBuf::from(path))
                        }).unwrap()
            ).unwrap();

            globals.set("__file_open",
                        scope.create_function(|_, (path, mode): (PathType, String)| {
                            functions::file_open(uninstall, inverses, path, mode, temp)?;
                            Ok(())
                        }).unwrap()
            ).unwrap();

            globals.set("__get_registry_data",
                        scope.create_function(|c, (root, key): (String, String)| {
                            extra_functions::get_registry_data(c, &RootKey::from(root.as_str()), key)
                        }).unwrap()
            ).unwrap();

            globals.set("__get_abs_path",
                        scope.create_function(|_, path: PathType| {
                            Ok(path.to_absolute_path(temp).to_str().unwrap().to_string())
                        }).unwrap()
            ).unwrap();

            globals.set("__create_symlink",
                        scope.create_function(|_, (original, link): (PathType, PathType)| {
                            functions::create_symlink(inverses, &original, &link, temp)?;
                            Ok(())
                        }).unwrap()
            ).unwrap();

            globals.set("__set_attributes",
                        scope.create_function(|_, (path, attr): (PathType, u32)| -> rlua::Result<()> {
                            functions::set_attributes(inverses, &path, attr, temp)?;
                            Ok(())
                        }).unwrap()
            ).unwrap();







            match ctx.load(code.as_str()).exec() {
                Ok(_) => {Ok(())}
                Err(e) => {

                    if let rlua::Error::CallbackError { traceback, cause } = &e {
                        println!("Callback error: {} \n\n{}", traceback, cause.to_string());
                    } else {
                        println!("Other error: {}", e);
                    }

                    println!("Problematic code: \n{}", code);


                    Err(e.clone())
                }
            }


        })
    })

    //Delete the file and io functions

    //Add the other oak functions

        //Add our owm oan functions
        // - oak.zip
        // - oak.unzip
        // - oak.delete_registry_entry
        // - oak.write_registry_value (one for each value type)
        // - oak.write_registry_key
        // - oak.download
        // - oak.edit



    /*lua.context(|ctx| {
        let f= ctx.create_function(|_, path: PathType| {
            crate::functions::delete(uninstall, inverses, &path, temp);
            Ok(())
        }).unwrap();

        ctx.globals().set("__test", f).unwrap();
    });*/

}

impl<'l> FromLua<'l> for PathType {
    fn from_lua(lua_value: Value<'l>, lua: Context<'l>) -> rlua::Result<Self> {
        let table = Table::from_lua(lua_value, lua)?;

        let ident: String = table.get("ident")?;

        let path: String = table.get("path")?;

        match ident.as_str() {
            "s" => {
                let special: String = table.get("special")?;

                Ok(PathType::Special(PathBuf::from(special), PathBuf::from(path)))
            }
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
    fn from_lua(lua_value: Value<'lua>, _lua: Context<'lua>) -> rlua::Result<Self> {
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
                            "qword" => {
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

                        let is_multiline = move || -> rlua::Result<_> {
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

                                    *(multi.get_mut(ind - 1).ok_or(rlua::Error::FromLuaConversionError {
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

impl<'lua> ToLua<'lua> for Data {
    fn to_lua(self, lua: Context<'lua>) -> Result<Value<'lua>> {
        match self {
            Data::None => {Ok(Value::Nil)}
            Data::String(s) => {Ok(Value::String(lua.create_string(&s).unwrap()))}
            Data::ExpandString(s) => {
                let table = lua.create_table().unwrap();

                table.set(Value::String(lua.create_string("value").unwrap()), Value::String(lua.create_string(&s).unwrap()))?;
                table.set(Value::String(lua.create_string("ident").unwrap()), Value::String(lua.create_string("expanded").unwrap()))?;

                Ok(Value::Table(table))
            }
            Data::Binary(b) => {Ok(Value::Table(lua.create_sequence_from(b)?))}
            Data::U32(n) => {Ok(Value::Integer(n.into()))}
            Data::U32BE(_) => {todo!()}
            Data::Link => {Ok(Value::Nil)}
            Data::MultiString(ms) => {

                Ok(Value::Table(lua.create_sequence_from(ms)?))
            }
            Data::ResourceList => {Ok(Value::Nil)}
            Data::FullResourceDescriptor => {Ok(Value::Nil)}
            Data::ResourceRequirementsList => {Ok(Value::Nil)}
            Data::U64(n) => {

                let table = lua.create_table().unwrap();

                table.set(Value::String(lua.create_string("value").unwrap()), Value::Integer(n as i64))?;
                table.set(Value::String(lua.create_string("ident").unwrap()), Value::String(lua.create_string("qword").unwrap()))?;

                Ok(Value::Table(table))
            }
        }
    }
}


impl From<Error> for LuaError {
    fn from(e: Error) -> Self {
        LuaError::ExternalError(Arc::new(e))

    }
}

impl<'lua> FromLua<'lua> for RootKey {
    fn from_lua(lua_value: Value<'lua>, lua: Context<'lua>) -> rlua::Result<Self> {
        let rk = String::from_lua(lua_value, lua)?;

        Ok(RootKey::from(rk.as_str()))
    }
}

pub fn data_to_code(data: &Data) -> String {
    match data {
        Data::None => {format!("null")}
        Data::String(s) => {format!("{:?}",s)}
        Data::ExpandString(s) => {format!("registry.expanded({:?})", s)}
        Data::U32(n) => {format!("{}", n)}
        Data::MultiString(m) => {
            let mut s = String::from("{");

            for line in m {
                s.push_str(format!("{:?}, ", line).as_str());
            }

            s.push_str("}");

            s
        }
        Data::U64(n) => {format!("registry.qword({:?})", n)}
        _ => {panic!("Invalid data")}
    }
}
