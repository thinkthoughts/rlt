//! RLT-004b: statement records and `rlt explain`.
//!
//! - X01–X04: the records in statements/ (exit 0), with golden text and JSON
//!   for the Modulo 30 record.
//! - tests/explain/expected.tsv: invalid (2), stale (3), proof-failure (1, 2)
//!   and assessment-state (0) fixtures, each with a pinned report line.
//! - Properties: the kernel block is `rlt check`'s result for the same bytes;
//!   semantic and physical assessments never change kernel fields or exit;
//!   JSON parses back to the same values; output is deterministic;
//!   malformed inputs still yield a valid report.

use rlt_cli::explain::{explain, json_string, json_write, render_json, render_text, Json, STALE};
use rlt_cli::{check_file, ACCEPT, MALFORMED, REJECT};
use std::path::{Path, PathBuf};
use yaml_rust2::{Yaml, YamlLoader};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn records() -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(root().join("statements"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .filter(|n| n.ends_with(".yaml"))
        .map(|n| format!("statements/{n}"))
        .collect();
    v.sort();
    v
}

fn parse_json(s: &str) -> Yaml {
    let mut docs = YamlLoader::load_from_str(s).expect("JSON output parses");
    assert_eq!(docs.len(), 1);
    docs.remove(0)
}

/// A scratch directory inside the repository (records must be repository-relative).
fn scratch(name: &str) -> (PathBuf, String) {
    let rel = format!("target/explain-scratch/{name}");
    let dir = root().join(&rel);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    (dir, rel)
}

#[test]
fn x01_to_x04_records_are_valid_current_and_accepted() {
    let rs = records();
    assert_eq!(
        rs,
        [
            "statements/CONDITIONAL-2X3.yaml",
            "statements/DVD-2-8.yaml",
            "statements/MATH29-PAIRING.yaml",
            "statements/MOD30-CLASSIFICATION.yaml"
        ]
    );
    for r in rs {
        let x = explain(&root(), &r);
        assert_eq!(x.exit, ACCEPT, "{r}:\n{}", render_text(&x));
        assert_eq!(x.state, "current");
        assert_eq!(x.kernel.as_ref().unwrap().verdict, "ACCEPT");
    }
}

#[test]
fn x01_golden_text_and_json() {
    let x = explain(&root(), "statements/MOD30-CLASSIFICATION.yaml");
    let golden =
        |f: &str| std::fs::read_to_string(root().join("tests/explain-golden").join(f)).unwrap();
    assert_eq!(render_text(&x), golden("MOD30-CLASSIFICATION.txt"));
    assert_eq!(render_json(&x), golden("MOD30-CLASSIFICATION.json"));
}

#[test]
fn x02_x03_x04_sections() {
    let d = explain(&root(), "statements/DVD-2-8.yaml");
    assert_eq!(d.semantic.as_ref().unwrap().status, "unassessed");
    assert_eq!(d.semantic.as_ref().unwrap().assessment, "none");
    assert_eq!(d.physical.as_ref().unwrap().applicability, "not-applicable");
    assert!(d.physical.as_ref().unwrap().entries.is_empty());

    let m = explain(&root(), "statements/MATH29-PAIRING.yaml");
    let p = m.physical.as_ref().unwrap();
    assert_eq!(p.applicability, "relevant");
    assert_eq!(p.entries.len(), 1);
    assert_eq!(p.entries[0].id, "math29.context");
    assert_eq!(p.entries[0].status, "unestablished");
    assert_eq!(p.entries[0].evidence, 0);
    assert!(
        m.source.as_ref().unwrap().commit.is_none(),
        "conversation source is unpinned"
    );

    let c = explain(&root(), "statements/CONDITIONAL-2X3.yaml");
    assert_eq!(
        c.kernel.as_ref().unwrap().assumptions.as_deref(),
        Some("Γ = { h : 2 × 3 = 7 }")
    );
    assert!(render_text(&c).contains("  assumptions   Γ = { h : 2 × 3 = 7 }"));
}

#[test]
fn fixtures_match_expected_exit_and_line() {
    let tsv = std::fs::read_to_string(root().join("tests/explain/expected.tsv")).unwrap();
    let mut n = 0;
    for l in tsv.lines().filter(|l| !l.starts_with('#') && !l.is_empty()) {
        let f: Vec<&str> = l.splitn(3, '\t').collect();
        let x = explain(&root(), &format!("tests/explain/{}", f[0]));
        let text = render_text(&x);
        assert_eq!(x.exit.to_string(), f[1], "{}:\n{text}", f[0]);
        assert!(
            text.contains(f[2]),
            "{}: missing `{}` in\n{text}",
            f[0],
            f[2]
        );
        let j = parse_json(&render_json(&x));
        assert_eq!(j["exit"].as_i64(), Some(x.exit as i64), "{}", f[0]);
        n += 1;
    }
    assert_eq!(n, 44);
}

#[test]
fn xr05_symlinked_proof_path_is_invalid() {
    let (dir, rel) = scratch("xr05");
    #[cfg(unix)]
    std::os::unix::fs::symlink(
        root().join("tests/m3c/accept/CA01_two_divides_eight.rlt"),
        dir.join("link.rlt"),
    )
    .unwrap();
    let base = std::fs::read_to_string(root().join("statements/DVD-2-8.yaml")).unwrap();
    let rec = base.replace("id: DVD-2-8", "id: XR05-SYMLINK").replace(
        "tests/m3c/accept/CA01_two_divides_eight.rlt",
        &format!("{rel}/link.rlt"),
    );
    std::fs::write(dir.join("XR05-SYMLINK.yaml"), rec).unwrap();
    let x = explain(&root(), &format!("{rel}/XR05-SYMLINK.yaml"));
    assert_eq!(x.exit, MALFORMED);
    assert!(x.problems[0].contains("symlink"), "{:?}", x.problems);
    // A symlinked record is refused too.
    std::os::unix::fs::symlink(
        root().join("statements/DVD-2-8.yaml"),
        dir.join("DVD-2-8.yaml"),
    )
    .unwrap();
    let y = explain(&root(), &format!("{rel}/DVD-2-8.yaml"));
    assert_eq!(y.exit, MALFORMED);
    assert!(y.problems[0].contains("symbolic link"), "{:?}", y.problems);
}

/// The kernel block is `rlt check`'s result for the same file, line for line.
#[test]
fn kernel_block_equals_rlt_check() {
    let tsv = std::fs::read_to_string(root().join("tests/explain/expected.tsv")).unwrap();
    let mut recs: Vec<String> = records();
    recs.extend(
        tsv.lines()
            .filter(|l| l.starts_with("XS") || l.starts_with("XF") || l.starts_with("XA"))
            .map(|l| format!("tests/explain/{}", l.split('\t').next().unwrap())),
    );
    for r in recs {
        let x = explain(&root(), &r);
        let k = x.kernel.as_ref().unwrap();
        let rep = check_file(&root(), &root().join(&k.proof));
        assert_eq!(k.check_report, rep.text, "{r}");
        assert_eq!(k.exit, rep.code, "{r}");
        let get = |label: &str| {
            rep.text
                .lines()
                .find(|l| l.starts_with(label) && l[..16].trim_end() == label)
                .map(|l| l[16..].to_string())
        };
        assert_eq!(k.checked_goal, get("checked goal"), "{r}");
        assert_eq!(k.written_goal, get("written goal"), "{r}");
        assert_eq!(k.proved, get("proved"), "{r}");
        assert_eq!(k.assumptions, get("assumptions"), "{r}");
        assert_eq!(k.axioms_used, get("axioms used"), "{r}");
        assert_eq!(k.rules_used, get("rules used"), "{r}");
        assert_eq!(Some(k.verdict_line.clone()), get("verdict"), "{r}");
        let bytes = std::fs::read(root().join(&k.proof)).unwrap();
        use sha2::{Digest, Sha256};
        let h: String = Sha256::digest(&bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert_eq!(k.sha256, h, "{r}");
        // The exit never reports acceptance the checker did not give.
        if x.exit == ACCEPT || x.exit == STALE {
            assert_eq!(rep.code, ACCEPT, "{r}");
        }
        if rep.code == REJECT {
            assert_eq!(x.exit, REJECT, "{r}");
        }
    }
}

/// Rewriting a record's semantic status through every value and its physical
/// applicability changes no kernel field and no exit status.
#[test]
fn assessments_never_change_kernel_fields_or_exit() {
    let (dir, rel) = scratch("independence");
    let base = std::fs::read_to_string(root().join("statements/DVD-2-8.yaml")).unwrap();
    let reference = explain(&root(), "statements/DVD-2-8.yaml");
    let attributed = "  assessed_by: \"a reviewer\"\n  date: 2026-11-11\n  basis: {path: docs/STATEMENT-RECORDS.md, sha256: ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff}\n  assessed_goal: \"∃k. 8 = 2 × k\"\n";
    let semantics = [
        "semantic:\n  status: unassessed\n".to_string(),
        format!("semantic:\n  status: matches\n{attributed}  matches: [\"m\"]\n"),
        format!(
            "semantic:\n  status: partial\n{attributed}  matches: [\"m\"]\n  differs: [\"d\"]\n"
        ),
        format!("semantic:\n  status: differs\n{attributed}  differs: [\"d\"]\n"),
    ];
    let physicals = [
        "physical:\n  applicability: not-applicable\n  reason: \"r\"\n",
        "physical:\n  applicability: relevant\n  reading_point: math29\n  correspondence: [math29.context]\n",
    ];
    let head = &base[..base.find("semantic:").unwrap()];
    let mut n = 0;
    for (i, s) in semantics.iter().enumerate() {
        for (j, p) in physicals.iter().enumerate() {
            let id = format!("IND-{i}-{j}");
            let text = format!(
                "{}{s}{p}",
                head.replace("id: DVD-2-8", &format!("id: {id}"))
            );
            std::fs::write(dir.join(format!("{id}.yaml")), text).unwrap();
            let x = explain(&root(), &format!("{rel}/{id}.yaml"));
            assert_eq!(x.exit, reference.exit, "{id}:\n{}", render_text(&x));
            assert_eq!(x.state, reference.state, "{id}");
            let (a, b) = (
                x.kernel.as_ref().unwrap(),
                reference.kernel.as_ref().unwrap(),
            );
            assert_eq!(a.check_report, b.check_report, "{id}");
            assert_eq!(a.verdict_line, b.verdict_line, "{id}");
            assert_eq!(a.checked_goal, b.checked_goal, "{id}");
            n += 1;
        }
    }
    assert_eq!(n, 8);
}

/// A stale assessment is visible in text and JSON, and the exit stays the checker's.
#[test]
fn stale_assessment_is_visible_and_separate_from_stale_records() {
    for f in ["XA01-ASSESSED-GOAL-DIFFERS", "XA02-BASIS-SHA-DIFFERS"] {
        let x = explain(&root(), &format!("tests/explain/{f}.yaml"));
        assert_eq!(x.exit, ACCEPT, "{f}");
        assert_eq!(
            x.state, "current",
            "{f}: a stale assessment is not a stale record"
        );
        assert!(render_text(&x).contains("assessment stale"), "{f}");
        let j = parse_json(&render_json(&x));
        assert_eq!(j["semantic"]["assessment"].as_str(), Some("stale"), "{f}");
        assert_eq!(j["record"]["state"].as_str(), Some("current"), "{f}");
        assert!(
            !j["semantic"]["stale_reasons"].as_vec().unwrap().is_empty(),
            "{f}"
        );
    }
    let s = explain(&root(), "tests/explain/XS01-PROOF-SHA.yaml");
    let j = parse_json(&render_json(&s));
    assert_eq!(j["record"]["state"].as_str(), Some("stale"));
    assert_eq!(j["semantic"]["assessment"].as_str(), Some("none"));
}

/// Registry fidelity and upstream verification are separate fields.
#[test]
fn registry_fidelity_is_distinct_from_upstream_verification() {
    let x = explain(&root(), "statements/MOD30-CLASSIFICATION.yaml");
    let j = parse_json(&render_json(&x));
    assert_eq!(j["source"]["registry_fidelity"].as_str(), Some("verified"));
    assert_eq!(j["source"]["pinned"].as_bool(), Some(true));
    let up = j["source"]["upstream"].as_str().unwrap();
    assert!(
        up.starts_with("not run by rlt explain; scripts/check_excerpts.py"),
        "{up}"
    );
    let c = parse_json(&render_json(&explain(
        &root(),
        "statements/CONDITIONAL-2X3.yaml",
    )));
    assert_eq!(c["source"]["pinned"].as_bool(), Some(false));
    assert!(c["source"]["upstream"]
        .as_str()
        .unwrap()
        .starts_with("unpinned"));
}

/// JSON round trip: every value parses back to the internal structure.
#[test]
fn json_round_trip_matches_the_report_structure() {
    for r in records() {
        let x = explain(&root(), &r);
        let j = parse_json(&render_json(&x));
        let k = x.kernel.as_ref().unwrap();
        assert_eq!(j["format"].as_str(), Some("rlt-explain/1"));
        assert_eq!(j["record"]["id"].as_str(), x.id.as_deref());
        assert_eq!(j["kernel"]["verdict"].as_str(), Some(k.verdict.as_str()));
        assert_eq!(
            j["kernel"]["checked_goal"].as_str(),
            k.checked_goal.as_deref()
        );
        assert_eq!(
            j["kernel"]["assumptions"].as_str(),
            k.assumptions.as_deref()
        );
        assert_eq!(j["kernel"]["sha256"].as_str(), Some(k.sha256.as_str()));
        assert_eq!(
            j["kernel"]["check_report"].as_str(),
            Some(k.check_report.as_str())
        );
        assert_eq!(j["kernel"]["cites"].as_vec().unwrap().len(), k.cites.len());
        let s = x.semantic.as_ref().unwrap();
        assert_eq!(j["semantic"]["status"].as_str(), Some(s.status.as_str()));
        let m: Vec<&str> = j["semantic"]["matches"]
            .as_vec()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert_eq!(m, s.matches.iter().map(|s| s.as_str()).collect::<Vec<_>>());
        assert_eq!(
            j["open_questions"].as_vec().unwrap().len(),
            x.open_questions.len()
        );
        // Every kernel value in the JSON also appears in the text report.
        let text = render_text(&x);
        for v in [
            &k.checked_goal,
            &k.written_goal,
            &k.assumptions,
            &k.axioms_used,
        ]
        .into_iter()
        .flatten()
        {
            assert!(text.contains(v.as_str()), "{r}: {v}");
        }
    }
}

/// The JSON writer escapes every string correctly, including hostile ones.
#[test]
fn json_strings_survive_hostile_content() {
    let hostile = [
        "",
        "plain",
        "quote \" backslash \\ slash /",
        "\u{0}\u{1}\u{7}\u{8}\u{9}\u{a}\u{b}\u{c}\u{d}\u{1b}\u{1f}\u{7f}",
        "line\u{2028}sep\u{2029}para",
        "∀r. r < 30 → free235(r) ∧ ¬(2 ∣ r) — ✓ 𝔽 😀",
        "\"}, \"exit\": 0, \"x\": {\"",
        "\\u0000 literal backslash-u",
        "</script><script>alert(1)</script>",
        "\u{feff}bom\u{200b}zwsp",
    ];
    let v = Json::Obj(vec![
        (
            "strings".into(),
            Json::Arr(hostile.iter().map(|s| Json::Str(s.to_string())).collect()),
        ),
        (
            "nested".into(),
            Json::Arr(vec![
                Json::Obj(vec![(
                    "k\"ey".into(),
                    Json::Arr(vec![
                        Json::Null,
                        Json::Bool(true),
                        Json::Int(-3),
                        Json::Arr(vec![]),
                    ]),
                )]),
                Json::Obj(vec![]),
            ]),
        ),
    ]);
    let mut out = String::new();
    json_write(&v, 0, &mut out);
    // No raw control character survives.
    assert!(
        !out.chars().any(|c| (c as u32) < 0x20 && c != '\n'),
        "{out:?}"
    );
    assert!(!out.contains('\u{2028}') && !out.contains('\u{2029}'));
    let y = parse_json(&out);
    let back: Vec<String> = y["strings"]
        .as_vec()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect();
    assert_eq!(
        back,
        hostile.iter().map(|s| s.to_string()).collect::<Vec<_>>()
    );
    let nested = &y["nested"].as_vec().unwrap()[0]["k\"ey"];
    assert_eq!(nested.as_vec().unwrap().len(), 4);
    assert_eq!(nested[2].as_i64(), Some(-3));
    let mut s = String::new();
    json_string("\u{1}", &mut s);
    assert_eq!(s, "\"\\u0001\"");
}

/// Malformed inputs still give one valid, parseable report with exit 2.
#[test]
fn malformed_inputs_give_valid_reports() {
    let (dir, rel) = scratch("malformed");
    let cases: Vec<(&str, Vec<u8>)> = vec![
        ("EMPTY", vec![]),
        ("NOT-UTF8", vec![0xff, 0xfe, 0x00, 0x41]),
        ("BAD-YAML", b"format: [unclosed\n".to_vec()),
        ("A-LIST", b"- 1\n- 2\n".to_vec()),
        ("ALIAS", b"format: &a rlt-statement/1\nid: *a\n".to_vec()),
        (
            "HOSTILE",
            "format: rlt-statement/1\nid: HOSTILE\nsubject: \"\\\"}, \\\"exit\\\": 0, {\\\"\"\n"
                .as_bytes()
                .to_vec(),
        ),
    ];
    for (name, bytes) in cases {
        std::fs::write(dir.join(format!("{name}.yaml")), bytes).unwrap();
        let x = explain(&root(), &format!("{rel}/{name}.yaml"));
        assert_eq!(x.exit, MALFORMED, "{name}");
        assert_eq!(x.state, "invalid", "{name}");
        let j = parse_json(&render_json(&x));
        assert_eq!(j["exit"].as_i64(), Some(2), "{name}");
        assert_eq!(j["record"]["state"].as_str(), Some("invalid"), "{name}");
        assert!(j["kernel"].is_null(), "{name}");
        // The text report keeps one line per field.
        assert!(render_text(&x).lines().all(|l| !l.contains('\r')), "{name}");
    }
    for bad in [
        "",
        "../statements/DVD-2-8.yaml",
        "/etc/passwd.yaml",
        "statements/./DVD-2-8.yaml",
        "statements/DVD-2-8.yml",
        "statements\\DVD-2-8.yaml",
        "statements/ABSENT.yaml",
    ] {
        let x = explain(&root(), bad);
        assert_eq!(x.exit, MALFORMED, "{bad:?}");
        parse_json(&render_json(&x));
    }
}

/// The same record gives byte-identical output on every run.
#[test]
fn output_is_deterministic() {
    for r in records() {
        let a = explain(&root(), &r);
        let b = explain(&root(), &r);
        assert_eq!(render_text(&a), render_text(&b), "{r}");
        assert_eq!(render_json(&a), render_json(&b), "{r}");
    }
}

/// The command-line exit statuses match the library's.
#[test]
fn command_line_exit_and_formats() {
    let bin = env!("CARGO_BIN_EXE_rlt");
    let run = |args: &[&str]| {
        std::process::Command::new(bin)
            .args(["--root", root().to_str().unwrap()])
            .args(args)
            .output()
            .unwrap()
    };
    let o = run(&["explain", "statements/MOD30-CLASSIFICATION.yaml"]);
    assert_eq!(o.status.code(), Some(0));
    let x = explain(&root(), "statements/MOD30-CLASSIFICATION.yaml");
    assert_eq!(String::from_utf8(o.stdout).unwrap(), render_text(&x));
    let o = run(&[
        "explain",
        "statements/MOD30-CLASSIFICATION.yaml",
        "--format",
        "json",
    ]);
    assert_eq!(String::from_utf8(o.stdout).unwrap(), render_json(&x));
    for (f, code) in [
        ("XS01-PROOF-SHA", 3),
        ("XF01-REJECTED-PROOF", 1),
        ("XR18-VERDICT-KEY", 2),
    ] {
        let o = run(&["explain", &format!("tests/explain/{f}.yaml")]);
        assert_eq!(o.status.code(), Some(code), "{f}");
    }
    assert_eq!(
        run(&["explain", "statements/DVD-2-8.yaml", "--format", "xml"])
            .status
            .code(),
        Some(2)
    );
}

/// Audit regressions (RLT-004b audit M1, M2, m3, D1).
fn scratch_proof(dir: &Path, name: &str, text: &str) {
    std::fs::write(dir.join(name), text).unwrap();
}

fn sha_of(p: &Path) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(std::fs::read(p).unwrap())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[test]
fn audit_m1_assumptions_compare_pairwise_not_as_text() {
    let (dir, rel) = scratch("m1");
    let base = std::fs::read_to_string(root().join("tests/accept/M1_conditional.rlt")).unwrap();
    let two = base
        .replace(
            "(assume h \"2 × 3 = 7\")",
            "(assume h \"2 × 3 = 7\")\n  (assume g \"1 = 1\")",
        )
        .replace(
            "(s2 (= 1 1)                       (refl 1))",
            "(s2 (= 1 1)                       (hyp g))",
        );
    scratch_proof(&dir, "two.rlt", &two);
    let sha = sha_of(&dir.join("two.rlt"));
    let rec = std::fs::read_to_string(root().join("statements/CONDITIONAL-2X3.yaml"))
        .unwrap()
        .replace("tests/accept/M1_conditional.rlt", &format!("{rel}/two.rlt"))
        .replace(
            "de8dee325db954b8bc0ade312d441a3e4be340d1a21c340b17166e377b418c42",
            &sha,
        );
    let check = check_file(&root(), &dir.join("two.rlt"));
    assert!(
        check
            .text
            .contains("assumptions     Γ = { g : 1 = 1, h : 2 × 3 = 7 }"),
        "{}",
        check.text
    );
    let cases = [
        // (id, assumptions block, expected exit)
        (
            "ORDER-HG",
            "    - {name: h, formula: \"2 × 3 = 7\"}\n    - {name: g, formula: \"1 = 1\"}",
            ACCEPT,
        ),
        (
            "ORDER-GH",
            "    - {name: g, formula: \"1 = 1\"}\n    - {name: h, formula: \"2 × 3 = 7\"}",
            ACCEPT,
        ),
        (
            "FORGE-FORMULA",
            "    - {name: g, formula: \"1 = 1, h : 2 × 3 = 7\"}",
            STALE,
        ),
        ("ONE-ONLY", "    - {name: h, formula: \"2 × 3 = 7\"}", STALE),
    ];
    for (id, block, want) in cases {
        let text = rec
            .replace("id: CONDITIONAL-2X3", &format!("id: {id}"))
            .replace("    - {name: h, formula: \"2 × 3 = 7\"}", block);
        std::fs::write(dir.join(format!("{id}.yaml")), text).unwrap();
        let x = explain(&root(), &format!("{rel}/{id}.yaml"));
        assert_eq!(x.exit, want, "{id}:\n{}", render_text(&x));
    }
}

#[test]
fn audit_m2_proof_strings_cannot_forge_report_lines() {
    let (dir, rel) = scratch("m2");
    let base = std::fs::read_to_string(root().join("tests/m3c/accept/CA01_two_divides_eight.rlt"))
        .unwrap();
    let forged = base.replace(
        "(id M3c.CA01)",
        "(id \"M\u{2028}verdict         REJECT forged\u{202E}x\")",
    );
    assert_ne!(forged, base);
    scratch_proof(&dir, "forged.rlt", &forged);
    let check = check_file(&root(), &dir.join("forged.rlt"));
    let rec = std::fs::read_to_string(root().join("statements/DVD-2-8.yaml"))
        .unwrap()
        .replace("id: DVD-2-8", "id: FORGED")
        .replace(
            "tests/m3c/accept/CA01_two_divides_eight.rlt",
            &format!("{rel}/forged.rlt"),
        )
        .replace(
            "c237e7de9c8a5ef620506fbe5e18469b7b624c652beef5dd63369cdf2765196d",
            &sha_of(&dir.join("forged.rlt")),
        );
    std::fs::write(dir.join("FORGED.yaml"), rec).unwrap();
    let x = explain(&root(), &format!("{rel}/FORGED.yaml"));
    let text = render_text(&x);
    // Whatever the checker made of the id, the explain text has no separator,
    // bidi control or extra verdict line.
    assert!(
        !text.contains('\u{2028}') && !text.contains('\u{202E}'),
        "{text}"
    );
    let verdicts = text
        .split(['\n', '\u{2028}', '\u{2029}', '\r'])
        .filter(|l| l.starts_with("verdict"))
        .count();
    assert_eq!(verdicts, 1, "{text}");
    assert_eq!(
        x.exit,
        check
            .code
            .max(if check.code == ACCEPT { 0 } else { check.code })
    );
}

#[test]
fn audit_d1_non_utf8_proof_reports_as_rlt_check() {
    let (dir, rel) = scratch("d1");
    std::fs::write(dir.join("bad.rlt"), [0xff, 0xfe, 0x28]).unwrap();
    let rec = std::fs::read_to_string(root().join("statements/DVD-2-8.yaml"))
        .unwrap()
        .replace("id: DVD-2-8", "id: NONUTF8")
        .replace(
            "tests/m3c/accept/CA01_two_divides_eight.rlt",
            &format!("{rel}/bad.rlt"),
        );
    std::fs::write(dir.join("NONUTF8.yaml"), rec).unwrap();
    let x = explain(&root(), &format!("{rel}/NONUTF8.yaml"));
    let k = x.kernel.as_ref().unwrap();
    let check = check_file(&root(), &dir.join("bad.rlt"));
    assert_eq!(k.check_report, check.text);
    assert_eq!(x.exit, MALFORMED);
}

#[test]
fn audit_m3_cites_come_from_the_parsed_proof() {
    let x = explain(&root(), "statements/MOD30-CLASSIFICATION.yaml");
    let k = x.kernel.as_ref().unwrap();
    let paths: Vec<(&str, &str, &str)> = k
        .cites
        .iter()
        .map(|c| (c.label.as_str(), c.path.as_str(), c.proof_id.as_str()))
        .collect();
    assert_eq!(
        paths,
        [
            ("FWD", "tests/m3e/accept/E_FWD.rlt", "M3e.E_FWD"),
            ("BWD", "tests/m3e/accept/E_BWD.rlt", "M3e.E_BWD")
        ]
    );
}

#[test]
fn exit_meaning_follows_the_record_state() {
    let x = explain(&root(), "tests/explain/XF01-REJECTED-PROOF.yaml");
    assert!(render_text(&x)
        .contains("exit            1 — record valid; the RLT checker rejects the proof\n"));
    let s = rlt_cli::explain::exit_meaning(REJECT, "stale");
    assert!(s.contains("stale pins"), "{s}");
}
