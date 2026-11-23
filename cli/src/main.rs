use rlt_cli::{
    check_file, chem_report, lean_export, points_report, rp_report, validate, written_report,
    MALFORMED,
};
use std::path::PathBuf;

const USAGE: &str = "\
usage:
  rlt check FILE...                      replay proof files (exit 0 accept, 1 reject, 2 malformed)
  rlt definitions [ID] [--format text|json]  the definition registry (read-only; exit 0 consistent, 2 not)
  rlt written A CONV_A B CONV_B          written identity and elaboration of two expressions
  rlt rp ID                              a reading point: meanings, relationships, correspondence
  rlt validate                           consistency of sources, conventions and records
  rlt lean FILE...                       Lean 4 replay of the accepted statements
  rlt points FILE                        specification records: reading points, counts, relations
  rlt chem WRITTEN CONVENTION            how a species-table convention reads a written formula
  rlt gen EQUATION ID                    untrusted generator: a PEANO-ARITH proof file for a closed equation
  rlt prove GOAL [--out FILE] [--depth N] [--rounds N] [--nodes N] [--witnesses N] [--budget N]
                                         bounded proof search; every proof found is replayed by the kernel
                                         (exit 0 PROVED, 1 NOT_FOUND_WITHIN_BOUNDS, 2 ERROR)
  rlt discover DIR [--two-level] [--limit N]  bounded theorem discovery; stores only kernel-replayed theorems
  rlt discover --definitions FILE DIR [--limit N]  discovery over the functions registered in FILE (NAT-FUN)
  rlt discover --verify DIR              re-hash and re-check every theorem in DIR/manifest.tsv
  rlt residues FILE                      closed quotient–remainder witnesses and their registered readings
  rlt explain RECORD [--format text|json] a statement record: kernel, source, semantic, physical
                                         (exit 0 current+accept, 1 reject, 2 invalid/malformed, 3 stale)
  rlt deps ID|FILE [--format text|json]  a theorem's dependency graph and library entry (same exits)
  rlt deps --verify-lib [--format text|json]  every lib/ entry against its checked proof
  rlt compose BUNDLE                     layered composition of kernel-checked theorems (RLT-007d;
                                         exit 0 composed, 1 refused, 2 malformed or missing, 3 stale)
options:
  --root DIR                             repository root (default: current directory)
  --unregistered-definitions             with check, prove or discover --definitions: check derivations
                                         over unregistered definitions files (exit 4 EXPLORATORY; never an
                                         authoritative result, never usable by records, citations or lib/)";

fn main() {
    // Kernel terms are trees and numerals are unary chains; give the checker a
    // generous stack so every input within the parser limits completes.
    let worker = std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(run)
        .expect("spawn checker thread");
    std::process::exit(worker.join().unwrap_or(MALFORMED));
}

