# Todo

- Add flags to any commands with multiplicity (see NSIS commands)
- Implement create instruction and create shortcut
- Add a SHIT ton of tests (use a tempdir for this, and create files filled with garbage)
  - Test the following:
    - More complex iunstallers
    - Convert code to string and back again, compare code or strings to test compiler
- Convert specs list into a table with 3 columns, step name, action and inverse
- Add a more comprehensive Oak scripting language
  - A Script that can perform optimisations, and recognise files that can be moved to a temporary directory
  - Add some functions that help with displaying installation progress
  - Add high level install and uninstall functions to oak script (to install and uninstall 3rd party)
  - Function to add the uninstaller to a path, and add this to the registry
    - Function to check to see if already installed
- Add some path-specific functions (the inverses remove the added path, instead of saving the original path value)
- Add reg_append/remove_multi_key function that adds/removes a line from a multiline value 
- Add function that adds an uninstaller entry to HKEY_LOCAL_MACHINE\Software\Microsoft\Windows\CurrentVersion\Uninstall
- Add special windows variables (%appdata%, %programfiles%, etc.)
- Add extra functions