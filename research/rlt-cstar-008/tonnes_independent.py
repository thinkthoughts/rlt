"""Tonnes of CO2 per mmol of carbon from IUPAC 2007 atomic weights in exact rational
arithmetic, vs the constant of the tree on PYTHONPATH and end to end."""
from fractions import Fraction as F
import numpy as np, xarray as xr, roms_tools
from roms_tools.analysis import cdr_analysis as ca
C, O = F("12.0107"), F("15.9994")
M = C + 2 * O
t_per_mmol = M * F(1, 1000) * F(1, 10**6)
print("tree", roms_tools.__file__.rsplit("/roms_tools/", 1)[0])
print("independent: M =", M, "g/mol; t CO2 per mmol C =", t_per_mmol, "=", float(t_per_mmol))
print("code constant _MOLAR_MASS_CO2_KG_PER_MOL =", ca._MOLAR_MASS_CO2_KG_PER_MOL, "; equal to M/1000:", F(ca._MOLAR_MASS_CO2_KG_PER_MOL).limit_denominator(10**9) == M / 1000)
d3, d4 = ("time", "eta_rho", "xi_rho"), ("time", "s_rho", "eta_rho", "xi_rho")
ds = xr.Dataset({"avg_begin_time": ("time", [0.]), "avg_end_time": ("time", [1.]),
    "ALK_source": (d4, np.zeros((1, 1, 3, 3))), "DIC_source": (d4, np.zeros((1, 1, 3, 3))),
    "FG_CO2": (d3, np.ones((1, 3, 3))), "FG_ALT_CO2": (d3, np.zeros((1, 3, 3))),
    "hDIC": (d4, np.ones((1, 1, 3, 3))), "hDIC_ALT_CO2": (d4, np.zeros((1, 1, 3, 3)))})
g = xr.Dataset({"pm": (("eta_rho", "xi_rho"), np.ones((3, 3))), "pn": (("eta_rho", "xi_rho"), np.ones((3, 3)))})
v = float(ca.compute_cdr_metrics(ds, g)["cdr_carbon_uptake_from_flux"][-1])
print("end to end: 1 mmol (one interior cell, 1 m^2, 1 s) ->", v, "t; relative difference", abs(v - float(t_per_mmol)) / float(t_per_mmol))
