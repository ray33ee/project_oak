# Todo

- Find out why `modify_attributes_test` test keeps failing

- Redo the entire install, uninstall, create system such
  - Must use the `Source` struct to create an installer
  - Must work with tests
  - Must be able to add archives to executables for installers and uninstallers
  - Create a struct that reads and writes structs from exe?

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

- To combine the executable and installer, we simply concatenate the archive to the end of the executable.
  - To create an installer executable, we take the oak executable itself and append the archive, then we append a 4 byte 
    address that contains the location of the archive.
  - When we execute an installer, we look to the lasxt 4 bytes of the installer, which contains the offset. We use this to
    obtain the archive.
  - NOTE: When we create an empty oak exe, we must append 4 bytes of zeros, this tells the exe that it is not an installer or uninstaller
- If the oak exe is called without a subcommand the oak exe first checks the last 4 bytes. If it is a zero, we are a 


- Add some path-specific functions (the inverses remove the added path, instead of saving the original path value)
  - Dafuq does this mean?