use std::path::Path;
use std::result::Result;
use crate::{OakRead, OakWrite, PathType};

use std::cell::UnsafeCell;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use hlua::Lua;
use crate::path_type::Inverse;


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



    {}", code);

    lua.execute::<()>(code.as_str()).unwrap();

    Ok(())
}

