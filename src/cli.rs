use clap::Parser;
use clap::ValueEnum;

#[derive(ValueEnum, Clone)]
pub enum Weight { Blosum50, Blosum62, Pam120  }

#[derive(Parser)]
#[clap(version = "0.1")]
#[clap(about = "Alignment nucleotide seq by smith-waterman algorithm")]
pub struct CliArgs
{
    #[clap(name = "miss", long, short = 'u', display_order = 1)]
    #[clap(default_value_t = 1)]
    #[clap(value_parser = is_integer)]
    #[clap(help = "Penalty score when two residue missmatch")]
    pub _miss: i16,

    #[clap(name = "match", long, short = 'm', display_order = 2)]
    #[clap(default_value_t = 1)]
    #[clap(value_parser = is_integer)]
    #[clap(help = "Add this score when two residue match")]
    pub _match: i16,
    
    #[clap(name = "gap-open", long, short = 'o', display_order = 3)]
    #[clap(default_value_t = 3)]
    #[clap(value_parser = is_positive_integer)]
    #[clap(help = "Gap open penalty score, positive integer")]
    pub gap_open: u16,

    #[clap(name = "gap-extend", long, short = 'e', display_order = 4)]
    #[clap(default_value_t = 2)]
    #[clap(value_parser = is_positive_integer)]
    #[clap(help = "Gap extend penalty score, positive integer")]
    pub gap_extend: u16,

    #[clap(name = "protein", long, short = 'p', display_order = 5)]
    #[clap(conflicts_with_all = &["miss", "match"])]
    #[clap(help = "Performing protein sequence alignment")]
    pub is_protein: bool,

    #[clap(value_enum)]
    #[clap(name = "weight", long = "weight", short = 'w', display_order = 6)]
    #[clap(conflicts_with_all = &["miss", "match"])]
    #[clap(default_value_t = Weight::Blosum62)]
    // #[clap(help = "blosum50 | blosum62 | pam120")]
    pub weight: Weight,

    #[clap(name = "db")]
    #[clap(help = "Database sequence file path(fasta/fastq)")]
    pub db: String,

    #[clap(name = "query")]
    #[clap(help = "Query sequence file path(fasta/fastq)")]
    pub query: String
}

fn is_integer(arg: &str) -> Result<i16, String>
{
    let score = arg.parse().map_err(|_expection| format!("{} not a i8 integer", arg))?;
    Ok(score)
}

fn is_positive_integer(arg: &str) -> Result<u16, String>
{
    let score = arg.parse().map_err(|_expection| format!("{} not a non-negative integer", arg))?;
    Ok(score)
}