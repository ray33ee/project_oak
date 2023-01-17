# Todo

- Define a struct that contains all the information needed to create an installer.
  - Define a function to convert this struct into an installer
  - Use serde to serialise it from json or xml form

- Add flags to any commands with multiplicity (see NSIS commands)

- Add higher functions
  - install and uninstall functions to oak script (to install and uninstall 3rd party)
  - Add function that adds an uninstaller entry to HKEY_LOCAL_MACHINE\Software\Microsoft\Windows\CurrentVersion\Uninstall
  - Function to check to see if already installed
  - Add reg_append/remove_multi_key function that adds/removes a line from a multiline value
  - Add shortcut to desktop/start menu/taskbar, etc.

- Add a bunch of tests (use a tempdir for this, and create files filled with garbage)
  - Test the following:
    - More complex uninstallers
- Figure out how to add the installer to the .data portion of executables
  - Create two executables, differing only in .data size and compare

- Add some path-specific functions (the inverses remove the added path, instead of saving the original path value)
  - Dafuq does this mean?