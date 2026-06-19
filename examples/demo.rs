use std::process::ExitCode;
use tiny_args::*;

fn main() -> ExitCode {
    let mut ta = TinyArgs::new();

    // Help definitions:
    ta.define_program_name("demo");
    ta.define_description("A demo program for TinyArgs");
    ta.define_usage("[OPTIONS] [COMMAND] [ARGS]...");
    ta.define_example("--name=test some/path/  - Sets some values");

    // Global Options
    let name = ta.define_option_txt("name", None, "test", "A name of something");
    let context = ta.define_option_num("context", 'c', 4, "Context lines");
    let verbose = ta.define_option_bool("verbose", 'v', false, "Verbose mode");

    // Commands
    let list = ta.define_command("list", "List vargs");
    let version = ta.define_command("version", "Display version");

    // Command Options
    let list_all = ta
        .command(list)
        .define_option_bool("all", 'a', false, "List All");
    let list_max = ta
        .command(list)
        .define_option_num("max", 'm', 0, "Maximum items");

    ta.command(list).define_example("list --max=10");
    ta.command(list).define_example("list --all ");

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
