import json, random, sys
from cstar.applications.forge.namelist_model import settings_support_problems, run_time_settings_for_ref, validate_run_time_sections
rng = random.Random(7)
vals = [None, True, False, 0, 1, "1", "yes", "garbage", [], {}]
out = []
for i in range(4000):
    s = {}
    if rng.random() < .9:
        s["param"] = {k: rng.choice([None, 0, 1, 2, 3, 4, 7, 8, 16, 33, "8", "x"]) for k in ("llm","mmm","np_xi","np_eta") if rng.random() < .9}
    for sec in ("blk_frc", "bgc"):
        r = rng.random()
        if r < .7: s[sec] = {"interp_frc": rng.choice(vals)}
        elif r < .8: s[sec] = None
    if rng.random() < .7: s["cppdefs"] = {"auto_tiling": rng.choice(vals)}
    if rng.random() < .1: s["pipe_frc"] = {"pipe_source": rng.choice(vals)}
    if rng.random() < .1: s["bogus"] = {}
    try: r1 = settings_support_problems(s, run_time_settings_for_ref(None))
    except Exception as e: r1 = f"EXC {type(e).__name__}: {e}"
    try: r2 = validate_run_time_sections(s)
    except Exception as e: r2 = f"EXC {type(e).__name__}: {e}"
    out.append([r1, r2])
json.dump(out, open(sys.argv[1], "w"))
