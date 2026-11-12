//! RLT-004c: the theorem library and `rlt deps`.
//!
//! - The library in lib/ is valid and every entry current (`--verify-lib`).
//! - The graph comes from the one check: for every proof in tests/, the root
//!   equals `rlt check` and the edges equal the file's `cites` block.
//! - Statement fingerprints change with every part that fixes meaning.
//! - Negative cases on scratch copies of the repository: stale hashes, forged
//!   identities, replacements, invalid citations, cycles and both limits.

use rlt_cli::deps::{
    deps, fingerprint_text, load_library, render_json, render_text, verify_lib, StatementParts,
};
use rlt_cli::{check_file, ACCEPT, MALFORMED, REJECT};
use std::path::{Path, PathBuf};
use yaml_rust2::YamlLoader;

const STALE: i32 = 3;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let t = to.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copy_dir(&e.path(), &t);
        } else {
            std::fs::copy(e.path(), &t).unwrap();
        }
    }
}

/// A scratch copy of everything `rlt deps` and `rlt explain` read.
fn scratch_root(name: &str) -> PathBuf {
    let r = root().join("target/deps-scratch").join(name);
    let _ = std::fs::remove_dir_all(&r);
    for d in [
        "spec",
        "conventions",
        "readingpoints",
        "sources",
        "statements",
        "lib",
        "tests/accept",
        "tests/m3c/accept",
        "tests/m3d/accept",
        "tests/m3e/accept",
        "correspondence",
        "residues",
        "points",
    ] {
        if root().join(d).exists() {
            copy_dir(&root().join(d), &r.join(d));
        }
    }
    for f in ["docs/RLT-003-M3e.md", "lean/M3eReadings.lean"] {
        std::fs::create_dir_all(r.join(f).parent().unwrap()).unwrap();
        std::fs::copy(root().join(f), r.join(f)).unwrap();
    }
    r
}

fn edit(r: &Path, file: &str, from: &str, to: &str) {
    let p = r.join(file);
    let s = std::fs::read_to_string(&p).unwrap();
    assert!(s.contains(from), "{file}: `{from}` not found");
    std::fs::write(&p, s.replacen(from, to, 1)).unwrap();
}

fn sha(p: &Path) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(std::fs::read(p).unwrap())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[test]
fn the_library_is_valid_and_every_entry_current() {
    let lib = load_library(&root()).expect("lib/ loads");
    // 68 entries over 70 proofs at RLT-004c; RLT-005 adds 19 of each.
    assert_eq!(lib.entries.len(), 87);
    let proofs: usize = lib
        .entries
        .iter()
        .map(|(_, e)| 1 + e.alternatives.len())
        .sum();
    assert_eq!(
        proofs, 89,
        "every accepted proof of M3c–M3e, RLT-005 and the statement records is indexed"
    );
    let r = verify_lib(&root());
    assert_eq!(r.exit, ACCEPT, "{}", rlt_cli::deps::render_lib_text(&r));
    assert!(r.entries.iter().all(|e| e.1 == ACCEPT));
    // C7: the conditional theorem is listed and not citable.
    let c = lib.get("arith.conditional.2x3_7").unwrap();
    assert!(!c.citable);
    assert_eq!(
        c.assumptions,
        vec![("h".to_string(), "2 × 3 = 7".to_string())]
    );
}

#[test]
fn e_t1_graph_counts_limits_and_links() {
    let d = deps(&root(), "mod30.classification");
    assert_eq!(d.exit, ACCEPT, "{}", render_text(&d));
    assert_eq!((d.distinct, d.checks, d.chain), (50, 49, 2));
    assert_eq!(d.axioms.len(), 7);
    assert_eq!(d.definitions, ["dvd", "free235", "lt"]);
    assert_eq!(d.warnings.len(), 1, "49 of 64 is above 75%");
    assert_eq!(d.statement_records.len(), 1);
    assert_eq!(d.statement_records[0].2, ACCEPT);
    let root_node = d.root.as_ref().unwrap();
    let labels: Vec<_> = root_node
        .children
        .iter()
        .map(|c| c.label.clone().unwrap())
        .collect();
    assert_eq!(labels, ["FWD", "BWD"]);
    assert_eq!(
        root_node.children[0].lib_id.as_deref(),
        Some("mod30.forward")
    );
    let text = render_text(&d);
    assert!(text.contains("budget          cited-file checks 49 of 64 · citing-file chain 2 of 8"));
    assert!(text.contains("theorems        50 distinct · 49 cited-file checks"));
    // JSON parses and carries the same counts.
    let j = &YamlLoader::load_from_str(&render_json(&d)).unwrap()[0];
    assert_eq!(j["summary"]["cited_file_checks"].as_i64(), Some(49));
    assert_eq!(j["root"]["children"].as_vec().unwrap().len(), 2);
    // Deterministic.
    assert_eq!(
        render_json(&d),
        render_json(&deps(&root(), "mod30.classification"))
    );
}

