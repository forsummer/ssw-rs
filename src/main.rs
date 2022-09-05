mod cli;
mod avx;
mod load;
mod score;
mod pairwise;

use std::time::Instant;

use clap::Parser;

use load::fastx_parser;
use cli::{ Weight, CliArgs };
use score::{ pam120, blosum50, blosum62 };
use pairwise::{ AlignFlag, smith_waterman_avx2 };
// use pairwise::smith_waterman_scalar;

fn main()
{
    let cli_args = CliArgs::parse();

    let go = cli_args.gap_open;
    let ge = cli_args.gap_extend;
    
    let d_set = fastx_parser(cli_args.db);
    let q_set = fastx_parser(cli_args.query);

    let is_protein = cli_args.is_protein;
    if !is_protein
    {
        let miss_ = -(cli_args._miss.abs());
        let match_ = cli_args._match.abs();
        let score = |r1, r2| if r1 == r2 { match_ } else { miss_ };
        let flag = match cli_args.print_path
        {
            true  => AlignFlag::Path,
            false => AlignFlag::End,
        };

        let mut time_cost_total = 0.0;
        let mut res_set = Vec::with_capacity(d_set.len() * q_set.len());
        for d in d_set.iter()
        {
            for q in q_set.iter()
            {
                let tick = Instant::now();
                let res = smith_waterman_avx2(d, q, go, ge, &flag, score)
                    .expect("Overflow, d/q sequence is too long");
                time_cost_total = time_cost_total + tick.elapsed().as_secs_f64();
                res_set.push(res);
                // let res = smith_waterman_scalar(d, q, go as u32, ge as u32, score)
                //     .expect("Overflow, d/q sequence is too long");
            }
        }

        for res in res_set.iter()
        {
            println!("{}", res);
        }
        println!("time-cost: {}s", time_cost_total);
    }
    else
    {
        let score = match cli_args.weight
        {
            Weight::Pam120   => pam120(),
            Weight::Blosum50 => blosum50(),
            Weight::Blosum62 => blosum62(),
        };

        let flag = match cli_args.print_path
        {
            true  => AlignFlag::Path,
            false => AlignFlag::End,
        };
        
        let mut time_cost_total = 0.0;
        let mut res_set = Vec::with_capacity(d_set.len() * q_set.len());
        for d in d_set.iter()
        {
            for q in q_set.iter()
            {
                let tick = Instant::now();
                let res = smith_waterman_avx2(d, q, go, ge, &flag, &score).expect("Overflow, d/q sequence is too long");
                time_cost_total = time_cost_total + tick.elapsed().as_secs_f64();
                res_set.push(res);
            }
        }

        for res in res_set.iter()
        {
            println!("{}", res);
        }
        println!("time-cost: {}s", time_cost_total);
    }
}