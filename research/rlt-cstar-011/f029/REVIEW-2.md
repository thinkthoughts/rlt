# F-029 revision — independent review, rounds 2 and 3 (condensed)

The same reviewer as round 1 worked read-only on copies under `/tmp/f029v2_review/`.

## Round 2 (`19cdfff`): ACCEPT, with two wording fixes

**What the reviewer checked:**
- There is one tile check per `blueprint_problems` call, on `generated_tile_settings`.
- The author's oracle reproduced 12 of 12.
- The reviewer's own oracle covers 21 scenarios, adding mixed modes, an integer flag, missing sections, a stale `llm`, a 16×1 tiling and a stale BGC flag with physics only. Wrong results: parent 12, revision 0.
- On 4,000 random inputs, the output with no `tile_settings` is byte-identical to the parent's.
- The resolver, the executor and the wizard call the check without `tile_settings`.
- Three broken copies (M1–M3) fail 6, 5 and 6 unit tests respectively, and M1 also fails the Tier 1 test.

**Must-fix (wording):** the docstring and the commit message wrongly said that a reused file overrides an explicit mode.

**Nice-to-have:**
- read the sections from `TILE_SIZE_SECTIONS`;
- align the wizard's load-time check;
- refuse inconsistent `auto` modes before generation (an existing issue);
- add grid-file and reused-file cases.

## Missed by round 2, caught by the author's chunked unit run

The stub `fake_validate(model_settings, *, roms_ref)` in `test_hash_pin_tier.py` does not accept the new keyword, so the test failed with a `TypeError`.

## Round 3 (`1f56f37`): ACCEPT

- Both wording fixes are confirmed.
- `TILE_SIZE_SECTIONS` is now the single list of sections.
- The wizard change has no import cycle, and its test fails without it.
- `tile_settings=None` is the right fix for the stub. A search of the whole test tree found no other stub of the four functions.
- Unit test files on `1f56f37`:

| File | Passed |
|---|---|
| `test_hash_pin_tier.py` | 9 |
| `test_early_refusals.py` | 101 |
| `test_forge_blueprint.py` | 739 |
| `test_namelist_model.py` | 698 |
| `wizard/test_wizard.py` | 489 |

## Re-run by the author on `1f56f37`

- Oracle: 0 wrong.
- Tier 1: 101 passed.
- Non-regression on 4,000 inputs: byte-identical.
- Full chunked unit suite: 10,029 passed, with the same 4 local-environment failures as at the parent.
