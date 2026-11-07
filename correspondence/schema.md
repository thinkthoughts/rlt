# Physical correspondence records (component E)

Correspondence records live inside each reading point file (`readingpoints/*.yaml`, key `correspondence`). Each record has:

| Field | Meaning |
|---|---|
| `id` | record id, e.g. `math0.imagining` |
| `kind` | `behavior` (a situated human act), `representation` (a written sign standing for a referent), or `referent` (the thing a sign or act is about) |
| `description` | plain words |
| `status` | from `status.yaml` → `correspondence_status` |
| `evidence` | list of `{type, ref}`; `type` from `allowed_support` |
| `source` | registered source id for the description |

Rules checked by `rlt validate`:

- `status` is in the vocabulary.
- `imagined` applies to `referent` records only. Imagining itself is a `behavior` record and is graded like any other behavior.
- A record with empty `evidence` holds `unestablished`, `inapplicable` or `imagined`. `supported` and `rejected` require evidence.
- Evidence types are `physical_measurement` or `experimentally_grounded_observation`. A kernel theorem is never an evidence type.

Behavior records describe human roles ("a reader", "a driver"). The RLT software checks and records; it is never the agent of a behavior record.

The kernel reads none of these records, and `rlt check` prints them beside its verdict without changing them.
