//! Input hardening before the M2 merge (audit at 11e7be7, inherited issues):
//! H1 exponential YAML expansion, H2 symbolic links on proof paths, H3
//! non-decimal numerals. Each case is reproducible on a scratch copy; H0
//! checks that every committed record keeps exactly its meaning.

use rlt_cli::residues::residues_report;
use rlt_cli::{validate, ACCEPT, MALFORMED, REJECT};
use rlt_syntax::records::{decimal_integer_spelling, load_yaml};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use yaml_rust2::YamlLoader;

fn root() -> PathBuf {
    let manifest = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").to_string());
    Path::new(&manifest).parent().unwrap().to_path_buf()
}

const DIRS: [&str; 8] = [
    "sources",
    "conventions",
    "correspondence",
    "readingpoints",
    "points",
    "spec",
    "residues",
    "tests",
];

fn copy_dir(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap();
    for e in std::fs::read_dir(src).unwrap() {
        let p = e.unwrap().path();
        // RLT-006 generated certificates are never copied into scratch trees.
        if p.ends_with("tests/rlt006/gen")
            || p.ends_with("tests/rlt007s1/gen")
            || p.ends_with("tests/rlt007s4/gen")
        {
            continue;
        }
        let to = dst.join(p.file_name().unwrap());
        if p.is_dir() {
            copy_dir(&p, &to);
        } else {
            std::fs::copy(&p, &to).unwrap();
        }
    }
}

fn scratch(name: &str) -> PathBuf {
    let s = std::env::temp_dir().join(format!("rlt-h-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&s);
    for d in DIRS {
        copy_dir(&root().join(d), &s.join(d));
    }
    s
}

fn edit(s: &Path, path: &str, from: &str, to: &str) {
    let f = s.join(path);
    let t = std::fs::read_to_string(&f).unwrap();
    assert!(
        t.contains(from),
        "fixture edit target missing in {path}: {from}"
    );
    std::fs::write(&f, t.replacen(from, to, 1)).unwrap();
}

const REC: &str = "residues/residue-30.yaml";

// ── H0: valid inputs keep their meanings ───────────────────────────────────

/// Every committed YAML record loads, and the hardened loader returns the
/// same tree as the unhardened YAML loader.
#[test]
fn h0_every_committed_record_keeps_its_meaning() {
    let mut n = 0;
    let mut stack: Vec<PathBuf> = DIRS[..7].iter().map(|d| root().join(d)).collect();
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "yaml") {
                let src = std::fs::read_to_string(&p).unwrap();
                let plain = YamlLoader::load_from_str(&src).unwrap().remove(0);
                let hardened = load_yaml(&p).unwrap_or_else(|e| panic!("{e}"));
                assert_eq!(plain, hardened, "{}", p.display());
                n += 1;
            }
        }
    }
    assert!(n >= 15, "only {n} records found");
    assert_eq!(validate(&root()).code, ACCEPT);
}

// ── H1: exponential expansion ──────────────────────────────────────────────

fn laughs() -> String {
    let mut y = String::from(
        "a: &a [\"lol\", \"lol\", \"lol\", \"lol\", \"lol\", \"lol\", \"lol\", \"lol\", \"lol\"]\n",
    );
    for (i, c) in "bcdefghijklmnopqrstuvwxyz".chars().enumerate() {
        let prev = (b'a' + i as u8) as char;
        y.push_str(&format!(
            "{c}: &{c} [*{prev}, *{prev}, *{prev}, *{prev}, *{prev}, *{prev}, *{prev}, *{prev}, *{prev}]\n"
        ));
    }
    y
}

#[test]
fn h1_alias_expansion_is_refused_quickly_in_every_record_family() {
    let s = scratch("h1");
    for path in [
        "residues/lol.yaml",
        "points/lol.yaml",
        "readingpoints/lol.yaml",
        "conventions/lol.yaml",
    ] {
        std::fs::write(s.join(path), laughs()).unwrap();
        let t = Instant::now();
        let e = load_yaml(&s.join(path)).unwrap_err();
        assert!(t.elapsed() < Duration::from_secs(2), "{path}: slow");
        assert!(
            e.contains("YAML aliases are refused") || e.contains("YAML anchors are refused"),
            "{e}"
        );
    }
    let t = Instant::now();
    let r = residues_report(&s, &s.join("residues/lol.yaml"));
    assert_eq!(r.code, MALFORMED, "{}", r.text);
    let v = validate(&s);
    assert!(t.elapsed() < Duration::from_secs(30));
    assert_ne!(v.code, ACCEPT, "{}", v.text);
    let _ = std::fs::remove_dir_all(&s);
}

