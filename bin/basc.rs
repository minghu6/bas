use std::path::PathBuf;

use clap::{IntoApp, Parser, Subcommand};
use clap_complete::Shell;

use bas::shell::gen_completions;
use bas::driver::RunCompiler;

/// Bas Lang Compiler
#[derive(Parser)]
#[clap()]
struct Cli {
    /// Genrerate completion for bin
    #[clap(long = "generate", arg_enum)]
    generator: Option<Shell>,

    #[clap(subcommand)]
    command: Option<SubCommand>,

    src: PathBuf

}

#[derive(Subcommand)]
enum SubCommand {
}

// fn format_u32_str(s: &str) -> Result<u32, String> {
//     let s = s.replace("_", "");
//     u32::from_str_radix(&s, 10).or(Err(s))
// }

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if let Some(generator) = cli.generator {
        let mut cmd = Cli::command();
        gen_completions(generator, &mut cmd);
        return Ok(());
    }

    if let Some(command) = cli.command {
        match command {
        }
    }

    RunCompiler::new(&cli.src)?;

    Ok(())
}
