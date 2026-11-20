//! `rlt prove` — bounded, deterministic proof search (UNTRUSTED).
//!
//! Given a goal file (a statement, its convention, its mode, its declared
//! assumptions and a registered axiom set, with no proof), this module
//! searches for a derivation using only the kernel's 20 registered rules and
//! the axioms of that set. Nothing here is trusted. A found derivation is
//! written as an ordinary `rlt-proof` certificate and handed, as bytes, to
//! the same function `rlt check` uses (`check_bytes_outcome`): the file is
//! re-parsed, the written goal re-elaborated, the axiom set re-loaded against
//! its pin, and every step replayed by the kernel. `PROVED` is reported only
//! when that replay accepts; a search that ends without a proof is reported
//! as `NOT_FOUND_WITHIN_BOUNDS`, which says nothing about the goal's truth.
//!
//! The search has two layers:
//! - an equational core: a proof-producing congruence closure (an e-graph
//!   with a proof forest) saturated by bounded e-matching of the equational
//!   axioms (oriented left to right), of universally quantified equational
//!   assumptions, and of Horn rules `l = r → l' = r'` and `l = r → ⊥` (Q1, Q2
//!   and negated equations). Every merge records a justification from which a
//!   chain of `refl`, `symm`, `trans`, `cong`, `inst`, `forall-elim` and
//!   `imp-elim` steps is extracted.
//! - a goal-directed natural-deduction search, iteratively deepened: the
//!   invertible rules (∀, →, ∧ on the goal; ∧, ∃, ⊥ in the context) are
//!   applied eagerly; disjunctions on the goal are tried left then right;
//!   the non-invertible choices (∃ witnesses, case splits on disjunctions,
//!   modus ponens, instantiating quantified facts and axioms, backward
//!   chaining) each cost one unit of depth.

use crate::{
    check_bytes_outcome, load_axiom_set, load_definitions, pinned_conventions, sha256_hex, ACCEPT,
};
use rlt_kernel::{Axiom, Formula, Mode, Prop, Term, Theorem};
use rlt_syntax::sexpr::{parse_all, Sexp};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::rc::Rc;

// ── Proof trees ────────────────────────────────────────────────────────────

/// A derivation built by the search. Each node records the conclusion the
/// search *expects* its rule to yield; the kernel recomputes every one.
#[derive(Debug)]
pub struct P {
    pub concl: Prop,
    pub rule: R,
}
pub type Pf = Rc<P>;

#[derive(Debug)]
pub enum R {
    Hyp(String),
    Refl(Term),
    Symm(Pf),
    Trans(Pf, Pf),
    Cong(String, Vec<Pf>),
    Inst(String, Vec<(String, Term)>),
    AndI(Pf, Pf),
    AndL(Pf),
    AndR(Pf),
    OrL(Pf),
    OrR(Pf),
    OrCases(Pf, String, Pf, String, Pf),
    ImpI(String, Pf),
    ImpE(Pf, Pf),
    FalseE(Pf),
    ForallI(String, Pf),
    ForallE(Pf, Term),
    ExistsI(Term, Pf),
    ExistsE(Pf, String, String, Pf),
}

fn mk(concl: Prop, rule: R) -> Pf {
    Rc::new(P { concl, rule })
}

fn eqn(p: &Pf) -> &Formula {
    match &p.concl {
        Prop::Eq(f) => f,
        other => panic!("internal: expected an equation, found {other}"),
    }
}

fn hyp(name: &str, phi: &Prop) -> Pf {
    mk(phi.clone(), R::Hyp(name.to_string()))
}
fn refl(t: &Term) -> Pf {
    mk(Prop::eq(t.clone(), t.clone()), R::Refl(t.clone()))
}
fn is_refl(p: &Pf) -> bool {
    matches!(p.rule, R::Refl(_))
}
fn symm(p: Pf) -> Pf {
    if is_refl(&p) {
        return p;
    }
    let f = eqn(&p);
    mk(Prop::eq(f.rhs.clone(), f.lhs.clone()), R::Symm(p))
}
fn trans(p: Pf, q: Pf) -> Pf {
    if is_refl(&p) {
        return q;
    }
    if is_refl(&q) {
        return p;
    }
    let c = Prop::eq(eqn(&p).lhs.clone(), eqn(&q).rhs.clone());
    mk(c, R::Trans(p, q))
}
fn cong(f: &str, ps: Vec<Pf>) -> Pf {
    let l = Term::App(f.into(), ps.iter().map(|p| eqn(p).lhs.clone()).collect());
    if ps.iter().all(is_refl) {
        return refl(&l);
    }
    let r = Term::App(f.into(), ps.iter().map(|p| eqn(p).rhs.clone()).collect());
    mk(Prop::eq(l, r), R::Cong(f.into(), ps))
}
fn inst(ax: &Axiom, sigma: &BTreeMap<String, Term>) -> Option<Pf> {
    let mut binds = Vec::new();
    for v in ax.vars() {
        binds.push((v.clone(), sigma.get(v)?.clone()));
    }
    let s: BTreeMap<String, Term> = binds.iter().cloned().collect();
    Some(mk(
        ax.body().subst_vars(&s),
        R::Inst(ax.name().to_string(), binds),
    ))
}
fn forall_e(p: Pf, t: &Term) -> Pf {
    let c = match &p.concl {
        Prop::Forall(_, b) => b.open(t),
        other => panic!("internal: forall-elim on {other}"),
    };
    mk(c, R::ForallE(p, t.clone()))
}
fn imp_e(p: Pf, q: Pf) -> Pf {
    let c = match &p.concl {
        Prop::Imp(_, b) => (**b).clone(),
        other => panic!("internal: imp-elim on {other}"),
    };
    mk(c, R::ImpE(p, q))
}
fn and_l(p: Pf) -> Pf {
    let c = match &p.concl {
        Prop::And(a, _) => (**a).clone(),
        other => panic!("internal: and-left on {other}"),
    };
    mk(c, R::AndL(p))
}
fn and_r(p: Pf) -> Pf {
    let c = match &p.concl {
        Prop::And(_, b) => (**b).clone(),
        other => panic!("internal: and-right on {other}"),
    };
    mk(c, R::AndR(p))
}
fn false_e(p: Pf, goal: &Prop) -> Pf {
    if *goal == Prop::False {
        return p;
    }
    mk(goal.clone(), R::FalseE(p))
}

// ── E-graph with a proof forest ────────────────────────────────────────────

enum Just {
    Given(Pf),
    Cong,
}

struct Edge {
    a: usize,
    b: usize,
    j: Just,
}

/// How an instance of a rule schema is proved.
#[derive(Clone)]
enum Source {
    /// A registered axiom, instantiated with `inst`.
    Axiom(Axiom),
    /// A fact `∀?0 … ∀?k-1. body`, instantiated with `forall-elim` k times.
    Fact(Pf, usize),
}

#[derive(Clone)]
enum Kind {
    /// `l = r`, used left to right (`flip`: the schema states `r = l`).
    Eq { l: Term, r: Term, flip: bool },
    /// `al = ar → cl = cr`, or `al = ar → ⊥` when `concl` is `None`.
    Horn {
        al: Term,
        ar: Term,
        concl: Option<(Term, Term)>,
    },
}

#[derive(Clone)]
struct Rule {
    vars: Vec<String>,
    kind: Kind,
    src: Source,
}

impl Rule {
    fn instance(&self, sigma: &BTreeMap<String, Term>) -> Option<Pf> {
        let p = match &self.src {
            Source::Axiom(ax) => inst(ax, sigma)?,
            Source::Fact(pf, k) => {
                let mut p = pf.clone();
                for i in 0..*k {
                    p = forall_e(p, sigma.get(&format!("?{i}"))?);
                }
                p
            }
        };
        Some(match self.kind {
            Kind::Eq { flip: true, .. } => symm(p),
            _ => p,
        })
    }
}

fn term_vars(t: &Term, out: &mut BTreeSet<String>) {
    let mut v = Vec::new();
    t.vars(&mut v);
    out.extend(v);
}

