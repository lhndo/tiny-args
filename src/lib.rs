/*!
A tiny command line argument parser with automatic help generation, and argument validation.

- Inputs are categorized as `commands`, `options`, and `va args`.
- Options Flags are defined with the `-` or `--` prefixes.
- Options Flags can be both global, and command specific (see full example).
- These can hold values representing booleans, numbers and text values, (stored internally as bool, f64 and Strings).
- Short name option groups are supported for boolean type values. E.g.: `-abc`
- Arguments with values are strictly defined by using the equal sign `=`, i.e. `--arg=value`.
- Commands and va args have no dash prefix. First argument of such kind is stored as the command, and the rest into a va_args "bucket", which can be retrived with `get_va_args()`.
- Help sections such as description, usage, and examples can be redefined if needed using the
  provided functions: `define_...()`.
- The help call (-h --help) is hard coded during argument parsing.


 ## Minimal Example
```rust
use std::process::ExitCode;
use tiny_args::*;

fn main() -> ExitCode {
    let mut ta = TinyArgs::new();

    // Global Options
    let name = ta.define_option_txt("name", None, "test", "A name of something");
    let context = ta.define_option_num("context", 'c', 4, "Context lines");
    let verbose = ta.define_option_bool("verbose", 'v', false, "Verbose mode");

    // Commands
    let list = ta.define_command("list", "List items");
    let version = ta.define_command("version", "Display version");

    // Parse
    if let Err(e) = ta.parse_arguments() {
        eprintln!("Error: {e}");
        return ExitCode::FAILURE;
    }

    // Run
    println!("name:    {}", ta.get_option(name));
    println!("context: {}", ta.get_option(context));
    println!("verbose: {}", ta.get_option(verbose));

    if ta.get_active_command() == version {
        println!("Version: 1.2.3.4");
    }

    if ta.get_active_command() == list {
        for arg in ta.get_va_args() {
            print!("|{arg}");
        }
    }

    ExitCode::SUCCESS
}
 ```

### Minimal Generated Help

```none
>demo_minimal --help

Help

  Usage: demo_minimal [OPTIONS] [COMMANDS] [ARGS]...

  Commands:

      list                     List items
      version                  Display version

  For more information run: [COMMAND] --help

  Options:

    -c, --context=<context>    Context lines [Default: 4]
    -h, --help                 Display this help message
        --name=<name>          A name of something [Default: test]
    -v, --verbose              Verbose mode
```


## Full Example

```rust
use std::process::ExitCode;
use tiny_args::*;

fn main() -> ExitCode {
    let mut ta = TinyArgs::new();

    // Help definitions:
    ta.define_program_name("demo");
    ta.define_description("A demo program for tinyargs.");
    ta.define_usage("[OPTIONS] [COMMAND] [ARGS]...");
    ta.define_example("--name=test some/path/  - Sets some values");

    // Global Options
    let name = ta.define_option_txt("name", None, "test", "A name of something");
    let context = ta.define_option_num("context", 'c', 4, "Context lines");
    let verbose = ta.define_option_bool("verbose", 'v', false, "Verbose mode");

    // Commands
    let version = ta.define_command("version", "Display version");
    let list = ta.define_command("list", "List items");

    // Command Options
    let list_all = ta
        .command(list)
        .define_option_bool("user", 'u', false, "List user items");
    let list_max = ta
        .command(list)
        .define_option_num("max", 'm', 0, "Maximum items");

    // Command Examples
    ta.command(list)
        .define_example("list --max=10 - Lists max 10 items");
    ta.command(list)
        .define_example("list --user   - Lists user Items");

    // Parse
    if let Err(e) = ta.parse_arguments() {
        eprintln!("Error: {e}");
        return ExitCode::FAILURE;
    }

    // Run
    println!("name:    {}", ta.get_option(name));
    println!("context: {}", ta.get_option(context));
    println!("verbose: {}", ta.get_option(verbose));

    if ta.get_active_command() == version {
        println!("Version: 1.2.3.4");
    }

    if ta.get_active_command() == list {
        println!("List all: {}", ta.get_option(list_all));
        println!("List max: {}", ta.get_option(list_max));
        print!("Arguments: ");

        for arg in ta.get_va_args() {
            print!("|{arg}");
        }
    }

    ExitCode::SUCCESS
}
```

### Generated Help

```none
>demo_minimal --help

A demo program for tinyargs.

Help

  Usage: demo [OPTIONS] [COMMAND] [ARGS]...

  Commands:

      list                     List items
      version                  Display version

  For more information run: [COMMAND] --help

  Options:

    -c, --context=<context>    Context lines [Default: 4]
    -h, --help                 Display this help message
        --name=<name>          A name of something [Default: test]
    -v, --verbose              Verbose mode

Examples:

  demo --name=test some/path/  - Sets some values

```


*/

