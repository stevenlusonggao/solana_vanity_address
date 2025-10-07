use clap::{Parser, ValueEnum};
use rayon::{prelude::*, ThreadPoolBuilder};
use solana_keypair::Keypair;
use solana_signer::Signer;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(name = "solana-vanity-address")]
#[command(about = "A CLI tool for generating solana vanity addresses")]
struct Args {
    // pattern to find
    #[arg(short = 'f', long, value_parser = validate_find)]
    find: String,

    // number of threads to create
    #[arg(short = 't', long, default_value_t = 2, value_parser = validate_threads)]
    threads: usize,

    // match type to use
    #[arg(short = 'm', long, value_enum, default_value_t = MatchType::Prefix)]
    match_type: MatchType,

    // enable case sensitivity
    #[arg(short = 's', long, default_value_t = false, action = clap::ArgAction::Set)]
    case_sensitivity: bool,

    // enable flexible character set
    #[arg(short = 'l', long, default_value_t = true, action = clap::ArgAction::Set)]
    flexible_chars: bool,
}

// Check if all characters are valid base58, and is an appropriate length
const CHAR_LIMIT: usize = 18; //arbitrary number that is shorter than the pubkey char limit but also is an unreasonably long substring to search for
const BASE58_SET: &str = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
fn validate_find(s: &str) -> Result<String, String> {
    if s.len() > CHAR_LIMIT {
        return Err(format!(
            "Pattern is too long to search for; current char limit: {}",
            CHAR_LIMIT
        ));
    }

    for ch in s.chars() {
        if !BASE58_SET.contains(ch) {
            return Err(format!(
                "Invalid character '{}' in pattern. Only base58 characters allowed: {}",
                ch, BASE58_SET
            ));
        }
    }

    Ok(s.to_string())
}