/// Rules from a schema `∀vars. body` (vars occur as `Term::Var`).
fn rules_of(vars: &[String], body: &Prop, src: Source, both_ways: bool) -> Vec<Rule> {
    let all: BTreeSet<String> = vars.iter().cloned().collect();
    let vs = |t: &Term| {
        let mut s = BTreeSet::new();
        term_vars(t, &mut s);
        s
    };
    let mut out = Vec::new();
    match body {
        Prop::Eq(f) => {
            let (lv, rv) = (vs(&f.lhs), vs(&f.rhs));
            if matches!(f.lhs, Term::App(..) | Term::Param(_)) && lv == all && rv.is_subset(&lv) {
                out.push(Rule {
                    vars: vars.to_vec(),
                    kind: Kind::Eq {
                        l: f.lhs.clone(),
                        r: f.rhs.clone(),
                        flip: false,
                    },
                    src: src.clone(),
                });
            }
            if both_ways
                && matches!(f.rhs, Term::App(..) | Term::Param(_))
                && rv == all
                && lv.is_subset(&rv)
            {
                out.push(Rule {
                    vars: vars.to_vec(),
                    kind: Kind::Eq {
                        l: f.rhs.clone(),
                        r: f.lhs.clone(),
                        flip: true,
                    },
                    src,
                });
            }
        }
        Prop::Imp(a, b) => {
            if let Prop::Eq(fa) = &**a {
                let mut av = vs(&fa.lhs);
                av.extend(vs(&fa.rhs));
                let concl = match &**b {
                    Prop::Eq(fb) => Some((fb.lhs.clone(), fb.rhs.clone())),
                    Prop::False => None,
                    _ => return out,
                };
                if av == all && !matches!(fa.lhs, Term::Var(_)) {
                    out.push(Rule {
                        vars: vars.to_vec(),
                        kind: Kind::Horn {
                            al: fa.lhs.clone(),
                            ar: fa.rhs.clone(),
                            concl,
                        },
                        src,
                    });
                }
            }
        }
        _ => {}
    }
    out
}

/// Strip a ∀ prefix, naming the bound variables `?0`, `?1`, … .
fn strip_forall(p: &Prop) -> (usize, Prop) {
    let mut k = 0;
    let mut cur = p.clone();
    while let Prop::Forall(_, b) = &cur {
        let next = b.open(&Term::Var(format!("?{k}")));
        cur = next;
        k += 1;
    }
    (k, cur)
}

struct EGraph {
    terms: Vec<Term>,
    ids: HashMap<Term, usize>,
    head: Vec<String>,
    kids: Vec<Vec<usize>>,
    parent: Vec<usize>,
    members: Vec<Vec<usize>>,
    adj: Vec<Vec<usize>>,
    edges: Vec<Edge>,
    memo: HashMap<(usize, usize), Pf>,
    contradiction: Option<Pf>,
    max_nodes: usize,
    overflow: bool,
}

fn head_of(t: &Term) -> Option<String> {
    match t {
        Term::App(f, _) => Some(f.clone()),
        Term::Param(a) => Some(format!("@{a}")),
        _ => None,
    }
}

impl EGraph {
    fn new(max_nodes: usize) -> EGraph {
        EGraph {
            terms: Vec::new(),
            ids: HashMap::new(),
            head: Vec::new(),
            kids: Vec::new(),
            parent: Vec::new(),
            members: Vec::new(),
            adj: Vec::new(),
            edges: Vec::new(),
            memo: HashMap::new(),
            contradiction: None,
            max_nodes,
            overflow: false,
        }
    }

    fn add(&mut self, t: &Term) -> Option<usize> {
        if let Some(&i) = self.ids.get(t) {
            return Some(i);
        }
        let h = head_of(t)?;
        let ks = match t {
            Term::App(_, args) => args
                .iter()
                .map(|a| self.add(a))
                .collect::<Option<Vec<_>>>()?,
            _ => Vec::new(),
        };
        if self.terms.len() >= self.max_nodes {
            self.overflow = true;
            return None;
        }
        let i = self.terms.len();
        self.terms.push(t.clone());
        self.ids.insert(t.clone(), i);
        self.head.push(h);
        self.kids.push(ks);
        self.parent.push(i);
        self.members.push(vec![i]);
        self.adj.push(Vec::new());
        Some(i)
    }

    fn find(&mut self, mut x: usize) -> usize {
        while self.parent[x] != x {
            let g = self.parent[self.parent[x]];
            self.parent[x] = g;
            x = g;
        }
        x
    }

    fn find_ro(&self, mut x: usize) -> usize {
        while self.parent[x] != x {
            x = self.parent[x];
        }
        x
    }

    fn merge(&mut self, a: usize, b: usize, j: Just) -> bool {
        let (ra, rb) = (self.find(a), self.find(b));
        if ra == rb {
            return false;
        }
        let e = self.edges.len();
        self.edges.push(Edge { a, b, j });
        self.adj[a].push(e);
        self.adj[b].push(e);
        let (big, small) = if self.members[ra].len() >= self.members[rb].len() {
            (ra, rb)
        } else {
            (rb, ra)
        };
        self.parent[small] = big;
        let moved = std::mem::take(&mut self.members[small]);
        self.members[big].extend(moved);
        true
    }

    /// Congruence closure to a fixpoint.
    fn rebuild(&mut self) -> bool {
        let mut any = false;
        loop {
            let mut sig: HashMap<(String, Vec<usize>), usize> = HashMap::new();
            let mut todo = Vec::new();
            for n in 0..self.terms.len() {
                if self.kids[n].is_empty() {
                    continue;
                }
                let ks: Vec<usize> = self.kids[n].clone();
                let key = (
                    self.head[n].clone(),
                    ks.iter().map(|&k| self.find(k)).collect(),
                );
                match sig.get(&key) {
                    Some(&m) => {
                        if self.find(m) != self.find(n) {
                            todo.push((m, n));
                        }
                    }
                    None => {
                        sig.insert(key, n);
                    }
                }
            }
            if todo.is_empty() {
                return any;
            }
            for (m, n) in todo {
                if self.merge(m, n, Just::Cong) {
                    any = true;
                }
            }
        }
    }

    /// A proof of terms[a] = terms[b]; the two nodes are in one class.
    fn explain(&mut self, a: usize, b: usize) -> Pf {
        if a == b {
            return refl(&self.terms[a]);
        }
        if let Some(p) = self.memo.get(&(a, b)) {
            return p.clone();
        }
        // The forest path from a to b is unique.
        let mut prev: HashMap<usize, (usize, usize)> = HashMap::new();
        let mut q = VecDeque::from([a]);
        let mut seen = HashSet::from([a]);
        while let Some(u) = q.pop_front() {
            if u == b {
                break;
            }
            for &e in &self.adj[u] {
                let v = if self.edges[e].a == u {
                    self.edges[e].b
                } else {
                    self.edges[e].a
                };
                if seen.insert(v) {
                    prev.insert(v, (u, e));
                    q.push_back(v);
                }
            }
        }
        let mut path = Vec::new();
        let mut cur = b;
        while cur != a {
            let (u, e) = prev[&cur];
            path.push((u, e));
            cur = u;
        }
        path.reverse();
        let mut acc = refl(&self.terms[a]);
        for (u, e) in path {
            let step = self.edge_pf(e, u);
            acc = trans(acc, step);
        }
        self.memo.insert((a, b), acc.clone());
        acc
    }

    fn edge_pf(&mut self, e: usize, from: usize) -> Pf {
        let (a, b) = (self.edges[e].a, self.edges[e].b);
        match &self.edges[e].j {
            Just::Given(p) => {
                let p = p.clone();
                if a == from {
                    p
                } else {
                    symm(p)
                }
            }
            Just::Cong => {
                let (u, v) = if a == from { (a, b) } else { (b, a) };
                let (ku, kv) = (self.kids[u].clone(), self.kids[v].clone());
                let f = self.head[u].clone();
                let ps = ku
                    .iter()
                    .zip(kv.iter())
                    .map(|(&x, &y)| self.explain(x, y))
                    .collect();
                cong(&f, ps)
            }
        }
    }