/// The root of every graph is `rlt check`'s result for the same file, and its
/// edges are the file's own citations, in order.
#[test]
fn every_proof_agrees_with_rlt_check() {
    let mut files = Vec::new();
    fn collect(d: &Path, out: &mut Vec<PathBuf>) {
        for e in std::fs::read_dir(d).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                collect(&p, out);
            } else if p.extension().is_some_and(|x| x == "rlt") {
                out.push(p);
            }
        }
    }
    collect(&root().join("tests"), &mut files);
    files.sort();
    assert!(files.len() > 400);
    for f in files {
        let rel = f
            .strip_prefix(root())
            .unwrap()
            .to_string_lossy()
            .to_string();
        let d = deps(&root(), &rel);
        let rep = check_file(&root(), &f);
        if !d.problems.is_empty() {
            // Paths `rlt check` reads but `rlt deps` refuses (none expected).
            panic!("{rel}: {:?}", d.problems);
        }
        let n = d.root.as_ref().unwrap();
        assert_eq!(n.kernel.check_report, rep.text, "{rel}");
        assert_eq!(n.kernel.exit, rep.code, "{rel}");
        match rep.code {
            ACCEPT => assert!(d.exit == ACCEPT, "{rel}"),
            REJECT => assert_eq!(d.exit, REJECT, "{rel}"),
            _ => assert_eq!(d.exit, MALFORMED, "{rel}"),
        }
        let cited: Vec<String> = rep
            .text
            .lines()
            .filter_map(|l| l.strip_prefix("cites           "))
            .map(|l| l.split(" = ").next().unwrap().to_string())
            .collect();
        let labels: Vec<String> = n
            .children
            .iter()
            .filter(|c| c.problem.is_none())
            .map(|c| c.label.clone().unwrap())
            .collect();
        // An accepted proof's edges are exactly its citations; a failed one
        // reports every citation that passed, then the one that failed.
        if rep.code == ACCEPT {
            assert_eq!(labels, cited, "{rel}");
        } else {
            assert!(
                labels.starts_with(&cited) && labels.len() <= cited.len() + 1,
                "{rel}"
            );
        }
    }
}

#[test]
fn the_fingerprint_changes_with_every_part_that_fixes_meaning() {
    let base = StatementParts {
        convention: "NAT-DEF".into(),
        convention_digest: "a".repeat(64),
        axiom_set: "ROBINSON-Q".into(),
        axiom_file: "spec/robinson-q.rlt".into(),
        axiom_sha256: "b".repeat(64),
        definitions: Some((
            "spec/definitions.rlt".into(),
            "c".repeat(64),
            vec!["dvd".into()],
        )),
        goal: "¬(∃k. S(S(S(S(S(S(S(0))))))) = *(S(S(0)), k))".into(),
        assumptions: vec![],
    };
    let t0 = fingerprint_text(&base);
    type Variant = Box<dyn Fn(&mut StatementParts)>;
    let variants: Vec<Variant> = vec![
        Box::new(|p| p.convention = "NAT-LOGIC".into()),
        Box::new(|p| p.convention_digest = "d".repeat(64)),
        Box::new(|p| p.axiom_set = "PEANO-ARITH".into()),
        Box::new(|p| p.axiom_file = "spec/peano-arith.rlt".into()),
        Box::new(|p| p.axiom_sha256 = "e".repeat(64)),
        Box::new(|p| p.definitions = None),
        Box::new(|p| {
            p.definitions = Some((
                "spec/definitions.rlt".into(),
                "f".repeat(64),
                vec!["dvd".into()],
            ))
        }),
        Box::new(|p| {
            p.definitions = Some((
                "spec/definitions.rlt".into(),
                "c".repeat(64),
                vec!["dvd".into(), "lt".into()],
            ))
        }),
        Box::new(|p| p.goal = "¬(∃k. S(S(S(S(S(S(S(0))))))) = *(k, S(S(0))))".into()),
        Box::new(|p| p.assumptions = vec!["(= (S/1 (0/0)) (S/1 (0/0)))".into()]),
    ];
    for (i, v) in variants.iter().enumerate() {
        let mut p = base.clone();
        v(&mut p);
        assert_ne!(fingerprint_text(&p), t0, "variant {i}");
    }
    // The same statement proved twice has one fingerprint (CA02 and CA09).
    let lib = load_library(&root()).unwrap();
    let e = lib.get("dvd.not.2_7").unwrap();
    assert_eq!(e.alternatives.len(), 1);
    assert!(e.alternatives[0]
        .path
        .ends_with("CA09_witnesswise_gives_not_dvd.rlt"));
}

