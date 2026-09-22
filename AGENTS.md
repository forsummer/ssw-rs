# AGENTS.md

This file gives AI coding agents (and future-you) the context needed to work in this
repository. Read it before making changes.

## What this project is

A Rust implementation of the Smith-Waterman sequence alignment algorithm, centered on
striped SIMD acceleration (AVX2 / SSE2), with a scalar implementation kept as the
correctness reference.

- `ssw` (library, `src/lib.rs`): public API, error types, module re-exports
- `smith-waterman` (CLI, `src/main.rs`): clap command line
- Dependencies: `needletail` (reads FASTA/FASTQ, including .gz), `clap`, `thiserror`;
  `pyo3` is declared but **not yet used in `src/`**

## Build & test

```bash
cargo build
cargo test
cargo run --release -- --help
```

## ⚠️ Critical: CPU instruction set & portability

`.cargo/config.toml` injects:

```toml
rustflags = ["-C", "target-feature=+avx2", "-C", "target-feature=+sse2", "-C", "target-cpu=native"]
```

- This ensures AVX2 is used locally and is the core of the performance. Do **not**
  casually remove these flags "for portability".
- Side effect: a local `--release` binary only runs on this machine (or same-class CPUs).
  On x86_64 machines without AVX2 it will raise an illegal instruction. To distribute,
  build on the target machine, or drop `target-cpu=native` / add runtime CPU dispatch,
  rather than copying the `target/release` artifact around.
- The AVX2 code in `src/avx.rs` and `src/pairwise.rs` is gated by
  `#[cfg(all(target_feature="avx", target_feature="avx2"))]`; on non-AVX2 machines those
  modules do not compile (only the SSE2 and scalar paths remain).

## Code style (important)

- The codebase is K&R style (rustfmt defaults). The only custom rustfmt setting is
  `chain_width = 60` in `rustfmt.toml`, which wraps long method chains earlier.
- Always run `cargo fmt` after editing. Do not "tidy up" formatting by hand, or you will
  fight rustfmt and produce meaningless diffs.
- Comment language: README / public docs are in English, in-source comments are mixed
  Chinese/English. Match the surrounding code; no need to unify the whole repo.

## Architecture map

| File | Responsibility |
| --- | --- |
| `src/pairwise.rs` | Core implementation: `smith_waterman_avx2` / `smith_waterman_sse2` / `smith_waterman_scalar`, plus public `AlignFlag` and `AlignResult` (~2400 lines) |
| `src/avx.rs` | AVX2 vector wrappers `M256Epu8` / `M256Epu16` (`__m256i`) |
| `src/sse2.rs` | SSE2 vector wrappers `M128Epu8` / `M128Epu16` (`__m128i`) |
| `src/score.rs` | BLOSUM50 / BLOSUM62 / PAM120 scoring matrices, wrapped as `Fn(u8, u8) -> Option<i8>` |
| `src/lib.rs` | Crate root: `Error` enum, module re-exports |
| `src/main.rs` | clap CLI args and entry point |
| `examples/` | Test data: FASTA / FASTQ / .gz (including 100k and 10M sizes) |

## Conventions & guardrails

- Public APIs return `Result<AlignResult, Error>`; the error type lives in `src/lib.rs`.
- Scoring functions use the signature `Fn(u8, u8) -> Option<i8>`; `AlignFlag` controls the
  return content (`End` / `Path` / `OptOnly`).
- The scalar implementation `smith_waterman_scalar` is the correctness baseline: when
  changing SIMD code, cross-check against it rather than only "it runs".
- Understand data packing / alignment before hand-editing SIMD code; when unsure, keep the
  three implementations (avx2 / sse2 / scalar) behaviorally consistent.

## Known cleanup items

- The `pyo3` dependency is currently unused by `src/`; either remove it or add the Python
  bindings deliberately.
