/-
RLT-PROVER-002 (audit D6): the authoritative Lean declaration
correspondence check for lean/ProverDefinitions.lean.

Run by scripts/check-definitions-lean.py as
  lean --run DeclCheck.lean SYSROOT SPEC USER_DIR USER_MODULE EXP_DIR EXP_MODULE

This program never imports the companion file. Its own source is elaborated
against core Lean only, so notation, macros, instances or options declared
in the companion cannot change how this checker is read. At run time it
loads two compiled environments separately:

- USER_MODULE: lean/ProverDefinitions.lean, compiled by Lean (kernel-checked);
- EXP_MODULE: the expected module that the script generated from RLT's own
  printed statements and the registered written definitions, compiled by
  Lean on its own (never importing the companion). For each expected theorem
  T it holds `def T.expected : Prop := STATEMENT`; for each `FN_unfolds` it
  also holds `theorem FN_unfolds.by_rfl : FN_unfolds.expected := Iff.rfl`,
  so Lean's kernel checks the unfolding by definitional equality in a clean
  environment.

The verdict compares elaborated declarations (`Expr`s, up to alpha
equivalence; binder names and binder annotations are ignored), never source
text or pretty-printed output:

1. the companion module imports only `Init`;
2. every constant the companion module declares is an expected name, or an
   internal auxiliary theorem (`T._proof_…`) under an expected theorem T;
3. each expected `def` is a safe, non-opaque definition with no universe
   parameters, whose type and value equal the expected module's;
4. each expected theorem is a theorem with no universe parameters, whose
   type equals the expected module's `T.expected`;
5. each `FN_unfolds.by_rfl` exists in the expected module as a theorem of
   type `FN_unfolds.expected`;
6. each expected theorem depends only on propext, Classical.choice and
   Quot.sound (collected from the kernel terms, not from `#print`).

Any missing, extra, differently typed or differently valued declaration
fails closed. Output ends with `RESULT PASS N` or `RESULT FAIL N`.
-/
import Lean
open Lean

def standardAxioms : List Name := [``propext, ``Classical.choice, ``Quot.sound]

def loadModule (sysroot dir : System.FilePath) (m : Name) : IO Environment := do
  searchPathRef.set [sysroot / "lib" / "lean", dir]
  importModules #[{ module := m }] {} (trustLevel := 0)

def kindOf : ConstantInfo → String
  | .thmInfo _ => "theorem" | .defnInfo _ => "def" | .axiomInfo _ => "axiom"
  | .opaqueInfo _ => "opaque" | .inductInfo _ => "inductive" | .ctorInfo _ => "constructor"
  | .recInfo _ => "recursor" | .quotInfo _ => "quot"

