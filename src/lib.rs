/*!
A tiny command line argument parser with automatic help generation, and argument validation.

- Inputs are categorized as `commands`, `options`, and `va args`.
- Options are defined with the `-` or `--` prefixes.
- These can hold values representing booleans, numbers and text values, (stored internally as bool, f64 and Strings).
- Boolean short name options can be set as groups. E.g.: `-abc`
- Arguments with values are strictly defined by using the equal sign `=`, i.e. `--arg=value`.
- Commands and va args have no dash prefix. First argument of such kind is stored as the command, and the rest into a va_args "bucket", which can be retrived with `get_va_args()`.
- Help sections such as description, usage, and examples can be redefined if needed using the
  provided functions: `define_help_...()`.
- The help call (-h --help) is hard coded during argument parsing.


 ## Example
```
use std::process::ExitCode;
use tiny_args::*;

fn main() -> ExitCode {
    let mut args = TinyArgs::new();

    // Optional help definitions:
    args.define_help_program_name("demo");
    args.define_help_description("A demo program for TinyArgs");
    args.define_help_usage("[OPTIONS] [COMMAND] [ARGS]...");
    args.define_help_example("--name=test some/path/  - Sets some values");

    let list = args.define_command("list", "List vargs");
    let version = args.define_command("version", "Display version");

    let name = args.define_option_txt("name", None, "test", "A name of something");
    let context = args.define_option_num("context", 'c', 4, "Context lines");
    let verbose = args.define_option_bool("verbose", 'v', false, "Verbose mode");

    if let Err(e) = args.parse_arguments() {
        eprintln!("Error: {e}");
        return ExitCode::FAILURE;
    }

    println!("name: {}", args.get_option(name));
    println!("context: {}", args.get_option(context));
    println!("verbose: {}", args.get_option(verbose));

    if args.command() == version {
        println!("Version: 1.2.3.4");
    }

    if args.command() == list {
        for arg in args.get_va_args() {
            println!("{arg}");
        }
    }

    ExitCode::SUCCESS
  }
 ```

## Generated Help

```none
>demo_program --help

A demo program for TinyArgs

Help:

  Usage: demo [OPTIONS] [COMMAND] [ARGS]...

  Commands:

      list                     List args
      version                  Display version

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

#[derive(Clone, Debug)]
pub enum Error {
    ParseValue { value: String, arg: String },
    UnknownOpt(String),
    UnknownCmd(String),
    Parse(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ParseValue { value, arg } => {
                write!(f, "Cannot parse value: {} for argument: {}", value, arg)
            }
            Error::UnknownOpt(s) => write!(f, "Unknown option: {}", s),
            Error::UnknownCmd(s) => write!(f, "Unknown command: {}", s),
            Error::Parse(s) => f.write_str(s),
        }
    }
}

impl std::error::Error for Error {}

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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct OptHandle<T> {
    name: &'static str,
    _p: PhantomData<T>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct CmdHandle {
    name: &'static str,
}

impl CmdHandle {
    const NONE: Self = CmdHandle { name: "" };
}

#[derive(Debug, Clone, PartialEq)]
pub struct Argument {
    pub name: &'static str,
    pub short_name: Option<char>,
    pub description: &'static str,
    pub default: Value,
    pub value: Value,
    pub was_set: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Command {
    pub name: &'static str,
    pub description: &'static str,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct TinyArgs {
    pub program_name: String,
    pub description: String,
    pub help: String,
    pub usage: String,
    pub examples: Vec<String>,
    pub cmds: HashMap<String, Command>,
    pub opts: HashMap<String, Argument>,
    pub va_args: Vec<String>,
    pub active_cmd: Option<Command>,
}

impl TinyArgs {
    /// Create a TinyArgs instance
    #[must_use]
    pub fn new() -> Self {
        let mut res = Self {
            program_name: String::new(),
            description: String::new(),
            help: String::new(),
            usage: String::new(),
            examples: vec![],
            cmds: HashMap::new(),
            opts: HashMap::new(),
            va_args: vec![],
            active_cmd: None,
        };

        let _ = res.define_option_bool("help", 'h', false, "Display this help message");
        res
    }

    /// Define the program name displayed in the help section
    /// If not defined, the program name is automatically derived from the command line
    pub fn define_help_program_name(&mut self, name: &str) {
        self.program_name = name.to_owned();
    }

    /// Define the program description for the help section
    pub fn define_help_description(&mut self, description: &str) {
        self.description = description.into();
    }

    /// Define program usage for the help section
    /// The program name gets automatically prefixed,
    pub fn define_help_usage(&mut self, usage: &str) {
        self.usage = usage.into();
    }

    /// Define examples in for the help section
    /// You can this function multiple times to add more execution examples
    /// The program name gets automatically prefixed
    pub fn define_help_example(&mut self, examples: &str) {
        self.examples.push(examples.to_string());
    }

    /// Define a command
    #[must_use]
    pub fn define_command(&mut self, name: &'static str, description: &'static str) -> CmdHandle {
        let arg = Command { name, description };
        self.cmds.insert(name.to_owned(), arg);

        CmdHandle { name }
    }

    /// Define a boolean option
    #[must_use]
    pub fn define_option_bool(
        &mut self,
        name: &'static str,
        short_name: impl Into<Option<char>>,
        default_value: bool,
        description: &'static str,
    ) -> OptHandle<bool> {
        self.define_argument(name, short_name, Value::Bool(default_value), description);

        OptHandle {
            name,
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
    ) -> OptHandle<f64> {
        self.define_argument(
            name,
            short_name,
            Value::Num(default_value.into()),
            description,
        );

        OptHandle {
            name,
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
    ) -> OptHandle<String> {
        self.define_argument(
            name,
            short_name,
            Value::Txt(default_value.into()),
            description,
        );

        OptHandle {
            name,
            _p: PhantomData::<String>,
        }
    }

    /// Internal
    fn define_argument(
        &mut self,
        name: &'static str,
        short_name: impl Into<Option<char>>,
        default_value: Value,
        description: &'static str,
    ) {
        let arg = Argument {
            name,
            short_name: short_name.into(),
            description,
            value: default_value.clone(),
            default: default_value,
            was_set: false,
        };
        self.opts.insert(name.to_owned(), arg);
    }

    /// Get the option's value from the stored handle
    #[must_use]
    pub fn get_option<T: TryFrom<Value>>(&self, opt_handle: OptHandle<T>) -> T {
        let val = &self.find_argument(opt_handle.name).value;

        T::try_from(val.clone()).unwrap_or_else(|_| {
            panic!(
                "type mismatch for argument {} when converting from {:?} to {}",
                opt_handle.name,
                val,
                type_name::<T>()
            )
        })
    }

    /// Get the active command handle
    /// CmdHandle::NONE is returned if no command is set
    /// Example:
    ///  ```
    ///      if args.command() == version {
    ///          println!("Version: 1.2.3.4");
    ///      }
    ///  ```
    pub fn command(&self) -> CmdHandle {
        let name = self.active_cmd.as_ref().map_or_else(|| "", |c| c.name);

        if name.is_empty() {
            return CmdHandle::NONE;
        }

        CmdHandle { name }
    }

    /// This function MUST be run for the input arguments to be processed
    /// Automatically handles the help printout if "help" or "h" is encountered
    /// Call example:
    /// ```
    ///
    ///    if let Err(e) = args.parse_arguments() {
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
            Error::Parse("Failed parsing first argument (executable path)".to_owned())
        })?;

        // We derive the program name if none was defined by the user
        if self.program_name.is_empty() {
            let split: Vec<&str> = input_name.split(|c| "\\/".contains(c)).collect();

            self.program_name = split
                .last()
                .map_or("program_name".to_owned(), |s| s.to_string())
        }

        // Iter though the arguments
        for input in args_iter {
            let mut trimmed_input = input.to_owned();
            let mut prefix_dash_count = 0;

            // Trimming - or -- prefixes and counting the dashes
            for _ in 0..2 {
                if let Some(trimmed) = trimmed_input.strip_prefix('-') {
                    prefix_dash_count += 1;
                    trimmed_input = trimmed.to_owned();
                } else {
                    break;
                }
            }

            if trimmed_input.is_empty() {
                return Err(Error::Parse("Invalid argument prefixed by '-'".to_owned()));
            }

            // Parsing command or va_arg
            // No - or -- prefix
            if prefix_dash_count == 0 {
                // Argument was not prefixed with - or --
                if let Some(cmd) = self.cmds.get_mut(&trimmed_input)
                    && self.active_cmd.is_none()
                {
                    // No command was registered, and command is valid
                    self.active_cmd = Some(cmd.clone());
                    //
                } else if self.active_cmd.is_some() || self.cmds.is_empty() {
                    // Va args
                    self.va_args.push(trimmed_input); // We add it to the va args bucket
                } else {
                    // Commands are defined, this is the first command input, but we don't recognise this specific one
                    return Err(Error::UnknownCmd(trimmed_input));
                }
                continue; // We continue to next arg
            }

            let mut input_arg = trimmed_input;
            let mut input_val = String::new();

            // Try splitting arg=value into separate parts
            //
            // If value is not present, then the value string stays empty
            if let Some((left, right)) = input_arg.split_once('=') {
                if left.is_empty() {
                    return Err(Error::Parse(format!("Argument missing before ={}", right)));
                }

                if right.is_empty() {
                    return Err(Error::Parse(format!("Value missing after {}=", left)));
                }
                input_val = right.to_owned();
                input_arg = left.to_owned();
            }

            // We don't allow grouped short options with value assignments: e.g. -abc=10
            if prefix_dash_count == 1 && input_arg.chars().count() > 1 && !input_val.is_empty() {
                return Err(Error::Parse(format!(
                    "Grouped options cannot have assigned values: '-{input_arg}={input_val}'"
                )));
            }

            // We catch help option flags and display it immediately
            if input_arg == "help" || input_arg == "h" {
                self.print_help_and_exit(0);
            }

            // Grouped short option: -abc
            // We know that value is empty since we validated above
            if prefix_dash_count == 1 && input_arg.chars().count() > 1 {
                // We iterate though all characters part of the short name arg combo
                for short_name in input_arg.chars() {
                    // Auto help print
                    if short_name == 'h' {
                        self.print_help_and_exit(0);
                    }

                    let found_arg = self.opts.iter_mut().find_map(|(_, a)| {
                        if Some(short_name) == a.short_name {
                            Some(a)
                        } else {
                            None
                        }
                    });

                    // Verify if arg is defined
                    if let Some(argument) = found_arg {
                        argument.was_set = true;
                        // Only boolean options can be part of groups
                        if matches!(argument.value, Value::Bool(_)) {
                            argument.value = Value::Bool(true)
                        } else {
                            return Err(Error::Parse(format!(
                                "Only boolean type options can be part of grouped options: '-{input_arg}', option: '{short_name}', '{}'",
                                argument.name
                            )));
                        }
                    } else {
                        return Err(Error::UnknownOpt(short_name.into()));
                    }
                }

                continue;
            }

            // Find the argument against user registered ones
            let found_arg = self.opts.iter_mut().find_map(|(_, a)| {
                if (prefix_dash_count == 2 && input_arg == a.name)
                    || (prefix_dash_count == 1
                        && input_arg == a.short_name.unwrap_or(' ').to_string())
                {
                    Some(a)
                } else {
                    None
                }
            });

            if let Some(argument) = found_arg {
                argument.was_set = true;
                // Only boolean options/flags can be set without an explicit value
                if input_val.is_empty() {
                    if matches!(argument.value, Value::Bool(_)) {
                        argument.value = Value::Bool(true)
                    } else {
                        return Err(Error::Parse(format!(
                            "No value specified for option: '{}'",
                            argument.name
                        )));
                    }
                }
                // Options with explicit value assignment arg=val
                else {
                    argument.value = match argument.value {
                        Value::Txt(_) => Value::Txt(input_val),
                        Value::Num(_) => {
                            Value::parse_as_num(&input_val).map_err(|_| Error::ParseValue {
                                value: input_val,
                                arg: input_arg,
                            })?
                        }
                        Value::Bool(_) => {
                            Value::parse_as_bool(&input_val).map_err(|_| Error::ParseValue {
                                value: input_val,
                                arg: input_arg,
                            })?
                        }
                    }
                }
            } else {
                // Argument not defined - unknown
                return Err(Error::UnknownOpt(input_arg));
            }
        }

        Ok(())
    }

    /// Internal - Acts as get, should not fail
    fn find_argument(&self, name: &str) -> &Argument {
        self.opts
            .get(name)
            .unwrap_or_else(|| panic!("Could not find argument: {name}"))
    }

    /// Find if an argument was explicitly set by the user
    pub fn was_option_set<T>(&self, arg_handle: OptHandle<T>) -> bool {
        self.find_argument(arg_handle.name).was_set
    }

    /// Retrieve the rest of input va args
    pub fn get_va_args(&self) -> std::slice::Iter<'_, String> {
        self.va_args.iter()
    }

    fn generate_help(&mut self) {
        if self.usage.is_empty() {
            self.usage = {
                let mut options = "";
                let mut commands = "";

                if !self.opts.is_empty() {
                    options = "[OPTIONS] "
                };

                if !self.cmds.is_empty() {
                    commands = "[COMMANDS] "
                };

                format!("{}{}[ARGS]...", options, commands)
            }
        }

        let examples = {
            let mut res = String::new();

            if !self.examples.is_empty() {
                res = "\nExamples:\n\n".to_owned() + &res;
                self.examples.iter().for_each(|s| {
                    res.push_str(&format!("  {program} {s}\n", program = self.program_name))
                });
            }

            res
        };

        self.help = format!(
            "
{description}

Help:

  Usage: {program} {usage}
{commands} {arguments} {examples}
",
            description = self.description,
            program = self.program_name,
            usage = self.usage,
            commands = if !self.cmds.is_empty() {
                "\n  Commands:\n\n".to_string() + &self.generate_cmds_help_list()
            } else {
                "".to_string()
            },
            arguments = if !self.opts.is_empty() {
                "\n  Options:\n\n".to_string() + &self.generate_args_help_list()
            } else {
                "".to_string()
            },
        );
    }

    fn generate_args_help_list(&self) -> String {
        let mut args_help = String::new();

        let mut keys: Vec<&String> = self.opts.keys().collect();
        keys.sort();

        for arg in keys.iter().map(|&k| self.opts.get(k).unwrap()) {
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

        args_help
    }

    fn generate_cmds_help_list(&self) -> String {
        let mut cmds_help = String::new();

        let mut keys: Vec<&String> = self.cmds.keys().collect();
        keys.sort();

        for cmd in keys.iter().map(|&k| self.cmds.get(k).unwrap()) {
            let line = &format!(
                "{space:6}{name:25}{desc}\n",
                space = "",
                name = cmd.name,
                desc = cmd.description
            );

            cmds_help.push_str(line);
        }

        cmds_help
    }

    /// Get help as str
    pub fn get_help_text(&mut self) -> &str {
        if self.help.is_empty() {
            self.generate_help();
        }

        &self.help
    }

    /// Print the program help
    pub fn print_help(&mut self) {
        println!("{}", self.get_help_text());
    }

    /// Print the program help and exit program with code
    pub fn print_help_and_exit(&mut self, exit_code: i32) {
        println!("{}", self.get_help_text());
        std::process::exit(exit_code);
    }
}
