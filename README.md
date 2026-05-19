A tiny command line argument parser with automatic help generation, and argument validation.

- Internally, the arguments are stored inside a Value enum containing three simple types:
  bool, String, and f64 representing booleans, numbers and text
- Arguments not matching `-` or `--` prefixes are stored inside a Vec "bucket" of VARGS
- Argument with values are set like: `--arg=value`
- Help sections such as description, usage, and examples can be re-defined from default using the
  provided functions: `define_help_...()`
- The help argument call is hard coded

## Example

```rust
use tiny_args::*;
use std::process::ExitCode;


fn main() -> ExitCode {
    let mut args = TinyArgs::new();

    // Optional help definitions:
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

```
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