/// (case, setup, target, expected exit, a line fragment the text report contains)
type Setup = fn(&Path);

fn cases() -> Vec<(&'static str, Setup, &'static str, i32, &'static str)> {
    vec![
        // A changed proof cannot keep its verified identity: the pin is stale.
        (
            "proof-bytes-changed",
            |r| {
                let p = r.join("tests/m3e/accept/E_T1.rlt");
                let mut s = std::fs::read_to_string(&p).unwrap();
                s.push_str("; edited\n");
                std::fs::write(p, s).unwrap();
            },
            "mod30.classification",
            STALE,
            "  stale         proof sha256: entry e11712c6",
        ),
        (
            "checked-goal-edited",
            |r| {
                edit(
                    r,
                    "lib/dvd/dvd.2_8.yaml",
                    "checked_goal: \"∃k. 8 = 2 × k\"",
                    "checked_goal: \"∃k. 8 = k × 2\"",
                )
            },
            "dvd.2_8",
            STALE,
            "stale         checked goal",
        ),
        (
            "uses-reordered",
            |r| {
                edit(
                    r,
                    "lib/mod30/mod30.classification.yaml",
                    "  - mod30.forward\n  - mod30.backward",
                    "  - mod30.backward\n  - mod30.forward",
                )
            },
            "mod30.classification",
            STALE,
            "stale         uses: entry mod30.backward, mod30.forward",
        ),
        (
            "proof-id-edited",
            |r| {
                edit(
                    r,
                    "lib/dvd/dvd.2_8.yaml",
                    "proof_id: M3c.CA01",
                    "proof_id: M3c.CA99",
                )
            },
            "dvd.2_8",
            STALE,
            "stale         proof id",
        ),
        (
            "alternative-sha-edited",
            |r| {
                let s = std::fs::read_to_string(r.join("lib/dvd/dvd.not.2_7.yaml")).unwrap();
                let i = s.find("alternatives:").unwrap();
                let j = s[i..].find("sha256: ").unwrap() + i + 8;
                let mut b = s.into_bytes();
                b[j] = if b[j] == b'f' { b'e' } else { b'f' };
                std::fs::write(r.join("lib/dvd/dvd.not.2_7.yaml"), b).unwrap();
            },
            "dvd.not.2_7",
            STALE,
            "stale         alternative tests/m3c/accept/CA09_witnesswise_gives_not_dvd.rlt sha256",
        ),
        (
            "statement-record-pin-differs",
            |r| {
                edit(
                    r,
                    "statements/DVD-2-8.yaml",
                    "sha256: c237e7de",
                    "sha256: d237e7de",
                )
            },
            "dvd.2_8",
            STALE,
            "stale         statement record statements/DVD-2-8.yaml proof pin",
        ),
        // Forged or changed identities are invalid.
        (
            "fingerprint-forged",
            |r| edit(r, "lib/dvd/dvd.2_8.yaml", "fingerprint: ", "fingerprint: 0"),
            "dvd.2_8",
            MALFORMED,
            "must be 64 lower-case hexadecimal digits",
        ),
        (
            "fingerprint-not-registered",
            |r| {
                let fp = std::fs::read_to_string(r.join("lib/dvd/dvd.2_4.yaml")).unwrap();
                let fp = fp
                    .lines()
                    .find(|l| l.starts_with("fingerprint: "))
                    .unwrap()
                    .to_string();
                let s = std::fs::read_to_string(r.join("lib/dvd/dvd.2_8.yaml")).unwrap();
                let old = s
                    .lines()
                    .find(|l| l.starts_with("fingerprint: "))
                    .unwrap()
                    .to_string();
                std::fs::write(r.join("lib/dvd/dvd.2_8.yaml"), s.replace(&old, &fp)).unwrap();
            },
            "dvd.2_8",
            MALFORMED,
            "a library id is bound to one statement",
        ),
        // A semantically different proof under an existing id: E_FWD's proof
        // and statement placed under mod30.classification.
        (
            "statement-swapped-under-id",
            |r| {
                let fwd = std::fs::read_to_string(r.join("lib/mod30/mod30.forward.yaml")).unwrap();
                let get = |k: &str| fwd.lines().find(|l| l.starts_with(k)).unwrap().to_string();
                let s =
                    std::fs::read_to_string(r.join("lib/mod30/mod30.classification.yaml")).unwrap();
                let mut out = s.clone();
                for k in ["fingerprint: ", "proof: ", "proof_id: ", "checked_goal: "] {
                    let old = s.lines().find(|l| l.starts_with(k)).unwrap();
                    out = out.replace(old, &get(k));
                }
                std::fs::write(r.join("lib/mod30/mod30.classification.yaml"), out).unwrap();
            },
            "mod30.classification",
            MALFORMED,
            "a library id is bound to one statement",
        ),
        (
            "unregistered-id",
            |r| {
                std::fs::copy(
                    r.join("lib/dvd/dvd.2_8.yaml"),
                    r.join("lib/dvd/dvd.2_8b.yaml"),
                )
                .unwrap();
                edit(r, "lib/dvd/dvd.2_8b.yaml", "id: dvd.2_8", "id: dvd.2_8b");
            },
            "dvd.2_8",
            MALFORMED,
            "is not registered in lib/identities.tsv",
        ),
        (
            "identity-without-entry",
            |r| std::fs::remove_file(r.join("lib/order/order.below.3.yaml")).unwrap(),
            "dvd.2_8",
            MALFORMED,
            "identities are never retired",
        ),
        (
            "id-in-wrong-area",
            |r| {
                std::fs::rename(
                    r.join("lib/dvd/dvd.2_8.yaml"),
                    r.join("lib/order/dvd.2_8.yaml"),
                )
                .unwrap()
            },
            "dvd.2_8",
            MALFORMED,
            "must live at lib/dvd/dvd.2_8.yaml",
        ),
        (
            "replacement-without-history",
            |r| {
                let s = std::fs::read_to_string(r.join("lib/dvd/dvd.not.2_7.yaml")).unwrap();
                let p0 = s
                    .lines()
                    .find(|l| l.starts_with("proof: "))
                    .unwrap()
                    .to_string();
                let a0 = s
                    .lines()
                    .find(|l| l.starts_with("  - {path: tests/m3c/accept/CA09"))
                    .unwrap()
                    .to_string();
                let out = s
                    .replace(&p0, &format!("proof: {}", &a0[4..]))
                    .replace(&a0, &format!("  - {}", &p0[7..]))
                    .replace("proof_id: M3c.CA02", "proof_id: M3c.CA09");
                std::fs::write(r.join("lib/dvd/dvd.not.2_7.yaml"), out).unwrap();
            },
            "dvd.not.2_7",
            MALFORMED,
            "history omits it",
        ),
        (
            "history-with-other-statement",
            |r| {
                let fp = std::fs::read_to_string(r.join("lib/dvd/dvd.2_4.yaml")).unwrap();
                let fp =
                    fp.lines().find(|l| l.starts_with("fingerprint: ")).unwrap()[13..].to_string();
                edit(r, "lib/dvd/dvd.2_8.yaml", "history: []", &format!("history:\n  - {{path: tests/m3e/accept/PD_04.rlt, sha256: {}, fingerprint: {fp}, replaced_on: 2026-11-11, reviewed_by: \"a reviewer\"}}", "a".repeat(64)));
            },
            "dvd.2_8",
            MALFORMED,
            "a replacement must prove the identical statement",
        ),
        (
            "uses-conditional",
            |r| {
                edit(
                    r,
                    "lib/mod30/mod30.classification.yaml",
                    "  - mod30.backward\n",
                    "  - mod30.backward\n  - arith.conditional.2x3_7\n",
                )
            },
            "mod30.classification",
            MALFORMED,
            "conditional and not citable",
        ),
        (
            "uses-unknown",
            |r| {
                edit(
                    r,
                    "lib/mod30/mod30.classification.yaml",
                    "  - mod30.backward\n",
                    "  - mod30.backwards\n",
                )
            },
            "mod30.classification",
            MALFORMED,
            "unknown id `mod30.backwards`",
        ),
        (
            "citable-conditional",
            |r| {
                edit(
                    r,
                    "lib/arith/arith.conditional.2x3_7.yaml",
                    "citable: false",
                    "citable: true",
                )
            },
            "arith.conditional.2x3_7",
            MALFORMED,
            "only unconditional theorems can be cited",
        ),
        (
            "duplicate-fingerprint",
            |r| {
                let fp = std::fs::read_to_string(r.join("lib/dvd/dvd.2_4.yaml")).unwrap();
                let fpl =
                    fp.lines().find(|l| l.starts_with("fingerprint: ")).unwrap()[13..].to_string();
                let s = std::fs::read_to_string(r.join("lib/dvd/dvd.2_8.yaml")).unwrap();
                let old =
                    s.lines().find(|l| l.starts_with("fingerprint: ")).unwrap()[13..].to_string();
                std::fs::write(r.join("lib/dvd/dvd.2_8.yaml"), s.replace(&old, &fpl)).unwrap();
                edit(
                    r,
                    "lib/identities.tsv",
                    &format!("dvd.2_8\t{old}"),
                    &format!("dvd.2_8\t{fpl}"),
                );
            },
            "dvd.2_8",
            MALFORMED,
            "registered under two ids",
        ),
        (
            "unknown-target",
            |_| {},
            "dvd.2_9",
            MALFORMED,
            "`dvd.2_9` is not a library id",
        ),
    ]
}

