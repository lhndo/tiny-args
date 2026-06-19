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
