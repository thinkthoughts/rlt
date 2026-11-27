//! RLT-DEF-001a: the canonical definition registry. DR-01 … DR-22, the
//! adversarial cases of RLT-DEF-001-PROPOSAL revision 2 §5.2. Every case
//! fails closed: a derivation over definitions that are not the registered
//! bytes never gets an authoritative verdict, and an exploratory result never
//! feeds a citation, a record, a library entry, discovery verification or a
//! composition.

use rlt_cli::defreg::{
    expansion_sha256, load_definitions_registry, load_definitions_registry_pinned, Authority, Pins,
    DEFINITIONS_REGISTRY, PINNED_DEFINITION_FILES,
};
use rlt_cli::prove::{prove_source, prove_source_as, render, Verdict, EXPLORATORY_CERT_LINE};
use rlt_cli::{
    check_file_exploratory, check_file_outcome, read_function_definitions, sha256_hex, ACCEPT,
    EXPLORATORY, MALFORMED,
};
use rlt_syntax::proof::DefinitionsRef;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    let manifest = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").to_string());
    Path::new(&manifest).parent().unwrap().to_path_buf()
}

fn read(p: &Path) -> String {
    std::fs::read_to_string(p).unwrap()
}

fn sha_at(root: &Path, rel: &str) -> String {
    sha256_hex(&std::fs::read(root.join(rel)).unwrap())
}

/// A scratch file under target/def001a/ (repository-relative path returned).
fn scratch(name: &str, body: &str) -> String {
    let rel = format!("target/def001a/{name}");
    let full = root().join(&rel);
    std::fs::create_dir_all(full.parent().unwrap()).unwrap();
    std::fs::write(&full, body).unwrap();
    rel
}

const EVIL: &str = "(rlt-definitions\n  (id DOUBLE)\n  (convention NAT-LOGIC)\n  (domain Nat)\n  (define-function double (x) (+ x 1)\n    (written \"double(x) := x + 1\")\n    (origin rlt-authored)))\n";

fn goal(id: &str, defs_rel: &str, defs_sha: &str, statement: &str) -> String {
    format!(
        "(rlt-goal\n  (id {id})\n  (convention NAT-FUN)\n  (definitions \"{defs_rel}\" \"{defs_sha}\")\n  (goal \"{statement}\")\n  (mode unconditional)\n  (axioms ROBINSON-Q))\n"
    )
}

fn text(r: &rlt_cli::prove::ProveOutcome) -> String {
    let mut t = String::new();
    render(&mut t, r);
    t
}

fn check(root: &Path, rel: &str) -> rlt_cli::CheckOutcome {
    check_file_outcome(root, &root.join(rel))
}

/// DR-01 fixture: the audit's D2 case — a second file declaring
/// `(id DOUBLE)` with `double(x) := x + 1`, and a kernel-valid certificate
/// for `double(2) = 3` over it (made by an exploratory run).
fn evil_certificate(tag: &str) -> (String, String) {
    let defs = scratch(&format!("{tag}/evil.rlt"), EVIL);
    let g = goal("evil.two", &defs, &sha_at(&root(), &defs), "double(2) = 3");
    let out = format!("target/def001a/{tag}/evil_two.rlt");
    let _ = std::fs::remove_file(root().join(&out));
    let r = prove_source_as(
        &root(),
        &g,
        &[],
        Some(Path::new(&out)),
        Authority::ExploratoryAllowed,
    );
    assert!(matches!(r.verdict, Verdict::Proved { .. }), "{}", text(&r));
    assert_eq!(r.code(), EXPLORATORY, "{}", text(&r));
    (defs, out)
}

// ── Mini repository roots, for cases that alter registered files ──────────

fn copy_dir(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap();
    for e in std::fs::read_dir(src).unwrap() {
        let e = e.unwrap();
        let t = e.file_type().unwrap();
        let to = dst.join(e.file_name());
        if t.is_dir() {
            copy_dir(&e.path(), &to);
        } else if t.is_file() {
            std::fs::copy(e.path(), &to).unwrap();
        }
    }
}

/// A copy of the parts of the repository a NAT-FUN check reads.
fn mini_root(tag: &str) -> PathBuf {
    let r = root().join("target/def001a/roots").join(tag);
    let _ = std::fs::remove_dir_all(&r);
    for d in [
        "spec",
        "definitions",
        "sources",
        "conventions",
        "correspondence",
        "readingpoints",
        "tests/definitions",
    ] {
        copy_dir(&root().join(d), &r.join(d));
    }
    r
}

fn registry_text(r: &Path) -> String {
    read(&r.join(DEFINITIONS_REGISTRY))
}