#[test]
fn negative_cases_on_scratch_repositories() {
    for (name, setup, target, want, line) in cases() {
        let r = scratch_root(name);
        setup(&r);
        let d = deps(&r, target);
        let text = render_text(&d);
        assert_eq!(d.exit, want, "{name}:\n{text}");
        assert!(text.contains(line), "{name}: missing `{line}` in\n{text}");
        YamlLoader::load_from_str(&render_json(&d)).expect("JSON parses");
    }
}

#[test]
fn a_reviewed_replacement_with_history_is_accepted() {
    let r = scratch_root("replacement-with-history");
    let s = std::fs::read_to_string(r.join("lib/dvd/dvd.not.2_7.yaml")).unwrap();
    let fp = s.lines().find(|l| l.starts_with("fingerprint: ")).unwrap()[13..].to_string();
    let p0 = s
        .lines()
        .find(|l| l.starts_with("proof: "))
        .unwrap()
        .to_string();
    let a0 = s
        .lines()
        .find(|l| l.starts_with("  - {path: tests/m3c/accept/CA09"))
        .unwrap()
        .to_string();
    let first_sha = &p0[p0.find("sha256: ").unwrap() + 8..p0.len() - 1];
    let out = s
        .replace(&p0, &format!("proof: {}", &a0[4..]))
        .replace(&format!("alternatives:\n{a0}\n"), "")
        .replace("proof_id: M3c.CA02", "proof_id: M3c.CA09")
        .replace("uses: []", "uses:\n  - dvd.witnesswise.2_7")
        .replace(
            "history: []",
            &format!("history:\n  - {{path: tests/m3c/accept/CA02_two_not_divides_seven.rlt, sha256: {first_sha}, fingerprint: {fp}, replaced_on: 2026-11-11, reviewed_by: \"RLT-004c test\"}}"),
        );
    std::fs::write(r.join("lib/dvd/dvd.not.2_7.yaml"), out).unwrap();
    let d0 = deps(&r, "dvd.not.2_7");
    assert!(
        d0.notes.is_empty(),
        "the history pin's bytes exist and were re-checked"
    );
    // A history pin whose bytes are gone is reported as a note.
    let s2 = std::fs::read_to_string(r.join("lib/dvd/dvd.not.2_7.yaml")).unwrap();
    std::fs::write(
        r.join("lib/dvd/dvd.not.2_7.yaml"),
        s2.replace(
            "path: tests/m3c/accept/CA02_two_not_divides_seven.rlt, sha256",
            "path: tests/m3c/accept/CA02_gone.rlt, sha256",
        ),
    )
    .unwrap();
    let dn = deps(&r, "dvd.not.2_7");
    assert_eq!(dn.exit, ACCEPT);
    assert!(
        render_text(&dn).contains("note            history tests/m3c/accept/CA02_gone.rlt"),
        "{}",
        render_text(&dn)
    );
    std::fs::write(r.join("lib/dvd/dvd.not.2_7.yaml"), s2).unwrap();
    // The entry is valid, but E_BWD cites CA02, now in history only: the
    // entry for mod30.backward names an unlisted dependency.
    let d = deps(&r, "dvd.not.2_7");
    assert_eq!(d.exit, ACCEPT, "{}", render_text(&d));
    let b = deps(&r, "mod30.backward");
    assert_eq!(b.exit, STALE, "{}", render_text(&b));
    assert!(render_text(&b).contains("(unlisted tests/m3c/accept/CA02_two_not_divides_seven.rlt)"));
}