use std::any::type_name;
use std::collections::HashMap;
use std::fmt::Display;
use std::marker::PhantomData;
use std::num::ParseFloatError;
use std::str::ParseBoolError;

// ——————————————————————————————————————————————————————————————————————————————————————
//                                         Error
// ——————————————————————————————————————————————————————————————————————————————————————

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    ParseValue { value: String, arg: String },
    ParseArgument(String),
    UnknownOpt(String),
    UnknownCmd(String),
    Command(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ParseValue { value, arg } => {
                write!(f, "Cannot parse value: {} for argument: {}", value, arg)
            }
            Error::UnknownOpt(s) => write!(f, "Unknown option: {}", s),
            Error::UnknownCmd(s) => write!(f, "Unknown command: {}", s),
            Error::ParseArgument(s) => f.write_str(s),
            Error::Command(s) => f.write_str(s),
        }
    }
}

impl std::error::Error for Error {}

// ——————————————————————————————————————————————————————————————————————————————————————
//                                         Value
// ——————————————————————————————————————————————————————————————————————————————————————

/// Possible argument values
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Bool(bool),
    Num(f64),
    Txt(String),
}

impl Value {
    /// Parse str as bool Val
    pub fn parse_as_bool(input_val: &str) -> Result<Self, ParseBoolError> {
        let b = input_val.parse::<bool>()?;
        Ok(Value::Bool(b))
    }

    /// Parse str as num Val
    pub fn parse_as_num(input_val: &str) -> Result<Self, ParseFloatError> {
        let num = input_val.parse::<f64>()?;
        Ok(Value::Num(num))
    }
}

impl TryFrom<Value> for bool {
    type Error = &'static str;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Bool(v) = value {
            Ok(v)
        } else {
            Err("parsing bool")
        }
    }
}

impl TryFrom<Value> for f64 {
    type Error = &'static str;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Num(v) = value {
            Ok(v)
        } else {
            Err("parsing num")
        }
    }
}

impl TryFrom<Value> for String {
    type Error = &'static str;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Txt(v) = value {
            Ok(v)
        } else {
            Err("parsing string")
        }
    }
}

// ——————————————————————————————————————————————————————————————————————————————————————
//                                         Handles
// ——————————————————————————————————————————————————————————————————————————————————————

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct OptionHandle<T> {
    name: &'static str,
    cmd_root: CommandHandle,
    _p: PhantomData<T>,
}

impl<T> OptionHandle<T> {
    pub const HELP: OptionHandle<bool> = OptionHandle {
        name: "help",
        cmd_root: CommandHandle::NONE,
        _p: PhantomData::<bool>,
    };
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct CommandHandle {
    name: &'static str,
}

impl CommandHandle {
    const NONE: Self = CommandHandle { name: "" };
}

// ——————————————————————————————————————————————————————————————————————————————————————
//                                Options and Commands
// ——————————————————————————————————————————————————————————————————————————————————————

#[derive(Debug, Clone, PartialEq)]
pub struct OptionFlag {
    name: &'static str,
    short_name: Option<char>,
    description: &'static str,
    default: Value,
    value: Value,
    was_set: bool,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Command {
    name: &'static str,
    description: &'static str,
    usage: &'static str,
    examples: Vec<String>,
    options: HashMap<String, OptionFlag>,
}

impl Command {
    pub fn new() -> Self {
        Self {
            name: "",
            description: "",
            usage: "",
            examples: Vec::new(),
            options: HashMap::new(),
        }
    }

    /// Define the program description for the help section
    pub fn define_description(&mut self, description: &'static str) {
        self.description = description.into();
    }

