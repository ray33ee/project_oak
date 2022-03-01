# Project Oak

## Layout

An installation is described by a set of commands. These commands are executed out of order and concurrently.
A command can be a single step, or a list of steps, executed in order. Each step has an inverse. Inverses can be
user defined or default. Each command also has an inverse, that is either user defined, infered, or implemented as the inverses
of the list of steps in the reverse order.

## Script

The script is used to create an installer. The uninstaller is created when the installer finishes correctly.
If the installer fails at any point, any for commands that finished or are being executed, their inverses are called.

## Step

Each step has an action, to be executed during an install, and an inverse, used to undo the action during an uninstall.
Here is a list of the steps, and their actions and inverses:

- File
    - Action: Move the installation file or folder (recursively) to destination
    - Inverse: Delete the moved file or folder (Delete)
- Move
    - Action: Move the file or folder (recursively) from source to destination
    - Inverse: Move the file or folder (recursively) from destination to source (Move)
- Delete
    - Action: Remove the file or folder and store a copy for inverse
    - Inverse: Restore the file or folder (File)
- Create
    - Action: Create a file or folder or shortcut
    - Inverse: Delete file or folder (Delete)
- Copy 
    - Action: Copy file or folder (recursively) from source to destination
    - Inverse: Delete destination file or folder (Delete)
- Rename
    - Action: Rename a file or folder (store original file name for inverse)
    - Inverse: Rename to the original stored name (Rename)
- Environment
    - Action: Create or modify environment variables (store original variable value for inverse)
    - Inverse: Delete or restore variable (Environment)
- Regedit
    - Action: Add or modify registry entries (store original of modified entry for inverse)
    - Inverse: Remove registry entry (Regedit)
- Download
    - Action: Download a file from a website
    - Inverse: Delete downloaded file (Delete)
- Execute
    - Action: Run an external executable
    - Inverse: Run an external executable (preferebly uninstaller, Automatically deduce uninstaller path via registry ) (Execute)
- Edit
    - Action: Edit a text file (via line number, regex match, etc.) (store copy of file before modification for inverse)
    - Inverse: Replace the modified file with a copy of the original (File)
- If 
    - Action: If statement with else
    - Inverse: Keep track of which path is taken, then use that inverse in the uninstaller
- Print
    - Action: Print a message to the console
    - Inverse: None
- Input
    - Action: Get user input and store in variable
    - Inverse: None
- String
    - Action: Bunch of string handling functions, comparisons, searches, replacing, etc.
    - Inverse: None
    
## This repo

This repo does not attempt to create Windows installers or uninstallers, it merely serves as a proof of concept for the action/inverse and step/command systems

This repo is an interpretor for the Oak language. In 'install' mode, all actions are executed, and an uninstaller Oak file is created. 

If any step fails, all other executed steps will be reversed correctly, as outlined in 'Uninstall'

## Uninstall/Inverse

Inverses are used

- in the uninstaller
- if the installer fails

Uninstaller inverses are quite simple, the inverses of each command are used. 
Inverses of a command are calculated as the inverse of each step, in the reverse order.

If the installer fails, the inverse of each fully-completed command are executed. 
For any partially completed commands, the inverse of the set of already completed steps are called, in reverse order.

## Interpretor subcommands

- 'install'
- 'uninstall'
- 'list' - List all the commands and steps in an Oak file

## Oak format

Oak files are zip archives containing the command list (as a serialised Vec<(Command, Option<Command>)>) and a set of files used in installation/uninstallation