use clap::{App, AppSettings, Arg, Shell, SubCommand};
use std::str::FromStr;

pub fn get_cli<'a, 'b>() -> App<'a, 'b> {
    App::new("backup-rat")
        .version("0.6.0")
        .author("System.rat <system.rodent@gmail.com>")
        .about("A versatile backup program")
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("backup")
                .about("backup operations")
                .alias("bu")
                .arg(
                    Arg::with_name("TARGET")
                        .help("The target to backup (if excluded backs up all non-optional targets)")
                        .index(1),
                )
        )
        .subcommand(
            SubCommand::with_name("restore")
                .about("restoring operations")
                .arg(
                    Arg::with_name("TARGET")
                        .help("The target to restore")
                        .index(1)
                        .required(true)
                )
        )
        .subcommand(
            SubCommand::with_name("completion")
                .about("generate shell completions")
                .arg(
                    Arg::with_name("SHELL")
                        .help("The shell to generate for. Available shells are: bash, zsh, fish, powershell, elvish")
                        .required(true)
                        .index(1)
                )
        )
}

pub fn print_completions(shell: String) {
    get_cli().gen_completions_to(
        "backup-rat",
        Shell::from_str(shell.as_ref()).unwrap_or_else(|_| {
            eprintln!("Unknown shell!");
            std::process::exit(1);
        }),
        &mut std::io::stdout(),
    );
}