/-- Axioms reachable from `c`, by walking the kernel terms (types and values)
of every constant it uses. No precomputed cache stored in an .olean is
consulted. A constant that cannot be found is reported as `unknown.NAME`,
which is never a standard axiom. -/
partial def collectAxioms (env : Environment) (c : Name) : StateM (NameSet × NameSet) Unit := do
  if (← get).1.contains c then return
  modify fun (s, a) => (s.insert c, a)
  match env.find? c with
  | some (.axiomInfo _) => modify fun (s, a) => (s, a.insert c)
  | some ci =>
    let used := ci.type.getUsedConstants ++
      ((ci.value? (allowOpaque := true)).map (·.getUsedConstants) |>.getD #[])
    for d in used do collectAxioms env d
  | none => modify fun (s, a) => (s, a.insert (Name.mkSimple "unknown" ++ c))

def axiomsOf (env : Environment) (c : Name) : List Name :=
  let ((), (_, ax)) := (collectAxioms env c).run ({}, {})
  ax.toList

/-- The non-internal prefix of a name: `F4_holds._proof_1_1` ↦ `F4_holds`. -/
partial def publicPrefix : Name → Name
  | n@(.str p _) => if n.isInternal then publicPrefix p else n
  | n@(.num p _) => if n.isInternal then publicPrefix p else n
  | .anonymous => .anonymous

def main (args : List String) : IO UInt32 := do
  let [sysroot, spec, udir, umod, edir, emod] := args
    | IO.println "usage: DeclCheck SYSROOT SPEC USER_DIR USER_MODULE EXP_DIR EXP_MODULE"; return 2
  let mut defs : Array Name := #[]
  let mut thms : Array Name := #[]
  let mut unfolds : Array Name := #[]
  for line in (← IO.FS.lines spec) do
    match line.splitOn " " with
    | ["def", n] => defs := defs.push n.toName
    | ["theorem", n] => thms := thms.push n.toName
    | ["unfolds", n] => unfolds := unfolds.push n.toName
    | [""] => pure ()
    | _ => IO.println s!"RESULT FAIL 0 (malformed spec line `{line}`)"; return 2
  let uenv ← loadModule sysroot udir umod.toName
  let eenv ← loadModule sysroot edir emod.toName
  let mut bad : Nat := 0
  let mut ok : Nat := 0
  -- 1. imports
  let some uidx := uenv.getModuleIdx? umod.toName
    | IO.println s!"RESULT FAIL 0 (module {umod} not loaded)"; return 1
  let udata := uenv.header.moduleData[uidx.toNat]!
  for imp in udata.imports do
    if imp.module != `Init then
      IO.println s!"  REFUSED: the companion imports {imp.module}; only Init is allowed"
      bad := bad + 1
  -- 2. no constant beyond the expected names and their internal proofs; an
  --    internal auxiliary theorem counts only if its expected parent's proof
  --    actually uses it (Lean's own `T._proof_…` lemmas), so a hand-written
  --    theorem cannot hide under an internal-looking name.
  for n in udata.constNames do
    let expected := defs.contains n || thms.contains n
    let parent := publicPrefix n
    let usedByParent := match uenv.find? parent with
      | some (.thmInfo p) => p.value.getUsedConstants.contains n
      | _ => false
    let aux := n.isInternal && thms.contains parent && usedByParent &&
      (match uenv.find? n with | some (.thmInfo _) => true | _ => false)
    unless expected || aux do
      let k := (uenv.find? n).map kindOf |>.getD "?"
      IO.println s!"  REFUSED: unexpected {k} `{n}` declared by the companion"
      bad := bad + 1
  -- 3. definitions: type and value as registered
  for d in defs do
    match uenv.find? d, eenv.find? d with
    | some (.defnInfo u), some (.defnInfo e) =>
      if u.safety != .safe then
        IO.println s!"  REFUSED: def `{d}` is not a safe definition"; bad := bad + 1
      else if !u.levelParams.isEmpty then
        IO.println s!"  REFUSED: def `{d}` has universe parameters"; bad := bad + 1
      else if !(u.type.eqv e.type) then
        IO.println s!"  REFUSED: def `{d}` has type {u.type}; registered {e.type}"; bad := bad + 1
      else if !(u.value.eqv e.value) then
        IO.println s!"  REFUSED: def `{d}` has value {u.value}; registered {e.value}"; bad := bad + 1
      else
        IO.println s!"  def {d}: elaborated type and value equal the registered definition"; ok := ok + 1
    | some c, _ => IO.println s!"  REFUSED: `{d}` is a {kindOf c}, not a definition"; bad := bad + 1
    | none, _ => IO.println s!"  REFUSED: def `{d}` is missing from the companion"; bad := bad + 1
  -- 4 and 6. theorems: elaborated type equal to the expected proposition
  for t in thms do
    let en := t ++ `expected
    match uenv.find? t, eenv.find? en with
    | some (.thmInfo u), some (.defnInfo e) =>
      let ax := axiomsOf uenv t
      let extra := ax.filter (fun a => !standardAxioms.contains a)
      if !u.levelParams.isEmpty then
        IO.println s!"  REFUSED: theorem `{t}` has universe parameters"; bad := bad + 1
      else if !(u.type.eqv e.value) then
        IO.println s!"  REFUSED: theorem `{t}` has elaborated type {u.type}; expected {e.value}"; bad := bad + 1
      else if !extra.isEmpty then
        IO.println s!"  REFUSED: theorem `{t}` depends on non-standard axioms {extra}"; bad := bad + 1
      else
        IO.println s!"  theorem {t}: elaborated type equals the expected proposition; axioms {ax}"; ok := ok + 1
    | some c, some _ => IO.println s!"  REFUSED: `{t}` is a {kindOf c}, not a theorem"; bad := bad + 1
    | none, _ => IO.println s!"  REFUSED: theorem `{t}` is missing from the companion"; bad := bad + 1
    | _, none => IO.println s!"  REFUSED: the expected module lacks `{en}`"; bad := bad + 1
  -- 5. unfoldings checked by Iff.rfl in the clean expected module
  for t in unfolds do
    match eenv.find? (t ++ `by_rfl) with
    | some (.thmInfo r) =>
      if r.type.eqv (mkConst (t ++ `expected)) then
        IO.println s!"  {t}: Lean's kernel accepts Iff.rfl for the expected proposition (definitional equality, clean environment)"
      else
        IO.println s!"  REFUSED: {t}.by_rfl has an unexpected type"; bad := bad + 1
    | _ => IO.println s!"  REFUSED: the expected module lacks {t}.by_rfl"; bad := bad + 1
  if bad == 0 && ok == defs.size + thms.size then
    IO.println s!"RESULT PASS {ok}"; return 0
  else
    IO.println s!"RESULT FAIL {bad}"; return 1
