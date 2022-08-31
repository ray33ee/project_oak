/*

Contains functions to assist installation, but do not change target computer state (i.e. do not have inverses
and do not contribute to the uninstaller)

File
    - exists(String)
    - created/modified/accessed
    - tmppath: get the path of the temporary folder

Registry
    - load: Loads the subkeys and values of the specified registry entry into a lua table. Recursion optional
    - expand: expands environment variables

Misc
    - reboot
    - assert_admin: Makes sure the installer is run with admin privileges
    - get: Get some input from the user

*/