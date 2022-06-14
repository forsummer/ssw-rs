mod args;
mod load;
mod utils;
mod pairwise;

use clap::Parser;

use args::CliArgs;
use load::load_fastx;
use pairwise::smith_waterman_avx2;

fn main()
{
    let cli_args = CliArgs::parse();

    let d_set = load_fastx(cli_args.db_path);
    let q_set = load_fastx(cli_args.q_path);

    for i in 0..d_set.len()
    {
        for j in 0..q_set.len()
        {
            let result = smith_waterman_avx2(&d_set[i], &q_set[j], cli_args._match, cli_args._miss, cli_args.gap_open, cli_args.gap_extend);
            println!("{}", &result);
        }
    }
}