#[test]
fn h1_a_single_anchor_or_alias_is_refused() {
    let s = scratch("h1b");
    edit(
        &s,
        REC,
        "references: [math29]",
        "references: &refs [math29]",
    );
    let r = residues_report(&s, &s.join(REC));
    assert_eq!(r.code, MALFORMED, "{}", r.text);
    assert!(r.text.contains("YAML anchors are refused"), "{}", r.text);
    let _ = std::fs::remove_dir_all(&s);
}

#[test]
fn h1_tags_cannot_retype_a_value() {
    let s = scratch("h1c");
    edit(&s, REC, "  - r: 7\n", "  - r: !!int \"7\"\n");
    let r = residues_report(&s, &s.join(REC));
    assert_eq!(r.code, MALFORMED, "{}", r.text);
    assert!(r.text.contains("YAML tags are refused"), "{}", r.text);
    let _ = std::fs::remove_dir_all(&s);
}

// ── H2: symbolic links on proof paths ──────────────────────────────────────

#[cfg(unix)]
#[test]
fn h2_symlinked_tests_directory_is_refused() {
    let s = scratch("h2");
    let outside = std::env::temp_dir().join(format!("rlt-h2-outside-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&outside);
    std::fs::rename(s.join("tests"), &outside).unwrap();
    std::os::unix::fs::symlink(&outside, s.join("tests")).unwrap();
    let r = residues_report(&s, &s.join(REC));
    assert_eq!(r.code, REJECT, "{}", r.text);
    assert!(
        r.text.contains("is a symbolic link or lies under one"),
        "{}",
        r.text
    );
    assert!(!r.text.contains("verdict         VALID"), "{}", r.text);
    // Reading-point relationships use the same binding rule.
    let v = validate(&s);
    assert_ne!(v.code, ACCEPT, "{}", v.text);
    assert!(
        v.text.contains("is a symbolic link or lies under one"),
        "{}",
        v.text
    );
    let _ = std::fs::remove_dir_all(&s);
    let _ = std::fs::remove_dir_all(&outside);
}

#[cfg(unix)]
#[test]
fn h2_symlinked_subdirectory_and_file_stay_refused() {
    let s = scratch("h2b");
    std::fs::rename(s.join("tests/m2"), s.join("tests/m2-real")).unwrap();
    std::os::unix::fs::symlink(s.join("tests/m2-real"), s.join("tests/m2")).unwrap();
    let r = residues_report(&s, &s.join(REC));
    assert!(
        r.text.contains("is a symbolic link or lies under one"),
        "{}",
        r.text
    );
    assert_ne!(r.code, ACCEPT);
    let _ = std::fs::remove_dir_all(&s);
}

// ── H3: decimal numerals ───────────────────────────────────────────────────

#[test]
fn h3_non_decimal_numerals_are_refused() {
    for spelled in ["0x07", "0o7", "+7", "07", "-07", "0x0"] {
        let s = scratch("h3");
        edit(&s, REC, "  - r: 7\n", &format!("  - r: {spelled}\n"));
        let r = residues_report(&s, &s.join(REC));
        assert_eq!(r.code, MALFORMED, "{spelled}: {}", r.text);
        assert!(
            r.text
                .contains(&format!("numeral `{spelled}` is not decimal")),
            "{}",
            r.text
        );
        let _ = std::fs::remove_dir_all(&s);
    }
}

#[test]
fn h3_decimal_spelling_rule() {
    for ok in ["0", "7", "29", "1000", "-1", "-30"] {
        assert!(decimal_integer_spelling(ok), "{ok}");
    }
    for bad in ["07", "+7", "0x07", "0o7", "-0", "-07", "", "-", "7_0"] {
        assert!(!decimal_integer_spelling(bad), "{bad}");
    }
}

#[test]
fn h3_quoted_hex_text_stays_text() {
    // A quoted string is a string: it is refused where a numeral is required
    // (as before) and is never read as a number.
    let s = scratch("h3b");
    edit(&s, REC, "  - r: 7\n", "  - r: \"0x07\"\n");
    let r = residues_report(&s, &s.join(REC));
    assert_eq!(r.code, MALFORMED, "{}", r.text);
    assert!(r.text.contains("must be a numeral 0..1000"), "{}", r.text);
    let _ = std::fs::remove_dir_all(&s);
}