    fn class_nodes(&self, c: usize) -> &[usize] {
        &self.members[self.find_ro(c)]
    }

    /// E-matching of a pattern (axiom or schema variables as `Var`) against a class.
    fn ematch(
        &self,
        pat: &Term,
        class: usize,
        sub: &BTreeMap<String, usize>,
        out: &mut Vec<BTreeMap<String, usize>>,
        limit: usize,
    ) {
        if out.len() >= limit {
            return;
        }
        match pat {
            Term::Var(v) => match sub.get(v) {
                Some(&n) => {
                    if self.find_ro(n) == self.find_ro(class) {
                        out.push(sub.clone());
                    }
                }
                None => {
                    let rep = *self
                        .class_nodes(class)
                        .iter()
                        .min()
                        .expect("nonempty class");
                    let mut s = sub.clone();
                    s.insert(v.clone(), rep);
                    out.push(s);
                }
            },
            Term::Bound(_) => {}
            _ => {
                let Some(h) = head_of(pat) else { return };
                let ps: &[Term] = match pat {
                    Term::App(_, args) => args,
                    _ => &[],
                };
                let mut ms: Vec<usize> = self.class_nodes(class).to_vec();
                ms.sort_unstable();
                for m in ms {
                    if self.head[m] != h || self.kids[m].len() != ps.len() {
                        continue;
                    }
                    let mut subs = vec![sub.clone()];
                    for (p, &k) in ps.iter().zip(self.kids[m].iter()) {
                        let mut next = Vec::new();
                        for s in &subs {
                            self.ematch(p, k, s, &mut next, limit);
                        }
                        subs = next;
                        if subs.is_empty() {
                            break;
                        }
                    }
                    for s in subs {
                        if out.len() >= limit {
                            return;
                        }
                        out.push(s);
                    }
                }
            }
        }
    }

    fn subst(&self, t: &Term, sub: &BTreeMap<String, usize>) -> Term {
        match t {
            Term::Var(v) => self.terms[sub[v]].clone(),
            Term::App(f, args) => {
                Term::App(f.clone(), args.iter().map(|a| self.subst(a, sub)).collect())
            }
            other => other.clone(),
        }
    }

    fn classes(&self) -> Vec<usize> {
        (0..self.terms.len())
            .filter(|&i| self.parent[i] == i)
            .collect()
    }

    /// Bounded saturation; returns the number of rounds run.
    fn saturate(&mut self, rules: &[Rule], rounds: usize) -> usize {
        const MATCH_LIMIT: usize = 4096;
        let mut applied: HashSet<(usize, Vec<Term>)> = HashSet::new();
        self.rebuild();
        for round in 0..rounds {
            if self.contradiction.is_some() || self.overflow {
                return round;
            }
            let mut changed = false;
            for (ri, rule) in rules.iter().enumerate() {
                let trigger = match &rule.kind {
                    Kind::Eq { l, .. } => l,
                    Kind::Horn { al, .. } => al,
                };
                let mut matches: Vec<BTreeMap<String, usize>> = Vec::new();
                for c in self.classes() {
                    let mut m1 = Vec::new();
                    self.ematch(trigger, c, &BTreeMap::new(), &mut m1, MATCH_LIMIT);
                    match &rule.kind {
                        // The antecedent's right side, in the class of its left side.
                        Kind::Horn { ar, .. } => {
                            for s in &m1 {
                                self.ematch(ar, c, s, &mut matches, MATCH_LIMIT);
                            }
                        }
                        Kind::Eq { .. } => matches.extend(m1),
                    }
                }
                for s in matches {
                    if rule.vars.iter().any(|v| !s.contains_key(v)) {
                        continue;
                    }
                    let key_terms: Vec<Term> =
                        rule.vars.iter().map(|v| self.terms[s[v]].clone()).collect();
                    if !applied.insert((ri, key_terms.clone())) {
                        continue;
                    }
                    let sigma: BTreeMap<String, Term> =
                        rule.vars.iter().cloned().zip(key_terms).collect();
                    let Some(ipf) = rule.instance(&sigma) else {
                        continue;
                    };
                    match &rule.kind {
                        Kind::Eq { l, r, .. } => {
                            let (lt, rt) = (self.subst(l, &s), self.subst(r, &s));
                            let (Some(ln), Some(rn)) = (self.add(&lt), self.add(&rt)) else {
                                return round + 1;
                            };
                            changed |= self.merge(ln, rn, Just::Given(ipf));
                        }
                        Kind::Horn { al, ar, concl } => {
                            let (at, art) = (self.subst(al, &s), self.subst(ar, &s));
                            let (Some(an), Some(rn)) = (self.add(&at), self.add(&art)) else {
                                return round + 1;
                            };
                            self.rebuild();
                            if self.find(an) != self.find(rn) {
                                continue;
                            }
                            let ante = self.explain(an, rn);
                            let mp = imp_e(ipf, ante);
                            match concl {
                                None => {
                                    self.contradiction = Some(mp);
                                    return round + 1;
                                }
                                Some((cl, cr)) => {
                                    let (ct, crt) = (self.subst(cl, &s), self.subst(cr, &s));
                                    let (Some(cn), Some(dn)) = (self.add(&ct), self.add(&crt))
                                    else {
                                        return round + 1;
                                    };
                                    changed |= self.merge(cn, dn, Just::Given(mp));
                                }
                            }
                        }
                    }
                }
                changed |= self.rebuild();
            }
            if !changed {
                return round + 1;
            }
        }
        rounds
    }
}

/// Bodies the equational core uses as rules: `l = r`, `l = r → l' = r'`, `l = r → ⊥`.
fn is_rule_body(p: &Prop) -> bool {
    match p {
        Prop::Eq(_) => true,
        Prop::Imp(a, b) => matches!(**a, Prop::Eq(_)) && matches!(**b, Prop::Eq(_) | Prop::False),
        _ => false,
    }
}

// ── Prop matching (backward chaining) ──────────────────────────────────────

fn match_term(pat: &Term, t: &Term, s: &mut BTreeMap<String, Term>) -> bool {
    match pat {
        Term::Var(v) => {
            if t.has_var() || t.has_bound() {
                return false;
            }
            match s.get(v) {
                Some(u) => u == t,
                None => {
                    s.insert(v.clone(), t.clone());
                    true
                }
            }
        }
        Term::App(f, ps) => match t {
            Term::App(g, ts) if f == g && ps.len() == ts.len() => {
                ps.iter().zip(ts).all(|(p, x)| match_term(p, x, s))
            }
            _ => false,
        },
        _ => pat == t,
    }
}

trait BoundFree {
    fn has_bound(&self) -> bool;
}
impl BoundFree for Term {
    /// Whether this term mentions a bound index (it cannot then be a witness).
    fn has_bound(&self) -> bool {
        match self {
            Term::Bound(_) => true,
            Term::App(_, a) => a.iter().any(|x| x.has_bound()),
            _ => false,
        }
    }
}

fn match_prop(pat: &Prop, p: &Prop, s: &mut BTreeMap<String, Term>) -> bool {
    match (pat, p) {
        (Prop::Eq(a), Prop::Eq(b)) => {
            match_term(&a.lhs, &b.lhs, s) && match_term(&a.rhs, &b.rhs, s)
        }
        (Prop::False, Prop::False) => true,
        (Prop::And(a, b), Prop::And(c, d))
        | (Prop::Or(a, b), Prop::Or(c, d))
        | (Prop::Imp(a, b), Prop::Imp(c, d)) => match_prop(a, c, s) && match_prop(b, d, s),
        (Prop::Forall(_, a), Prop::Forall(_, b)) | (Prop::Exists(_, a), Prop::Exists(_, b)) => {
            match_prop(a, b, s)
        }
        _ => false,
    }
}

// ── Search ─────────────────────────────────────────────────────────────────

