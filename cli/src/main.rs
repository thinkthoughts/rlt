use rlt_cli::{
    check_file, chem_report, lean_export, points_report, rp_report, validate, written_report,
    MALFORMED,
};
use std::path::PathBuf;

const USAGE: &str = "\
usage:
  rlt check FILE...                      replay proof files (exit 0 accept, 1 reject, 2 malformed)
  rlt written A CONV_A B CONV_B          written identity and elaboration of two expressions
  rlt rp ID                              a reading point: meanings, relationships, correspondence
  rlt validate                           consistency of sources, conventions and records
  rlt lean FILE...                       Lean 4 replay of the accepted statements
  rlt points FILE                        specification records: reading points, counts, relations
  rlt chem WRITTEN CONVENTION            how a species-table convention reads a written formula
  rlt gen EQUATION ID                    untrusted generator: a PEANO-ARITH proof file for a closed equation
  rlt residues FILE                      closed quotient–remainder witnesses and their registered readings
  rlt explain RECORD [--format text|json] a statement record: kernel, source, semantic, physical
                                         (exit 0 current+accept, 1 reject, 2 invalid/malformed, 3 stale)
options:
  --root DIR                             repository root (default: current directory)";

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
    let code = match args.first().map(|s| s.as_str()) {
        Some("check") if args.len() >= 2 => {
            let mut worst = 0;
            for (i, f) in args[1..].iter().enumerate() {
                if i > 0 {
                    println!();
                }
                let r = check_file(&root, &root.join(f));
                print!("{}", r.text);
                worst = worst.max(r.code);
            }
            worst
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
