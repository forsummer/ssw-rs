//! ### SSW: A fastest striped-smith-waterman implementation in Rust
//! SSW is the most fastest Rust implementation of smith-waterman algorithm,
//! which use **AVX2** instruction to parallelizez the algorithm in data-level.
//! It can be used to alignment two sequences, return **best aignment score**,
//! **alignment end** and **optimal traceback path**.
//! 
//! This library provides the following two kinds of API in Rust:
//! - smith-waterman algorithm implementaion which accelerated by **AVX2**
//! - Wrapping of scoring matrix: `blosum50`, `blosum62` and `pam120`
//! 
//! ### Example: Use smith-waterman implementation accelerated by AVX2
//! ```rust
//! use ssw::score::blosum50;
//! use ssw::pairwise::{ AlignFlag, smith_waterman_avx2 };
//! 
//! fn main()
//! {
//!     // DataBase protein sequence
//!     let d = "CLKQTQMRTDHARCGDFWEESHHHHHHFTLCIA".as_bytes();
//!     
//!     // Query protein sequence
//!     let q = "CLKQTQMRTDHAMCGDFWEESHHHFTLCIA".as_bytes();
//!     
//!     // Gap open penalty score
//!     let go = 3;
//!     
//!     // Gap extennd penalty score
//!     let ge = 2;
//!     
//!     // When `smith_waterman_avx2()` receives `AlignFlag::End`,
//!     // it will return only the best alignment score and position.
//!     // Conversely, when it receives `AlignFlag::Path`, it will return optimal score,
//!     // start and end position of best alignment and best traceback path
//!     let flag = AlignFlag::Path;
//!     
//!     // Module `score` packaging `blosum50`, `blosum62` and `pam120` matrix to a closure
//!     // which like `Fn(u8, u8) -> Option<i8>`. Therefore, the user does not need to
//!     // care about the index arrangement of the scoring matrix and whether a pair
//!     // has a corresponding score in the matrix
//!     let pair_score = blosum50();
//!     
//!     // Alignment two sequence
//!     let align_res = smith_waterman_avx2(d, q, go, ge, &flag, &pair_score)
//!         .map_or_else(|err| panic!("{}", err), |res| res);
//!     
//!     println!("{}", res);
//!     // `AlignResult` has implemented `std::fmt::Display` trait.
//!     // Therefore, the alignment results can be printed directly,
//!     // the printed results are as follows:
//!     //
//!     // optimal_alignment_score: 216, d_start 1, d_end: 33, q_start 1, q_end: 30
//! 	//
//! 	// d_best: 1 CLKQTQMRTDHARCGDFWEESHHHHHHFTLCIA 33
//!   	//           ||||||||||||*|||||||||||   |||||| 
//! 	// q_best: 1 CLKQTQMRTDHAMCGDFWEESHHH---FTLCIA 30
//! }
//! ```

mod avx;

/// Provide wrapping of scoring matrix, such as **blosum50**, **blosum62** and **pam120**.
/// (The scoring matrix is wrapped into a closure like `Fn(u8, u8) -> Option<i8>`)
/// When those function be called, it will return a closure like `Box<Fn(u8, u8) -> Option<i8>>`.
/// The closure accept a character pairs(nucleotide or amino acid) and return corresponding score in scoring matrix.
/// If character pairs has no corresponding score in scoring matrix, closure will return `None`.
pub mod score;

/// Contain striped-smith-waterman implementation accelerated by AVX2 and a serial implementation.
pub mod pairwise;