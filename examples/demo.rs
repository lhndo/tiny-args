use std::process::ExitCode;
use tiny_args::*;

fn main() -> ExitCode {
    let mut args = TinyArgs::new();

    // Optional help definitions:
    args.define_help_program_name("demo_program");
    args.define_help_description("A demo for TinyArgs");
    args.define_help_usage("[OPTION] [PATHS]...");
    args.define_help_example("--name=test some/path/  - Sets some values");

    let name = args.define_arg_txt("name", "", "test", "A name of something");
    let times = args.define_arg_num("times", "t", 22, "How many times");
    let version = args.define_arg_bool("version", "v", false, "Display version number");

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
