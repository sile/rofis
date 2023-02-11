use clap::Parser;
use orfail::OrFail;
use rofis::dirs_index::DirsIndex;

#[derive(Debug, Parser)]
#[clap(version)]
struct Args {}

fn main() -> orfail::Result<()> {
    env_logger::try_init().or_fail()?;
    let _args = Args::parse();
    let dirs_index = DirsIndex::build(std::env::current_dir().or_fail()?).or_fail()?;
    Ok(())
}
