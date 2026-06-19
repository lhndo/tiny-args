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

```
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
