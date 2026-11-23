#!/bin/bash
# Negative controls for check_rlt_cstar.py. Usage: manifest_controls.sh RLT_REPO RLT_BIN
R=$1; RLT=$2; base=$(mktemp -d)
git -C $R archive HEAD | tar -x -C $base          # complete committed tree (kernel registries included)
cp $R/research/rlt-cstar-009/check_rlt_cstar.py $base/research/rlt-cstar-009/   # checker under test
M2=research/rlt-cstar-002/certificates/MANIFEST.tsv; M7=research/rlt-cstar-007/certificates/MANIFEST.tsv
run() { # name, expected exit, expected message regex, mutation commands...
  name=$1; want=$2; rx=$3; shift 3; d=$(mktemp -d); cp -r $base/. $d/; (cd $d && eval "$@")
  out=$(cd $d && python3 research/rlt-cstar-009/check_rlt_cstar.py $RLT --expect ACCEPT=17 MALFORMED=1 2>&1); rc=$?
  hit=$(echo "$out" | grep -cE "$rx"); ok=$([ $rc = $want ] && { [ -z "$rx" ] || [ $hit -gt 0 ]; } && echo PASS || echo "CONTROL BROKEN")
  printf "%-16s %-44s exit %s (want %s)  %s\n" "$ok" "$name" $rc $want "$(echo "$out" | grep -E "^FAIL [a-z]|^FAIL .*: " | grep -vE "^FAIL research.*  (ACCEPT|MALFORMED)$" | head -1 | cut -c1-90)"
  rm -rf $d; }
run "unchanged"                        0 "" "true"
run "002 manifest without header"      0 "" "sed -i 1d $M2"
run "007 manifest with header"         0 "" "sed -i '1i file\tsha256\texit\tverdict' $M7"
run "omitted entry (G03 removed)"      1 "omitted: .*G03" "sed -i '/^G03_/d' $M2"
run "omitted entry (G08 removed)"      1 "omitted: .*G08" "sed -i '/^G08_/d' $M7"
run "duplicated entry (G01 twice)"     1 "duplicated: G01" "l=\$(grep '^G01_' $M2); echo \"\$l\" >> $M2"
run "entry in both manifests (G08)"    1 "duplicated: G08" "grep '^G08_' $M7 >> $M2"
run "modified certificate (G04 +byte)" 1 "G04.*modified" "echo ' ' >> research/rlt-cstar-002/certificates/G04_coarse_10_8.rlt"
run "modified goal, sha updated"       1 "incorrectly classified|tampered" "f=research/rlt-cstar-002/certificates/G02_tiles_10_3.rlt; python3 -c \"import re,sys,hashlib;p=sys.argv[1];s=open(p).read();m=re.search(r'\\(goal \\\"([^\\\"]*)\\\"\\)',s);g=m.group(1);i=max(k for k,c in enumerate(g) if c.isdigit());s=s[:m.start(1)]+g[:i]+str((int(g[i])+1)%10)+g[i+1:]+s[m.end(1):];open(p,'w').write(s);h=hashlib.sha256(open(p,'rb').read()).hexdigest();q=sys.argv[2];t=open(q).read().splitlines();t=[(l.split('\t')[0]+'\t'+h+'\t'+'\t'.join(l.split('\t')[2:])) if l.startswith('G02_') else l for l in t];open(q,'w').write('\n'.join(t)+'\n')\" \$f $M2"
run "misclassified (MALFORMED as ACCEPT)" 1 "incorrectly classified" "sed -i 's/^\(C_dvd_16_512.rlt\t[^\t]*\t[^\t]*\t\)MALFORMED/\1ACCEPT/' $M2"
run "misclassified (ACCEPT as REJECT)" 1 "incorrectly classified" "sed -i 's/^\(G05_coarse_10_4.rlt\t[^\t]*\t[^\t]*\t\)ACCEPT/\1REJECT/' $M2"
run "listed file missing (G06 deleted)" 1 "missing file: .*G06" "rm research/rlt-cstar-002/certificates/G06_join_ambiguity.rlt"
run "extra unlisted certificate"       1 "omitted: .*EXTRA" "cp research/rlt-cstar-002/certificates/G01_ceil_10_3.rlt research/rlt-cstar-002/certificates/EXTRA.rlt"
run "G08 entry and file removed"       1 "expected 17 ACCEPT" "sed -i '/^G08_/d' $M7; rm research/rlt-cstar-007/certificates/G08_coarse_edge_150_21.rlt"
run "malformed manifest line"          1 "malformed entry" "echo 'G09_x.rlt	nothex	0	ACCEPT' >> $M7"
rm -rf $base
