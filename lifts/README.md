# PiπΠ lifts (component G)

A lift records a proposed link from one reading point (the leading specification, PI-5) to another, carried by explicit relations (PI-2). In RLT-001 lifts live inside the reading-point files (`readingpoints/*.yaml`, key `lifts`):

```yaml
lifts:
  - id: math0-to-math5
    to: math5
    via: "(1+1) × 3 = 6"
    status: proposed
```

Status values come from `correspondence/status.yaml` → `lift_status` (PI-4). A lift becomes `verified` only when every relation in `via` is a kernel theorem; multiplication enters the kernel in RLT-002, so `rlt validate` refuses `verified` in RLT-001.