// Check if number of threads is create is realistic
fn validate_threads(s: &str) -> Result<usize, String> {
    let threads = s
        .parse()
        .map_err(|_| format!("'{}' is not a valid number", s))?;

    if threads == 0 {
        return Err("Number of threads must be at least 1".to_string());
    }

    let available_threads = match std::thread::available_parallelism() {
        Ok(a) => a.get(),
        Err(e) => {
            return Err(format!(
                "Cannot get number of available threads in system: {}",
                e
            ))
        }
    };

    if threads > available_threads {
        return Err(format!(
            "Requested {} threads but only {} hardware threads (logical cores) available, which may cause performance degradation",
            threads, available_threads
        ));
    }

    Ok(threads)
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum MatchType {
    Prefix,
    Suffix,
    Either,
}

fn main() {
    let args = Args::parse();
    println!("Now searching with the following config:");
    println!("  Pattern: {}", args.find);
    println!("  Threads: {}", args.threads);
    println!("  Match Type: {:?}", args.match_type);
    println!("  Case Sensitivity: {}", args.case_sensitivity);
    println!("  Flexible Char Set: {}", args.flexible_chars);

    let start = Instant::now();

    let pattern = args.find;
    let match_type = args.match_type;
    let case_sensitivity = args.case_sensitivity;
    let flexible_chars = args.flexible_chars;

    ThreadPoolBuilder::new()
        .num_threads(args.threads)
        .build_global()
        .unwrap();

    let found = Arc::new(AtomicBool::new(false));
    let result = (0..args.threads).into_par_iter().find_map_any(|_| {
        while !found.load(Ordering::Relaxed) {
            let keypair = Keypair::new();
            let pubkey_str = keypair.pubkey().to_string();

            if matches_pattern(
                pubkey_str.as_bytes(),
                &pattern.as_bytes(),
                match_type,
                case_sensitivity,
                flexible_chars,
            ) {
                found.store(true, Ordering::Relaxed);
                return Some(keypair);
            }
        }
        None
    });

    match result {
        Some(keypair) => {
            println!("Found address: {}", keypair.pubkey());
            println!("KP: {}", keypair.to_base58_string());
        }
        None => {
            println!("No matching keypair found");
        }
    }
    println!("Took {:.2} minutes", start.elapsed().as_secs_f64() / 60.0);
}

// Pattern finder
fn matches_pattern(
    pubkey: &[u8],
    pattern: &[u8],
    match_type: MatchType,
    case_sensitive: bool,
    flexible_chars: bool,
) -> bool {
    let pubkey_len = pubkey.len();
    let pattern_len = pattern.len();
    let flexible_chars = if case_sensitive {
        false
    } else {
        flexible_chars
    };

    match match_type {
        MatchType::Prefix => {
            for i in 0..pattern_len {
                if !matches_char(pubkey[i], pattern[i], case_sensitive, flexible_chars) {
                    return false;
                }
            }
            true
        }
        MatchType::Suffix => {
            let start_idx = pubkey_len - pattern_len;
            for i in 0..pattern_len {
                if !matches_char(
                    pubkey[start_idx + i],
                    pattern[i],
                    case_sensitive,
                    flexible_chars,
                ) {
                    return false;
                }
            }
            true
        }
        MatchType::Either => {
            // check prefix first (early return on match)
            let mut prefix_matches = true;
            for i in 0..pattern_len {
                if !matches_char(pubkey[i], pattern[i], case_sensitive, flexible_chars) {
                    prefix_matches = false;
                    break;
                }
            }
            if prefix_matches {
                return true;
            }
            // check suffix if prefix doesn't match
            let start_idx = pubkey_len - pattern_len;
            for i in 0..pattern_len {
                if !matches_char(
                    pubkey[start_idx + i],
                    pattern[i],
                    case_sensitive,
                    flexible_chars,
                ) {
                    return false;
                }
            }
            true
        }
    }
}

// Checks which pattern finder method to use
#[inline]
fn matches_char(c: u8, target: u8, case_sensitive: bool, flexible: bool) -> bool {
    // case_sensitivity is true, flexible_chars is false
    if case_sensitive {
        c == target
    // flexible_chars is true, case_sensitivity is false
    } else if flexible {
        matches_flexible(c, target)
    // flexible_chars is false, case_sensitivity is false
    } else {
        c.eq_ignore_ascii_case(&target)
    }
}

// Flexible char pattern finder that looks for similar chars
#[inline]
fn matches_flexible(c: u8, target: u8) -> bool {
    match target {
        b'1' => matches!(c, b'1' | b'i' | b'L'),
        b'2' => matches!(c, b'2' | b'z' | b'Z'),
        b'3' => matches!(c, b'3' | b'E'),
        b'4' => matches!(c, b'4' | b'A'),
        b'5' => matches!(c, b'5' | b's' | b'S'),
        b'6' => matches!(c, b'6' | b'b' | b'G'),
        b'7' => matches!(c, b'7' | b'T'),
        b'8' => matches!(c, b'8' | b'B'),
        b'9' => matches!(c, b'9' | b'g'),

        b'a' => matches!(c, b'a' | b'A' | b'4'),
        b'b' => matches!(c, b'b' | b'B' | b'6'),
        b'c' => matches!(c, b'c' | b'C'),
        b'd' => matches!(c, b'd' | b'D'),
        b'e' => matches!(c, b'e' | b'E' | b'3'),
        b'f' => matches!(c, b'f' | b'F'),
        b'g' => matches!(c, b'g' | b'G' | b'6' | b'9'),
        b'h' => matches!(c, b'h' | b'H'),
        b'i' => matches!(c, b'i' | b'1'),
        b'j' => matches!(c, b'j' | b'J'),
        b'k' => matches!(c, b'k' | b'K'),
        b'm' => matches!(c, b'm' | b'M'),
        b'n' => matches!(c, b'n' | b'N'),
        b'o' => matches!(c, b'o'),
        b'p' => matches!(c, b'p' | b'P'),
        b'q' => matches!(c, b'q' | b'Q'),
        b'r' => matches!(c, b'r' | b'R'),
        b's' => matches!(c, b's' | b'S' | b'5'),
        b't' => matches!(c, b't' | b'T' | b'7'),
        b'u' => matches!(c, b'u' | b'U'),
        b'v' => matches!(c, b'v' | b'V'),
        b'w' => matches!(c, b'w' | b'W'),
        b'x' => matches!(c, b'x' | b'X'),
        b'y' => matches!(c, b'y' | b'Y'),
        b'z' => matches!(c, b'z' | b'Z' | b'2'),

        b'A' => matches!(c, b'a' | b'A' | b'4'),
        b'B' => matches!(c, b'b' | b'B' | b'6' | b'8'),
        b'C' => matches!(c, b'c' | b'C'),
        b'D' => matches!(c, b'd' | b'D'),
        b'E' => matches!(c, b'e' | b'E' | b'3'),
        b'F' => matches!(c, b'f' | b'F'),
        b'G' => matches!(c, b'g' | b'G' | b'6' | b'9'),
        b'H' => matches!(c, b'h' | b'H'),
        b'J' => matches!(c, b'j' | b'J'),
        b'K' => matches!(c, b'k' | b'K'),
        b'L' => matches!(c, b'L' | b'1'),
        b'M' => matches!(c, b'm' | b'M'),
        b'N' => matches!(c, b'n' | b'N'),
        b'P' => matches!(c, b'p' | b'P'),
        b'Q' => matches!(c, b'q' | b'Q'),
        b'R' => matches!(c, b'r' | b'R'),
        b'S' => matches!(c, b's' | b'S' | b'5'),
        b'T' => matches!(c, b't' | b'T' | b'7'),
        b'U' => matches!(c, b'u' | b'U'),
        b'V' => matches!(c, b'v' | b'V'),
        b'W' => matches!(c, b'w' | b'W'),
        b'X' => matches!(c, b'x' | b'X'),
        b'Y' => matches!(c, b'y' | b'Y'),
        b'Z' => matches!(c, b'z' | b'Z' | b'2'),

        _ => c.eq_ignore_ascii_case(&target),
    }
}
