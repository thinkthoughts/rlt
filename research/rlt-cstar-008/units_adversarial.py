"""Adversarial unit strings (none in the ROMS-Tools test suite before RLT-CSTAR-008)
through _parse_units of the tree on PYTHONPATH."""
from roms_tools.analysis.cdr_analysis import _parse_units, _FLUX_DIMS, _INVENTORY_DIMS
cases = ["mmol m-2 d-1", "mmol/m2/sec", "mol m^-2 s^-1", "kmol/m^2/s", "mmol/m^2/s ", " mmol/m^2/s",
         "Mmol/m^2/s", "MMOL/M^2/S", "mmol/M^2/s", "mmol/m^2/S", "mmol/m^2/s/s*s", "mmol/m^2*s^-1",
         "mmol/m/m/s", "mmol m-2 s-1 m0", "mmol/m^2/s^1", "mmol/m^2/s^2", "mmol/m^-2/s", "mol/m^2/s mmol/mol",
         "mmol.m-2.s-1", "mmol/m²/s", "mmol/m**2/s", "1e-3 mol/m^2/s", "mEq/s", "meq/s", "meters mMol m-3"]
for u in cases:
    try:
        f, d = _parse_units(u)
        ok = "flux" if d == _FLUX_DIMS else ("inventory" if d == _INVENTORY_DIMS else f"dims {d}")
        print(f"{u!r:28} -> {f:g} {ok}")
    except Exception as e:
        print(f"{u!r:28} -> refused: {str(e)[:90]}")
