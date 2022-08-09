# Todo

- Automatically add the uninstaller to a path, and add this to the registry
- Add more sophisticated steps (some of these steps may requre user defined inverses)
  - Registry  
  - Zip
  - Environment
- Ensure each step executes exactly as expected, and the inverses are correct
- Add a more comprehensive Oak scripting language
  - A lexer to convert the script into a list of tokens
  - Take that list of tokens and convert into an AST
  - Create an interpreter that uses the language
    - Enable the interpreter to display the installation progress to the terminal/GUI
- Convert specs list into a table with 3 columns, step name, action and inverse
- If no command line arguments are supplied, look for a file in the environment named 'installer',
  if it exists, run it as an installer. If this fails, look for an uninstaller
- For all actions, swap out the main function with a wrapper, so that it can be replaced with a duimmy function.
  This means that at the flip of a variable, operations can be applied to a virtual machine, instead of 
  a real one while developing to protect the developer's computer.
