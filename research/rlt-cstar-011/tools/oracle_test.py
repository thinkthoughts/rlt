"""RLT-CSTAR-011 F-029 oracle (scratch; copied into the forge test dir, never committed).

For each scenario: P = the tile messages blueprint_problems reports before generation;
G = what process_forge_blueprint(validate=False) does: generate, then configure_build
(which checks the tiles on the settings it wrote). The pre-generation check is right
when P is empty exactly when G succeeds (or fails for a non-tile reason), and P names
the values configure_build names."""
import json, os, re, traceback
import pytest
from cstar.applications.forge.blueprint import CoarseGridMode, Partitioning, SurfaceType
from cstar.applications.forge.engine import blueprint_problems, process_forge_blueprint
from cstar.applications.forge.host import HostPaths

OUT = os.environ.get("RLT_ORACLE_OUT", "/tmp/oracle.jsonl")
TILE = re.compile(r"param\.np_(xi|eta)=")

def modes(cfg, m, keep=None):
    items = [i for i in cfg.forcing.surface if keep is None or i.type in keep]
    cfg.forcing.surface = [i.model_copy(update={"coarse_grid_mode": m}) for i in items]

def flags(cfg, v):
    for s in ("blk_frc", "bgc"):
        if s in cfg.model_settings: cfg.model_settings[s]["interp_frc"] = v

def S_consistent(cfg): modes(cfg, CoarseGridMode.ALWAYS); flags(cfg, True)
def S_stale_flag(cfg): modes(cfg, CoarseGridMode.ALWAYS); flags(cfg, False)
def S_A_stale_tiles_good_partition(cfg):
    modes(cfg, CoarseGridMode.ALWAYS); flags(cfg, True); cfg.domain.partitioning = Partitioning(n_procs_x=1, n_procs_y=1)
def S_C_stale_tiles_bad_partition(cfg):
    modes(cfg, CoarseGridMode.ALWAYS); flags(cfg, True); cfg.model_settings["param"]["np_xi"] = 4
def S_2x1_snapshot(cfg):
    modes(cfg, CoarseGridMode.ALWAYS); flags(cfg, True); cfg.model_settings["param"]["np_eta"] = 1
def S_E_never_stale_true(cfg): modes(cfg, CoarseGridMode.NEVER); flags(cfg, True)
def S_auto_stale_true(cfg): modes(cfg, CoarseGridMode.AUTO); flags(cfg, True)
def S_auto_stale_false(cfg): modes(cfg, CoarseGridMode.AUTO); flags(cfg, False)
def S_bgc_only_always(cfg): modes(cfg, CoarseGridMode.ALWAYS, keep={SurfaceType.BGC}); flags(cfg, False)
def S_physics_only_always(cfg): modes(cfg, CoarseGridMode.ALWAYS, keep={SurfaceType.PHYSICS}); flags(cfg, False)
def S_auto_tiling_stale_cppdef(cfg):
    modes(cfg, CoarseGridMode.ALWAYS); flags(cfg, True)
    cfg.domain.partitioning = Partitioning(auto_tiling=True, n_cores=4)
def S_one_tile_consistent(cfg):
    modes(cfg, CoarseGridMode.ALWAYS); flags(cfg, True); cfg.domain.partitioning = Partitioning(n_procs_x=1, n_procs_y=1)
    cfg.model_settings["param"].update(np_xi=1, np_eta=1)

SCEN = [v for k, v in sorted(globals().items()) if k.startswith("S_")]

@pytest.mark.parametrize("scen", SCEN, ids=[s.__name__[2:] for s in SCEN])
def test_oracle(tmp_path, forge_blueprint_factory, scen):
    cfg, _ = forge_blueprint_factory("unified", tmp_path, name="oracle")
    rec = {"scenario": scen.__name__[2:], "surface": [(i.type.value, i.coarse_grid_mode.value) for i in cfg.forcing.surface]}
    try:
        scen(cfg)
    except Exception as e:
        rec["setup_error"] = f"{type(e).__name__}: {e}"[:300]
        open(OUT, "a").write(json.dumps(rec) + "\n"); return
    rec["surface"] = [(i.type.value, i.coarse_grid_mode.value) for i in cfg.forcing.surface]
    try:
        probs = blueprint_problems(cfg)
    except Exception as e:
        probs = [f"blueprint_problems raised {type(e).__name__}: {e}"]
    rec["pre_tile"] = [p for p in probs if TILE.search(p)]
    rec["pre_other"] = [p[:160] for p in probs if not TILE.search(p)]
    host = HostPaths(working_dir=tmp_path, source_data_cache=tmp_path / "src", system="test")
    try:
        process_forge_blueprint(cfg, host=host, use_dask=False, validate=False)
        rec["generated"] = "ok"
    except Exception as e:
        msg = f"{type(e).__name__}: {e}"
        rec["generated"] = "tile refusal" if TILE.search(msg) else "other failure"
        rec["generated_msg"] = msg[:600]
    rec["nc_files"] = len(list(tmp_path.rglob("*.nc")))
    open(OUT, "a").write(json.dumps(rec) + "\n")