fn write_proof(dir: &Path, name: &str, cites: &[(String, String, String)], body: &str) {
    let mut c = String::new();
    if !cites.is_empty() {
        c.push_str("  (cites\n");
        for (l, p, h) in cites {
            c.push_str(&format!("    ({l} \"{p}\" \"{h}\")\n"));
        }
        c.push_str("  )\n");
    }
    let src = std::fs::read_to_string(root().join("tests/m3d/accept/DA03_below_one.rlt")).unwrap();
    let i = src.find("(steps").unwrap();
    std::fs::write(
        dir.join(name),
        format!("{}{c}{body}{}", &src[..i], &src[i..]).replace(
            "(id M3d.DA03)",
            &format!("(id T.{})", name.trim_end_matches(".rlt")),
        ),
    )
    .unwrap();
}

/// Citation failures, cycles and both limits, as `rlt check` reports them.
#[test]
fn citations_cycles_and_limits() {
    let dir = root().join("target/deps-scratch/citations");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let rel = "target/deps-scratch/citations";
    let da03 = "tests/m3d/accept/DA03_below_one.rlt".to_string();
    let da03_sha = sha(&root().join(&da03));
    // Wrong hash.
    write_proof(
        &dir,
        "badhash.rlt",
        &[("A".into(), da03.clone(), "f".repeat(64))],
        "",
    );
    // Outside path.
    write_proof(
        &dir,
        "outside.rlt",
        &[("A".into(), "../x.rlt".into(), "f".repeat(64))],
        "",
    );
    // A chain of n citing files: c1 cites DA03, c(k+1) cites ck.
    let mut prev = (da03.clone(), da03_sha.clone());
    for k in 1..=9 {
        let n = format!("chain{k}.rlt");
        write_proof(
            &dir,
            &n,
            &[("A".into(), prev.0.clone(), prev.1.clone())],
            "",
        );
        prev = (format!("{rel}/{n}"), sha(&dir.join(&n)));
    }
    // 64 and 65 citations of DA03 in one file.
    let many = |n: usize| {
        (0..n)
            .map(|i| (format!("A{i}"), da03.clone(), da03_sha.clone()))
            .collect::<Vec<_>>()
    };
    write_proof(&dir, "budget64.rlt", &many(64), "");
    write_proof(&dir, "budget65.rlt", &many(65), "");
    // Cycles: a file citing itself cannot carry its own hash, so cycles are
    // built through the cycle check, which runs before the hash check.
    write_proof(
        &dir,
        "self.rlt",
        &[("A".into(), format!("{rel}/self.rlt"), "f".repeat(64))],
        "",
    );
    write_proof(
        &dir,
        "cyc_a.rlt",
        &[("B".into(), format!("{rel}/cyc_b.rlt"), "f".repeat(64))],
        "",
    );
    let a_sha = sha(&dir.join("cyc_a.rlt"));
    write_proof(
        &dir,
        "cyc_b.rlt",
        &[("A".into(), format!("{rel}/cyc_a.rlt"), a_sha)],
        "",
    );
    let b_sha = sha(&dir.join("cyc_b.rlt"));
    write_proof(
        &dir,
        "cyc_a.rlt",
        &[("B".into(), format!("{rel}/cyc_b.rlt"), b_sha.clone())],
        "",
    );

    let run = |f: &str| deps(&root(), &format!("{rel}/{f}"));
    let expect = |f: &str, code: i32, frag: &str| {
        let d = run(f);
        let t = render_text(&d);
        assert_eq!(d.exit, code, "{f}:\n{t}");
        assert!(t.contains(frag), "{f}: missing `{frag}` in\n{t}");
        let rep = check_file(&root(), &root().join(rel).join(f));
        assert_eq!(
            rep.code,
            d.root.as_ref().map(|n| n.kernel.exit).unwrap_or(MALFORMED),
            "{f}"
        );
        d
    };
    expect(
        "badhash.rlt",
        MALFORMED,
        "citation failed: citation `A`: tests/m3d/accept/DA03_below_one.rlt has SHA-256",
    );
    expect("outside.rlt", MALFORMED, "citation failed: citation `A`");
    let c8 = run("chain8.rlt");
    assert_eq!(c8.exit, ACCEPT, "{}", render_text(&c8));
    assert_eq!(c8.chain, 8);
    assert!(render_text(&c8).contains("citing-file chain 8 of 8"));
    expect(
        "chain9.rlt",
        MALFORMED,
        "citations nest deeper than 8 files",
    );
    let b64 = expect("budget64.rlt", ACCEPT, "cited-file checks 64 of 64");
    assert_eq!(b64.distinct, 2, "one root and DA03, checked 64 times");
    assert_eq!(b64.checks, 64);
    expect("budget65.rlt", MALFORMED, "would check more than 64 files");
    expect("budget65.rlt", MALFORMED, "cited-file checks 65 of 64");
    expect("self.rlt", MALFORMED, "citation cycle through");
    expect("cyc_a.rlt", MALFORMED, "citation cycle through");
}

