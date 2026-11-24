| scenario | generation + configure_build | parent_5d35301 | v1_134183a | v2_1f56f37 |
|---|---|---|---|---|
| 2x1_snapshot | tile refusal | refuse (1 msg, 1 sentences) ✓, text differs | refuse (2 msg, 3 sentences) ✓, text differs | refuse (1 msg, 2 sentences) ✓, same text |
| A_stale_tiles_good_partition | ok | refuse (1 msg, 2 sentences) ✗ | refuse (1 msg, 2 sentences) ✗ | accept (0 msg, 0 sentences) ✓ |
| C_stale_tiles_bad_partition | tile refusal | refuse (1 msg, 2 sentences) ✓, text differs | refuse (2 msg, 4 sentences) ✓, text differs | refuse (1 msg, 2 sentences) ✓, same text |
| E_never_stale_true | ok | refuse (1 msg, 2 sentences) ✗ | refuse (1 msg, 2 sentences) ✗ | accept (0 msg, 0 sentences) ✓ |
| auto_stale_false | other failure | accept (0 msg, 0 sentences) | accept (0 msg, 0 sentences) | accept (0 msg, 0 sentences) |
| auto_stale_true | other failure | refuse (1 msg, 2 sentences) | refuse (1 msg, 2 sentences) | refuse (1 msg, 2 sentences) |
| auto_tiling_stale_cppdef | tile refusal | refuse (1 msg, 2 sentences) ✓, same text | refuse (1 msg, 2 sentences) ✓, same text | refuse (1 msg, 2 sentences) ✓, same text |
| bgc_only_always | tile refusal | accept (0 msg, 0 sentences) ✗ | refuse (1 msg, 2 sentences) ✓, same text | refuse (1 msg, 2 sentences) ✓, same text |
| consistent | tile refusal | refuse (1 msg, 2 sentences) ✓, same text | refuse (1 msg, 2 sentences) ✓, same text | refuse (1 msg, 2 sentences) ✓, same text |
| one_tile_consistent | ok | accept (0 msg, 0 sentences) ✓ | accept (0 msg, 0 sentences) ✓ | accept (0 msg, 0 sentences) ✓ |
| physics_only_always | tile refusal | accept (0 msg, 0 sentences) ✗ | refuse (1 msg, 2 sentences) ✓, same text | refuse (1 msg, 2 sentences) ✓, same text |
| stale_flag | tile refusal | accept (0 msg, 0 sentences) ✗ | refuse (1 msg, 2 sentences) ✓, same text | refuse (1 msg, 2 sentences) ✓, same text |

wrong verdicts or messages: parent_5d35301: 7, v1_134183a: 4, v2_1f56f37: 0
