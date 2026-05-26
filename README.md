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

```rust
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

```
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