/// lib/identities.tsv is append-only across every committed version.
#[test]
fn identity_registry_is_append_only() {
    let dir = root().join("target/deps-scratch/gitgate");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("lib")).unwrap();
    let git = |args: &[&str]| {
        std::process::Command::new("git")
            .args(["-c", "user.email=t@example.invalid", "-c", "user.name=t"])
            .args(args)
            .current_dir(&dir)
            .output()
            .unwrap()
    };
    let gate = || {
        std::process::Command::new(root().join("scripts/check-identities.sh"))
            .arg("HEAD")
            .current_dir(&dir)
            .output()
            .unwrap()
            .status
            .code()
    };
    let reg = std::fs::read_to_string(root().join("lib/identities.tsv")).unwrap();
    std::fs::write(dir.join("lib/identities.tsv"), &reg).unwrap();
    git(&["init", "-q"]);
    git(&["add", "."]);
    git(&["commit", "-q", "-m", "base"]);
    assert_eq!(gate(), Some(0));
    // Appending a new identity is allowed, committed or not.
    let grown = format!("{reg}new.id\t{}\t{}\n", "a".repeat(64), "b".repeat(64));
    std::fs::write(dir.join("lib/identities.tsv"), &grown).unwrap();
    assert_eq!(gate(), Some(0));
    git(&["commit", "-qam", "grow"]);
    assert_eq!(gate(), Some(0));
    // Changing an existing identity's fingerprint is refused, in the working
    // tree and after it is committed (the check reads every version).
    let line = reg
        .lines()
        .find(|l| l.starts_with("mod30.classification\t"))
        .unwrap();
    let mut f: Vec<&str> = line.split('\t').collect();
    let forged = "c".repeat(64);
    f[1] = &forged;
    std::fs::write(
        dir.join("lib/identities.tsv"),
        grown.replace(line, &f.join("\t")),
    )
    .unwrap();
    assert_eq!(gate(), Some(1));
    git(&["commit", "-qam", "rebind"]);
    assert_eq!(gate(), Some(1), "a committed rebinding is still refused");
    // Removing the file is refused.
    std::fs::remove_file(dir.join("lib/identities.tsv")).unwrap();
    assert_eq!(gate(), Some(1));
    // A branch's drafts merged with a merge commit are not main's versions:
    // main's history is read by first parent.
    git(&["checkout", "-q", "--", "lib/identities.tsv"]);
    git(&["reset", "-q", "--hard", "HEAD~1"]);
    git(&["checkout", "-q", "-b", "draft"]);
    std::fs::write(
        dir.join("lib/identities.tsv"),
        format!("{grown}draft.id\t{}\t{}\n", "d".repeat(64), "e".repeat(64)),
    )
    .unwrap();
    git(&["commit", "-qam", "draft"]);
    std::fs::write(
        dir.join("lib/identities.tsv"),
        format!("{grown}draft.id\t{}\t{}\n", "f".repeat(64), "e".repeat(64)),
    )
    .unwrap();
    git(&["commit", "-qam", "draft revised"]);
    git(&["checkout", "-q", "-"]);
    git(&["merge", "-q", "--no-ff", "-m", "merge draft", "draft"]);
    assert_eq!(
        gate(),
        Some(0),
        "the merged branch's earlier draft does not bind main"
    );
}

