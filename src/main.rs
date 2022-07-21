mod cli;
mod load;
mod utils;
mod score;
mod pairwise;

use clap::Parser;
use cli::CliArgs;
use load::load_fastx;

// use score::pam120;
use score::blosum62;
// use score::blosum50;
// use pairwise::smith_waterman_avx2;
use pairwise::smith_waterman_scalar;

fn main()
{
    let cli_args = CliArgs::parse();

    let go = cli_args.gap_open;
    let ge = cli_args.gap_extend;

    let d_set = load_fastx(cli_args.db);
    let q_set = load_fastx(cli_args.query);

    let is_protein = cli_args.is_protein;
    if !is_protein
    {
        let miss_ = -(cli_args._miss.abs());
        let match_ = cli_args._match.abs();

        let ref score = |r1, r2| if r1 == r2 { match_ } else { miss_ };
        for d in d_set.iter()
        {
            for q in q_set.iter()
            {
                println!("{}", smith_waterman_scalar(d, q, go, ge, score));
            }
        }
    }
    else
    {
        let ref score = if let None = cli_args.weight { blosum62() } else { todo!() };
        for d in d_set.iter()
        {
            for q in q_set.iter()
            {
                println!("{}", smith_waterman_scalar(d, q, go, ge, score));
            }
        }
    }
}