    /// Define program usage for the help section
    /// The program name gets automatically prefixed,
    pub fn define_usage(&mut self, usage: &'static str) {
        self.usage = usage.into();
    }

    /// Define examples in for the help section
    /// You can this function multiple times to add more execution examples
    /// The program name gets automatically prefixed
    pub fn define_example(&mut self, examples: &str) {
        self.examples.push(examples.to_string());
    }

    /// Internal - Add an option to list
    fn add_option(
        &mut self,
        name: &'static str,
        short_name: impl Into<Option<char>>,
        default_value: Value,
        description: &'static str,
    ) {
        let sn: Option<char> = short_name.into();

        // Validating short name uniqueness
        if let Some(sn) = sn {
            let existing = self.options.iter().find_map(|e| {
                if e.1.short_name.as_ref() == Some(&sn) {
                    Some(true)
                } else {
                    None
                }
            });

            debug_assert!(
                existing.is_none(),
                "Error: Option short name '-{}' already taken!",
                sn
            );
        }

        let arg = OptionFlag {
            name,
            short_name: sn,
            description,
            value: default_value.clone(),
            default: default_value,
            was_set: false,
        };

        let res = self.options.insert(name.to_owned(), arg);

        debug_assert!(
            res.is_none(),
            "Error: Option name '--{}' already taken!",
            name
        );
    }

    /// Define a boolean option
    #[must_use]
    pub fn define_option_bool(
        &mut self,
        name: &'static str,
        short_name: impl Into<Option<char>>,
        default_value: bool,
        description: &'static str,
    ) -> OptionHandle<bool> {
        self.add_option(name, short_name, Value::Bool(default_value), description);

        OptionHandle {
            name,
            cmd_root: CommandHandle { name: self.name },
            _p: PhantomData::<bool>,
        }
    }

    /// Define a numerical option
    #[must_use]
    pub fn define_option_num(
        &mut self,
        name: &'static str,
        short_name: impl Into<Option<char>>,
        default_value: impl Into<f64>,
        description: &'static str,
    ) -> OptionHandle<f64> {
        self.add_option(
            name,
            short_name,
            Value::Num(default_value.into()),
            description,
        );

        OptionHandle {
            name,
            cmd_root: CommandHandle { name: self.name },
            _p: PhantomData::<f64>,
        }
    }

    /// Define a text option
    #[must_use]
    pub fn define_option_txt(
        &mut self,
        name: &'static str,
        short_name: impl Into<Option<char>>,
        default_value: &str,
        description: &'static str,
    ) -> OptionHandle<String> {
        self.add_option(
            name,
            short_name,
            Value::Txt(default_value.into()),
            description,
        );

        OptionHandle {
            name,
            cmd_root: CommandHandle { name: self.name },
            _p: PhantomData::<String>,
        }
    }
}

// ——————————————————————————————————————————————————————————————————————————————————————
//                                         Args
// ——————————————————————————————————————————————————————————————————————————————————————

#[derive(Debug, Default, Clone, PartialEq)]
pub struct TinyArgs {
    pub program_name: String,
    pub root: Command,
    pub commands: HashMap<String, Command>,
    pub va_args: Vec<String>,
    pub active_cmd: Option<CommandHandle>,
}

impl TinyArgs {
    //
    /// Create a TinyArgs instance
    #[must_use]
    pub fn new() -> Self {
        let mut res = Self {
            program_name: String::new(),
            root: Command::new(),
            commands: HashMap::new(),
            va_args: vec![],
            active_cmd: None,
        };

        let _ = res.define_option_bool("help", 'h', false, "Display this help message");
        res
    }

    /// Define the program name displayed in the help section
    /// If not defined, the program name is automatically derived from the command line
    pub fn define_program_name(&mut self, name: &str) {
        self.program_name = name.to_owned();
    }

    /// Define the program description for the help section
    pub fn define_description(&mut self, description: &'static str) {
        self.root.define_description(description);
    }

    /// Define program usage for the help section
    /// The program name gets automatically prefixed,
    pub fn define_usage(&mut self, usage: &'static str) {
        self.root.define_usage(usage);
    }

