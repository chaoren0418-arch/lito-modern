#!/usr/bin/env bash
# 用法：bash evidence.sh <lito-modern 根> <lito-launcher-server 根> <暫存目錄>
set -u
M=$1; L=$2; T=$3; mkdir -p "$T"; unset JAVA_TOOL_OPTIONS
JAR=lito伺服器/LiTo381-Server/reference/base-0.6.71.jar
SRC=lito伺服器/LiTo381-Server/src/main/java/local/lito381/server/BattleSpellCatalog.java
echo "== E0 版本"; git -C "$M" rev-parse HEAD; git -C "$L" rev-parse HEAD; git -C "$M" rev-parse HEAD:docs/HISTORY.md
echo "== E1 JAR SHA256"; (cd "$L" && sha256sum "$JAR")
echo "== E2 catalog 常數"
(cd "$T" && unzip -o -q "$L/$JAR" 'local/lito381/server/BattleSpellCatalog*' 'local/lito381/server/ClientSession.class')
git -C "$L" show 2066774:"$SRC" | grep -oE "static final int [A-Z0-9_]+ *= *[0-9]+" | sed -E 's/static final int //; s/ *= */=/' | sort > "$T/src_consts.txt"
javap -p -constants "$T/local/lito381/server/BattleSpellCatalog.class" | grep -oE "static final int [A-Z0-9_]+ *= *[0-9]+" | sed -E 's/static final int //; s/ *= */=/' | sort > "$T/jar_consts.txt"
wc -l < "$T/src_consts.txt"; wc -l < "$T/jar_consts.txt"; diff "$T/src_consts.txt" "$T/jar_consts.txt" && echo "diff: 相同"
echo "src new *Spell( = $(git -C "$L" show 2066774:"$SRC" | grep -oE 'new \w+Spell\(' | wc -l)"
echo "jar new BattleSpellCatalog\$*Spell = $(javap -p -c "$T/local/lito381/server/BattleSpellCatalog.class" | grep -cE ': new .*BattleSpellCatalog\$\w+Spell$')"
sed -n '85p;192,193p' "$M/維護工具/generate-spell-request-shapes.py"
echo "== E3 技能集合"
python3 - "$M" <<'PY'
import re,sys
M=sys.argv[1]
pc=open(M+'/docs/PROTOCOL-CONTRACT.md',encoding='utf-8').read().split('\n')
rows={int(m[1]) for l in pc for m in [re.match(r"\| (\d+) \| `\w+` \| \d+ \| \d+ \|",l)] if m}
src=open(M+'/LiTo381-Modern/src/modern_lito/native_skills.rs',encoding='utf-8').read()
body=src[src.index('pub fn request('):]; body=body[:body.index('_ => anyhow::bail!')]
main={int(x) for arm in re.findall(r"^\s+([0-9 |]+)=>",body,re.M) for x in arm.split('|') if x.strip()}
g=re.search(r"matches!\(skill, ([0-9 |]+)\)",src[src.index('pub fn ground_request('):]); main|={int(x) for x in g[1].split('|')}
cand=set(range(1,99))&rows-main-{97,98}
rest=rows-main-cand-{97,98,99,100}
print('catalog',len(rows)); print('main',len(main),sorted(main))
print('candidate-only',len(cand),sorted(cand)); print('97-100',4); print('rest',len(rest),sorted(rest))
print('sum',len(main)+len(cand)+4+len(rest),'overlap',len(main&cand)+len(main&rest)+len(cand&rest))
PY
grep -rnE "vec!\[0x14|\[0x14," "$M/LiTo381-Modern/src" --include=*.rs | grep -v native_skills.rs; echo "（以上為 native_skills.rs 以外的 0x14 開頭請求，空白表示沒有）"
grep -rln "C_USE_SPELL" "$M/LiTo381-Modern/src" --include=*.rs
echo "== E4 ClientSession（E149）"
javap -p -c "$T/local/lito381/server/ClientSession.class" > "$T/cs.txt"
for sig in 'private void quitFight() throws' 'private void broadcastDefaultRoomReset(long) throws' 'private void exitRoom(boolean) throws' 'private boolean rentalItemSelectionContext();'; do
  n=$(grep -nF "  $sig" "$T/cs.txt" | head -1 | cut -d: -f1); echo "--- $sig（cs.txt:$n）"
  case "$sig" in *quitFight*) k=8;; *broadcastDefault*) k=15;; *exitRoom*) k=6;; *) k=34;; esac
  sed -n "$n,$((n+k))p" "$T/cs.txt"
done
echo "== E5 伺服器文件"
sed -n 15p "$L/PLAN.md"; sed -n 7p "$L/lito伺服器/LiTo381-Server/docs/maintenance/combat-closeout-current.zh-TW.md"
echo "== E6"; sed -n 14p "$L/lito伺服器/LiTo381-Server/docs/maintenance/combat-continuation-current.zh-TW.md" | python3 -c "import sys;[print(l.rstrip(\"\\n\")[:120]) for l in sys.stdin]"
echo "== E7 HISTORY 去重"
python3 - "$M" <<'PY'
import sys
L=open(sys.argv[1]+'/docs/HISTORY.md',encoding='utf-8').read().split('\n')
nz=[x for x in L if x.strip()]; print('lines',len(L),'nonempty',len(nz),'unique',len(set(nz)),'dup',len(nz)-len(set(nz)))
PY
echo "== E8 E149 內含的戰鬥候選層（8d26 鏈）與未含的下一層"
(cd "$L" && unzip -l "$JAR" | grep -E "NativeCancellationBattleRuntime\.class|NativeMagicalWeaponAction\.class|NativeMassTeleportAnswerLife\.class|NativeStaticNpcScene"; echo "（NativeStaticNpcScene 未出現＝cancellation-static-runtime 之後未部署）")
sed -n 50p "$L/lito登入器/LiTo381-Launcher/docs/maintenance/world-resource-compat-current.zh-TW.md" | python3 -c "import sys;[print(l.rstrip(\"\\n\")[:120]) for l in sys.stdin]"