fn run() -> i32 {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let mut root = PathBuf::from(".");
    if let Some(i) = args.iter().position(|a| a == "--root") {
        if i + 1 < args.len() {
            root = PathBuf::from(&args[i + 1]);
            args.drain(i..=i + 1);
        }
    }
    // RLT-DEF-001a: exploratory checking over unregistered definitions is
    // requested explicitly, and only for check, prove and discover --definitions.
    let mut explore = false;
    if let Some(i) = args.iter().position(|a| a == "--unregistered-definitions") {
        args.remove(i);
        explore = true;
        let allowed = match args.first().map(String::as_str) {
            Some("check") | Some("prove") => true,
            Some("discover") => args.get(1).map(String::as_str) == Some("--definitions"),
            _ => false,
        };
        if !allowed || args.iter().any(|a| a == "--unregistered-definitions") {
            eprintln!("{USAGE}");
            return MALFORMED;
        }
    }
    let authority = if explore {
        rlt_cli::defreg::Authority::ExploratoryAllowed
    } else {
        rlt_cli::defreg::Authority::RegisteredOnly
    };
    let code = match args.first().map(|s| s.as_str()) {
        Some("check") if args.len() >= 2 => {
            let mut worst = 0;
            for (i, f) in args[1..].iter().enumerate() {
                if i > 0 {
                    println!();
                }
                let r = if explore {
                    rlt_cli::check_file_exploratory(&root, &root.join(f)).report
                } else {
                    check_file(&root, &root.join(f))
                };
                print!("{}", r.text);
                // RLT-DEF-001a audit H1: by severity, so EXPLORATORY (4)
                // never hides a REJECT or MALFORMED file in the batch.
                worst = rlt_cli::worse_check_code(worst, r.code);
            }
            worst
        }
        Some("definitions") if args.len() <= 4 => {
            let (mut id, mut json, mut i) = (None, false, 1);
            while i < args.len() {
                match (args[i].as_str(), args.get(i + 1).map(String::as_str)) {
                    ("--format", Some("text")) => i += 2,
                    ("--format", Some("json")) => {
                        json = true;
                        i += 2
                    }
                    (x, _) if id.is_none() && !x.starts_with("--") => {
                        id = Some(x.to_string());
                        i += 1
                    }
                    _ => {
                        eprintln!("{USAGE}");
                        return MALFORMED;
                    }
                }
            }
            let (text, code) = rlt_cli::defreg::definitions_report(&root, id.as_deref(), json);
            print!("{text}");
            code
        }
        Some("written") if args.len() == 5 => {
            let r = written_report(&root, &args[1], &args[2], &args[3], &args[4]);
            print!("{}", r.text);
            r.code
        }
        Some("rp") if args.len() == 2 => {
            let r = rp_report(&root, &args[1]);
            print!("{}", r.text);
            r.code
        }
        Some("lean") if args.len() >= 2 => {
            let files: Vec<PathBuf> = args[1..].iter().map(|f| root.join(f)).collect();
            let r = lean_export(&root, &files);
            print!("{}", r.text);
            r.code
        }
        Some("points") if args.len() == 2 => {
            let r = points_report(&root, &root.join(&args[1]));
            print!("{}", r.text);
            r.code
        }
        Some("gen") if args.len() == 3 => match rlt_cli::gen::generate(&root, &args[1], &args[2]) {
            Ok(text) => {
                print!("{text}");
                0
            }
            Err(e) => {
                eprintln!("rlt gen: {e}");
                1
            }
        },
        Some("prove") if args.len() >= 2 => {
            let mut overrides = Vec::new();
            let mut out = None;
            let mut i = 2;
            while i < args.len() {
                let (Some(flag), Some(val)) = (args[i].strip_prefix("--"), args.get(i + 1)) else {
                    eprintln!("{USAGE}");
                    return MALFORMED;
                };
                if flag == "out" {
                    out = Some(PathBuf::from(val));
                } else {
                    match val.parse::<u64>() {
                        Ok(n) => overrides.push((flag.to_string(), n)),
                        Err(_) => {
                            eprintln!("{USAGE}");
                            return MALFORMED;
                        }
                    }
                }
                i += 2;
            }
            let (text, code) = rlt_cli::prove::prove_report_as(
                &root,
                &root.join(&args[1]),
                &overrides,
                out.as_deref(),
                authority,
            );
            print!("{text}");
            code
        }
        Some("discover") if args.len() == 3 && args[1] == "--verify" => {
            // RLT-PROVER-002: a definition-aware record is recognised by its
            // manifest's first line and verified by its own checker.
            let (text, code) = if rlt_cli::discover_defs::is_record(&root, &args[2]) {
                rlt_cli::discover_defs::verify(&root, &args[2])
            } else {
                rlt_cli::discover::verify(&root, &args[2])
            };
            print!("{text}");
            code
        }
        Some("discover") if args.len() >= 4 && args[1] == "--definitions" => {
            let limit = match args.get(4).map(String::as_str) {
                None => None,
                Some("--limit") => match args.get(5).map(|v| v.parse::<usize>()) {
                    Some(Ok(n)) if args.len() == 6 => Some(n),
                    _ => {
                        eprintln!("{USAGE}");
                        return MALFORMED;
                    }
                },
                Some(_) => {
                    eprintln!("{USAGE}");
                    return MALFORMED;
                }
            };
            let (text, code) =
                rlt_cli::discover_defs::discover_as(&root, &args[2], &args[3], limit, authority);
            print!("{text}");
            code
        }
        Some("discover") if args.len() >= 2 => {
            let mut two_level = false;
            let mut limit = None;
            let mut i = 2;
            while i < args.len() {
                match (
                    args[i].as_str(),
                    args.get(i + 1).map(|v| v.parse::<usize>()),
                ) {
                    ("--two-level", _) => {
                        two_level = true;
                        i += 1;
                    }
                    ("--limit", Some(Ok(n))) => {
                        limit = Some(n);
                        i += 2;
                    }
                    _ => {
                        eprintln!("{USAGE}");
                        return MALFORMED;
                    }
                }
            }
            let (text, code) = rlt_cli::discover::discover(&root, &args[1], two_level, limit);
            print!("{text}");
            code
        }
        Some("residues") if args.len() == 2 => {
            let r = rlt_cli::residues::residues_report(&root, &root.join(&args[1]));
            print!("{}", r.text);
            r.code
        }
        Some("chem") if args.len() == 3 => {
            let r = chem_report(&root, &args[1], &args[2]);
            print!("{}", r.text);
            r.code
        }
        Some("explain") if args.len() == 2 || (args.len() == 4 && args[2] == "--format") => {
            let x = rlt_cli::explain::explain(&root, &args[1]);
            match args.get(3).map(|s| s.as_str()) {
                None | Some("text") => print!("{}", rlt_cli::explain::render_text(&x)),
                Some("json") => print!("{}", rlt_cli::explain::render_json(&x)),
                Some(_) => {
                    eprintln!("{USAGE}");
                    return MALFORMED;
                }
            }
            x.exit
        }
        Some("deps") if args.len() == 2 || (args.len() == 4 && args[2] == "--format") => {
            let json = match args.get(3).map(|s| s.as_str()) {
                None | Some("text") => false,
                Some("json") => true,
                Some(_) => {
                    eprintln!("{USAGE}");
                    return MALFORMED;
                }
            };
            if args[1] == "--verify-lib" {
                let r = rlt_cli::deps::verify_lib(&root);
                if json {
                    print!("{}", rlt_cli::deps::render_lib_json(&r));
                } else {
                    print!("{}", rlt_cli::deps::render_lib_text(&r));
                }
                r.exit
            } else {
                let d = rlt_cli::deps::deps(&root, &args[1]);
                if json {
                    print!("{}", rlt_cli::deps::render_json(&d));
                } else {
                    print!("{}", rlt_cli::deps::render_text(&d));
                }
                d.exit
            }
        }
        Some("compose") if args.len() == 2 => {
            let (text, code) = rlt_cli::compose::compose_report(&root, &args[1]);
            print!("{text}");
            code
        }
        Some("validate") if args.len() == 1 => {
            let r = validate(&root);
            print!("{}", r.text);
            r.code
        }
        _ => {
            eprintln!("{USAGE}");
            MALFORMED
        }
    };
    code
}