    /// Define examples in for the help section
    /// You can this function multiple times to add more execution examples
    /// The program name gets automatically prefixed
    pub fn define_example(&mut self, example: &str) {
        self.root.define_example(example);
    }

    /// Define a command
    #[must_use]
    pub fn define_command(
        &mut self,
        name: &'static str,
        description: &'static str,
    ) -> CommandHandle {
        {
            let arg = Command {
                name,
                description,
                usage: "",
                examples: Vec::new(),
                options: HashMap::new(),
            };
            let res = self.commands.insert(name.to_owned(), arg);
            debug_assert!(
                res.is_none(),
                "Error: Command name '{}' already taken!",
                name
            );

            CommandHandle { name }
        }
    }

    /// Define a boolean option
    #[must_use]
    pub fn define_option_bool(
        &mut self,
        name: &'static str,
        short_name: impl Into<Option<char>>,
        default_value: bool,
        description: &'static str,
    ) -> OptionHandle<bool> {
        self.root
            .define_option_bool(name, short_name, default_value, description)
    }

    /// Define a numerical option
    #[must_use]
    pub fn define_option_num(
        &mut self,
        name: &'static str,
        short_name: impl Into<Option<char>>,
        default_value: impl Into<f64>,
        description: &'static str,
    ) -> OptionHandle<f64> {
        self.root
            .define_option_num(name, short_name, default_value, description)
    }

    /// Define a text option
    #[must_use]
    pub fn define_option_txt(
        &mut self,
        name: &'static str,
        short_name: impl Into<Option<char>>,
        default_value: &str,
        description: &'static str,
    ) -> OptionHandle<String> {
        self.root
            .define_option_txt(name, short_name, default_value, description)
    }

    /// Get the option's value from the stored handle
    #[must_use]
    pub fn get_option<T: TryFrom<Value>>(&self, opt_handle: OptionHandle<T>) -> T {
        let name = opt_handle.name;
        let val = &self.find_option_by_handle(opt_handle).value;

        T::try_from(val.clone()).unwrap_or_else(|_| {
            panic!(
                "type mismatch for argument {} when converting from {:?} to {}",
                name,
                val,
                type_name::<T>()
            )
        })
    }

    /// Get the active command handle
    /// CmdHandle::NONE is returned if no command is set
    /// Example:
    ///  ```
    ///      if ta.get_active_command() == version {
    ///          println!("Version: 1.2.3.4");
    ///      }
    ///  ```
    pub fn get_active_command(&self) -> CommandHandle {
        if let Some(cmd) = self.active_cmd {
            cmd
        } else {
            return CommandHandle::NONE;
        }
    }

    pub fn command(&mut self, command: CommandHandle) -> &mut Command {
        self.find_command_by_handle_mut(command)
    }

    /// This function MUST be run for the input arguments to be processed
    /// Automatically handles the help printout if "help" or "h" is encountered
    /// Call example:
    /// ```
    ///
    ///    if let Err(e) = ta.parse_arguments() {
    ///        eprintln!("Error: {e}");
    ///        return ExitCode::FAILURE;
    ///    }
    ///
    /// ```
    pub fn parse_arguments(&mut self) -> Result<(), Error> {
        let args = std::env::args().collect();
        self.parse_arguments_from_vec(args)
    }

