use clap::Parser;
use orfail::OrFail;
use rofis::dirs_index::DirsIndex;

#[derive(Debug, Parser)]
#[clap(version)]
struct Args {}

fn main() -> orfail::Result<()> {
    env_logger::try_init().or_fail()?;
    let _args = Args::parse();

    log::info!("Starts building directories index");
    let dirs_index = DirsIndex::build(std::env::current_dir().or_fail()?).or_fail()?;
    log::info!(
        "Finished building directories index: entries={}",
        dirs_index.len()
    );

    Ok(())
}
