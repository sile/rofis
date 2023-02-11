use clap::Parser;
use orfail::OrFail;

#[derive(Debug, Parser)]
#[clap(version)]
struct Args {}

fn main() -> orfail::Result<()> {
    env_logger::try_init().or_fail()?;
    let _args = Args::parse();
    Ok(())
}