fn write(r: &Path, rel: &str, body: &str) {
    std::fs::create_dir_all(r.join(rel).parent().unwrap()).unwrap();
    std::fs::write(r.join(rel), body).unwrap();
}

/// Pins for a mini root's registry as it now is, with the given entries.
fn pins_for<'a>(
    r: &Path,
    files: &'a [(&'a str, &'a str, &'a str)],
) -> (String, &'a [(&'a str, &'a str, &'a str)]) {
    (sha_at(r, DEFINITIONS_REGISTRY), files)
}

fn registry_err(r: &Path, files: &[(&str, &str, &str)]) -> String {
    let (h, f) = pins_for(r, files);
    match load_definitions_registry_pinned(
        r,
        Pins {
            registry_sha256: &h,
            files: f,
        },
    ) {
        Ok(_) => panic!("registry accepted"),
        Err(e) => e,
    }
}

fn leak(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

/// The pinned file list with one entry's SHA-256 replaced.
fn files_with(id: &str, sha: &str) -> Vec<(&'static str, &'static str, &'static str)> {
    PINNED_DEFINITION_FILES
        .iter()
        .map(|&(i, p, h)| {
            if i == id {
                (i, p, leak(sha.to_string()))
            } else {
                (i, p, h)
            }
        })
        .collect()
}

const F3: &str = "tests/definitions/certificates/F3_double_two.rlt";
const DOUBLE_SHA: &str = "8dabb459817f7b225419456080a4e81eb675daf9a830dba529192ff50cfbc2f7";

// ── The registry as committed ────────────────────────────────────────────

#[test]
fn the_committed_registry_is_consistent_and_its_identities_are_the_specified_ones() {
    let reg = load_definitions_registry(&root()).unwrap();
    assert_eq!(
        reg.canonical.get("double").map(String::as_str),
        Some("066fb635c06d85cf6cbd288468b7d992566056793f44375b47f0ac30c09a1c16")
    );
    assert_eq!(
        reg.canonical.get("quadruple").map(String::as_str),
        Some("8a3513697d503ba5f365c237fd2f7e4b84006ff44766fc783d1241a91c81beda")
    );
    // `double` in DOUBLE and in QUADRUPLE: one definition, recognised by
    // content identity (file identities differ).
    let a = &reg.loaded["DOUBLE"].list[0];
    let b = &reg.loaded["QUADRUPLE"].list[0];
    assert_eq!(a.content_sha256, b.content_sha256);
    assert_eq!(expansion_sha256(a), expansion_sha256(b));
    assert_ne!(
        reg.loaded["DOUBLE"].file_sha256,
        reg.loaded["QUADRUPLE"].file_sha256
    );
    // Every committed NAT-FUN certificate is over a registered file and
    // keeps its verdict.
    for dir in [
        "tests/definitions/certificates",
        "research/definition-discovery",
    ] {
        for e in std::fs::read_dir(root().join(dir)).unwrap() {
            let p = e.unwrap().path();
            if p.extension().is_some_and(|x| x == "rlt") {
                let c = check_file_outcome(&root(), &p);
                assert_eq!(c.report.code, ACCEPT, "{}", c.report.text);
            }
        }
    }
}

// ── DR-01 … DR-03: files outside the registry ────────────────────────────

#[test]
fn dr01_a_second_file_claiming_id_double_is_refused_on_every_authoritative_path() {
    let (defs, cert) = evil_certificate("dr01");
    // check
    let c = check(&root(), &cert);
    assert_eq!(c.report.code, MALFORMED, "{}", c.report.text);
    assert!(c
        .report
        .text
        .contains("is not a registered definitions file"));
    assert!(c.theorem.is_none());
    assert!(c.definitions.is_none());
    let exploratory = check_file_exploratory(&root(), &root().join(&cert));
    assert_eq!(exploratory.report.code, EXPLORATORY);
    let provenance = exploratory.definitions.as_ref().unwrap();
    assert!(!provenance.registration.as_ref().unwrap().is_registered());
    let k = rlt_cli::explain::kernel_fields(&exploratory, &cert, &sha_at(&root(), &cert));
    assert!(k.definitions_sha256.is_none());
    // prove
    let g = goal("evil.two", &defs, &sha_at(&root(), &defs), "double(2) = 3");
    let r = prove_source(
        &root(),
        &g,
        &[],
        Some(Path::new("target/def001a/dr01/again.rlt")),
    );
    assert!(matches!(r.verdict, Verdict::Error(_)), "{}", text(&r));
    assert!(text(&r).contains("is not a registered definitions file"));
    // discover --definitions
    let (t, code) =
        rlt_cli::discover_defs::discover(&root(), &defs, "target/def001a/dr01/disc", Some(2));
    assert_eq!(code, 2, "{t}");
    assert!(t.contains("is not a registered definitions file"), "{t}");
    // deps on the certificate
    let d = rlt_cli::deps::deps(&root(), &cert);
    assert_ne!(d.exit, ACCEPT);
    assert!(rlt_cli::deps::render_text(&d).contains("is not a registered definitions file"));
}

#[test]
fn dr02_an_exact_copy_of_a_registered_file_at_another_path_is_refused() {
    let copy = scratch(
        "dr02/double.rlt",
        &read(&root().join("definitions/double.rlt")),
    );
    let cert = read(&root().join(F3)).replace("\"definitions/double.rlt\"", &format!("\"{copy}\""));
    let p = scratch("dr02/F3.rlt", &cert);
    let c = check(&root(), &p);
    assert_eq!(c.report.code, MALFORMED, "{}", c.report.text);
    assert!(c
        .report
        .text
        .contains("is not a registered definitions file"));
    // Recognised as the registered content, still refused.
    assert!(
        c.report.text.contains("double@066fb635"),
        "{}",
        c.report.text
    );
}

#[test]
fn dr03_a_layout_only_variant_shares_content_identity_and_is_refused() {
    let body = format!(
        "; a comment changes file identity only\n{}",
        read(&root().join("definitions/double.rlt"))
    );
    let copy = scratch("dr03/double.rlt", &body);
    let h = sha_at(&root(), &copy);
    assert_ne!(h, DOUBLE_SHA);
    let d = read_function_definitions(
        &root(),
        &DefinitionsRef {
            path: copy.clone(),
            sha256: h.clone(),
        },
    )
    .unwrap();
    assert_eq!(
        d.list[0].content_sha256,
        "066fb635c06d85cf6cbd288468b7d992566056793f44375b47f0ac30c09a1c16"
    );
    let cert = read(&root().join(F3))
        .replace("\"definitions/double.rlt\"", &format!("\"{copy}\""))
        .replace(DOUBLE_SHA, &h);
    let p = scratch("dr03/F3.rlt", &cert);
    let c = check(&root(), &p);
    assert_eq!(c.report.code, MALFORMED, "{}", c.report.text);
    assert!(
        c.report.text.contains("double@066fb635"),
        "{}",
        c.report.text
    );
}

// ── DR-04 … DR-06: registered files and the registry altered ─────────────

#[test]
fn dr04_an_edited_registered_file_is_refused_even_with_the_header_updated() {
    let r = mini_root("dr04");
    let edited = format!("{} ", read(&r.join("definitions/double.rlt")));
    write(&r, "definitions/double.rlt", &edited);
    let h = sha_at(&r, "definitions/double.rlt");
    write(
        &r,
        "target/F3.rlt",
        &read(&r.join(F3)).replace(DOUBLE_SHA, &h),
    );
    let c = check(&r, "target/F3.rlt");
    assert_eq!(c.report.code, MALFORMED, "{}", c.report.text);
    assert!(c.report.text.contains("registered"), "{}", c.report.text);
    // The committed certificate, against the edited file: refused too.
    assert_eq!(check(&r, F3).report.code, MALFORMED);
    // Exploratory: the registry itself no longer loads (its DOUBLE entry is
    // not the bytes on disk), so nothing is checked in any mode.
    let x = check_file_exploratory(&r, &r.join("target/F3.rlt"));
    assert_eq!(x.report.code, MALFORMED, "{}", x.report.text);
}

#[test]
fn dr05_a_registry_edit_without_a_new_binary_refuses_every_nat_fun_check() {
    let r = mini_root("dr05");
    write(
        &r,
        DEFINITIONS_REGISTRY,
        &format!("{}# edited\n", registry_text(&r)),
    );
    let c = check(&r, F3);
    assert_eq!(c.report.code, MALFORMED, "{}", c.report.text);
    assert!(c.report.text.contains("(I1)"), "{}", c.report.text);
    let x = check_file_exploratory(&r, &r.join(F3));
    assert_eq!(x.report.code, MALFORMED);
    let (t, code) = rlt_cli::defreg::definitions_report(&r, None, false);
    assert_eq!(code, MALFORMED, "{t}");
    assert!(t.contains("INCONSISTENT"));
    // `rlt validate` is unchanged by RLT-DEF-001a (its report stays
    // byte-identical on every root); the registry's own check is
    // `rlt definitions`, and every NAT-FUN load enforces it.
}

#[test]
fn dr06_registry_and_file_edited_together_still_need_the_binary_pin() {
    let r = mini_root("dr06");
    let edited = read(&r.join("definitions/double.rlt"))
        .replace("(origin rlt-authored)", "(origin rlt-authored) ");
    write(&r, "definitions/double.rlt", &edited);
    let h = sha_at(&r, "definitions/double.rlt");
    write(
        &r,
        DEFINITIONS_REGISTRY,
        &registry_text(&r).replace(DOUBLE_SHA, &h),
    );
    write(
        &r,
        "target/F3.rlt",
        &read(&r.join(F3)).replace(DOUBLE_SHA, &h),
    );
    let c = check(&r, "target/F3.rlt");
    assert_eq!(c.report.code, MALFORMED, "{}", c.report.text);
    assert!(c.report.text.contains("(I1)"), "{}", c.report.text);
    // With the registry pin updated but not the file list: I2.
    let e = registry_err(&r, PINNED_DEFINITION_FILES);
    assert!(e.contains("(I2)"), "{e}");
}

// ── DR-07 … DR-14: each registry rule on its own (test pins) ─────────────

#[test]
fn dr07_a_file_whose_id_convention_or_domain_differs_from_its_entry_is_refused() {
    for (from, to) in [
        ("(id DOUBLE)", "(id DOUBLEX)"),
        ("(domain Nat)", "(domain Int)"),
    ] {
        let r = mini_root("dr07");
        let edited = read(&r.join("definitions/double.rlt")).replace(from, to);
        write(&r, "definitions/double.rlt", &edited);
        let h = sha_at(&r, "definitions/double.rlt");
        write(
            &r,
            DEFINITIONS_REGISTRY,
            &registry_text(&r).replace(DOUBLE_SHA, &h),
        );
        let files = files_with("DOUBLE", &h);
        let e = registry_err(&r, &files);
        assert!(e.contains("(I5)") || e.contains("domain"), "{from}: {e}");
    }
}

#[test]
fn dr08_defines_must_list_exactly_the_computed_identities() {
    let wrong = "066fb635c06d85cf6cbd288468b7d992566056793f44375b47f0ac30c09a1c16";
    let zero = "a".repeat(64);
    let cases: Vec<(&str, String)> = vec![
        // a wrong content identity
        ("wrong", format!("content_sha256: {zero}")),
    ];
    for (tag, _) in &cases {
        let r = mini_root(&format!("dr08{tag}"));
        // Only DOUBLE's own line (the first occurrence) is changed.
        let t = registry_text(&r).replacen(
            &format!("content_sha256: {wrong}"),
            &format!("content_sha256: {zero}"),
            1,
        );
        write(&r, DEFINITIONS_REGISTRY, &t);
        let e = registry_err(&r, PINNED_DEFINITION_FILES);
        assert!(e.contains("(I6)"), "{e}");
    }
    // an omitted name
    let r = mini_root("dr08omit");
    let t = registry_text(&r).replacen(
        "    defines:\n      - name: double\n        content_sha256: 066fb635c06d85cf6cbd288468b7d992566056793f44375b47f0ac30c09a1c16\n  - id: QUADRUPLE",
        "    defines: []\n  - id: QUADRUPLE",
        1,
    );
    assert_ne!(t, registry_text(&r));
    write(&r, DEFINITIONS_REGISTRY, &t);
    let e = registry_err(&r, PINNED_DEFINITION_FILES);
    assert!(e.contains("(I6)"), "{e}");
    // an extra name
    let r = mini_root("dr08extra");
    let t = registry_text(&r).replacen(
        "        content_sha256: 066fb635c06d85cf6cbd288468b7d992566056793f44375b47f0ac30c09a1c16\n  - id: QUADRUPLE",
        "        content_sha256: 066fb635c06d85cf6cbd288468b7d992566056793f44375b47f0ac30c09a1c16\n      - name: triple\n        content_sha256: 066fb635c06d85cf6cbd288468b7d992566056793f44375b47f0ac30c09a1c16\n  - id: QUADRUPLE",
        1,
    );
    write(&r, DEFINITIONS_REGISTRY, &t);
    let e = registry_err(&r, PINNED_DEFINITION_FILES);
    assert!(e.contains("(I6)"), "{e}");
}

#[test]
fn dr09_one_name_with_two_content_identities_is_refused() {
    let r = mini_root("dr09");
    let y = "(rlt-definitions\n  (id DOUBLEY)\n  (convention NAT-LOGIC)\n  (domain Nat)\n  (define-function double (y) (+ y y)\n    (written \"double(y) := y + y\")\n    (origin rlt-authored)))\n";
    write(&r, "definitions/double-y.rlt", y);
    let hy = sha_at(&r, "definitions/double-y.rlt");
    let dy = read_function_definitions(
        &r,
        &DefinitionsRef {
            path: "definitions/double-y.rlt".into(),
            sha256: hy.clone(),
        },
    )
    .unwrap();
    let dx = read_function_definitions(
        &r,
        &DefinitionsRef {
            path: "definitions/double.rlt".into(),
            sha256: DOUBLE_SHA.into(),
        },
    )
    .unwrap();
    // Same expansion, different content identity (parameter names count).
    assert_eq!(expansion_sha256(&dy.list[0]), expansion_sha256(&dx.list[0]));
    assert_ne!(dy.list[0].content_sha256, dx.list[0].content_sha256);
    let entry = format!(
        "  - id: DOUBLEY\n    path: definitions/double-y.rlt\n    sha256: {hy}\n    kind: functions\n    convention: NAT-LOGIC\n    domain: Nat\n    defines:\n      - name: double\n        content_sha256: {}\n  - id: NAT-DEF\n",
        dy.list[0].content_sha256
    );
    write(
        &r,
        DEFINITIONS_REGISTRY,
        &registry_text(&r).replace("  - id: NAT-DEF\n", &entry),
    );
    let mut files: Vec<(&str, &str, &str)> = PINNED_DEFINITION_FILES[..2].to_vec();
    files.push(("DOUBLEY", "definitions/double-y.rlt", leak(hy)));
    files.push(PINNED_DEFINITION_FILES[2]);
    let e = registry_err(&r, &files);
    assert!(
        e.contains("(I7)") && e.contains("one name has one content identity"),
        "{e}"
    );
}

#[test]
fn dr10_duplicate_ids_shared_paths_and_unplain_paths_are_refused() {
    // duplicate id
    let r = mini_root("dr10id");
    write(
        &r,
        DEFINITIONS_REGISTRY,
        &registry_text(&r).replace("id: QUADRUPLE", "id: DOUBLE"),
    );
    let files = [
        PINNED_DEFINITION_FILES[0],
        (
            "DOUBLE",
            PINNED_DEFINITION_FILES[1].1,
            PINNED_DEFINITION_FILES[1].2,
        ),
        PINNED_DEFINITION_FILES[2],
    ];
    assert!(registry_err(&r, &files).contains("(I3)"));
    // one path under two ids
    let r = mini_root("dr10path");
    write(
        &r,
        DEFINITIONS_REGISTRY,
        &registry_text(&r).replace(
            "path: definitions/quadruple.rlt\n    sha256: 7f463b05541ed9643ab98b276bb37de31c2f39054f55502656bd1c20fb703932",
            &format!("path: definitions/double.rlt\n    sha256: {DOUBLE_SHA}"),
        ),
    );
    let files = [
        PINNED_DEFINITION_FILES[0],
        ("QUADRUPLE", "definitions/double.rlt", DOUBLE_SHA),
        PINNED_DEFINITION_FILES[2],
    ];
    assert!(registry_err(&r, &files).contains("(I3)"));
    // ./ and .. spellings in the registry
    for spelled in [
        "./definitions/double.rlt",
        "definitions/../definitions/double.rlt",
    ] {
        let r = mini_root("dr10dot");
        write(
            &r,
            DEFINITIONS_REGISTRY,
            &registry_text(&r).replace("path: definitions/double.rlt", &format!("path: {spelled}")),
        );
        let files = [
            (PINNED_DEFINITION_FILES[0].0, spelled, DOUBLE_SHA),
            PINNED_DEFINITION_FILES[1],
            PINNED_DEFINITION_FILES[2],
        ];
        assert!(registry_err(&r, &files).contains("(I3)"), "{spelled}");
    }
    // ./ and .. spellings in a header: refused by the path rules
    for spelled in [
        "./definitions/double.rlt",
        "definitions/../definitions/double.rlt",
    ] {
        let cert =
            read(&root().join(F3)).replace("\"definitions/double.rlt\"", &format!("\"{spelled}\""));
        let p = scratch("dr10/F3.rlt", &cert);
        let c = check(&root(), &p);
        assert_eq!(c.report.code, MALFORMED, "{}", c.report.text);
    }
}

#[cfg(unix)]
#[test]
fn dr11_symbolic_links_are_refused_for_the_registry_and_for_definitions() {
    let r = mini_root("dr11");
    std::fs::rename(r.join(DEFINITIONS_REGISTRY), r.join("spec/real.yaml")).unwrap();
    std::os::unix::fs::symlink("real.yaml", r.join(DEFINITIONS_REGISTRY)).unwrap();
    let c = check(&r, F3);
    assert_eq!(c.report.code, MALFORMED, "{}", c.report.text);
    assert!(
        c.report.text.contains("(I1)") && c.report.text.contains("symbolic link"),
        "{}",
        c.report.text
    );
    // A header naming a symlink to the registered file.
    let r = mini_root("dr11b");
    std::os::unix::fs::symlink("double.rlt", r.join("definitions/link.rlt")).unwrap();
    write(
        &r,
        "target/F3.rlt",
        &read(&r.join(F3)).replace("\"definitions/double.rlt\"", "\"definitions/link.rlt\""),
    );
    let c = check(&r, "target/F3.rlt");
    assert_eq!(c.report.code, MALFORMED, "{}", c.report.text);
    assert!(c.report.text.contains("symlink"), "{}", c.report.text);
}

#[test]
fn dr12_unknown_keys_duplicate_keys_and_other_versions_are_refused() {
    let edits: [(&str, &str, &str); 5] = [
        ("unknown top-level key", "files:\n", "note: x\nfiles:\n"),
        (
            "unknown entry key",
            "    kind: nat-def\n",
            "    kind: nat-def\n    extra: x\n",
        ),
        (
            "duplicate key",
            "    kind: nat-def\n",
            "    kind: nat-def\n    kind: nat-def\n",
        ),
        (
            "format",
            "format: rlt-definitions-registry/1",
            "format: rlt-definitions-registry/2",
        ),
        (
            "canonical text",
            "canonical_text: rlt-function-definition/1",
            "canonical_text: rlt-function-definition/2",
        ),
    ];
    for (what, from, to) in edits {
        let r = mini_root("dr12");
        let t = registry_text(&r);
        assert!(t.contains(from), "{what}");
        write(&r, DEFINITIONS_REGISTRY, &t.replacen(from, to, 1));
        let e = registry_err(&r, PINNED_DEFINITION_FILES);
        assert!(e.contains("(I1)"), "{what}: {e}");
    }
    // anchors and aliases
    let r = mini_root("dr12a");
    write(
        &r,
        DEFINITIONS_REGISTRY,
        &registry_text(&r).replacen("    domain: Nat\n", "    domain: &d Nat\n", 1),
    );
    assert!(registry_err(&r, PINNED_DEFINITION_FILES).contains("anchors"));
}

#[test]
fn dr13_the_nat_def_entry_must_equal_pinned_definitions() {
    let r = mini_root("dr13");
    let other = "f".repeat(64);
    write(
        &r,
        DEFINITIONS_REGISTRY,
        &registry_text(&r).replace(
            "122e37fc627b136023a1a5ffac0344162ed77ae95312248a9f6a76c2b639ecda",
            &other,
        ),
    );
    let files = files_with("NAT-DEF", &other);
    assert!(registry_err(&r, &files).contains("(I8)"));
    // names differing from the file's
    let r = mini_root("dr13n");
    write(
        &r,
        DEFINITIONS_REGISTRY,
        &registry_text(&r).replace("      - free235\n", ""),
    );
    assert!(registry_err(&r, PINNED_DEFINITION_FILES).contains("(I8)"));
}

#[test]
fn dr14_registered_names_stay_disjoint_from_rules_axioms_and_nat_def_names() {
    for name in ["dvd", "free235", "refl", "add_zero"] {
        let r = mini_root("dr14");
        let edited = read(&r.join("definitions/double.rlt"))
            .replace("define-function double", &format!("define-function {name}"))
            .replace("\"double(x) := x + x\"", &format!("\"{name}(x) := x + x\""));
        write(&r, "definitions/double.rlt", &edited);
        let h = sha_at(&r, "definitions/double.rlt");
        write(
            &r,
            DEFINITIONS_REGISTRY,
            &registry_text(&r).replace(DOUBLE_SHA, &h).replacen(
                "- name: double",
                &format!("- name: {name}"),
                1,
            ),
        );
        let files = files_with("DOUBLE", &h);
        let e = registry_err(&r, &files);
        assert!(e.contains("registered file DOUBLE"), "{name}: {e}");
    }
}

// ── DR-15 … DR-20: exploratory results stay contained ────────────────────

#[test]
fn dr15_an_exploratory_certificate_cannot_be_cited() {
    let (_, cert) = evil_certificate("dr15");
    let h = sha_at(&root(), &cert);
    let citing = format!(
        "(rlt-proof\n  (id DR15)\n  (convention NAT-FUN)\n  (definitions \"definitions/double.rlt\" \"{DOUBLE_SHA}\")\n  (goal \"2 + 1 = 3\")\n  (mode unconditional)\n  (axioms ROBINSON-Q)\n  (cites (evil \"{cert}\" \"{h}\"))\n  (steps\n    (s1 (= (+ 2 1) 3) (cite evil))))\n"
    );
    let p = scratch("dr15/citing.rlt", &citing);
    for c in [
        check(&root(), &p),
        check_file_exploratory(&root(), &root().join(&p)),
    ] {
        assert_eq!(c.report.code, MALFORMED, "{}", c.report.text);
        assert!(
            c.report.text.contains("citation `evil`"),
            "{}",
            c.report.text
        );
        assert!(
            c.report
                .text
                .contains("is not a registered definitions file"),
            "{}",
            c.report.text
        );
    }
}

#[test]
fn dr16_an_exploratory_discovery_record_is_refused_by_verify() {
    let defs = scratch("dr16/evil.rlt", EVIL);
    let dir = "target/def001a/dr16/record";
    let _ = std::fs::remove_dir_all(root().join(dir));
    let (t, code) = rlt_cli::discover_defs::discover_as(
        &root(),
        &defs,
        dir,
        Some(4),
        Authority::ExploratoryAllowed,
    );
    assert_eq!(code, EXPLORATORY, "{t}");
    assert!(t.contains("EXPLORATORY"), "{t}");
    assert!(rlt_cli::discover_defs::is_record(&root(), dir));
    let (v, vcode) = rlt_cli::discover_defs::verify(&root(), dir);
    assert_ne!(vcode, 0, "{v}");
    assert!(
        v.contains("REFUSED") && v.contains("is not a registered definitions file"),
        "{v}"
    );
    // The record with its EXPLORATORY lines removed: refused all the same.
    let m = root().join(dir).join("manifest.tsv");
    let cleaned: String = read(&m)
        .lines()
        .filter(|l| !l.contains("EXPLORATORY"))
        .map(|l| format!("{l}\n"))
        .collect();
    std::fs::write(&m, cleaned).unwrap();
    let s = root().join(dir).join("summary.txt");
    let cleaned: String = read(&s)
        .lines()
        .filter(|l| !l.contains("EXPLORATORY"))
        .map(|l| format!("{l}\n"))
        .collect();
    std::fs::write(&s, cleaned).unwrap();
    let (v, vcode) = rlt_cli::discover_defs::verify(&root(), dir);
    assert_ne!(vcode, 0, "{v}");
}

#[test]
fn dr17_records_libraries_and_composition_recheck_registration_from_bytes() {
    let (_, cert) = evil_certificate("dr17");
    let h = sha_at(&root(), &cert);
    // deps on the certificate
    let d = rlt_cli::deps::deps(&root(), &cert);
    assert_ne!(d.exit, ACCEPT);
    // a statement record naming it
    let rec = format!(
        "format: rlt-statement/1\nid: DR17-EVIL\nsubject: \"exploratory certificate\"\nsource:\n  registry: LEAN-CORE-NAT-DVD\n  commit: 6a10ac8c22beadecabdbb0919c2b50214762f91d\n  excerpts:\n    - \"dvd a b := Exists (fun c => b = a * c)\"\nkernel:\n  proof: {cert}\n  sha256: {h}\n  convention: NAT-FUN\n  axiom_set: ROBINSON-Q\n  axiom_set_sha256: 4ad55192222576e3901abc4ee7ea5a6760b64ff212e8c55796ab9847e423b00c\n  checked_goal: \"2 + 1 = 3\"\n  assumptions: []\nsemantic:\n  status: unassessed\nphysical:\n  applicability: not-applicable\n  reason: \"arithmetic\"\n"
    );
    let rp = scratch("dr17/DR17-EVIL.yaml", &rec);
    let x = rlt_cli::explain::explain(&root(), &rp);
    let xt = rlt_cli::explain::render_text(&x);
    assert_ne!(x.exit, ACCEPT, "{xt}");
    assert!(xt.contains("is not a registered definitions file"), "{xt}");
    // Structure: every path that gives authority (explain, deps, compose,
    // residues, discover verification, citations) reaches the checker only
    // through the authoritative entry points; the exploratory authority is
    // named only where the flag is handled and passed along.
    let src = root().join("cli/src");
    for f in [
        "explain.rs",
        "deps.rs",
        "compose.rs",
        "residues.rs",
        "discover.rs",
    ] {
        let s = read(&src.join(f));
        for forbidden in [
            "ExploratoryAllowed",
            "check_file_exploratory",
            "_outcome_as(",
            "load_function_definitions_as",
            "attach_functions_as",
            "prove_source_as",
        ] {
            assert!(!s.contains(forbidden), "{f} uses {forbidden}");
        }
    }
    let lib = read(&src.join("lib.rs"));
    // Citations are always checked authoritatively.
    assert!(lib.contains("// RLT-DEF-001a: a cited file is always checked authoritatively."));
    let n = lib.matches("defreg::Authority::ExploratoryAllowed").count();
    assert_eq!(n, 2, "lib.rs names ExploratoryAllowed {n} times (check_file_exploratory and the loader's match arm)");
}

#[test]
fn dr18_removing_the_exploratory_header_line_gains_nothing() {
    let (_, cert) = evil_certificate("dr18");
    let body = read(&root().join(&cert));
    assert!(body.contains(EXPLORATORY_CERT_LINE));
    let p = scratch(
        "dr18/stripped.rlt",
        &body.replace(EXPLORATORY_CERT_LINE, ""),
    );
    let c = check(&root(), &p);
    assert_eq!(c.report.code, MALFORMED, "{}", c.report.text);
    assert!(c
        .report
        .text
        .contains("is not a registered definitions file"));
    // Exploratory on request, with or without the line.
    assert_eq!(
        check_file_exploratory(&root(), &root().join(&p))
            .report
            .code,
        EXPLORATORY
    );
}

#[test]
fn dr19_exploratory_output_never_overwrites_a_tracked_file() {
    let defs = scratch("dr19/evil.rlt", EVIL);
    let g = goal("evil.two", &defs, &sha_at(&root(), &defs), "double(2) = 3");
    let before = std::fs::read(root().join(F3)).unwrap();
    let r = prove_source_as(
        &root(),
        &g,
        &[],
        Some(Path::new(F3)),
        Authority::ExploratoryAllowed,
    );
    assert!(matches!(r.verdict, Verdict::Error(_)), "{}", text(&r));
    assert!(
        text(&r).contains("tracked by the repository"),
        "{}",
        text(&r)
    );
    assert_eq!(std::fs::read(root().join(F3)).unwrap(), before);
}

#[test]
fn dr20_the_flag_changes_nothing_for_registered_definitions() {
    for e in std::fs::read_dir(root().join("tests/definitions/certificates")).unwrap() {
        let p = e.unwrap().path();
        let a = check_file_outcome(&root(), &p);
        let b = check_file_exploratory(&root(), &p);
        assert_eq!(a.report.code, ACCEPT);
        assert_eq!(
            (a.report.text, a.report.code),
            (b.report.text, b.report.code),
            "{}",
            p.display()
        );
    }
    for e in std::fs::read_dir(root().join("tests/definitions/goals")).unwrap() {
        let p = e.unwrap().path();
        let g = read(&p);
        let name = p.file_stem().unwrap().to_string_lossy().to_string();
        let oa = format!("target/def001a/dr20/{name}.a.rlt");
        let ob = format!("target/def001a/dr20/{name}.b.rlt");
        for o in [&oa, &ob] {
            let _ = std::fs::remove_file(root().join(o));
        }
        let a = prove_source(&root(), &g, &[], Some(Path::new(&oa)));
        let b = prove_source_as(
            &root(),
            &g,
            &[],
            Some(Path::new(&ob)),
            Authority::ExploratoryAllowed,
        );
        assert_eq!(a.code(), 0, "{}", text(&a));
        assert_eq!(text(&a).replace(&oa, "OUT"), text(&b).replace(&ob, "OUT"));
        assert_eq!(read(&root().join(&oa)), read(&root().join(&ob)));
    }
}

// ── DR-21, DR-22 ─────────────────────────────────────────────────────────

#[test]
fn dr21_a_malformed_unregistered_file_keeps_its_parse_message() {
    let rec = "(rlt-definitions\n  (id REC)\n  (convention NAT-LOGIC)\n  (domain Nat)\n  (define-function loop (x) (loop x)\n    (written \"loop(x) := loop(x)\")\n    (origin rlt-authored)))\n";
    let defs = scratch("dr21/rec.rlt", rec);
    let g = goal("rec", &defs, &sha_at(&root(), &defs), "loop(0) = 0");
    for authority in [Authority::RegisteredOnly, Authority::ExploratoryAllowed] {
        let r = prove_source_as(
            &root(),
            &g,
            &[],
            Some(Path::new("target/def001a/dr21/c.rlt")),
            authority,
        );
        let t = text(&r);
        assert!(matches!(r.verdict, Verdict::Error(_)), "{t}");
        assert!(!t.contains("is not a registered definitions file"), "{t}");
        assert!(t.contains("loop"), "{t}");
    }
}

#[test]
fn dr22_the_python_cross_check_agrees_and_detects_a_mutated_canonical_text() {
    let bin = env!("CARGO_BIN_EXE_rlt");
    let script = root().join("scripts/check-definition-identities.py");
    let run = |extra: &[&str]| {
        std::process::Command::new("python3")
            .arg("-I")
            .arg(&script)
            .arg(bin)
            .args(extra)
            .output()
            .expect("python3")
    };
    let ok = run(&[]);
    assert_eq!(
        ok.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&ok.stdout)
    );
    for field in ["body", "formal", "written", "params", "deps"] {
        let bad = run(&["--mutate", field]);
        assert_eq!(
            bad.status.code(),
            Some(1),
            "{field}: {}",
            String::from_utf8_lossy(&bad.stdout)
        );
    }
}
