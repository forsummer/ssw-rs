## SSW: Fastest Striped-Smith-Waterman algorithm implementation accelerated by AVX2

### Overview
SSW is the most fastest Rust implementation of smith-waterman algorithm, which use **AVX2** instruction to parallelizez the algorithm in data-level. It can be used to alignment two sequences, return **best aignment score**, **alignment end** and **optimal traceback path**. This library provides the following two kinds of API in Rust:

- Smith-Waterman(accelerated by **AVX2**) implementation in Rust
- Wrapping of the scoring matrix: `blosum50`, `blosum62` and `pam120`

We also provide a command line software name `simth-waterman` in this package which can alignment protein and genome sequence directly.



### How to use `ssw` interface in Rust

```Rust
use ssw::score::blosum50;
use ssw::pairwise::{ AlignFlag, smith_waterman_avx2 };

fn main()
{
    // DataBase protein sequence
    let d = "CLKQTQMRTDHARCGDFWEESHHHHHHFTLCIA".as_bytes();
    
    // Query protein sequence
    let q = "CLKQTQMRTDHAMCGDFWEESHHHFTLCIA".as_bytes();
    
    // Gap open penalty score
    let go = 3;
    
    // Gap extennd penalty score
    let ge = 2;
    
    // When `smith_waterman_avx2()` receives `AlignFlag::End`,
    // it will return only the best alignment score and position.
    // Conversely, when it receives `AlignFlag::Path`, it will return optimal score,
    // start and end position of best alignment and best traceback path
    let flag = AlignFlag::Path;
    
    // Module `score` packaging `blosum50`, `blosum62` and `pam120` matrix to a closure
    // which like `Fn(u8, u8) -> Option<i8>`. Therefore, the user does not need to
    // care about the index arrangement of the scoring matrix and whether a pair
    // has a corresponding score in the matrix
    let pair_score = blosum50();
    
    let res = smith_waterman_avx2(d, q, go, ge, &flag, &pair_score).unwrap();
    
    println!("{}", res);
    // `AlignResult` has implemented `std::fmt::Display` trait.
    // Therefore, the alignment results can be printed directly,
    // the printed results are as follows:
    //
    // optimal_alignment_score: 216, d_start 1, d_end: 33, q_start 1, q_end: 30
	//
	// d_best: 1 CLKQTQMRTDHARCGDFWEESHHHHHHFTLCIA 33
  	//           ||||||||||||*|||||||||||   |||||| 
	// q_best: 1 CLKQTQMRTDHAMCGDFWEESHHH---FTLCIA 30
}
```



### How to install the software

First, you should have `Rust` and `Cargo`, if not, you can run follow command to install `Rust` and `Cargo`:

```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then, you should download the source code package of `ssw`. After that, run follow command to install `smith-waterman` software:

```shell
cargo install /path/to/smith-waterman/score/code
```

After installation is complete, run command `smith-waterman --help` in your shell, you will see output follow like that:

```
smith-waterman 0.1
Alignment nucleotide seq by smith-waterman algorithm

USAGE:
    smith-waterman [OPTIONS] <db> <query>

ARGS:
    <db>       Database sequence file path(fasta/fastq)
    <query>    Query sequence file path(fasta/fastq)

OPTIONS:
    -u, --miss <miss>                Penalty score when two residue missmatch [default: 1]
    -m, --match <match>              Add this score when two residue match [default: 1]
    -o, --gap-open <gap-open>        Gap open penalty score, positive integer [default: 3]
    -e, --gap-extend <gap-extend>    Gap extend penalty score, positive integer [default: 2]
    -p, --protein                    Performing protein sequence alignment
    -c, --print-path                 Return optimal alignment path
    -w, --weight <weight>            [default: blosum62] [possible values: pam120, blosum50,
                                     blosum62]
    -h, --help                       Print help information
    -V, --version                    Print version information
```



### Warning

Currently the `ssw` library is only available for **x86_64** platforms that support the **AVX2** instruction set.



### Inspiration

- **Farrar' work about striped-smith-waterman**: Michael Farrar, Striped Smith–Waterman speeds database searches six times over other SIMD implementations, Bioinformatics, Volume 23, Issue 2, 15 January 2007, Pages 156–161, https://doi.org/10.1093/bioinformatics/btl582
- **Zhao's work about improve striped-smith-waterman algorithm implementation**: Zhao, Mengyao, et al. "SSW library: an SIMD Smith-Waterman C/C++ library for use in genomic applications." *PloS one* 8.12 (2013): e82138, https://doi.org/10.1371/journal.pone.0082138