fn scratch_file(dir: &Path, name: &str, text: &str) -> String {
    std::fs::write(dir.join(name), text).unwrap();
    sha(&dir.join(name))
}

/// Audit regressions (RLT-004c audit 1–7).
#[test]
fn audit_regressions() {
    let dir = root().join("target/deps-scratch/audit");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let rel = "target/deps-scratch/audit";
    // 6: alpha-variants and assumption names leave the fingerprint unchanged.
    let a02_path = std::fs::read_dir(root().join("tests/m3a/accept"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| p.file_name().unwrap().to_string_lossy().starts_with("A02"))
        .unwrap();
    let a02 = std::fs::read_to_string(&a02_path).unwrap();
    scratch_file(
        &dir,
        "a02y.rlt",
        &a02.replace("∀x. x × 0 = 0", "∀y. y × 0 = 0")
            .replace("(forall x (= (* x 0) 0))", "(forall y (= (* y 0) 0))"),
    );
    let fp = |f: &str| {
        deps(&root(), &format!("{rel}/{f}"))
            .root
            .unwrap()
            .fingerprint
            .unwrap()
    };
    let a02_rel = a02_path
        .strip_prefix(root())
        .unwrap()
        .to_string_lossy()
        .to_string();
    assert_eq!(
        deps(&root(), &a02_rel).root.unwrap().fingerprint.unwrap(),
        fp("a02y.rlt"),
        "an alpha-variant is the same statement"
    );
    let cond = std::fs::read_to_string(root().join("tests/accept/M1_conditional.rlt")).unwrap();
    scratch_file(
        &dir,
        "condg.rlt",
        &cond
            .replace("(assume h ", "(assume g ")
            .replace("(hyp h)", "(hyp g)"),
    );
    assert_eq!(
        deps(&root(), "tests/accept/M1_conditional.rlt")
            .root
            .unwrap()
            .fingerprint
            .unwrap(),
        fp("condg.rlt"),
        "an assumption's name is a label, not part of the statement"
    );
    // 2: a refused citation is shown as refused and adds nothing to the unions.
    let mut cc = std::fs::read_to_string(root().join("tests/accept/M1_thirty.rlt")).unwrap();
    let i = cc.find("(steps").unwrap();
    cc.insert_str(
        i,
        &format!(
            "(cites (C \"tests/accept/M1_conditional.rlt\" \"{}\"))\n  ",
            sha(&root().join("tests/accept/M1_conditional.rlt"))
        ),
    );
    scratch_file(&dir, "cc.rlt", &cc);
    let d = deps(&root(), &format!("{rel}/cc.rlt"));
    let t = render_text(&d);
    assert_eq!(d.exit, MALFORMED, "{t}");
    let c = &d.root.as_ref().unwrap().children[0];
    assert!(
        c.checked && c.problem.as_deref().unwrap().contains("conditional"),
        "{t}"
    );
    assert!(
        t.contains("citation refused: citation `C`: the cited theorem is conditional"),
        "{t}"
    );
    assert!(
        !d.axioms.contains(&"mul_succ".to_string()),
        "refused edge excluded: {:?}",
        d.axioms
    );
    // 4 and 5 and 7 on scratch repositories.
    let r = scratch_root("audit-stale-record");
    edit(
        &r,
        "statements/MATH29-PAIRING.yaml",
        "checked_goal: \"29 + 1 = 30\"",
        "checked_goal: \"29 + 1 = 31\"",
    );
    let d = deps(&r, "arith.add.29_1_30");
    assert_eq!(d.exit, STALE, "{}", render_text(&d));
    assert!(render_text(&d).contains("stale         statement record statements/MATH29-PAIRING.yaml: entry rlt explain exit 0 · current rlt explain exit 3"));
    let r = scratch_root("audit-invalid-lib");
    std::fs::write(r.join("lib/junk.txt"), "x").unwrap();
    let d = deps(&r, "tests/m3e/accept/PD_00.rlt");
    assert_eq!(d.exit, MALFORMED, "{}", render_text(&d));
    let r = scratch_root("audit-alt-rejected");
    let cr10 = "tests/m3c/reject/CR10_two_not_divides_eight.rlt";
    std::fs::create_dir_all(r.join("tests/m3c/reject")).unwrap();
    std::fs::copy(root().join(cr10), r.join(cr10)).unwrap();
    edit(
        &r,
        "lib/dvd/dvd.2_4.yaml",
        "uses: []",
        &format!(
            "uses: []\nalternatives:\n  - {{path: {cr10}, sha256: {}}}",
            sha(&r.join(cr10))
        ),
    );
    let d = deps(&r, "dvd.2_4");
    assert_eq!(d.exit, REJECT, "{}", render_text(&d));
    // An id ending in `.rlt` is refused.
    assert!(rlt_syntax::records::lib_id("arith.rlt", "t").is_err());
}
