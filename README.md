# Solana Vanity Address Generator CLI 

A high-performance, multi-threaded Solana vanity address generator CLI written in Rust. Generate custom Solana wallet addresses with specific prefixes or suffixes using flexible matching options.

A vanity public key is a Solana address that begins or ends with specific characters you choose. The more characters you want at the beginning of your vanity address, the longer it will take to generate one.

## Features

- ⚡ Optimized Multi-threaded Performance - Built with [Rayon](https://docs.rs/rayon/latest/rayon/) for efficient parallel processing. Utilize multiple CPU cores for maximum performance.
- 🎯 Flexible Matching - Match patterns with lookalike characters (e.g., s matches S, 5).
- 🔤 Case Sensitivity - Choose between case-sensitive, case-insensitive. 
- 🔍 Multiple Match Types - Search for prefix, suffix, or either.
- ✅ Base58 Validation - Automatically validates patterns against Solana's [Base58](https://digitalbazaar.github.io/base58-spec/) character set.

## Installation

### Prerequisites:

- [Install Rust](https://rust-lang.org/tools/install/)
- Cargo (comes with Rust)

### Build from Source

```bash
# Clone the repository
git clone https://github.com/stevenlusonggao/solana_vanity_address
cd solana_vanity_address

# Build in release mode (optimized)
cargo build --release
```

## Usage

### Basic Usage

```bash
# Generate address with "Punk" prefix using 8 threads
cargo run --release -- -f "Punk" -t 8
```

### Command-Line Options

```
Options:
  -f, --find <FIND>
        Pattern to find 

  -t, --threads <THREADS>
        Number of threads to use. [default: 2]     

  -m, --match-type <MATCH_TYPE>
        Where the pattern search should take place. [default: prefix] [possible values: prefix, suffix, either]

  -s, --case-sensitivity 
        Enable case sensitivity. [default: false]

  -l, --flexible-chars
        Enable flexible char find. [default: true]        

  -h, --help                     
        Print help
```

## Full Guide

[Click here](https://stevenlusonggao.github.io/blog/posts/solana-vanity-address/) for the full guide with more examples.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