    /// Parse arguments from a provided vector of Strings
    pub fn parse_arguments_from_vec(&mut self, args: Vec<String>) -> Result<(), Error> {
        let mut args_iter = args.iter();

        let input_name = args_iter.next().ok_or_else(|| {
            Error::ParseArgument("Failed parsing first argument (executable path)".to_owned())
        })?;

        // ——————————————————————————————————————— Program Name ———————————————————————————————————————

        // We derive the program name if none was defined by the user
        if self.program_name.is_empty() {
            let split: Vec<&str> = input_name.split(|c| "\\/".contains(c)).collect();

            self.program_name = split
                .last()
                .map_or("program_name".to_owned(), |s| s.to_string())
        }

        // ——————————————————————————————————————— Iter Args ———————————————————————————————————————
        for arg in args_iter {
            let mut arg = arg.to_owned();
            let mut prefix_dash_count = 0;
            // Trimming - or -- prefixes and counting the dashes
            for _ in 0..2 {
                if let Some(trimmed) = arg.strip_prefix('-') {
                    prefix_dash_count += 1;
                    arg = trimmed.to_owned();
                } else {
                    break;
                }
            }

            if arg.is_empty() {
                return Err(Error::ParseArgument(
                    "Invalid argument prefixed by '-'".to_owned(),
                ));
            }

            // ——————————————————————————————————————— Commands ———————————————————————————————————————
            // Parsing args with no - or -- prefixes
            if prefix_dash_count == 0 {
                // Commands are defined
                if !self.commands.is_empty() {
                    // Known command
                    if let Some(cmd) = self.commands.get(&arg) {
                        if self.active_cmd.is_none() {
                            // Register active command
                            self.active_cmd = Some(CommandHandle { name: cmd.name });
                        } else {
                            return Err(Error::Command(format!(
                                "Command \"{}\" was called before command \"{}\"",
                                &self.active_cmd.as_ref().unwrap().name,
                                arg
                            )));
                        }
                        .to_owned()

                    //Unknown command
                    } else if self.active_cmd.is_none() {
                        // Commands are defined, this is the first command input, but we don't recognise this specific one
                        return Err(Error::UnknownCmd(arg));
                    } else {
                        // Active command preset, we push into the va args bucket
                        self.va_args.push(arg);
                    }
                } else {
                    // If no commands are defined, we push it straight to va args bucket
                    self.va_args.push(arg);
                }
                continue; // Continue to next arg
            }

            // ——————————————————————————————————————— Arg with value; '=' ———————————————————————————————————————

            let mut arg_value = String::new();

            if let Some((left, right)) = arg.split_once('=') {
                if left.is_empty() {
                    return Err(Error::ParseArgument(format!(
                        "Argument missing before ={}",
                        right
                    )));
                }

                if right.is_empty() {
                    return Err(Error::ParseArgument(format!(
                        "Value missing after {}=",
                        left
                    )));
                }
                arg_value = right.to_owned();
                arg = left.to_owned();
            }

            // ——————————————————————————————————————— Grouped Option Flags ———————————————————————————————————————

            if prefix_dash_count == 1 && arg.chars().count() > 1 {
                if !arg_value.is_empty() {
                    // We don't allow grouped short options with value assignments: e.g. -abc=10
                    return Err(Error::ParseArgument(format!(
                        "Grouped options cannot have assigned values: '-{arg}={arg_value}'"
                    )));
                }

                // We iterate though all characters part of the short name arg combo
                for short_name in arg.chars() {
                    // Search global options
                    let mut found_arg = self.root.options.iter_mut().find_map(|(_, a)| {
                        if Some(short_name) == a.short_name {
                            Some(a)
                        } else {
                            None
                        }
                    });

                    // Search active command options
                    if found_arg.is_none()
                        && let Some(cmd_handle) = self.active_cmd
                    {
                        let command = self.find_command_by_handle_mut(cmd_handle);

                        found_arg = command.options.iter_mut().find_map(|(_, a)| {
                            if Some(short_name) == a.short_name {
                                Some(a)
                            } else {
                                None
                            }
                        });
                    }

                    // Validate if arg is valid (bool)
                    if let Some(argument) = found_arg {
                        // Only boolean options can be part of groups
                        if matches!(argument.value, Value::Bool(_)) {
                            argument.value = Value::Bool(true);
                            argument.was_set = true;
                        } else {
                            return Err(Error::ParseArgument(format!(
                                "Only boolean type options can be part of grouped options: '-{arg}', option: '-{short_name}', '--{}'",
                                argument.name
                            )));
                        }
                    } else {
                        return Err(Error::UnknownOpt(format!(
                            "'{}', part of: '-{}'",
                            short_name, arg
                        )));
                    }
                }

                continue;
            }

            // ——————————————————————————————————————— Regular Option Flag ———————————————————————————————————————

            // Check if global option flag is defined
            let mut found_arg = self.root.options.iter_mut().find_map(|(_, a)| {
                if (prefix_dash_count == 2 && arg == a.name)
                    || (prefix_dash_count == 1 && arg == a.short_name.unwrap_or(' ').to_string())
                {
                    Some(a)
                } else {
                    None
                }
            });

            // Search active command options
            if found_arg.is_none()
                && let Some(cmd_handle) = self.active_cmd
            {
                let command = self.find_command_by_handle_mut(cmd_handle);

                found_arg = command.options.iter_mut().find_map(|(_, a)| {
                    if (prefix_dash_count == 2 && arg == a.name)
                        || (prefix_dash_count == 1
                            && arg == a.short_name.unwrap_or(' ').to_string())
                    {
                        Some(a)
                    } else {
                        None
                    }
                });
            }

            if let Some(argument) = found_arg {
                argument.was_set = true;
                // Only boolean options/flags can be set without an explicit value
                if arg_value.is_empty() {
                    if matches!(argument.value, Value::Bool(_)) {
                        argument.value = Value::Bool(true)
                    } else {
                        return Err(Error::ParseArgument(format!(
                            "No value specified for option: '{}'",
                            argument.name
                        )));
                    }
                }
                // Options with explicit value assignment arg=val
                else {
                    argument.value = match argument.value {
                        Value::Txt(_) => Value::Txt(arg_value),
                        Value::Num(_) => {
                            Value::parse_as_num(&arg_value).map_err(|_| Error::ParseValue {
                                value: arg_value,
                                arg,
                            })?
                        }
                        Value::Bool(_) => {
                            Value::parse_as_bool(&arg_value).map_err(|_| Error::ParseValue {
                                value: arg_value,
                                arg,
                            })?
                        }
                    }
                }
            } else {
                // Argument not defined - unknown
                return Err(Error::UnknownOpt(arg));
            }
        }

        // ——————————————————————————————————————— Help Trigger ———————————————————————————————————————

        // Check for help argument and print help
        if self.get_option(OptionHandle::<bool>::HELP) {
            if let Some(cmd) = &self.active_cmd {
                self.print_command_help_and_exit(cmd.name, 0);
            } else {
                self.print_help_and_exit(0);
            }
        }

        Ok(())
    }

