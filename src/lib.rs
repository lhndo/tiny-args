/*!
 A tiny command line argument parser with automatic help generation, and argument validation.

 - Internally, the arguments are stored inside a Value enum containing three simple types:
   bool, String, and f64 representing booleans, numbers and text.
 - Arguments without the `-` or `--` prefixes are stored inside a Vec "bucket" of VARGS in the given order. This list can be accessed using the `get_vargs()` function.
 - Arguments with values are set like: `--arg=value`.
 - Help sections such as description, usage, and examples can be redefined if needed using the
   provided functions: `define_help_...()`.
 - The help call is hard coded.


 ## Example
 ```
 use tiny_args::*;
 use std::process::ExitCode;

 fn main() -> ExitCode {
     let mut args = TinyArgs::new();

     // Optional definitions:
     args.define_help_program_name("demo_program");
     args.define_help_description("A demo for TinyArgs");
     args.define_help_usage("[OPTION] [PATHS]...");
     args.define_help_example("--name=test some/path/  - Sets some values");

     let name = args.define_arg_txt("name", "", "test", "A name of something");
     let times = args.define_arg_num("times", "t", 22, "How many times");
     let version = args.define_arg_bool("version", "v", false, "Display version");

     if let Err(e) = args.parse_arguments() {
         eprintln!("Error: {e}");
         return ExitCode::FAILURE;
     }

     if args.get(version) {
         println!("Version: 1.2.3.4");
     }

     println!("name: {}", args.get(name));
     println!("times: {}", args.get(times));

     println!("Paths:");
     for arg in args.get_vargs() {
         println!("{arg}");
     }

     ExitCode::SUCCESS
 }
 ```
## Generated Help

```none
>demo_program --help

Help:

  Usage: demo_program [OPTION] [PATHS]...

  Options:

    -h, --help                   Display this help message
        --name=<name>            A name of something [Default: test]
    -t, --times=<times>          How many times [Default: 22]
    -v, --version                Display version number


Examples:

  demo_program --name=test some/path/  - Sets some values
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
    UnknownArg(String),
    Parse(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ParseValue { value, arg } => {
                write!(f, "Cannot parse value: {} for argument: {}", value, arg)
            }
            Error::UnknownArg(s) => write!(f, "Unknown argument: {}", s),
            Error::Parse(s) => f.write_str(s),
        }
    }
}

impl std::error::Error for Error {}

/// Possible argument values
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Bool(bool),
    Txt(String),
    Num(f64),
}

impl Value {
    /// Parse str as num Val
    pub fn parse_as_num(input_val: &str) -> Result<Self, ParseFloatError> {
        let num = input_val.parse::<f64>()?;
        Ok(Value::Num(num))
    }

    /// Parse str as bool Val
    pub fn parse_as_bool(input_val: &str) -> Result<Self, ParseBoolError> {
        let b = input_val.parse::<bool>()?;
        Ok(Value::Bool(b))
    }
}

pub trait FromValue: Sized {
    fn from_value(v: &Value) -> Option<Self>;
}

impl FromValue for bool {
    fn from_value(v: &Value) -> Option<Self> {
        if let Value::Bool(b) = v {
            Some(*b)
        } else {
            None
        }
    }
}

impl FromValue for f64 {
    fn from_value(v: &Value) -> Option<Self> {
        if let Value::Num(n) = v {
            Some(*n)
        } else {
            None
        }
    }
}

impl FromValue for String {
    fn from_value(v: &Value) -> Option<Self> {
        if let Value::Txt(s) = v {
            Some(s.clone())
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Argument {
    pub name: &'static str,
    pub short_name: &'static str,
    pub description: &'static str,
    pub default: Value,
    pub value: Value,
    pub was_set: bool,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct TinyArgs {
    pub program_name: String,
    pub description: String,
    pub help: String,
    pub usage: String,
    pub examples: Vec<String>,
    pub args: HashMap<String, Argument>,
    pub vargs: Vec<String>,
}

impl TinyArgs {
    /// Create a TinyArgs instance
    #[must_use]
    pub fn new() -> Self {
        let mut ta = Self {
            program_name: String::new(),
            description: String::new(),
            help: String::new(),
            usage: String::new(),
            examples: vec![],
            args: HashMap::new(),
            vargs: vec![],
        };

        ta.usage = "[OPTIONS] [VARGS]...".to_owned();

        let _ = ta.define_arg_bool("help", "h", false, "Display this help message");
        ta
    }

    /// Define the program name displayed in the help section
    /// If not defined, the program name is derived automatically from the command line
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

    /// Define a boolean argument
    #[must_use]
    pub fn define_arg_bool(
        &mut self,
        name: &'static str,
        short_name: &'static str,
        default_value: bool,
        description: &'static str,
    ) -> ArgHandle<bool> {
        self.define_arg(name, short_name, Value::Bool(default_value), description);

        ArgHandle {
            name,
            _p: PhantomData::<bool>,
        }
    }

    /// Define a numerical argument
    #[must_use]
    pub fn define_arg_num(
        &mut self,
        name: &'static str,
        short_name: &'static str,
        default_value: impl Into<f64>,
        description: &'static str,
    ) -> ArgHandle<f64> {
        self.define_arg(
            name,
            short_name,
            Value::Num(default_value.into()),
            description,
        );

        ArgHandle {
            name,
            _p: PhantomData::<f64>,
        }
    }

    /// Define a textual argument
    #[must_use]
    pub fn define_arg_txt(
        &mut self,
        name: &'static str,
        short_name: &'static str,
        default_value: &str,
        description: &'static str,
    ) -> ArgHandle<String> {
        self.define_arg(
            name,
            short_name,
            Value::Txt(default_value.into()),
            description,
        );

        ArgHandle {
            name,
            _p: PhantomData::<String>,
        }
    }

    fn define_arg(
        &mut self,
        name: &'static str,
        short_name: &'static str,
        default_value: Value,
        description: &'static str,
    ) {
        let arg = Argument {
            name,
            short_name,
            description,
            value: default_value.clone(),
            default: default_value,
            was_set: false,
        };
        self.args.insert(name.to_owned(), arg);
    }

    /// Gets the argument value from the stored handle
    /// T is known at compile time from the handle
    #[must_use]
    pub fn get<T: FromValue>(&self, arg_handle: ArgHandle<T>) -> T {
        let val = self.get_val(arg_handle.name);

        T::from_value(val).unwrap_or_else(|| {
            panic!(
                "type mismatch for argument {} when converting from {:?} to {}",
                arg_handle.name,
                val,
                type_name::<T>()
            )
        })
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
        let mut vargs: Vec<String> = vec![];
        let mut args_iter = std::env::args().peekable();

        let input_name = args_iter.next().ok_or_else(|| {
            Error::Parse("Failed parsing starting argument (executable path)".to_owned())
        })?;

        if self.program_name.is_empty() {
            self.program_name = input_name
        }

        for input in args_iter {
            // Trimming - prefixes
            let trimmed_input = input.trim_start_matches('-').to_owned();
            if trimmed_input.is_empty() {
                return Err(Error::Parse("Invalid argument starting with -".to_owned()));
            }

            // Argument was not prefixed with - or --
            // We add it to the VARGS bucket
            if trimmed_input == input {
                vargs.push(input);
                continue; // We continue to next arg
            }

            let mut input_arg = trimmed_input;
            let mut input_val = String::new();

            // Try Split arg=value
            //
            // If value is not present, value string stays empty
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

            // Help
            if input_arg == "help" || input_arg == "h" {
                self.print_help_and_exit(0);
            }

            // Find Arg
            let found_arg = self.args.iter_mut().find_map(|(_, a)| {
                if input_arg == a.name || input_arg == a.short_name {
                    Some(a)
                } else {
                    None
                }
            });

            if let Some(argument) = found_arg {
                argument.was_set = true;

                // Boolean arguments/flags can be set without an explicit value
                if input_val.is_empty() {
                    if matches!(argument.value, Value::Bool(_)) {
                        argument.value = Value::Bool(true)
                    }
                }
                // Argument with explicit value assignment arg=val
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
                return Err(Error::UnknownArg(input_arg));
            }
        }

        self.vargs = vargs;
        Ok(())
    }

    fn get_arg(&self, name: &str) -> &Argument {
        self.args
            .get(name)
            .unwrap_or_else(|| panic!("Could not find argument: {name}"))
    }

    fn get_val(&self, name: &str) -> &Value {
        &self.get_arg(name).value
    }

    /// Find if an argument was explicitly set by the user
    pub fn was_set<T>(&self, arg_handle: ArgHandle<T>) -> bool {
        self.get_arg(arg_handle.name).was_set
    }

    /// Retrieve the rest of input vargs
    pub fn get_vargs(&self) -> std::slice::Iter<'_, String> {
        self.vargs.iter()
    }

    fn generate_help(&mut self) {
        let examples = {
            let mut res = String::new();
            self.examples.iter().for_each(|s| {
                res.push_str(&format!("  {program} {s}\n", program = self.program_name))
            });

            if !res.is_empty() {
                res = "Examples:\n\n".to_owned() + &res;
            }

            res
        };

        self.help = format!(
            " in the given order
{description}

Help:

  Usage: {program} {usage}

  Options:

{arguments}

{examples}",
            description = self.description,
            program = self.program_name,
            usage = self.usage,
            arguments = self.generate_args_help_list(),
        );
    }

    fn generate_args_help_list(&self) -> String {
        let mut args_help = String::new();

        let mut keys: Vec<&String> = self.args.keys().collect();
        keys.sort();

        for arg in keys.iter().map(|&k| self.args.get(k).unwrap()) {
            let name = "--".to_owned() + arg.name;

            let short_name = {
                if !arg.short_name.is_empty() {
                    "-".to_owned() + arg.short_name + ", "
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
                "{space:2}{short_name:>6}{name_and_val:25}{desc} {default}\n",
                space = "",
                name_and_val = name + &value,
                desc = arg.description
            );

            args_help.push_str(line);
        }

        args_help
    }

    /// Get help as str
    pub fn get_help_txt(&mut self) -> &str {
        if self.help.is_empty() {
            self.generate_help();
        }

        &self.help
    }

    /// Print the program help
    pub fn print_help(&mut self) {
        println!("{}", self.get_help_txt());
    }

    /// Print the program help and exit program with code
    pub fn print_help_and_exit(&mut self, exit_code: i32) {
        println!("{}", self.get_help_txt());
        std::process::exit(exit_code);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArgHandle<T> {
    pub name: &'static str,
    _p: PhantomData<T>,
}
