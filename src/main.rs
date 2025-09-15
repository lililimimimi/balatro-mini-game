pub mod joker;
pub mod modifiers;
pub mod pokerhand;
pub mod score;

use std::{
    error::Error,
    fs::File,
    io::{Read, stdin},
    path::{Path, PathBuf},
};

use clap::Parser;
use ortalib::Round;
use score::ScoreManager;

#[derive(Parser)]

struct Opts {
    file: PathBuf,

    #[arg(long)]
    explain: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts = Opts::parse();
    let round = parse_round(&opts)?;

    let (chips, mult, explanation) = ScoreManager::score_with_explanation(&round);

    if opts.explain {
        println!("{}", explanation);
    } else {
        println!("{}", (chips * mult).floor());
    }
    Ok(())
}

fn parse_round(opts: &Opts) -> Result<Round, Box<dyn Error>> {
    let mut input = String::new();
    if opts.file == Path::new("-") {
        stdin().read_to_string(&mut input)?;
    } else {
        File::open(&opts.file)?.read_to_string(&mut input)?;
    }

    let round = serde_yaml::from_str(&input)?;
    Ok(round)
}
