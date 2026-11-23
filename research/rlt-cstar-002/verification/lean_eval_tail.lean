open RLTCSTAR002 in
def table : List String := Id.run do
  let mut out : List String := []
  for p in List.range 300 do
    for t in List.range 64 do
      let p := p + 1
      let t := t + 1
      for k in List.range t do
        let w := if t = 1 then p else if k = 0 then firstW p t else if k = t - 1 then lastW p t else ceilDiv p t
        let st := if t = 1 then 0 else start p t k
        out := s!"{p} {t} {k} {w} {st}" :: out
  return out.reverse

def main : IO Unit := do
  for l in table do IO.println l