    /// Internal - Acts as get, should not fail
    fn find_command_by_handle(&self, cmd_handle: CommandHandle) -> &Command {
        let name = cmd_handle.name;
        self.commands
            .get(name)
            .unwrap_or_else(|| panic!("Could not find command: {name}"))
    }

    /// Internal - Acts as get, should not fail
    fn find_command_by_handle_mut(&mut self, cmd_handle: CommandHandle) -> &mut Command {
        let name = cmd_handle.name;
        self.commands
            .get_mut(name)
            .unwrap_or_else(|| panic!("Could not find command: {name}"))
    }

    /// Internal - Acts as get, should not fail
    fn find_option_by_handle<T>(&self, opt_handle: OptionHandle<T>) -> &OptionFlag {
        let name = opt_handle.name;

        // Root option
        if opt_handle.cmd_root == CommandHandle::NONE {
            self.root
                .options
                .get(name)
                .unwrap_or_else(|| panic!("Could not find option: {name}"))
        } else {
            // Command option
            let command = self.find_command_by_handle(opt_handle.cmd_root);
            command
                .options
                .get(name)
                .unwrap_or_else(|| panic!("Could not find option: {name}"))
        }
    }

    /// Find if an argument was explicitly set by the user
    pub fn was_option_set<T>(&self, opt_handle: OptionHandle<T>) -> bool {
        self.find_option_by_handle(opt_handle).was_set
    }

    /// Retrieve the rest of input va args
    pub fn get_va_args(&self) -> std::slice::Iter<'_, String> {
        self.va_args.iter()
    }

    fn generate_help(&self) -> String {
        let description = &self.root.description;

        let usage = generate_help_usage_list(
            self.root.usage,
            &self.program_name,
            !self.root.options.is_empty(),
            !self.commands.is_empty(),
        );

        let examples = generate_help_example_list(&self.root.examples, &self.program_name);
        let commands = generate_help_command_list(&self.commands);
        let options = generate_help_option_list(&self.root.options, "Options");

        format!(
            "
{description}

Help

{usage}
{commands} {options} {examples}
",
        )
    }

