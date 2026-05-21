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

    let name = args.define_option_txt("name", "", "test", "A name of something");
    let context = args.define_option_num("context", "c", 4, "Context lines");
    let verbose = args.define_option_bool("verbose", "v", false, "Verbose mode");

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
