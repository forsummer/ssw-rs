use std::time::Instant;
use clap::{Parser, ValueEnum};
use needletail::parse_fastx_file;

use ssw::score::{pam120, blosum50, blosum62};
use ssw::pairwise::{AlignFlag, smith_waterman_avx2};


#[derive(ValueEnum, Clone)]
enum Weight {Pam120, Blosum50, Blosum62}

#[derive(Parser)]
#[clap(version = "0.1")]
#[clap(about = "Alignment seq by smith-waterman algorithm")]
struct CliArgs
{
    #[clap(name = "miss", long, short = 'u', display_order = 1)]
    #[clap(default_value_t = 1)]
    #[clap(value_parser = is_integer)]
    #[clap(help = "Penalty score when two residue missmatch")]
    pub _miss: i8,

    #[clap(name = "match", long, short = 'm', display_order = 2)]
    #[clap(default_value_t = 1)]
    #[clap(value_parser = is_integer)]
    #[clap(help = "Add this score when two residue match")]
    pub _match: i8,

    #[clap(name = "gap-open", long, short = 'o', display_order = 3)]
    #[clap(default_value_t = 3)]
    #[clap(value_parser = is_positive_integer)]
    #[clap(help = "Gap open penalty score, positive integer")]
    pub gap_open: u8,

    #[clap(name = "gap-extend", long, short = 'e', display_order = 4)]
    #[clap(default_value_t = 2)]
    #[clap(value_parser = is_positive_integer)]
    #[clap(help = "Gap extend penalty score, positive integer")]
    pub gap_extend: u8,

    #[clap(name = "protein", long, short = 'p', display_order = 5)]
    #[clap(conflicts_with_all = &["miss", "match"])]
    #[clap(help = "Performing protein sequence alignment")]
    pub is_protein: bool,

    #[clap(name = "print-path", long, short = 'c', display_order = 6)]
    #[clap(help = "Return optimal alignment path")]
    pub print_path: bool,

    #[clap(value_enum)]
    #[clap(name = "weight", long, short = 'w', display_order = 7)]
    #[clap(conflicts_with_all = &["miss", "match"])]
    #[clap(default_value_t = Weight::Blosum62)]
    pub weight: Weight,

    #[clap(name = "db")]
    #[clap(help = "Database sequence file path")]
    pub db: String,

    #[clap(name = "query")]
    #[clap(help = "Query sequence file path")]
    pub query: String
}

fn is_integer(arg: &str) -> Result<i8, String>
{
    let score = arg.parse()
        .map_err(|_| format!("{} not a i8 integer", arg))?;
    Ok(score)
}

fn is_positive_integer(arg: &str) -> Result<u8, String>
{
    let score = arg.parse()
        .map_err(|_| format!("{} not a non-negative integer", arg))?;
    Ok(score)
}

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

    if !cli_args.is_protein
    {
        let miss_ = -(cli_args._miss.abs());
        let match_ = cli_args._match.abs();
        let score = |r1, r2| if r1 == r2 { Some(match_) } else { Some(miss_) };
        let flag = match cli_args.print_path
        {
            true  => AlignFlag::Path,
            false => AlignFlag::End,
        };

        let mut time_cost_total = 0.0;
        let mut results = Vec::with_capacity(d_set.len() * q_set.len());
        for d in d_set.iter()
        {
            for q in q_set.iter()
            {
                let tick = Instant::now();
                let res = smith_waterman_avx2(&d.seq, &q.seq, go, ge, &flag, score)
                    .expect("Overflow, d/q sequence is too long");
                time_cost_total = time_cost_total + tick.elapsed().as_secs_f64();
                results.push((&d.id, &q.id, res));
            }
        }

        for (d_id, q_id, result) in results.iter()
        {
            println!("d_id: {}", d_id);
            println!("q_id: {}", q_id);
            println!("{}", result);
        }
        println!("time-cost: {}s", time_cost_total);
    }
    else
    {
        let score = match cli_args.weight
        {
            Weight::Pam120   => pam120,
            Weight::Blosum50 => blosum50,
            Weight::Blosum62 => blosum62,
        };

        let flag = match cli_args.print_path
        {
            true  => AlignFlag::Path,
            false => AlignFlag::End,
        };

        let mut time_cost_total = 0.0;
        let mut results = Vec::with_capacity(d_set.len() * q_set.len());
        for d in d_set.iter()
        {
            for q in q_set.iter()
            {
                let tick = Instant::now();
                let res = smith_waterman_avx2(&d.seq, &q.seq, go, ge, &flag, &score)
                    .expect("Overflow, d/q sequence is too long");
                time_cost_total = time_cost_total + tick.elapsed().as_secs_f64();
                results.push((&d.id, &q.id, res));
            }
        }

        for (d_id, q_id, result) in results.iter()
        {
            println!("d_id: {}", d_id);
            println!("q_id: {}", q_id);
            println!("{}", result);
        }
        println!("time-cost: {}s", time_cost_total);
    }
}
