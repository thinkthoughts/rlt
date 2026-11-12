# RLT theorem library (RLT-004c)

Each entry `lib/<area>/<id>.yaml` names one checked statement under one axiom set and points at the proof
files that prove it. Proof files stay where they were checked (`tests/…`); their citations keep naming
them by path and SHA-256, so no existing hash changes.

- **Identity.** A library id is bound, permanently, to a *statement fingerprint*: the SHA-256 of a canonical
  text holding the convention and its pinned interpretation digest, the axiom set with its file and
  SHA-256, the definitions file and the definitions used, and the checked formula and each assumption in a
  canonical form (bound variables as de Bruijn indices; assumption names dropped, since they label steps
  inside a proof). Alpha-variants therefore share one fingerprint. `rlt deps ID` prints the text.
- **Registry.** `identities.tsv` records each id with its fingerprint and first proof SHA-256. It is
  append-only: `scripts/check-identities.sh`, run by `scripts/verify.sh`, refuses any change to a line of any committed version of the file.
- **Replacement.** A proof may be replaced under the same id only by a proof whose fingerprint is identical.
  The entry's `history` keeps every earlier pin (path, SHA-256, fingerprint, date, reviewer); once the first
  registered proof is replaced, it must appear there.
- **Alternatives.** Further accepted proofs of exactly the same statement are listed under `alternatives`.
- **Citable.** Only unconditional theorems (Γ = ∅) are citable; conditional entries say `citable: false`.

Entries are metadata. `rlt deps --verify-lib` compares every field with the checked proofs; `rlt deps ID`
shows one theorem's complete dependency graph. Proposed with `scripts/gen-lib.py`, then reviewed.
