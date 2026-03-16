use gototranscoder::adapter::cbmc2esbmc;
use log::trace;

use clap::{Args, Parser, Subcommand};

fn init() {
    use env_logger::Env;
    let env = Env::default()
        .filter_or("LOG_LEVEL", "trace")
        .write_style_or("LOG_STYLE", "always");

    env_logger::init_from_env(env);
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Converts CBMC <INPUT> into ESBMC <OUTPUT>
    CBMC2ESBMC(CmdArgs),
}

#[derive(Args)]
struct CmdArgs {
    input: std::path::PathBuf,
    output: std::path::PathBuf,
}

fn main() {
    init();
    trace!("Starting goto-transcoder");
    let cli = Cli::parse();

    match cli.command {
        Commands::CBMC2ESBMC(args) => {
            cbmc2esbmc(
                args.input.to_str().expect("input path is not valid UTF-8"),
                args.output
                    .to_str()
                    .expect("output path is not valid UTF-8"),
            );
        }
    };

    trace!("Done");
}
