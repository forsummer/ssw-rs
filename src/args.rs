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
    #[clap(help = "pair residue matching score (integer: [-32768..32767])")]
    pub _match: i16,

    #[clap(long)]
    #[clap(default_value_t = 1)]
    #[clap(parse(try_from_str = is_integers))]
    #[clap(help = "pair residue miss match score (integer: [-32768..32767])")]
    pub _miss: i16,

    #[clap(long)]
    #[clap(default_value_t = 3)]
    #[clap(parse(try_from_str = is_non_negative_integers))]
    #[clap(help = "gap open score (positive integer: [0..65535])")]
    pub gap_open: u16,

    #[clap(long)]
    #[clap(default_value_t = 2)]
    #[clap(parse(try_from_str = is_non_negative_integers))]
    #[clap(help = "gap extend score (positive integer: [0..65535])")]
    pub gap_extend: u16,
}

fn is_integers(arg: &str) -> Result<i16, String>
{
    let score: i16 = arg.parse().map_err(|_expection| format!("{} not a integers", arg))?;
    Ok(score)
}

fn is_non_negative_integers(arg: &str) -> Result<u16, String>
{
    let score: u16 = arg.parse().map_err(|_expection| format!("{} not a Non-Negative integer", arg))?;
    Ok(score)
}