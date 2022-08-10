# Todo

- Create inverses for `WriteRegistryKey` and `DeleteRegistryEntry`
- Ensure each step executes exactly as expected, and the inverses are correct
- Add a more comprehensive Oak scripting language
  - A lexer to convert the script into a list of tokens
  - Take that list of tokens and convert into an AST
  - A Script that can perform optimisations, and recognise files that can be moved to a temporary directory
  - Create an interpreter that uses the language
    - Enable the interpreter to display the installation progress to the terminal/GUI
  - Add high level install and uninstall functions to oak script
  - Create python/gdscript style syntax
  - Function to add the uninstaller to a path, and add this to the registry
    - Function to check to see if already installed
- Convert specs list into a table with 3 columns, step name, action and inverse
- For all actions, swap out the main function with a wrapper, so that it can be replaced with a duimmy function.
  This means that at the flip of a variable, operations can be applied to a virtual machine, instead of 
  a real one while developing to protect the developer's computer.