/// Search bounds. Every bound is explicit and reported with every result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bounds {
    /// Maximum number of non-invertible choices on one branch.
    pub depth: usize,
    /// Maximum e-matching rounds per saturation.
    pub rounds: usize,
    /// Maximum e-graph nodes per saturation.
    pub nodes: usize,
    /// Largest numeral tried as an ∃ witness.
    pub witnesses: u64,
    /// Maximum search calls over the whole run.
    pub budget: u64,
}

impl Default for Bounds {
    fn default() -> Bounds {
        Bounds {
            depth: 3,
            rounds: 32,
            nodes: 4000,
            witnesses: 12,
            budget: 20_000,
        }
    }
}

impl Bounds {
    pub fn describe(&self) -> String {
        format!(
            "depth ≤ {}, saturation rounds ≤ {}, e-graph nodes ≤ {}, numeral witnesses ≤ {}, search calls ≤ {}",
            self.depth, self.rounds, self.nodes, self.witnesses, self.budget
        )
    }
}

/// What the search measured.
#[derive(Clone, Debug, Default)]
pub struct Stats {
    pub calls: u64,
    pub saturations: u64,
    pub max_nodes: usize,
    pub depth_reached: usize,
    pub budget_exhausted: bool,
    pub node_limit_hit: bool,
}

struct Fact {
    prop: Prop,
    pf: Pf,
}

#[derive(Clone, Default)]
struct Ctx {
    facts: Vec<Rc<Fact>>,
    pending_ex: Vec<Rc<Fact>>,
    bottom: Option<Pf>,
    params: Vec<String>,
    used: BTreeSet<String>,
}

impl Ctx {
    fn add(&mut self, prop: Prop, pf: Pf) {
        match &prop {
            Prop::False => {
                if self.bottom.is_none() {
                    self.bottom = Some(pf.clone());
                }
            }
            Prop::And(..) => {
                let (l, r) = (and_l(pf.clone()), and_r(pf.clone()));
                let (lp, rp) = (l.concl.clone(), r.concl.clone());
                self.add(lp, l);
                self.add(rp, r);
            }
            Prop::Exists(..) => {
                self.pending_ex.push(Rc::new(Fact {
                    prop: prop.clone(),
                    pf: pf.clone(),
                }));
            }
            _ => {}
        }
        if !self.facts.iter().any(|f| f.prop == prop) {
            self.facts.push(Rc::new(Fact { prop, pf }));
        }
    }
}

struct Search<'a> {
    axioms: &'a [Axiom],
    axiom_rules: Vec<Rule>,
    bounds: Bounds,
    stats: Stats,
    taken: BTreeSet<String>,
    next: usize,
    created_params: Vec<String>,
    created_hyps: Vec<String>,
}

fn collect_terms_prop(p: &Prop, out: &mut Vec<Term>) {
    match p {
        Prop::Eq(f) => {
            collect_terms(&f.lhs, out);
            collect_terms(&f.rhs, out);
        }
        Prop::False => {}
        Prop::And(a, b) | Prop::Or(a, b) | Prop::Imp(a, b) => {
            collect_terms_prop(a, out);
            collect_terms_prop(b, out);
        }
        Prop::Forall(_, b) | Prop::Exists(_, b) => collect_terms_prop(b, out),
    }
}

fn collect_terms(t: &Term, out: &mut Vec<Term>) {
    if !t.has_bound() && !t.has_var() && !out.contains(t) {
        out.push(t.clone());
    }
    if let Term::App(_, a) = t {
        a.iter().for_each(|x| collect_terms(x, out));
    }
}

