mod args;
mod load;
mod utils;
mod pairwise;

use clap::Parser;

use args::CliArgs;
use load::load_fastx;

// use pairwise::smith_waterman_avx2;
use pairwise::smith_waterman_scalar;

fn main()
{
    let cli_args = CliArgs::parse();
    let d_set = load_fastx(cli_args.d_path);
    let q_set = load_fastx(cli_args.q_path);
    
    let go = cli_args.gap_open;
    let ge = cli_args.gap_extend;
    let match_ = cli_args._match;
    let miss_ = -cli_args._miss.abs();
    let score = |r1, r2| if r1 == r2 { match_ } else { miss_ };
    
    for d in d_set.iter()
    {
        for q in q_set.iter()
        {
            let res = smith_waterman_scalar(d, q, go, ge, score);
            println!("{}", res);
        }
    }
}