//! RLT-003 M2: closed quotient–remainder witnesses modulo 30.
//!
//! Accepted cases (A), tampered witnesses (R), missing or wrong remainder
//! bounds (B), table coverage (C), altered representative sets (P), source
//! substitutions and quotation fidelity (S), assumption laundering (L), and
//! promotion of a reading to a theorem (M). Each rejected case edits a scratch
//! copy of the repository and names the problem it must produce.

use rlt_cli::residues::{classify, residues_report, READING};
use rlt_cli::{check_file, validate, Report, ACCEPT, MALFORMED, REJECT};
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    let manifest = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").to_string());
    Path::new(&manifest).parent().unwrap().to_path_buf()
}

const REC: &str = "residues/residue-30.yaml";

fn copy_dir(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap();
    for e in std::fs::read_dir(src).unwrap() {
        let p = e.unwrap().path();
        let to = dst.join(p.file_name().unwrap());
        if p.is_dir() {
            copy_dir(&p, &to);
        } else {
            std::fs::copy(&p, &to).unwrap();
        }
    }
}

/// A scratch repository with edits (path, from, to) applied; each `from` must occur.
fn scratch(name: &str, edits: &[(&str, &str, &str)]) -> PathBuf {
    let s = std::env::temp_dir().join(format!("rlt-m2-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&s);
    for dir in [
        "sources",
        "conventions",
        "correspondence",
        "readingpoints",
        "points",
        "spec",
        "tests/accept",
        "tests/m2",
        "residues",
    ] {
        copy_dir(&root().join(dir), &s.join(dir));
    }
    for (path, from, to) in edits {
        let f = s.join(path);
        let text = std::fs::read_to_string(&f).unwrap();
        assert!(
            text.contains(from),
            "fixture edit target missing in {path}: {from}"
        );
        std::fs::write(&f, text.replacen(from, to, 1)).unwrap();
    }
    s
}

fn run(name: &str, edits: &[(&str, &str, &str)]) -> Report {
    let s = scratch(name, edits);
    let r = residues_report(&s, &s.join(REC));
    let _ = std::fs::remove_dir_all(&s);
    r
}

fn refused(r: &Report, code: i32, needle: &str) {
    assert_eq!(r.code, code, "expected exit {code}:\n{}", r.text);
    assert!(
        r.text.contains(needle),
        "expected `{needle}` in:\n{}",
        r.text
    );
    assert!(
        !r.text.contains("verdict         VALID"),
        "a refused record printed VALID:\n{}",
        r.text
    );
}

fn reading_lines(r: &Report) -> Vec<&str> {
    r.text
        .lines()
        .filter(|l| l.trim_start().starts_with("reading "))
        .collect()
}

// ── Accepted ───────────────────────────────────────────────────────────────

#[test]
fn a1_the_record_is_valid_and_reports_the_registered_classification() {
    let r = residues_report(&root(), &root().join(REC));
    assert_eq!(r.code, ACCEPT, "{}", r.text);
    assert!(r.text.contains(
        "the registered classification of the 30 entries gives [1, 7, 11, 13, 17, 19, 23, 29] = the registered representatives (8 of 30) — consistency check, not a theorem"
    ));
    assert!(r.text.contains("40 record(s) valid"));
    assert!(r
        .text
        .contains("193 kernel-checked bindings over 112 distinct proof files"));
    assert_eq!(r.text.matches("\nresidue ").count(), 30);
    assert_eq!(r.text.matches("\nclass witness ").count(), 5);
}

#[test]
fn a2_kinds_of_line_are_kept_apart() {
    let r = residues_report(&root(), &root().join(REC));
    let readings = reading_lines(&r);
    assert_eq!(readings.len(), 35);
    for l in &readings {
        assert!(l.ends_with(READING), "{l}");
        assert!(
            !l.contains("ACCEPT") && !l.contains("theorem relative"),
            "{l}"
        );
    }
    for l in r
        .text
        .lines()
        .filter(|l| l.trim_start().starts_with("theorem "))
    {
        assert!(
            l.contains(
                "→ ACCEPT — unconditional theorem relative to registered axiom set PEANO-ARITH"
            ),
            "{l}"
        );
    }
    assert!(r
        .text
        .contains("not established uniqueness of remainders · non-divisibility · gcd(r, 30) = 1"));
    let coprime: Vec<&str> = readings
        .iter()
        .copied()
        .filter(|l| l.contains("coprime representative"))
        .collect();
    let others: Vec<&str> = readings
        .iter()
        .copied()
        .filter(|l| l.contains("not a coprime representative"))
        .collect();
    assert_eq!((coprime.len() - others.len(), others.len()), (8, 22));
}

#[test]
fn a3_all_111_proof_files_are_unconditional_theorems() {
    let mut files: Vec<PathBuf> = std::fs::read_dir(root().join("tests/m2"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "rlt"))
        .collect();
    files.sort();
    assert_eq!(files.len(), 111);
    for f in &files {
        let r = check_file(&root(), f);
        assert_eq!(r.code, ACCEPT, "{}", r.text);
        assert!(
            r.text
                .contains("unconditional theorem relative to registered axiom set PEANO-ARITH"),
            "{}",
            r.text
        );
        assert!(r.text.contains("\nresult kind     closed ("), "{}", r.text);
        assert!(r.text.contains("\nassumptions     Γ = ∅\n"), "{}", r.text);
    }
}

#[test]
fn a4_the_generator_reproduces_every_division_witness() {
    for r in 0..30u64 {
        for p in [2u64, 3, 5] {
            let id = format!("D_{r:02}_{p}");
            let w = format!("{r} = {p} × {} + {}", r / p, r % p);
            let g = rlt_cli::gen::generate(&root(), &w, &format!("M2.{id}")).unwrap();
            let f = std::fs::read_to_string(root().join(format!("tests/m2/{id}.rlt"))).unwrap();
            assert_eq!(g, f, "{id}");
        }
    }
}

#[test]
fn a5_validate_includes_the_witness_record() {
    let v = validate(&root());
    assert_eq!(v.code, ACCEPT, "{}", v.text);
    assert!(v
        .text
        .contains("residues        residue-30.yaml: 0 problem(s)"));
}

#[test]
fn a6_accurate_quotation_is_identified_and_leaves_its_source_unchanged() {
    let r = residues_report(&root(), &root().join(REC));
    assert!(r.text.contains(
        "quotation     \"{1, 7, 11, 13, 17, 19, 23, 29}: 17111317192329\"  [NEWS-MATH29] defines math29; readingpoints/math29.yaml is unchanged (quoted; the definition stays with its source)"
    ));
    assert!(r.text.contains(
        "references    math29 (its documented meanings stay in readingpoints/math29.yaml)"
    ));
}

#[test]
fn a7_classify_is_the_registered_rule() {
    let rt = rlt_syntax::records::load_conventions(&root().join("conventions"))
        .unwrap()
        .remove("RESIDUE-30")
        .unwrap()
        .residue
        .unwrap();
    assert!(classify(&rt, &[(2, 1), (3, 1), (5, 2)]).0);
    assert!(!classify(&rt, &[(2, 1), (3, 0), (5, 4)]).0);
}

// ── Tampered witnesses ─────────────────────────────────────────────────────

#[test]
fn r1_written_equation_differs_from_the_proof() {
    let r = run(
        "r1",
        &[(
            REC,
            "written: \"7 = 2 × 3 + 1\"",
            "written: \"7 = 2 × 2 + 3\"",
        )],
    );
    refused(
        &r,
        REJECT,
        "residue 7 witness for 2: record states `7 = (2 × 2) + 3`, proof checks `7 = (2 × 3) + 1`",
    );
}

#[test]
fn r2_proof_of_another_residue() {
    let r = run(
        "r2",
        &[(
            REC,
            "written: \"7 = 2 × 3 + 1\", proof: tests/m2/D_07_2.rlt",
            "written: \"8 = 2 × 4 + 0\", proof: tests/m2/D_08_2.rlt",
        )],
    );
    refused(&r, REJECT, "must have the form 7 = 2 × q + s");
}

#[test]
fn r3_tampered_proof_step() {
    let s = scratch("r3", &[]);
    let f = s.join("tests/m2/D_07_2.rlt");
    let t = std::fs::read_to_string(&f).unwrap();
    // Tamper the final step's justification: symm of a step proving 6 + 1 = 7.
    let bad = t.replace("(symm s44))))", "(symm s43))))");
    assert_ne!(bad, t);
    std::fs::write(&f, bad).unwrap();
    let r = residues_report(&s, &s.join(REC));
    let _ = std::fs::remove_dir_all(&s);
    assert_ne!(r.code, ACCEPT, "{}", r.text);
    assert!(r.text.contains("residue 7 witness for 2"), "{}", r.text);
    assert!(
        !r.text.contains("remainders 1, 1, 2 (all nonzero)"),
        "{}",
        r.text
    );
}

#[test]
fn r4_false_division_cannot_be_generated() {
    assert!(rlt_cli::gen::generate(&root(), "7 = 2 × 3 + 2", "X").is_err());
}

#[test]
fn r5_tampered_class_witness() {
    let r = run(
        "r5",
        &[(
            REC,
            "written: \"37 = 30 × 1 + 7\"",
            "written: \"37 = 30 × 1 + 8\"",
        )],
    );
    refused(&r, REJECT, "class witness 37");
}

// ── Remainder bounds ───────────────────────────────────────────────────────

#[test]
fn b1_missing_bound_is_malformed() {
    let r = run("b1", &[(REC, ", bound: \"1 + (0 + 1) = 2\", bound_proof: tests/m2/B_1_2.rlt}\n      - {p: 3, written: \"7 = 3 × 2 + 1\"", "}\n      - {p: 3, written: \"7 = 3 × 2 + 1\"")]);
    refused(&r, MALFORMED, "bound");
}

#[test]
fn b2_bound_for_another_remainder() {
    let r = run("b2", &[(REC, "written: \"7 = 5 × 1 + 2\", proof: tests/m2/D_07_5.rlt, bound: \"2 + (2 + 1) = 5\", bound_proof: tests/m2/B_2_5.rlt", "written: \"7 = 5 × 1 + 2\", proof: tests/m2/D_07_5.rlt, bound: \"1 + (3 + 1) = 5\", bound_proof: tests/m2/B_1_5.rlt")]);
    refused(&r, REJECT, "remainder 2 has no bound below 5");
}

#[test]
fn b3_remainder_at_least_the_divisor_has_no_bound() {
    // 7 = 2 × 2 + 3 is a true closed equation, but 3 is not a remainder by 2.
    let s = scratch("b3", &[]);
    let g = rlt_cli::gen::generate(&s, "7 = 2 × 2 + 3", "M2.bad").unwrap();
    std::fs::write(s.join("tests/m2/bad.rlt"), g).unwrap();
    let g = rlt_cli::gen::generate(&s, "3 + (0 + 1) = 4", "M2.bad_bound").unwrap();
    std::fs::write(s.join("tests/m2/bad_bound.rlt"), g).unwrap();
    let f = s.join(REC);
    let t = std::fs::read_to_string(&f).unwrap().replacen(
        "written: \"7 = 2 × 3 + 1\", proof: tests/m2/D_07_2.rlt, bound: \"1 + (0 + 1) = 2\", bound_proof: tests/m2/B_1_2.rlt",
        "written: \"7 = 2 × 2 + 3\", proof: tests/m2/bad.rlt, bound: \"3 + (0 + 1) = 4\", bound_proof: tests/m2/bad_bound.rlt",
        1,
    );
    std::fs::write(&f, t).unwrap();
    let r = residues_report(&s, &f);
    let _ = std::fs::remove_dir_all(&s);
    refused(&r, REJECT, "remainder 3 has no bound below 2");
}

#[test]
fn b4_bound_proof_that_proves_something_else() {
    let r = run("b4", &[(REC, "bound: \"1 + (0 + 1) = 2\", bound_proof: tests/m2/B_1_2.rlt}\n      - {p: 3, written: \"7 = 3 × 2 + 1\"", "bound: \"1 + (0 + 1) = 2\", bound_proof: tests/m2/B_0_2.rlt}\n      - {p: 3, written: \"7 = 3 × 2 + 1\"")]);
    refused(&r, REJECT, "residue 7 witness for 2 bound");
}

// ── Coverage ───────────────────────────────────────────────────────────────

#[test]
fn c1_duplicate_residue() {
    let r = run("c1", &[(REC, "  - r: 8\n", "  - r: 7\n")]);
    refused(&r, REJECT, "residue 7 is listed twice");
    assert!(r.text.contains("consistency   not reported"), "{}", r.text);
}

#[test]
fn c2_missing_residue() {
    let s = scratch("c2", &[]);
    let f = s.join(REC);
    let t = std::fs::read_to_string(&f).unwrap();
    let start = t.find("  - r: 29\n").unwrap();
    let end = t.find("\nclasses:").unwrap();
    std::fs::write(&f, format!("{}{}", &t[..start], &t[end..])).unwrap();
    let r = residues_report(&s, &f);
    let _ = std::fs::remove_dir_all(&s);
    refused(&r, REJECT, "residues must cover 0 to 29 exactly once each");
    assert!(r.text.contains("consistency   not reported"), "{}", r.text);
}

#[test]
fn c3_residue_out_of_range() {
    let r = run("c3", &[(REC, "  - r: 0\n", "  - r: 30\n")]);
    refused(&r, REJECT, "residue 30 is outside 0 to 29");
}

#[test]
fn c4_missing_and_extra_witnesses() {
    let r = run(
        "c4",
        &[(
            REC,
            "      - {p: 5, written: \"7 = 5 × 1 + 2\"",
            "      - {p: 3, written: \"7 = 5 × 1 + 2\"",
        )],
    );
    refused(&r, REJECT, "residue 7: missing witness for 5");
    assert!(
        r.text.contains("residue 7: 2 witnesses for 3"),
        "{}",
        r.text
    );
}

// ── Representative sets ────────────────────────────────────────────────────

#[test]
fn p1_altered_convention_representatives_break_the_pin() {
    let s = scratch(
        "p1",
        &[(
            "conventions/residue-30.yaml",
            "representatives: [1, 7, 11, 13, 17, 19, 23, 29]",
            "representatives: [1, 7, 11, 13, 17, 19, 23, 31]",
        )],
    );
    let r = residues_report(&s, &s.join(REC));
    let v = validate(&s);
    let _ = std::fs::remove_dir_all(&s);
    assert_ne!(r.code, ACCEPT, "{}", r.text);
    assert!(
        r.text
            .contains("differ from the convention pinned in this binary"),
        "{}",
        r.text
    );
    assert_ne!(v.code, ACCEPT, "{}", v.text);
}

#[test]
fn p2_altered_representatives_differ_from_the_quote_and_the_table() {
    // Bypass the pin: check a convention value directly.
    let mut convs = rlt_syntax::records::load_conventions(&root().join("conventions")).unwrap();
    let mut c = convs.remove("RESIDUE-30").unwrap();
    c.residue.as_mut().unwrap().representatives = vec![1, 7, 11, 13, 17, 19, 23];
    let reg = rlt_cli::pinned_registry(&root()).unwrap();
    let p = rlt_cli::residues::residue_table_problems(&root(), &c, &reg);
    assert!(
        p.iter().any(|x| x.contains("differ from the quoted set")),
        "{p:?}"
    );
}

#[test]
fn p3_tally_binds_an_altered_set() {
    let r = run(
        "p3",
        &[(
            REC,
            "binds: [1, 7, 11, 13, 17, 19, 23, 29]",
            "binds: [1, 7, 11, 13, 17, 19, 23, 31]",
        )],
    );
    refused(&r, REJECT, "one summand per registered representative");
}

#[test]
fn p4_tally_binds_a_representative_twice() {
    let r = run(
        "p4",
        &[(
            REC,
            "binds: [1, 7, 11, 13, 17, 19, 23, 29]",
            "binds: [1, 7, 7, 13, 17, 19, 23, 29]",
        )],
    );
    refused(&r, REJECT, "binding each once");
}

#[test]
fn p5_rule_cannot_be_replaced() {
    let s = scratch(
        "p5",
        &[(
            "conventions/residue-30.yaml",
            "rule: nonzero-remainders",
            "rule: gcd-one",
        )],
    );
    let r = residues_report(&s, &s.join(REC));
    let _ = std::fs::remove_dir_all(&s);
    assert_ne!(r.code, ACCEPT, "{}", r.text);
    assert!(
        r.text
            .contains("rule `gcd-one` is outside the registered rule nonzero-remainders"),
        "{}",
        r.text
    );
}

// ── Sources and quotations ─────────────────────────────────────────────────

#[test]
fn s1_basis_source_substituted() {
    let r = run(
        "s1",
        &[(REC, "source: RLT-003-M2-APPROVAL}", "source: NEWS-MATH29}")],
    );
    refused(&r, REJECT, "basis \"The eight representatives");
}

#[test]
fn s2_quotation_source_substituted() {
    let r = run(
        "s2",
        &[(
            REC,
            "17111317192329\", source: NEWS-MATH29}",
            "17111317192329\", source: RLT-INDEX}",
        )],
    );
    refused(&r, REJECT, "is not a whole unit of `RLT-INDEX`");
}

#[test]
fn s3_inaccurate_quotation() {
    let r = run(
        "s3",
        &[(
            REC,
            "  - {text: \"{1, 7, 11, 13, 17, 19, 23, 29}: 17111317192329\"",
            "  - {text: \"{1, 7, 11, 13, 17, 19, 23}: 17111317192329\"",
        )],
    );
    refused(&r, REJECT, "quotation \"{1, 7, 11, 13, 17, 19, 23}");
}

#[test]
fn s4_unregistered_source_and_unknown_reference() {
    let r = run(
        "s4",
        &[(REC, "references: [math29]", "references: [math30]")],
    );
    refused(&r, REJECT, "references unknown reading point `math30`");
    let r = run(
        "s4b",
        &[(REC, "source: RLT-003-M2-APPROVAL}", "source: GDT-INVENTED}")],
    );
    refused(&r, REJECT, "basis source `GDT-INVENTED`");
}

#[test]
fn s5_quotation_cannot_redefine_its_source() {
    // A quotation key that would carry a definition is refused, and the
    // reading point is byte-identical to its committed form.
    let r = run(
        "s5",
        &[(
            REC,
            "17111317192329\", source: NEWS-MATH29}",
            "17111317192329\", source: NEWS-MATH29, defines: residue-30}",
        )],
    );
    assert_eq!(r.code, MALFORMED, "{}", r.text);
    let math29 = std::fs::read(root().join("readingpoints/math29.yaml")).unwrap();
    let digest = sha256(&math29);
    assert_eq!(digest, MATH29_SHA, "readingpoints/math29.yaml changed");
}

#[test]
fn s6_convention_quote_source_substituted_breaks_the_pin() {
    let s = scratch(
        "s6",
        &[(
            "conventions/residue-30.yaml",
            "    source: NEWS-MATH29\n  # A residue",
            "    source: RLT-INDEX\n  # A residue",
        )],
    );
    let r = residues_report(&s, &s.join(REC));
    let _ = std::fs::remove_dir_all(&s);
    refused(
        &r,
        MALFORMED,
        "differ from the convention pinned in this binary",
    );
}

// ── Assumption laundering ──────────────────────────────────────────────────

#[test]
fn l1_conditional_proof_cannot_serve_as_a_witness() {
    let r = run(
        "l1",
        &[(
            REC,
            "written: \"7 = 2 × 3 + 1\", proof: tests/m2/D_07_2.rlt",
            "written: \"2 × 3 + 1 = 8\", proof: tests/accept/M1_conditional.rlt",
        )],
    );
    refused(
        &r,
        REJECT,
        "residue 7 witness for 2: record expects mode unconditional, proof requests conditional",
    );
    assert!(
        !r.text.contains("remainders 1, 1, 2 (all nonzero)"),
        "{}",
        r.text
    );
}

#[test]
fn l2_records_cannot_declare_assumptions_or_modes() {
    for (k, v) in [
        ("assumptions", "[h]"),
        ("mode", "conditional"),
        ("convention", "NAT-DEC"),
    ] {
        let r = run(
            &format!("l2-{k}"),
            &[(
                REC,
                "{p: 2, written: \"7 = 2 × 3 + 1\"",
                &format!("{{p: 2, {k}: {v}, written: \"7 = 2 × 3 + 1\""),
            )],
        );
        assert_eq!(r.code, MALFORMED, "{k}: {}", r.text);
    }
}

#[test]
fn l3_proof_outside_the_repository_is_refused() {
    let r = run(
        "l3",
        &[(REC, "proof: tests/m2/D_07_2.rlt", "proof: ../D_07_2.rlt")],
    );
    refused(
        &r,
        REJECT,
        "proof path `../D_07_2.rlt` must be a tests/…/*.rlt file",
    );
}

// ── Promotion of a reading ─────────────────────────────────────────────────

#[test]
fn m1_a_record_cannot_mark_a_reading_as_a_theorem() {
    for (from, to) in [
        (
            "references: [math29]",
            "references: [math29]\nstatus: theorem",
        ),
        (
            "references: [math29]",
            "references: [math29]\ntheorem: \"gcd(7, 30) = 1\"",
        ),
        ("  - r: 7\n", "  - r: 7\n    reading: theorem\n"),
    ] {
        let r = run("m1", &[(REC, from, to)]);
        assert_eq!(r.code, MALFORMED, "{to}: {}", r.text);
    }
}

#[test]
fn m2_a_residues_file_is_not_a_proof_file() {
    let r = check_file(&root(), &root().join(REC));
    assert_ne!(r.code, ACCEPT, "{}", r.text);
}

#[test]
fn m3_the_classification_is_never_reported_as_a_theorem() {
    let r = residues_report(&root(), &root().join(REC));
    for l in r.text.lines() {
        let t = l.trim_start();
        if (t.starts_with("reading ") || t.starts_with("consistency ") || t.starts_with("binding "))
            && (l.contains("classification")
                || l.contains("coprime")
                || l.contains("residue class")
                || l.contains("representative"))
        {
            assert!(!l.contains("ACCEPT"), "{l}");
            assert!(
                l.contains("not a theorem") || l.contains("counts the registered representatives"),
                "{l}"
            );
        }
    }
    assert!(r
        .text
        .contains("readings are registered interpretations, not theorems"));
}

#[test]
fn m4_residues_file_must_be_in_residues_dir() {
    let s = scratch("m4", &[]);
    std::fs::copy(s.join(REC), s.join("points/residue-30.yaml")).unwrap();
    let r = residues_report(&s, &s.join("points/residue-30.yaml"));
    let _ = std::fs::remove_dir_all(&s);
    assert_eq!(r.code, MALFORMED, "{}", r.text);
}

// ── helpers ────────────────────────────────────────────────────────────────

const MATH29_SHA: &str = "828ca404210423cc3a2f8f82aa34ee6f193326232182e959432205e7a99cb1d3";

fn sha256(b: &[u8]) -> String {
    use std::process::Command;
    let tmp = std::env::temp_dir().join(format!("rlt-m2-sha-{}", std::process::id()));
    std::fs::write(&tmp, b).unwrap();
    let out = Command::new("sha256sum").arg(&tmp).output().unwrap();
    let _ = std::fs::remove_file(&tmp);
    String::from_utf8(out.stdout)
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap()
        .to_string()
}

#[test]
fn a8_report_matches_its_golden_file() {
    let r = residues_report(&root(), &root().join(REC));
    let text = format!("{}exit {}\n", r.text, r.code);
    let f = root().join("tests/expected/residues-30.out");
    if std::env::var("UPDATE_GOLDEN").is_ok() {
        std::fs::write(&f, &text).unwrap();
    }
    assert_eq!(text, std::fs::read_to_string(&f).unwrap());
}