impl<'a> Search<'a> {
    fn new(axioms: &'a [Axiom], bounds: Bounds, taken: BTreeSet<String>) -> Search<'a> {
        let mut axiom_rules = Vec::new();
        for ax in axioms {
            axiom_rules.extend(rules_of(
                ax.vars(),
                ax.body(),
                Source::Axiom(ax.clone()),
                false,
            ));
        }
        Search {
            axioms,
            axiom_rules,
            bounds,
            stats: Stats::default(),
            taken,
            next: 0,
            created_params: Vec::new(),
            created_hyps: Vec::new(),
        }
    }

    fn fresh(&mut self, prefix: &str) -> String {
        loop {
            self.next += 1;
            let n = format!("{prefix}{}", self.next);
            if self.taken.insert(n.clone()) {
                return n;
            }
        }
    }

    fn fresh_hyp(&mut self) -> String {
        let h = self.fresh("h");
        self.created_hyps.push(h.clone());
        h
    }

    fn fresh_param(&mut self) -> String {
        let p = self.fresh("p");
        self.created_params.push(p.clone());
        p
    }

    /// The equational core: saturate an e-graph built from the context.
    fn equational(&mut self, ctx: &Ctx, goal: &Prop) -> Option<Pf> {
        self.stats.saturations += 1;
        let mut g = EGraph::new(self.bounds.nodes);
        let mut rules = self.axiom_rules.clone();
        let mut ground: Vec<(Formula, Pf)> = Vec::new();
        for f in &ctx.facts {
            let (k, body) = strip_forall(&f.prop);
            match (&body, k) {
                (Prop::Eq(e), 0) => ground.push((e.clone(), f.pf.clone())),
                (Prop::Eq(_) | Prop::Imp(..), _) => {
                    let vars: Vec<String> = (0..k).map(|i| format!("?{i}")).collect();
                    rules.extend(rules_of(&vars, &body, Source::Fact(f.pf.clone(), k), true));
                }
                _ => {}
            }
        }
        for (e, pf) in &ground {
            let (Some(a), Some(b)) = (g.add(&e.lhs), g.add(&e.rhs)) else {
                break;
            };
            g.merge(a, b, Just::Given(pf.clone()));
        }
        // Terms of ground Horn antecedents, so ground rules can fire.
        for r in &rules {
            if let Kind::Horn { al, ar, .. } = &r.kind {
                if r.vars.is_empty() {
                    g.add(al);
                    g.add(ar);
                }
            }
        }
        let goal_nodes = match goal {
            Prop::Eq(f) => match (g.add(&f.lhs), g.add(&f.rhs)) {
                (Some(a), Some(b)) => Some((a, b)),
                _ => None,
            },
            _ => None,
        };
        g.saturate(&rules, self.bounds.rounds);
        self.stats.max_nodes = self.stats.max_nodes.max(g.terms.len());
        if g.overflow {
            self.stats.node_limit_hit = true;
        }
        if let Some(c) = g.contradiction.clone() {
            return Some(false_e(c, goal));
        }
        if let Some((a, b)) = goal_nodes {
            if g.find(a) == g.find(b) {
                return Some(g.explain(a, b));
            }
        }
        None
    }

    fn candidates(&self, ctx: &Ctx, goal: &Prop, numerals: bool) -> Vec<Term> {
        let mut out: Vec<Term> = ctx
            .params
            .iter()
            .rev()
            .map(|p| Term::Param(p.clone()))
            .collect();
        let mut ts = Vec::new();
        collect_terms_prop(goal, &mut ts);
        for f in &ctx.facts {
            collect_terms_prop(&f.prop, &mut ts);
        }
        ts.sort_by_key(|t| t.size());
        for t in ts {
            if !out.contains(&t) {
                out.push(t);
            }
        }
        if numerals {
            for n in 0..=self.bounds.witnesses {
                let t = Term::numeral(n);
                if !out.contains(&t) {
                    out.push(t);
                }
            }
        }
        out
    }

    fn prove(&mut self, ctx: &Ctx, goal: &Prop, depth: usize) -> Option<Pf> {
        self.stats.calls += 1;
        if self.stats.calls > self.bounds.budget {
            self.stats.budget_exhausted = true;
            return None;
        }
        if let Some(b) = &ctx.bottom {
            return Some(false_e(b.clone(), goal));
        }
        // ∃ in the context: eliminate with a fresh parameter (invertible).
        if let Some(ex) = ctx.pending_ex.first().cloned() {
            let Prop::Exists(_, body) = &ex.prop else {
                unreachable!()
            };
            let p = self.fresh_param();
            let h = self.fresh_hyp();
            let inst = body.open(&Term::Param(p.clone()));
            let mut c2 = ctx.clone();
            c2.pending_ex.remove(0);
            c2.params.push(p.clone());
            c2.add(inst.clone(), hyp(&h, &inst));
            let sub = self.prove(&c2, goal, depth)?;
            let c = sub.concl.clone();
            return Some(mk(c, R::ExistsE(ex.pf.clone(), p, h, sub)));
        }
        // Invertible goal rules.
        match goal {
            Prop::Forall(hint, body) => {
                let p = self.fresh_param();
                let mut c2 = ctx.clone();
                c2.params.push(p.clone());
                let sub = self.prove(&c2, &body.open(&Term::Param(p.clone())), depth)?;
                let c = Prop::Forall(hint.clone(), Box::new(sub.concl.close(&p)));
                return Some(mk(c, R::ForallI(p, sub)));
            }
            Prop::Imp(a, b) => {
                let h = self.fresh_hyp();
                let mut c2 = ctx.clone();
                c2.add((**a).clone(), hyp(&h, a));
                let sub = self.prove(&c2, b, depth)?;
                let c = Prop::Imp(a.clone(), Box::new(sub.concl.clone()));
                return Some(mk(c, R::ImpI(h, sub)));
            }
            Prop::And(a, b) => {
                let pa = self.prove(ctx, a, depth)?;
                let pb = self.prove(ctx, b, depth)?;
                return Some(mk(goal.clone(), R::AndI(pa, pb)));
            }
            _ => {}
        }
        if let Some(f) = ctx.facts.iter().find(|f| f.prop == *goal) {
            return Some(f.pf.clone());
        }
        if let Some(p) = self.equational(ctx, goal) {
            return Some(p);
        }
        if let Prop::Or(l, r) = goal {
            if let Some(p) = self.prove(ctx, l, depth) {
                return Some(mk(goal.clone(), R::OrL(p)));
            }
            if let Some(p) = self.prove(ctx, r, depth) {
                return Some(mk(goal.clone(), R::OrR(p)));
            }
        }
        // Direct instances of quantified facts and axioms (no antecedents).
        if let Some(p) = self.chain(ctx, goal, depth, false) {
            return Some(p);
        }
        if depth == 0 {
            return None;
        }
        self.stats.depth_reached = self.stats.depth_reached.max(depth);
        let d = depth - 1;
        if let Prop::Exists(_, body) = goal {
            for t in self.candidates(ctx, goal, true) {
                if let Some(p) = self.prove(ctx, &body.open(&t), d) {
                    return Some(mk(goal.clone(), R::ExistsI(t, p)));
                }
                if self.stats.budget_exhausted {
                    return None;
                }
            }
        }
        if let Some(p) = self.chain(ctx, goal, depth, true) {
            return Some(p);
        }
        // Case split on a disjunction in the context.
        for (i, f) in ctx.facts.iter().enumerate() {
            let Prop::Or(l, r) = &f.prop else { continue };
            let key = format!("or:{i}");
            if ctx.used.contains(&key) {
                continue;
            }
            let (h1, h2) = (self.fresh_hyp(), self.fresh_hyp());
            let mut c1 = ctx.clone();
            c1.used.insert(key.clone());
            c1.add((**l).clone(), hyp(&h1, l));
            let Some(p1) = self.prove(&c1, goal, d) else {
                if self.stats.budget_exhausted {
                    return None;
                }
                continue;
            };
            let mut c2 = ctx.clone();
            c2.used.insert(key);
            c2.add((**r).clone(), hyp(&h2, r));
            let Some(p2) = self.prove(&c2, goal, d) else {
                if self.stats.budget_exhausted {
                    return None;
                }
                continue;
            };
            return Some(mk(goal.clone(), R::OrCases(f.pf.clone(), h1, p1, h2, p2)));
        }
        // Modus ponens on an implication in the context whose consequent is
        // not an equation or ⊥ (those are Horn rules of the equational core).
        for (i, f) in ctx.facts.iter().enumerate() {
            let Prop::Imp(a, b) = &f.prop else { continue };
            if matches!(**a, Prop::Eq(_)) && matches!(**b, Prop::Eq(_) | Prop::False) {
                continue;
            }
            let key = format!("mp:{i}");
            if ctx.used.contains(&key) {
                continue;
            }
            let Some(pa) = self.prove(ctx, a, d) else {
                continue;
            };
            let mut c2 = ctx.clone();
            c2.used.insert(key);
            let pb = imp_e(f.pf.clone(), pa);
            c2.add((**b).clone(), pb);
            if let Some(p) = self.prove(&c2, goal, d) {
                return Some(p);
            }
        }
        // Instantiate quantified facts and non-equational axioms.
        let cands = self.candidates(ctx, goal, false);
        for (i, f) in ctx.facts.iter().enumerate() {
            let (k, body) = strip_forall(&f.prop);
            if k == 0 || is_rule_body(&body) {
                continue;
            }
            for t in &cands {
                let key = format!("inst:{i}:{}", t.kernel_form());
                if ctx.used.contains(&key) {
                    continue;
                }
                let pf = forall_e(f.pf.clone(), t);
                let mut c2 = ctx.clone();
                c2.used.insert(key);
                c2.add(pf.concl.clone(), pf);
                if let Some(p) = self.prove(&c2, goal, d) {
                    return Some(p);
                }
                if self.stats.budget_exhausted {
                    return None;
                }
            }
        }
        for ax in self.axioms {
            let structural = !matches!(ax.body(), Prop::Eq(_))
                && rules_of(ax.vars(), ax.body(), Source::Axiom(ax.clone()), false).is_empty();
            if !structural || ax.vars().len() != 1 {
                continue;
            }
            for t in &cands {
                if !matches!(t, Term::Param(_)) {
                    continue;
                }
                let key = format!("ax:{}:{}", ax.name(), t.kernel_form());
                if ctx.used.contains(&key) {
                    continue;
                }
                let sigma = BTreeMap::from([(ax.vars()[0].clone(), t.clone())]);
                let Some(pf) = inst(ax, &sigma) else { continue };
                let mut c2 = ctx.clone();
                c2.used.insert(key);
                c2.add(pf.concl.clone(), pf);
                if let Some(p) = self.prove(&c2, goal, d) {
                    return Some(p);
                }
                if self.stats.budget_exhausted {
                    return None;
                }
            }
        }
        None
    }

    /// Backward chaining: a fact or axiom `∀x̄. A₁ → … → H` whose head H
    /// matches the goal. With `antecedents` false, only antecedent-free
    /// heads are used (no depth is spent).
    fn chain(&mut self, ctx: &Ctx, goal: &Prop, depth: usize, antecedents: bool) -> Option<Pf> {
        if matches!(goal, Prop::Eq(_) | Prop::False) && !antecedents {
            // Equations and ⊥ are the equational core's.
            return None;
        }
        let mut schemas: Vec<(Vec<String>, Prop, Source)> = Vec::new();
        for f in &ctx.facts {
            let (k, body) = strip_forall(&f.prop);
            let vars = (0..k).map(|i| format!("?{i}")).collect();
            schemas.push((vars, body, Source::Fact(f.pf.clone(), k)));
        }
        for ax in self.axioms {
            schemas.push((
                ax.vars().to_vec(),
                ax.body().clone(),
                Source::Axiom(ax.clone()),
            ));
        }
        for (vars, body, src) in schemas {
            let mut antes = Vec::new();
            let mut head = body;
            while let Prop::Imp(a, b) = head.clone() {
                if !antecedents {
                    break;
                }
                antes.push(*a);
                head = *b;
            }
            if antes.is_empty() == antecedents {
                continue;
            }
            if vars.is_empty() && antes.is_empty() {
                continue; // exact facts are found directly
            }
            let mut s = BTreeMap::new();
            if !match_prop(&head, goal, &mut s) || vars.iter().any(|v| !s.contains_key(v)) {
                continue;
            }
            let rule = Rule {
                vars: vars.clone(),
                kind: Kind::Horn {
                    al: Term::zero(),
                    ar: Term::zero(),
                    concl: None,
                },
                src,
            };
            let Some(mut pf) = rule.instance(&s) else {
                continue;
            };
            let mut ok = true;
            for a in &antes {
                let a_inst = a.subst_vars(&s);
                match self.prove(ctx, &a_inst, depth.saturating_sub(1)) {
                    Some(pa) => pf = imp_e(pf, pa),
                    None => {
                        ok = false;
                        break;
                    }
                }
            }
            if ok && pf.concl == *goal {
                return Some(pf);
            }
            if self.stats.budget_exhausted {
                return None;
            }
        }
        None
    }
}

// ── Certificates ───────────────────────────────────────────────────────────

struct Printer {
    lines: Vec<String>,
    memo: HashMap<String, String>,
    avoid: BTreeSet<String>,
}

fn is_lower_ident(s: &str) -> bool {
    let mut cs = s.chars();
    matches!(cs.next(), Some(c) if c.is_ascii_lowercase())
        && cs.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

impl Printer {
    fn term(&self, t: &Term, names: &[String]) -> String {
        if let Some(n) = t.as_numeral() {
            return n.to_string();
        }
        match t {
            Term::Param(a) => a.clone(),
            Term::Bound(i) => names[names.len() - 1 - i].clone(),
            Term::Var(v) => format!("?{v}"),
            Term::App(f, a) if a.is_empty() => f.clone(),
            Term::App(f, a) => {
                let inner: Vec<String> = a.iter().map(|x| self.term(x, names)).collect();
                format!("({f} {})", inner.join(" "))
            }
        }
    }

    fn prop(&self, p: &Prop, names: &mut Vec<String>) -> String {
        match p {
            Prop::Eq(f) => format!(
                "(= {} {})",
                self.term(&f.lhs, names),
                self.term(&f.rhs, names)
            ),
            Prop::False => "false".into(),
            Prop::Imp(a, b) if **b == Prop::False => format!("(not {})", self.prop(a, names)),
            Prop::And(a, b) => format!("(and {} {})", self.prop(a, names), self.prop(b, names)),
            Prop::Or(a, b) => format!("(or {} {})", self.prop(a, names), self.prop(b, names)),
            Prop::Imp(a, b) => format!("(imp {} {})", self.prop(a, names), self.prop(b, names)),
            Prop::Forall(h, b) | Prop::Exists(h, b) => {
                let q = if matches!(p, Prop::Forall(..)) {
                    "forall"
                } else {
                    "exists"
                };
                let base = if is_lower_ident(h) {
                    h.clone()
                } else {
                    "x".into()
                };
                let mut name = base.clone();
                let mut k = 0;
                while names.contains(&name) || self.avoid.contains(&name) {
                    k += 1;
                    name = format!("{base}{k}");
                }
                names.push(name.clone());
                let inner = self.prop(b, names);
                names.pop();
                format!("({q} {name} {inner})")
            }
        }
    }

    fn emit(&mut self, p: &Pf) -> String {
        let t = |s: &Self, x: &Term| s.term(x, &[]);
        let args = match &p.rule {
            R::Hyp(h) => format!("hyp {h}"),
            R::Refl(x) => format!("refl {}", t(self, x)),
            R::Symm(a) => format!("symm {}", self.emit(a)),
            R::Trans(a, b) => {
                let (x, y) = (self.emit(a), self.emit(b));
                format!("trans {x} {y}")
            }
            R::Cong(f, ps) => {
                let ls: Vec<String> = ps.iter().map(|q| self.emit(q)).collect();
                format!("cong {f} {}", ls.join(" "))
            }
            R::Inst(ax, binds) => {
                let bs: Vec<String> = binds
                    .iter()
                    .map(|(v, x)| format!("({v} {})", t(self, x)))
                    .collect();
                format!("inst {ax} {}", bs.join(" "))
            }
            R::AndI(a, b) => {
                let (x, y) = (self.emit(a), self.emit(b));
                format!("and-intro {x} {y}")
            }
            R::AndL(a) => format!("and-left {}", self.emit(a)),
            R::AndR(a) => format!("and-right {}", self.emit(a)),
            R::OrL(a) => format!("or-left {}", self.emit(a)),
            R::OrR(a) => format!("or-right {}", self.emit(a)),
            R::OrCases(a, h1, b, h2, c) => {
                let (x, y, z) = (self.emit(a), self.emit(b), self.emit(c));
                format!("or-cases {x} {h1} {y} {h2} {z}")
            }
            R::ImpI(h, a) => format!("imp-intro {h} {}", self.emit(a)),
            R::ImpE(a, b) => {
                let (x, y) = (self.emit(a), self.emit(b));
                format!("imp-elim {x} {y}")
            }
            R::FalseE(a) => format!("false-elim {}", self.emit(a)),
            R::ForallI(pp, a) => format!("forall-intro {pp} {}", self.emit(a)),
            R::ForallE(a, x) => {
                let l = self.emit(a);
                format!("forall-elim {l} {}", t(self, x))
            }
            R::ExistsI(x, a) => {
                let l = self.emit(a);
                format!("exists-intro {} {l}", t(self, x))
            }
            R::ExistsE(a, pp, h, b) => {
                let (x, y) = (self.emit(a), self.emit(b));
                format!("exists-elim {x} {pp} {h} {y}")
            }
        };
        let body = format!("{} ({args})", self.prop(&p.concl, &mut Vec::new()));
        if let Some(l) = self.memo.get(&body) {
            return l.clone();
        }
        let label = format!("s{}", self.lines.len() + 1);
        self.lines.push(format!("    ({label} {body})"));
        self.memo.insert(body, label.clone());
        label
    }
}

fn sexp_text(e: &Sexp) -> String {
    match e {
        Sexp::Sym(s) => s.clone(),
        Sexp::Int(n) => n.to_string(),
        Sexp::Str(s) => {
            let mut o = String::from("\"");
            for c in s.chars() {
                if c == '"' || c == '\\' {
                    o.push('\\');
                }
                o.push(c);
            }
            o.push('"');
            o
        }
        Sexp::List(v) => format!(
            "({})",
            v.iter().map(sexp_text).collect::<Vec<_>>().join(" ")
        ),
    }
}

// ── Goal files ─────────────────────────────────────────────────────────────

/// A parsed goal file: the header fields a certificate repeats verbatim.
pub struct GoalFile {
    pub id: String,
    /// Header fields, in order, exactly as a certificate states them.
    pub fields: Vec<Sexp>,
    pub bounds: Bounds,
}

const GOAL_FIELDS: [&str; 9] = [
    "id",
    "convention",
    "goal",
    "mode",
    "assume",
    "axioms",
    "readingpoint",
    "relationship",
    "context",
];

/// Read `(rlt-goal …)`: the fields of a proof file with no proof, plus
/// optional `(bounds (depth N) (rounds N) (nodes N) (witnesses N) (budget N))`.
pub fn parse_goal_file(src: &str) -> Result<GoalFile, String> {
    let top = parse_all(src)?;
    let [root] = top.as_slice() else {
        return Err("a goal file holds exactly one (rlt-goal …) form".into());
    };
    let Some(("rlt-goal", fields)) = root.head() else {
        return Err("expected (rlt-goal …)".into());
    };
    let mut out = Vec::new();
    let mut bounds = Bounds::default();
    let mut id = String::new();
    for f in fields {
        let Some((key, rest)) = f.head() else {
            return Err(format!("field: expected (key value…), found {f:?}"));
        };
        match key {
            "bounds" => {
                for b in rest {
                    match b.head() {
                        Some((k, [Sexp::Int(n)])) => set_bound(&mut bounds, k, *n)?,
                        _ => return Err("(bounds (NAME N) …)".into()),
                    }
                }
            }
            "steps" | "params" | "cites" => {
                return Err(format!(
                    "({key} …): a goal file states a goal and supplies no proof"
                ))
            }
            k if GOAL_FIELDS.contains(&k) => {
                if k == "id" {
                    if let [Sexp::Sym(s) | Sexp::Str(s)] = rest {
                        id = s.clone();
                    }
                }
                out.push(f.clone());
            }
            other => return Err(format!("unknown field `{other}`")),
        }
    }
    if id.is_empty() {
        return Err("missing field (id …)".into());
    }
    Ok(GoalFile {
        id,
        fields: out,
        bounds,
    })
}

pub fn set_bound(b: &mut Bounds, k: &str, n: u64) -> Result<(), String> {
    let cap = |n: u64, max: u64| -> Result<u64, String> {
        if n > max {
            Err(format!("bound {k} = {n} exceeds the ceiling {max}"))
        } else {
            Ok(n)
        }
    };
    match k {
        "depth" => b.depth = cap(n, 12)? as usize,
        "rounds" => b.rounds = cap(n, 1000)? as usize,
        "nodes" => b.nodes = cap(n, 200_000)? as usize,
        "witnesses" => b.witnesses = cap(n, 1000)?,
        "budget" => b.budget = cap(n, 10_000_000)?,
        other => return Err(format!("unknown bound `{other}`")),
    }
    Ok(())
}

fn header_text(fields: &[Sexp]) -> String {
    let mut s = String::from("(rlt-proof\n");
    for f in fields {
        let _ = writeln!(s, "  {}", sexp_text(f));
    }
    s
}

// ── The prover ─────────────────────────────────────────────────────────────

pub enum Verdict {
    /// The certificate was replayed and accepted by the kernel.
    Proved {
        certificate: String,
        sha256: String,
        theorem: Theorem,
        steps: usize,
        replay_report: String,
    },
    /// No proof within the bounds. Nothing follows about the goal's truth.
    NotFound,
    /// Malformed input, or a certificate the kernel refused.
    Error(String),
}

pub struct ProveOutcome {
    pub verdict: Verdict,
    pub bounds: Bounds,
    pub stats: Stats,
    pub goal_shown: Option<String>,
    pub depth_found: Option<usize>,
}

impl ProveOutcome {
    pub fn code(&self) -> i32 {
        match self.verdict {
            Verdict::Proved { .. } => 0,
            Verdict::NotFound => 1,
            Verdict::Error(_) => 2,
        }
    }
    pub fn label(&self) -> &'static str {
        match self.verdict {
            Verdict::Proved { .. } => "PROVED",
            Verdict::NotFound => "NOT_FOUND_WITHIN_BOUNDS",
            Verdict::Error(_) => "ERROR",
        }
    }
}

/// The elaborated header of a goal: what the search works on.
struct Elaborated {
    goal: Prop,
    mode: Mode,
    assumptions: Vec<(String, Prop)>,
    axioms: Vec<Axiom>,
    taken: BTreeSet<String>,
}

fn elaborate(root: &Path, header: &str) -> Result<Elaborated, String> {
    let defs =
        rlt_syntax::proof::reads_definitions(header).then(|| load_definitions(root).map(Rc::new));
    let pf = rlt_syntax::proof::parse_proof_file_with(header, defs.clone())?;
    let convs = pinned_conventions(root)?;
    let conv = convs
        .get(&pf.convention)
        .ok_or_else(|| format!("convention `{}` is unregistered", pf.convention))?;
    let mut convention = conv.convention.clone();
    let mut taken: BTreeSet<String> = rlt_kernel::RULES.iter().map(|s| s.to_string()).collect();
    if !convention.definitions.is_empty() {
        match &defs {
            Some(Ok(d)) => {
                taken.extend(d.list.iter().map(|x| x.name.clone()));
                convention.defs = Some(d.clone());
            }
            Some(Err(e)) => return Err(e.clone()),
            None => return Err("definitions: unavailable".into()),
        }
    }
    let po = pf.to_proof_object(&convention)?;
    let set = load_axiom_set(root, &pf.axiom_set)?;
    taken.extend(set.axioms.iter().map(|a| a.name().to_string()));
    taken.extend(po.assumptions.iter().map(|(n, _)| n.clone()));
    Ok(Elaborated {
        goal: po.goal,
        mode: po.mode,
        assumptions: po.assumptions,
        axioms: set.axioms,
        taken,
    })
}

fn split_tokens(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for c in s.chars() {
        if c.is_whitespace() || c == '(' || c == ')' {
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
            out.push(c.to_string());
        } else {
            cur.push(c);
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// Build the certificate text for a found proof. The search's parameter and
/// local-assumption names are renumbered in order of first appearance
/// (p1, p2, …; h1, h2, …), avoiding every other name in the file.
fn certificate(
    fields: &[Sexp],
    params: &[String],
    hyps: &[String],
    pf: &Pf,
    taken: &BTreeSet<String>,
) -> (String, usize) {
    let mut avoid = taken.clone();
    avoid.extend(params.iter().cloned());
    let mut pr = Printer {
        lines: Vec::new(),
        memo: HashMap::new(),
        avoid,
    };
    pr.emit(pf);
    let steps = pr.lines.len();
    let toks = split_tokens(&pr.lines.join("\n"));
    let created: BTreeSet<&str> = params.iter().chain(hyps).map(String::as_str).collect();
    let mut blocked: BTreeSet<String> = taken.clone();
    blocked.extend(
        toks.iter()
            .filter(|t| !created.contains(t.as_str()))
            .cloned(),
    );
    let mut rename: HashMap<String, String> = HashMap::new();
    let mut used_params = Vec::new();
    let (mut np, mut nh) = (0, 0);
    for t in &toks {
        if rename.contains_key(t) || !created.contains(t.as_str()) {
            continue;
        }
        let is_param = params.contains(t);
        let name = loop {
            let cand = if is_param {
                np += 1;
                format!("p{np}")
            } else {
                nh += 1;
                format!("h{nh}")
            };
            if !blocked.contains(&cand) {
                break cand;
            }
        };
        if is_param {
            used_params.push(name.clone());
        }
        rename.insert(t.clone(), name);
    }
    let body: String = toks
        .iter()
        .map(|t| rename.get(t).cloned().unwrap_or_else(|| t.clone()))
        .collect();
    let mut s = String::from(
        "; Generated by `rlt prove` (untrusted bounded search). Its authority is the\n; kernel replay alone: `rlt check` this file.\n",
    );
    s.push_str(&header_text(fields));
    if !used_params.is_empty() {
        let _ = writeln!(s, "  (params {})", used_params.join(" "));
    }
    s.push_str("  (steps\n");
    s.push_str(&body);
    s.push_str("))\n");
    (s, steps)
}

/// The single gate to `PROVED`: a certificate counts only if its header
/// states exactly the goal file's fields and the kernel replay of its bytes,
/// through the function `rlt check` uses, accepts. `cert.as_bytes()` is what
/// is replayed; callers hash the same bytes.
pub fn certify(
    root: &Path,
    goal: &GoalFile,
    cert_path: &Path,
    cert: &str,
) -> Result<(Theorem, String), String> {
    let top = parse_all(cert).map_err(|e| format!("certificate: {e}"))?;
    let [rootform] = top.as_slice() else {
        return Err("certificate: expected one (rlt-proof …) form".into());
    };
    let Some(("rlt-proof", fields)) = rootform.head() else {
        return Err("certificate: expected (rlt-proof …)".into());
    };
    let header: Vec<&Sexp> = fields
        .iter()
        .filter(|f| !matches!(f.head(), Some(("params" | "steps", _))))
        .collect();
    let expected: Vec<&Sexp> = goal.fields.iter().collect();
    if header != expected {
        return Err("certificate header differs from the goal file (goal, mode, assumptions, convention and axiom set must be stated exactly as given)".into());
    }
    let out = check_bytes_outcome(root, cert_path, cert.as_bytes().to_vec());
    if out.report.code != ACCEPT {
        return Err(format!(
            "the kernel did not accept the certificate:\n{}",
            out.report.text
        ));
    }
    let th = out
        .theorem
        .ok_or("internal: replay accepted without a theorem")?;
    Ok((th, out.report.text))
}

/// Prove a goal file's source; write the certificate to `out` when one is found.
pub fn prove_source(
    root: &Path,
    goal_src: &str,
    overrides: &[(String, u64)],
    out: Option<&Path>,
) -> ProveOutcome {
    let mut res = ProveOutcome {
        verdict: Verdict::NotFound,
        bounds: Bounds::default(),
        stats: Stats::default(),
        goal_shown: None,
        depth_found: None,
    };
    let goal = match parse_goal_file(goal_src) {
        Ok(g) => g,
        Err(e) => {
            res.verdict = Verdict::Error(e);
            return res;
        }
    };
    let mut bounds = goal.bounds;
    for (k, n) in overrides {
        if let Err(e) = set_bound(&mut bounds, k, *n) {
            res.verdict = Verdict::Error(e);
            return res;
        }
    }
    res.bounds = bounds;
    let header = format!("{})", header_text(&goal.fields));
    let el = match elaborate(root, &header) {
        Ok(e) => e,
        Err(e) => {
            res.verdict = Verdict::Error(e);
            return res;
        }
    };
    res.goal_shown = Some(el.goal.to_string());
    let mut search = Search::new(&el.axioms, bounds, el.taken.clone());
    let mut ctx = Ctx::default();
    // Declared assumptions enter the context only in conditional mode; an
    // unconditional goal is searched with Γ = ∅.
    if el.mode == Mode::Conditional {
        for (n, f) in &el.assumptions {
            ctx.add(f.clone(), hyp(n, f));
        }
    }
    let mut found = None;
    for d in 0..=bounds.depth {
        if let Some(p) = search.prove(&ctx, &el.goal, d) {
            found = Some((p, d));
            break;
        }
        if search.stats.budget_exhausted {
            break;
        }
    }
    res.stats = search.stats.clone();
    let Some((pf, d)) = found else {
        return res;
    };
    res.depth_found = Some(d);
    let (cert, steps) = certificate(
        &goal.fields,
        &search.created_params,
        &search.created_hyps,
        &pf,
        &el.taken,
    );
    let default_path =
        PathBuf::from("target/rlt-prove").join(format!("{}.rlt", safe_name(&goal.id)));
    let path = out.map(Path::to_path_buf).unwrap_or(default_path);
    let full = if path.is_absolute() {
        path.clone()
    } else {
        root.join(&path)
    };
    if let Some(dir) = full.parent() {
        if let Err(e) = std::fs::create_dir_all(dir) {
            res.verdict = Verdict::Error(format!("cannot create {}: {}", dir.display(), e.kind()));
            return res;
        }
    }
    if let Err(e) = std::fs::write(&full, &cert) {
        res.verdict = Verdict::Error(format!("cannot write the certificate: {}", e.kind()));
        return res;
    }
    // Replay the bytes on disk, read back, not the in-memory proof.
    let bytes = match std::fs::read(&full) {
        Ok(b) => b,
        Err(e) => {
            res.verdict = Verdict::Error(format!("cannot read the certificate back: {}", e.kind()));
            return res;
        }
    };
    // The bytes hashed are the bytes replayed (audit K9): no lossy decoding,
    // so `text.as_bytes()` is exactly `bytes`.
    let text = match String::from_utf8(bytes.clone()) {
        Ok(t) => t,
        Err(_) => {
            res.verdict = Verdict::Error(
                "certificate refused; nothing is claimed: the bytes read back are not UTF-8".into(),
            );
            return res;
        }
    };
    debug_assert_eq!(text.as_bytes(), bytes.as_slice());
    match certify(root, &goal, &full, &text) {
        Ok((theorem, replay_report)) => {
            res.verdict = Verdict::Proved {
                certificate: path.display().to_string(),
                sha256: sha256_hex(&bytes),
                theorem,
                steps,
                replay_report,
            }
        }
        Err(e) => {
            res.verdict = Verdict::Error(format!("certificate refused; nothing is claimed: {e}"))
        }
    }
    res
}

pub fn safe_name(id: &str) -> String {
    id.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// `rlt prove GOAL [--out FILE] [--depth N] …`: the report.
pub fn prove_report(
    root: &Path,
    goal_file: &Path,
    overrides: &[(String, u64)],
    out: Option<&Path>,
) -> (String, i32) {
    let mut t = String::new();
    let shown = goal_file
        .strip_prefix(root)
        .unwrap_or(goal_file)
        .display()
        .to_string();
    let _ = writeln!(t, "RLT prove       {}", shown.escape_debug());
    let src = match rlt_syntax::records::read_regular(goal_file) {
        Ok(s) => s,
        Err(e) => {
            let _ = writeln!(t, "verdict         ERROR — cannot read the goal file: {e}");
            return (t, 2);
        }
    };
    let r = prove_source(root, &src, overrides, out);
    render(&mut t, &r);
    (t, r.code())
}

pub fn render(t: &mut String, r: &ProveOutcome) {
    if let Some(g) = &r.goal_shown {
        let _ = writeln!(t, "goal            {g}");
    }
    let _ = writeln!(t, "bounds          {}", r.bounds.describe());
    let s = &r.stats;
    let _ = writeln!(
        t,
        "search          {} calls, {} saturations, largest e-graph {} nodes{}{}",
        s.calls,
        s.saturations,
        s.max_nodes,
        if s.budget_exhausted {
            "; call budget exhausted"
        } else {
            ""
        },
        if s.node_limit_hit {
            "; node limit reached"
        } else {
            ""
        }
    );
    match &r.verdict {
        Verdict::Proved {
            certificate,
            sha256,
            theorem,
            steps,
            ..
        } => {
            let _ = writeln!(
                t,
                "found           at depth {} — {steps} steps",
                r.depth_found.unwrap_or(0)
            );
            let _ = writeln!(t, "certificate     {certificate}");
            let _ = writeln!(t, "  sha256        {sha256}");
            let _ = writeln!(t, "replay          ACCEPT — the certificate bytes were re-read, re-parsed and replayed by the RLT kernel (`rlt check {certificate}`)");
            let g: Vec<String> = theorem
                .assumptions()
                .iter()
                .map(|(n, f)| format!("{n} : {f}"))
                .collect();
            let _ = writeln!(
                t,
                "  Γ             {}",
                if g.is_empty() {
                    "∅".to_string()
                } else {
                    format!("{{{}}}", g.join(", "))
                }
            );
            let ax: Vec<&String> = theorem.axioms_used().keys().collect();
            let _ = writeln!(
                t,
                "  axioms used   {}",
                if ax.is_empty() {
                    "none".to_string()
                } else {
                    ax.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
                }
            );
            let _ = writeln!(t, "verdict         PROVED — the kernel-checked theorem holds under the axioms used and Γ above; their truth stays assumed");
        }
        Verdict::NotFound => {
            let _ = writeln!(t, "verdict         NOT_FOUND_WITHIN_BOUNDS — no proof was found within the bounds above; this says nothing about whether the goal is true, false or independent of the axioms");
        }
        Verdict::Error(e) => {
            let _ = writeln!(t, "verdict         ERROR — {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn egraph_explains_transitive_and_congruent_equalities() {
        let (a, b, c) = (
            Term::Param("a".into()),
            Term::Param("b".into()),
            Term::Param("c".into()),
        );
        let mut g = EGraph::new(100);
        let ab = Prop::eq(a.clone(), b.clone());
        let bc = Prop::eq(b.clone(), c.clone());
        let (na, nb, nc) = (g.add(&a).unwrap(), g.add(&b).unwrap(), g.add(&c).unwrap());
        g.merge(na, nb, Just::Given(hyp("h1", &ab)));
        g.merge(nb, nc, Just::Given(hyp("h2", &bc)));
        let sa = g.add(&Term::succ(a.clone())).unwrap();
        let sc = g.add(&Term::succ(c.clone())).unwrap();
        g.rebuild();
        assert_eq!(g.find(sa), g.find(sc));
        let p = g.explain(sa, sc);
        assert_eq!(p.concl, Prop::eq(Term::succ(a), Term::succ(c)));
    }
}
