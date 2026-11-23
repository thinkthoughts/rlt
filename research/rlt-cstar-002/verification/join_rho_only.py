"""Rho-only round trips: how many fail, split into even and uneven tilings."""
import numpy as np, xarray as xr
from roms_tools.tiling.join import join_datasets
from roms_tools.tiling.partition import partition
res = {"even": [0, 0], "uneven": [0, 0]}
for nx in range(1, 25):
    for ny in range(1, 25):
        ds = xr.Dataset({"h": (("eta_rho", "xi_rho"), np.random.default_rng(nx*99+ny).random((ny + 2, nx + 2)))})
        for npx in range(1, 7):
            for npe in range(1, 7):
                try:
                    _, parts = partition(ds, np_eta=npe, np_xi=npx)
                except ValueError:
                    continue
                k = "even" if nx % npx == 0 and ny % npe == 0 else "uneven"
                try:
                    ok = join_datasets(parts).identical(ds)
                except Exception:
                    ok = False
                res[k][0] += 1; res[k][1] += not ok
print({k: f"{v[1]} of {v[0]} fail" for k, v in res.items()})
