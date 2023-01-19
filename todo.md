# Todo

- Find out why `modify_attributes_test` test keeps failing

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

- Add some environment variable PATH-specific functions (the inverses remove the added PATH, instead of saving the original PATH value)
