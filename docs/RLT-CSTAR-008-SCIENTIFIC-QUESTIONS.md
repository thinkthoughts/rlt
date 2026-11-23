# RLT-CSTAR-008 · Scientific questions for [C]Worthy

Independent research by thinkthoughts (RLT); not affiliated with or endorsed by [C]Worthy. These questions concern the meaning of reported quantities, so the engineering review cannot settle them. Each states the quantity, what is established numerically, and the decision needed. Evidence: `RLT-CSTAR-008-VERIFICATION.md` §6, `research/rlt-cstar-008/f018_initial_difference.{py,out}`, and the oracle `roms_tools/tests/test_analysis/test_cdr_uptake_analytic.py` on the hold branches.

## Notation

Over the interior of a domain, per output window: U(t) is the cumulative extra air–sea uptake, Σ(FG_CO2 − FG_ALT_CO2)·A·Δt; X(t) = Σ(hDIC − hDIC_ALT_CO2)·A is the inventory difference; R(t) is the cumulative release of DIC, Σ(DIC_source − DIC_ALT_source)·Δt; Δ₀ = X(0). ucla-roms adds a release to DIC and leaves DIC_ALT_CO2 without it, so in a closed domain X(t) = Δ₀ + U(t) + R(t) + C(t) − E(t), with C the CaCO₃-dissolution response and E the exported perturbation.

## Q1 · What does `cdr_carbon_uptake_from_DIC_difference` report? (F-009)

- **Established.** For alkalinity-only releases (R = 0), the two methods agree. For releases that remove DIC (R < 0), the pinned code reports X(t), which equals U + R. For pure DIC removal, the closed-form model gives a DIC-based efficiency of −0.518, against an analytic air–sea efficiency of 0.482. Reporting X − R makes both methods equal the analytic uptake to 10⁻⁹ (proposed commit `9cc9e95d`, held).
- **Inferred.** ROMS-Tools' CDR documentation describes both methods as estimates of the *extra CO₂ taken up from the atmosphere*, and efficiency as moles transferred from the atmosphere per mole of alkalinity.
- **Decision needed.** Is the quantity the extra air–sea uptake (then the proposed change applies) or the net DIC held in the domain (then the pinned behaviour stands, and the documentation should say so for DIC-removing releases)?

## Q2 · Reference state of the inventory method (F-018)

The model was extended with an initial difference Δ₀ and an alkalinity difference A₀: dU/dt = k(c(A₀ + a·t) + r·t − Δ₀ − U). Results, established for OAE and DOR alike:

- The two methods always differ by exactly Δ₀ (plus C − E), constantly in time. The difference is already visible at the first output record (DIC-based minus flux-based, with the F-009 change), so **Δ₀ can be detected from the output alone**.
- **Unsupported offset** (A₀ = 0, e.g. source data supplying their own `DIC_ALT_CO2`): the offset outgasses, and *both* methods depart from the design uptake U₀. For OAE at three days the flux method gives 611,220 mmol against U₀ = 999,655, and the inventory method 1,111,220. The design uptake is then not recoverable from these outputs.
- **Restart of a perturbed run near equilibrium** (Δ₀ = c·A₀): the flux method gives this segment's uptake exactly. The inventory method gives the cumulative uptake since the original start (Δ₀ + U₀).
- **Decisions needed.** (a) Is `DIC_ALT_CO2 = DIC` at the start a requirement of the CDR experiment design? If so, should ROMS-Tools warn when the first-record gap is large? (b) For restarted runs, should the reported uptake be per segment or cumulative, and should the two methods be made to agree on that choice?

## Q3 · CaCO₃ dissolution response

- **Established.** MARBL gives DIC_ALT_CO2 the CaCO₃ dissolution of its own carbonate system. When the perturbed run dissolves more, the inventory method gains carbon that no air–sea flux brought in, and the model shows this as an exact gap d·t between the methods.
- **Decision needed.** Does this carbon count towards the reported removal, and in which metric?

## Q4 · Exported perturbation

- **Established.** Carbon taken up and then carried across the domain's sides stays in the flux-based uptake and leaves the inventory-based one; the gap equals the export exactly.
- **Inferred.** The Isometric module cited by ROMS-Tools does not credit uptake outside the modelled domain.
- **Decision needed.** Which metric is reported for crediting, and how is the gap between the methods interpreted: as a diagnostic, an uncertainty, or a correction?

## Q5 · Interpolation halo of narrow edge coarse tiles (F-021, for ucla-roms)

- **Established.** ucla-roms' `init_coarse` tests the coarse tile width before taking the surplus from the edge tiles. It therefore runs tilings whose edge coarse tiles hold 1 or 2 points: 2,293 of the tile cases for `llm ≤ 300` with ≤ 2 points are not empty. C-Star now refuses only empty edge tiles.
- **Decision needed (ucla-roms).** Is the interpolation halo (`cbf ≥ 3`) filled correctly for an edge tile of 1–2 coarse points, or should `init_coarse` test the edge widths too?

## Q6 · RivR2O ratios (F-012, carried over)

- DON and DOP from DOC_sl use 103/2583 and 1/2583, which RivR2O states for labile DOM. The POC ratios 25/276 and 1/276 remain untraced.
- **Decision needed.** Which protocol do these ratios follow?
