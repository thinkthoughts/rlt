"""For every history file of the head (/tmp/pt_new) and base (/tmp/pt_base) runs, compare the
Cs_r/Cs_w attributes ROMS wrote with a 50-digit evaluation of ucla-roms' stretching
(Vtransform 2 / Shchepetkin 2010, the formula CSF implements):
  sc = (k - N)/N (w) or (k - N - 0.5)/N (r);  C = (1 - cosh(ts*sc))/(cosh(ts) - 1) for ts > 0 else -sc^2
  Cs = (exp(tb*C) - 1)/(1 - exp(-tb)) for tb > 0 else C.
Prints, per distinct (theta_s, theta_b, N), max |Cs - exact| for base and head."""
import glob, xarray as xr, numpy as np, mpmath as mp
mp.mp.dps = 50
def exact(ts, tb, N, kind):
    out = []
    ks = range(0, N + 1) if kind == "w" else range(1, N + 1)
    for k in ks:
        sc = mp.mpf(k - N) / N if kind == "w" else (mp.mpf(k - N) - mp.mpf("0.5")) / N
        ts_, tb_ = mp.mpf(ts), mp.mpf(tb)
        C = (1 - mp.cosh(ts_ * sc)) / (mp.cosh(ts_) - 1) if ts_ > 0 else -sc**2
        Cs = (mp.exp(tb_ * C) - 1) / (1 - mp.exp(-tb_)) if tb_ > 0 else C
        out.append(Cs)
    return out
seen = {}
for root in ("/tmp/pt_base", "/tmp/pt_new"):
    for f in glob.glob(f"{root}/**/*his*.nc", recursive=True):
        try: a = xr.open_dataset(f, decode_times=False).attrs
        except Exception: continue
        if "Cs_r" not in a: continue
        ts, tb = float(np.atleast_1d(a["theta_s"])[0]), float(np.atleast_1d(a["theta_b"])[0])
        cr, cw = np.atleast_1d(a["Cs_r"]).astype(float), np.atleast_1d(a["Cs_w"]).astype(float)
        N = cr.size; key = (ts, tb, N)
        er, ew = exact(ts, tb, N, "r"), exact(ts, tb, N, "w")
        err = max(max(abs(mp.mpf(x) - e) for x, e in zip(cr, er)), max(abs(mp.mpf(x) - e) for x, e in zip(cw, ew)))
        d = seen.setdefault(key, {})
        side = "base" if root.endswith("base") else "head"
        d[side] = max(d.get(side, 0), float(err)); d.setdefault("files_" + side, 0); d["files_" + side] += 1
print("| theta_s | theta_b | N | files base/head | max err base | max err head |")
for (ts, tb, N), d in sorted(seen.items()):
    print(f"| {ts:g} | {tb:g} | {N} | {d.get('files_base',0)}/{d.get('files_head',0)} | {d.get('base',float('nan')):.2e} | {d.get('head',float('nan')):.2e} |")
