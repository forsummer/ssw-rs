use clap::Parser;

#[derive(Parser)]
#[clap(version = "0.1")]
#[clap(about = "Alignment nucleotide seq by smith-waterman algorithm")]
pub struct CliArgs
{
    #[clap(name = "db")]
    #[clap(help = "database file path [format: fasta]")]
    pub d_path: String,

    #[clap(name = "query")]
    #[clap(help = "query file path [format: fasta]")]
    pub q_path: String,

    #[clap(long)]
    #[clap(default_value_t = 1)]
    #[clap(parse(try_from_str = is_integers))]
    #[clap(help = "pair residue matching score")]
    pub _match: i32,

    #[clap(long)]
    #[clap(default_value_t = -1)]
    #[clap(parse(try_from_str = is_integers))]
    #[clap(help = "pair residue miss match score")]
    pub _miss: i32,

    #[clap(long)]
    #[clap(default_value_t = 3)]
    #[clap(parse(try_from_str = is_non_negative_integers))]
    #[clap(help = "gap open score, expect positive integers")]
    pub gap_open: i32,

    #[clap(long)]
    #[clap(default_value_t = 2)]
    #[clap(parse(try_from_str = is_non_negative_integers))]
    #[clap(help = "gap extend score, expect positive integers")]
    pub gap_extend: i32,
}

fn is_integers(arg: &str) -> Result<i32, String>
{
    let score: i32 = arg.parse().map_err(|_expection| format!("{} not a integers", arg))?;
    Ok(score)
}

fn is_non_negative_integers(arg: &str) -> Result<i32, String>
{
    let score: i32 = arg.parse().map_err(|_expection| format!("{} not a Non-negative integers", arg))?;
    if score >= 0 { Ok(score) } else { Err(format!("{} not a Non-negative integers", score)) }
}