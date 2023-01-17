# Project Oak

An oak installer contains a set of atomic commands that each have their own inverse that is used to automatically create an uninstaller, and
also to undo changes made if an installer fails part way through.

## Inverses

The most important part of an oak script is that every operation that changes the state of the target machine has an inverse.
Each inverse is selected such that it leaves the target machine in the same state as it was before installation.

For example, if we create a file at path A, the inverse is to delete the file at path A. If we move a file from A to B, the inverse
is to move the file from B to A.

During installation each command that is executed saves its inverse in a list. The uninstaller then executes these inverses in reverse
order to complete the uninstallation.

Please note: All writes to the target machine during installation will be undone during uninstall. If you wish to have locations that are preserved
after uninstall, they should be located outside any installation directory and created by the application itself (not the installer)

## Temporary location

Some steps do not really need inverses. Say we move a file from A to B and rename this file (B) to C. The inverse would be a rename followed by a 
deletion. However, since we delete the file anyway, the rename step is reduntant. To remove redundancy, we allow some locations to be explicitly
identified as temporary. Since temporary locations will be deleted when the installation finishes they do not affect the target machine's state and therefor
do not need inverses.

## Lua functions

The majority of Lua functions work the same, but some are modified to conform to the Oak rules and some have been removed altogether

### Path Type

Since most modified functions can take either an absolute path or a temporary path, we expose helper functions `pathtype.absolute` and `pathtype.temp` that take a path as an argument.

### Deleted functions

Some functions do not conform or are made redundant when used with Oak. These include

- `os.tmpname` and `io.tmpfile` since we use our own temporary files
- `os.execute` since executables can execute code that modifies the target machine without adding to the uninstaller thus breaking the Oak rules

### Modified functions

Most other functions that modify the state of the target machine have Oak alternatives, such as

- `os.remove`
- `os.rename`

### Registry type

We support the following registry types:

- `REG_NONE` via the Lua nil
- `REG_DWORD` via Lua integer or number
- `REG_SZ` via Lua string
- `REG_MULTI_SZ` via an array of strings
- `REG_QWORD` via helper function `registry.qword` which takes a number or integer as a u64
- `REG_EXPAND_SZ` via helper function `registry.expanded` which takes a string


