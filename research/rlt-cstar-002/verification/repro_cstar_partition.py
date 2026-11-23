"""RLT-CSTAR-002 Phase 1: the actual C-Star code path on a real ROMS-Tools grid.

1. C-Star Forge's early refusal (`check_tile_sizes`) on llm=mmm=10, np_xi=3, np_eta=1.
2. C-Star `ROMSModelGrid.get()` then `.partition(np_xi=3, np_eta=1)`, which calls
   `roms_tools.partition_netcdf` (the PARTITIONED layout's pre-run step).
"""
import logging, sys, shutil
from pathlib import Path
from cstar.applications.forge.namelist_model import check_tile_sizes, _edge_tile_width
from cstar.roms.input_dataset import ROMSModelGrid

np_xi, np_eta = int(sys.argv[1]), int(sys.argv[2])
grid = Path(sys.argv[3]).resolve()
param = {"llm": 10, "mmm": 10, "np_xi": np_xi, "np_eta": np_eta}
try:
    check_tile_sizes(param, auto_tiling=False)
    print(f"[Forge] check_tile_sizes{param}: accepted (edge widths xi={_edge_tile_width(10, np_xi)}, eta={_edge_tile_width(10, np_eta)})")
except ValueError as e:
    print(f"[Forge] check_tile_sizes{param}: refused: {e}")

work = Path("stage"); shutil.rmtree(work, ignore_errors=True); work.mkdir()
ds = ROMSModelGrid(location=str(grid))
ds.get(local_dir=work)
try:
    ds.partition(np_xi=np_xi, np_eta=np_eta)
    print(f"[pre_run] ROMSModelGrid.partition({np_xi},{np_eta}): wrote {len(ds.partitioning.files)} files")
except Exception as e:
    print(f"[pre_run] ROMSModelGrid.partition({np_xi},{np_eta}): {type(e).__name__}: {e}")
