"""F-018: an initial difference DIC - DIC_ALT_CO2 in the box model of
test_cdr_uptake_analytic.py, against compute_cdr_metrics of the tree on PYTHONPATH.

The perturbed run starts with an inventory difference D0 (mmol) and an alkalinity
difference A0 (meq). The extra uptake U relaxes the carbon deficit at rate k:
    dU/dt = k (c (A0 + a t) + r t - D0 - U),  U(0) = 0,
    X(t) = D0 + U(t) - r t            (inventory difference, closed domain)
    U(t) = U0(t) + (c A0 - D0)(1 - exp(-k t)),
where U0 is the uptake of the same release started from equal tracers. Cases:
  1. D0 = 0, A0 = 0     the design case: U = U0.
  2. D0 > 0, A0 = 0     unsupported offset (e.g. source data carrying their own
                        DIC_ALT_CO2): the offset outgasses, U = U0 - D0 (1 - e^-kt).
  3. D0 = c A0          restart of a perturbed run near equilibrium: U = U0, and
                        X - X(0) holds this segment's uptake while X holds all of it.
Output written the ucla-roms way: FG window averages, hDIC instantaneous at window
ends; no output record at t = 0 (the first record ends the first window).
Usage: PYTHONPATH=TREE python f018_initial_difference.py
"""
import numpy as np, xarray as xr, roms_tools
from roms_tools.analysis.cdr_analysis import compute_cdr_metrics

c, k, DT, NT = 0.8, 1 / (2 * 86400.0), 3600.0, 72
def run(a, r, D0, A0):
    te = DT * np.arange(1, NT + 1); tb = te - DT
    U0 = lambda t: (c * a + r) * t - (c * a + r) * (1 - np.exp(-k * t)) / k
    U = lambda t: U0(t) + (c * A0 - D0) * (1 - np.exp(-k * t))
    X = lambda t: D0 + U(t) - r * t
    area = 1.0e6; ny = nx = 3   # one interior cell
    d3, d4 = ("time", "eta_rho", "xi_rho"), ("time", "s_rho", "eta_rho", "xi_rho")
    fg = np.zeros((NT, ny, nx)); fg[:, 1, 1] = (U(te) - U(tb)) / DT / area
    h = np.zeros((NT, 1, ny, nx)); h[:, 0, 1, 1] = X(te) / area
    alk = np.zeros((NT, 1, ny, nx)); alk[:, 0, 1, 1] = a
    dic = np.zeros((NT, 1, ny, nx)); dic[:, 0, 1, 1] = -r
    ds = xr.Dataset({"avg_begin_time": ("time", tb), "avg_end_time": ("time", te),
        "ALK_source": (d4, alk), "DIC_source": (d4, dic),
        "FG_CO2": (d3, fg), "FG_ALT_CO2": (d3, 0 * fg),
        "hDIC": (d4, 2000 + h), "hDIC_ALT_CO2": (d4, 2000 + 0 * h)})
    g = xr.Dataset({"pm": (("eta_rho", "xi_rho"), np.full((ny, nx), 1e-3)), "pn": (("eta_rho", "xi_rho"), np.full((ny, nx), 1e-3))})
    out = compute_cdr_metrics(ds, g)
    nat = lambda v: (out[v] / out[v].attrs["tonnes_co2_per_native_unit"]).values
    return te, U0(te), U(te), nat("cdr_carbon_uptake_from_flux"), nat("cdr_carbon_uptake_from_DIC_difference")

print("tree", roms_tools.__file__.rsplit("/roms_tools/", 1)[0])
print(f"{'case':42} {'U0 (design)':>12} {'U actual':>12} {'flux':>12} {'DIC diff':>12} {'gap DIC-flux':>13} {'gap at 1st record':>18}")
for label, a, r, D0, A0 in [("1 OAE, equal start", 10, 0, 0, 0), ("1 DOR, equal start", 0, 10, 0, 0),
                            ("2 OAE, unsupported offset D0=5e5", 10, 0, 5e5, 0), ("2 DOR, unsupported offset D0=5e5", 0, 10, 5e5, 0),
                            ("3 OAE restart, D0 = c A0 = 5e5", 10, 0, 5e5, 5e5 / c), ("3 DOR restart, D0 = c A0 = 5e5", 0, 10, 5e5, 5e5 / c)]:
    t, u0, u, f, i = run(a, r, D0, A0)
    print(f"{label:42} {u0[-1]:12.0f} {u[-1]:12.0f} {f[-1]:12.0f} {i[-1]:12.0f} {i[-1]-f[-1]:13.0f} {i[0]-f[0]:18.0f}")
