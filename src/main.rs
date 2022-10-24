mod cli;
mod avx;
mod score;
mod pairwise;

use std::time::Instant;

use clap::Parser;
use needletail::parse_fastx_file;

use cli::{ Weight, CliArgs };
use score::{ pam120, blosum50, blosum62 };
use pairwise::{ AlignFlag, smith_waterman_avx2 };

struct Seq { pub id: String, pub seq: Vec<u8> }

fn fastx_parser<P>(path: P) -> Vec<Seq>
where
    P: AsRef<std::path::Path>
{
    let mut reader = parse_fastx_file(path)
        .map_or_else(|err| panic!("{}", err.msg), |reader| reader);

    let mut seq_set = Vec::new();
    while let Some(item) = reader.next()
    {
        let record = item.map_or_else(|err| panic!("{}", err.msg), |r| r);
        let id = String::from_utf8(record.id()
                        .to_vec())
                        .map_or_else(|err| panic!("{}", err), |s| s);
        let seq = record.seq().to_vec();
        seq_set.push(Seq { id, seq });
    }
    seq_set
}

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
                let res = smith_waterman_avx2(&d.seq, &q.seq, go, ge, &flag, score)
                    .expect("Overflow, d/q sequence is too long");
                time_cost_total = time_cost_total + tick.elapsed().as_secs_f64();
                res_set.push((&d.id, &q.id, res));
            }
        }

        for res in res_set.iter()
        {
            println!("d_id: {}", res.0);
            println!("q_id: {}", res.1);
            println!("{}", res.2);
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
                let res = smith_waterman_avx2(&d.seq, &q.seq, go, ge, &flag, &score).expect("Overflow, d/q sequence is too long");
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