    fn generate_command_help(&self, command_name: &str) -> String {
        let command = self
            .commands
            .get(command_name)
            .expect("Valid command definition");

        let command_name = command.name;
        let command_description = command.description;

        let pcn = format!("{} {}", self.program_name.clone(), &command.name);
        let usage =
            generate_help_usage_list(&command.usage, &pcn, !command.options.is_empty(), false);

        let examples = generate_help_example_list(&command.examples, &self.program_name);
        let command_options = generate_help_option_list(&command.options, "Command Options");
        let global_options = generate_help_option_list(&self.root.options, "Global Options");

        format!(
            "
Help

Command: 

  {command_name} - {command_description}


{usage}
{command_options} {global_options} {examples}
"
        )
    }

    /// Print the program help and exit program with code
    pub fn print_help_and_exit(&self, exit_code: i32) {
        println!("{}", self.generate_help());
        std::process::exit(exit_code);
    }

    /// Print the program help and exit program with code
    pub fn print_command_help_and_exit(&self, command_name: &str, exit_code: i32) {
        println!("{}", self.generate_command_help(command_name));
        std::process::exit(exit_code);
    }
}

// ——————————————————————————————————————————————————————————————————————————————————————
//                                    Free Functions
// ——————————————————————————————————————————————————————————————————————————————————————

/// Generate command list
fn generate_help_command_list(commands: &HashMap<String, Command>) -> String {
    let mut cmds_help = String::new();

    let mut keys: Vec<&String> = commands.keys().collect();
    keys.sort();

    for cmd in keys.iter().map(|&k| commands.get(k).unwrap()) {
        let line = &format!(
            "{space:6}{name:25}{desc}\n",
            space = "",
            name = cmd.name,
            desc = cmd.description
        );

        cmds_help.push_str(line);
    }

    if !cmds_help.is_empty() {
        cmds_help = "\n  Commands:\n\n".to_string()
            + &cmds_help
            + "\n  For more information run: [COMMAND] --help\n"
    };

    cmds_help
}

/// Generate options list
fn generate_help_option_list(options: &HashMap<String, OptionFlag>, title: &str) -> String {
    let mut args_help = String::new();

    let mut keys: Vec<&String> = options.keys().collect();
    keys.sort();

    for arg in keys.iter().map(|&k| options.get(k).unwrap()) {
        let name = "--".to_owned() + arg.name;

        let short_name = {
            if let Some(short_name) = arg.short_name {
                "-".to_owned() + &short_name.to_string() + ", "
            } else {
                "".to_string()
            }
        };

        let mut default = match &arg.default {
            Value::Bool(true) => "true".to_string(),
            Value::Txt(s) => {
                if s.is_empty() {
                    "".to_string()
                } else {
                    s.clone()
                }
            }
            Value::Num(n) => n.to_string(),
            _ => "".to_string(),
        };

        let value = {
            match arg.default {
                Value::Bool(_) => "".to_string(),
                _ => format!("=<{}>", arg.name),
            }
        };

        if !default.is_empty() {
            default = format!("[Default: {}]", default);
        }

        let line = &format!(
            "{space:2}{short_name:>6}{name_and_val:23}{desc} {default}\n",
            space = "",
            name_and_val = name + &value,
            desc = arg.description
        );

        args_help.push_str(line);
    }

    if !args_help.is_empty() {
        args_help = format!("\n  {title}:\n\n") + &args_help
    };

    args_help
}

/// Generate example list
fn generate_help_example_list(examples: &[String], program_name: &str) -> String {
    let mut res = String::new();

    if !examples.is_empty() {
        res = "\nExamples:\n\n".to_owned() + &res;
        examples
            .iter()
            .for_each(|ex| res.push_str(&format!("  {program_name} {ex}\n",)));
    }

    res
}

/// Generate usage list
fn generate_help_usage_list(
    usage: &str,
    program_name: &str,
    has_options: bool,
    has_commands: bool,
) -> String {
    if usage.is_empty() {
        let options = if has_options { "[OPTIONS] " } else { "" };
        let commands = if has_commands { "[COMMANDS] " } else { "" };
        format!("  Usage: {program_name} {}{}[ARGS]...", options, commands,)
    } else {
        format!("  Usage: {program_name} {usage}")
    }
}
