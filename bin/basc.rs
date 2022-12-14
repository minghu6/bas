use std::path::PathBuf;

use bas::driver::RunCompiler;
use bas::shell::gen_completions;
use clap::{IntoApp, Parser};
use clap_complete::Shell;
use inkwellkit::config::*;


/// Bas Lang Compiler
#[derive(Parser)]
#[clap()]
struct Cli {
    /// Genrerate completion for bin
    #[clap(long = "generate", arg_enum)]
    generator: Option<Shell>,

    // #[clap(subcommand)]
    // command: Option<SubCommand>,
    #[clap(short = 'O', arg_enum)]
    opt: Option<OptLv>,

    #[clap(short = 't', long = "target_type", arg_enum, default_value_t = TargetType::default())]
    target_type: TargetType,

    #[clap(short = 'e', long = "emit_type", arg_enum, default_value_t = EmitType::default())]
    emit_type: EmitType,

    src: PathBuf,

    output: PathBuf,
}

// #[derive(Subcommand)]
// enum SubCommand {
// }


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if let Some(generator) = cli.generator {
        let mut cmd = Cli::command();
        gen_completions(generator, &mut cmd);
        return Ok(());
    }

    let optlv = cli.opt.unwrap_or(OptLv::Debug);
    let target_type = cli.target_type;
    let emit_type = cli.emit_type;
    let print_type = if cli.output == PathBuf::from("stderr") {
        PrintTy::StdErr
    } else {
        PrintTy::File(cli.output)
    };

    let config = CompilerConfig {
        optlv,
        target_type,
        emit_type,
        print_type,
    };

    RunCompiler::new(&cli.src, config)?;

    Ok(())
}
