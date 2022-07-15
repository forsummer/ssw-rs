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
    #[clap(help = "pair residue matching score (positive integer)")]
    pub _match: i32,

    #[clap(long)]
    #[clap(default_value_t = 1)]
    #[clap(parse(try_from_str = is_integers))]
    #[clap(help = "pair residue miss match score (positive integer)")]
    pub _miss: i32,

    #[clap(long)]
    #[clap(default_value_t = 3)]
    #[clap(parse(try_from_str = is_non_negative_integer))]
    #[clap(help = "gap open score (positive integer)")]
    pub gap_open: u32,

    #[clap(long)]
    #[clap(default_value_t = 2)]
    #[clap(parse(try_from_str = is_non_negative_integer))]
    #[clap(help = "gap extend score (positive integer)")]
    pub gap_extend: u32,
}

fn is_integers(arg: &str) -> Result<i32, String>
{
    let score: i32 = arg.parse().map_err(|_expection| format!("{} not a integer", arg))?;
    Ok(score)
}

fn is_non_negative_integer(arg: &str) -> Result<u32, String>
{
    let score: u32 = arg.parse().map_err(|_expection| format!("{} not a non-negative integer", arg))?;
    Ok(score)
}