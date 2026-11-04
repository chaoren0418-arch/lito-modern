#!/usr/bin/env python3
"""產生 C_USE_SPELL 技能 (row,column) → 尾段形狀對照表（只讀 lito-launcher-server 來源）。

usage: python3 -I 維護工具/generate-spell-request-shapes.py <lito-launcher-server root> [--markdown]

判定順序與伺服器 dispatch 相同：
  1. IllusionSkillRequests.ATTACK/OTHER、DragonSkillRequests.TARGETED → 線上一律 ⑩（11 bytes），
     其他長度被拒絕；轉交內層時截成 3 bytes。
  2. TELEPORT(5) → ⑤ teleportSpellRequest。
  3. 其餘依 BattleSpellCatalog 記錄類型 → BATTLE 路徑 handler 呼叫的 parser；
     有多種形狀的類型，依每筆實例的欄位判定（ShieldSpell.targeting、requestShape、
     FreezeAttackSpell.areaRadius/lineTargeted、AreaAttackSpell.maximumRange 等）。
任何無法判定的技能都會列為 UNRESOLVED，且以非零結束碼結束。
"""
import collections, pathlib, re, subprocess, sys

EXPECTED_HEAD = "2066774824b314b2c65834f9afe72b68f7391f06"
root = pathlib.Path(sys.argv[1])
markdown = "--markdown" in sys.argv
SRV = "lito伺服器/LiTo381-Server/src/main/java/local/lito381/server"
def source(name):
    """優先從固定 commit 讀（工作目錄已移到別的版本也能重現），否則讀工作目錄。"""
    r = subprocess.run(["git", "-C", str(root), "show", f"{EXPECTED_HEAD}:{SRV}/{name}"], capture_output=True)
    if r.returncode == 0:
        return r.stdout.decode("utf-8"), f"git:{EXPECTED_HEAD[:7]}"
    return (root / SRV / name).read_text(encoding="utf-8"), "worktree"
FILES = ["ClientSession.java", "BattleSpellCatalog.java", "IllusionSkillRequests.java", "DragonSkillRequests.java"]
loaded = {name: source(name) for name in FILES}
MODES = {name: mode for name, (_, mode) in loaded.items()}
cs_lines = loaded["ClientSession.java"][0].split("\n")
cat = loaded["BattleSpellCatalog.java"][0]
illusion = loaded["IllusionSkillRequests.java"][0]
dragon = loaded["DragonSkillRequests.java"][0]
SOURCE_MODE = ",".join(sorted(set(MODES.values())))

SHAPE = {
    "selectorOnlySpellRequest": "① 無尾段", "selfSpellRequest": "② d(clientTick)",
    "groundSpellRequest": "③ h(x) h(y)", "objectSpellRequest": "④ d(target)",
    "teleportSpellRequest": "⑤ h(map) h(x) h(y)", "massTeleportSpellRequest": "⑥ h(map) h(x) h(y)",
    "callClanSpellRequest": "⑦ s(name)", "runClanSpellRequest": "⑧ s(name)",
    "summonSpellRequest": "⑨ d(selection)", "attackSpellRequest": "⑩ d(target) h(x) h(y)",
    "trueTargetRequest": "⑪ d(target) h(x) h(y) s(text)",
}

def ints(src, name):
    m = re.search(r"Set<Integer> %s\s*=\s*Set\.of\(([^)]*)\)" % name, src)
    return {int(x) for x in m.group(1).split(",")}

wrapper = {s: "IllusionSkillRequests.ATTACK" for s in ints(illusion, "ATTACK")}
wrapper.update({s: "IllusionSkillRequests.OTHER" for s in ints(illusion, "OTHER")})
wrapper.update({s: "DragonSkillRequests.TARGETED" for s in ints(dragon, "TARGETED")})

# handler (method) → parsers, keyed by BattleSpellCatalog record type in its signature
meth = re.compile(r"^    (?=\S)(?:private |static |public |protected |synchronized |final )*[\w<>\[\], .]+ (\w+)\((.*)$")
by_type = collections.defaultdict(lambda: collections.defaultdict(set))
cur, sig = None, ""
for i, line in enumerate(cs_lines, 1):
    m = meth.match(line)
    if m:
        cur, sig, j = (m.group(1), i), line, i
        while ")" not in sig and j < len(cs_lines):
            sig += cs_lines[j]; j += 1
    for p in SHAPE:
        if "Protocol.%s(" % p in line and cur:
            for t in set(re.findall(r"BattleSpellCatalog\.(\w+)", sig)) or {"-"}:
                by_type[t][cur].add(p)

consts = {m.group(1): int(m.group(2)) for m in re.finditer(r"static final int (\w+)\s*=\s*(\d+)\s*;", cat)}

def record_params(name):
    m = re.search(r"record %s\((.*?)\)\s*(?:implements[^{]*)?\{" % name, cat, re.S)
    return [p.strip().split()[-1] for p in re.sub(r"\s+", " ", m.group(1)).split(",")] if m else []

def split_args(s):
    out, depth, cur_arg = [], 0, ""
    for ch in s:
        depth += ch in "([{"; depth -= ch in ")]}"
        if ch == "," and depth == 0:
            out.append(cur_arg.strip()); cur_arg = ""
        else:
            cur_arg += ch
    return out + [cur_arg.strip()]

instances = []  # (record type, const, skillId, row, column, fields)
for m in re.finditer(r"new (\w+Spell)\(", cat):
    rec, i, depth, j = m.group(1), m.end(), 1, m.end()
    while depth:
        depth += cat[j] == "("; depth -= cat[j] == ")"; j += 1
    args = [re.sub(r"\s+", " ", a) for a in split_args(re.sub(r"//[^\n]*", "", cat[i:j - 1]))]
    # 包裝記錄（例如 new ConfusionSpell(new AttackSpell(...), ...)）的內層實例已單獨計入
    if len(args) < 3 or args[0].startswith("new ") or not args[1].isdigit() or not args[2].isdigit():
        continue
    sid = consts.get(args[0], int(args[0]) if args[0].isdigit() else None)
    instances.append((rec, args[0], sid, int(args[1]), int(args[2]), dict(zip(record_params(rec), args))))

ALIAS = {"PolluteWaterSpell": "TargetNegativeStatusSpell", "StrikerGaleSpell": "TargetNegativeStatusSpell"}

hmap = collections.defaultdict(list)  # handler name -> [(line, parsers)]
for d in by_type.values():
    for (name, line), ps in d.items():
        if (line, ps) not in hmap[name]:
            hmap[name].append((line, ps))

def H(name):
    """handler 名稱 → (名稱, 行號, parser)。同名多載只取直接呼叫 parser 的那個；
    一個方法內有多種 parser 時 parser 為 None，由呼叫端依實例欄位決定。"""
    cands = hmap.get(name, [])
    if len(cands) != 1:
        raise SystemExit(f"handler {name} 定位失敗：{cands}")
    line, ps = cands[0]
    return name, line, next(iter(ps)) if len(ps) == 1 else None

def typed(rec, world):
    d = by_type.get(ALIAS.get(rec, rec), {})
    return {k: v for k, v in d.items() if k[0].startswith("useClassicWorld") == world}

def line_of(src, needle):
    for i, l in enumerate(src.split("\n"), 1):
        if needle in l:
            return i
    return None

def inner(rec, const, sid, f, world):
    """回傳 (parser, handler, line, 條件說明)。world=True 表示 WORLD 路徑。"""
    rs = (f.get("requestShape") or "").split(".")[-1]
    if rec == "ShieldSpell":
        h = H("useShieldSpell"); p = "objectSpellRequest" if f.get("targeting", "").endswith("PLAYER_OBJECT") else "selectorOnlySpellRequest"
        return p, h[0], h[1], f"targeting={f.get('targeting', '').split('.')[-1]}"
    if rec in ("AbilityBuffSpell", "InvisibilitySpell"):
        name = "useAbilityBuffSpell" if rec == "AbilityBuffSpell" else "useInvisibilitySpell"
        line = hmap[name][0][0]
        p = {"OBJECT_TARGET": "objectSpellRequest", "SELF_TICK": "selfSpellRequest", "SELECTOR_ONLY": "selectorOnlySpellRequest"}.get(rs)
        return p, name, line, f"requestShape={rs}"
    if rec == "FreezeAttackSpell":
        centered, line_t = int(f.get("areaRadius", "0")) > 0, f.get("lineTargeted") == "true"
        name = (("useClassicWorldCenteredFreezeAttackSpell" if world else "useCenteredFreezeAttackSpell") if centered
                else "useSelectorOnlyTargetedFreezeAttackSpell" if line_t
                else ("useClassicWorldFreezeAttackSpell" if world else "useFreezeAttackSpell"))
        h = H(name)
        return h[2], name, h[1], f"areaRadius={f.get('areaRadius')} lineTargeted={f.get('lineTargeted')}（分支 CS:{'33056–33069' if world else '33607–33617'}）"
    if rec == "AreaAttackSpell":
        rng = int(f.get("maximumRange", "-1")); sel = rng == 0 or const == "MAGMA_BREATH"
        if world:
            name = "useClassicWorldCenteredAreaAttackSpell" if rng == 0 else "useSelectorOnlyTargetedAreaAttackSpell" if sel else "useClassicWorldAreaAttackSpell"
        else:
            name = "useSelectorOnlyTargetedAreaAttackSpell" if sel and rng > 0 else "useCenteredAreaAttackSpell" if rng == 0 else "useAreaAttackSpell"
        h = H(name)
        return h[2], name, h[1], f"maximumRange={rng}{'，MAGMA_BREATH 固定 SELECTOR_ONLY' if const == 'MAGMA_BREATH' else ''}（分支 CS:{'33205–33219' if world else '33683–33697'}）"
    if rec == "AttackSpell":
        special = sid in (192, 202, 203, 207, 208, 213)  # thunderGrab/confusion/smash/mindBreak/boneBreak/armBreaker
        name = "useSelectorOnlyTargetedAttackSpell" if special else ("useClassicWorldAttackSpell" if world else "useAttackSpell")
        cands = hmap.get(name, [])
        line, ps = cands[0]
        return next(iter(ps)), name, line, f"{'特殊 6 技能' if special else '一般'}（分支 CS:{'33184–33203' if world else '33698–33717'}）"
    hs = typed(rec, world) or typed(rec, False)  # WORLD 沒有專用 handler 時沿用 BATTLE handler
    shapes = set().union(*hs.values()) if hs else set()
    if len(shapes) != 1:
        return None, ",".join(k[0] for k in hs), None, "多形狀未處理"
    k = sorted(hs, key=lambda k: k[1])[0]
    return next(iter(shapes)), k[0], k[1], "、".join(f"{x[0]} CS:{x[1]}" for x in sorted(hs, key=lambda k: k[1])[1:])

WRAP = {}
for cls, src, setname in (("IllusionSkillRequests", illusion, None), ("DragonSkillRequests", dragon, None)):
    WRAP[cls] = (line_of(src, "static boolean dispatch" if cls.startswith("Illusion") else "static void dispatch"),
                 line_of(src, "payload.length!=11" if cls.startswith("Illusion") else "payload.length != 11"),
                 line_of(src, "Protocol.attackSpellRequest(payload)"),
                 line_of(src, "Arrays.copyOf(payload,3)" if cls.startswith("Illusion") else "Arrays.copyOf(payload,3)"))

rows, unresolved, mode = [], [], collections.Counter()
for rec, const, sid, r, c, f in sorted(instances, key=lambda x: (x[2] is None, x[2] or 0)):
    b = inner(rec, const, sid, f, False); w = inner(rec, const, sid, f, True)
    if sid in wrapper:
        cls = wrapper[sid].split(".")[0]; d0, lchk, lp, lfw = WRAP[cls]
        parser = "attackSpellRequest"
        why = (f"{wrapper[sid]}：{cls}.dispatch {cls}.java:{d0}（長度 11 檢查 :{lchk}、parser :{lp}、截 3 bytes 轉交 :{lfw}）"
               f" → 內層 {b[1]} CS:{b[2]}")
        mode["wrapper（兩種模式都先經過外層）"] += 1
    else:
        parser = b[0]
        why = f"{b[1]} CS:{b[2]}" + (f"；{b[3]}" if b[3] else "")
        if b[0] != w[0]:
            unresolved.append((sid, const, rec, f"WORLD 形狀不同：BATTLE {b} / WORLD {w}"))
        if w[1] != b[1]:
            why += f"；WORLD：{w[1]} CS:{w[2]}（同形狀）"
            mode["BATTLE／WORLD 不同 handler、逐筆比對形狀相同"] += 1
        else:
            mode["兩種模式共用同一 handler"] += 1
    if sid != (r << 3) + c + 1 or parser is None or b[2] is None:
        unresolved.append((sid, const, rec, why))
    rows.append((sid, const, r, c, rec, SHAPE.get(parser, "?"), why))
tl = hmap["useTeleportSpell"][0][0]
rows.append((5, "TELEPORT", 0, 4, "（dispatcher 特例）", SHAPE["teleportSpellRequest"],
             f"useTeleportSpell CS:{tl}；dispatcher 在 phase 檢查前轉交（useSpellBeforeDragonTargets CS:32165 起）"))
mode["兩種模式共用同一 handler"] += 1
rows.sort(key=lambda x: (x[0] is None, x[0] or 0))
wrapper_missing = sorted(s for s in wrapper if s not in {x[0] for x in rows})

head = subprocess.run(["git", "-C", str(root), "rev-parse", "HEAD"], capture_output=True, text=True).stdout.strip()
# 來源驗證：四個檔案都從固定 commit 物件讀到時，以該 commit 為準（checkout 的 HEAD 不影響結果）；
# 任何一個檔案退回讀工作目錄時，才要求工作目錄 HEAD 必須等於固定 commit。
all_pinned = all(m.startswith("git:") for m in MODES.values())
source_commit = EXPECTED_HEAD if all_pinned else head
if markdown:
    print("| skillId | 常數 | row | col | 記錄類型 | 尾段形狀 | 判定依據 |\n|---:|---|---:|---:|---|---|---|")
    for x in rows:
        print("| %d | `%s` | %d | %d | %s | %s | %s |" % x)
else:
    cnt = collections.Counter(x[5] for x in rows)
    print(f"source commit {source_commit} ({'pinned git objects' if all_pinned else 'worktree'}; checkout HEAD {head}); skills={len(rows)} unresolved={len(unresolved)} wrapper-not-in-catalog={wrapper_missing}")
    print("shapes: " + "; ".join(f"{k}={v}" for k, v in sorted(cnt.items())))
    for u in unresolved:
        print("UNRESOLVED", u)
    print(f"source={SOURCE_MODE}; mode check per skill: " + "; ".join(f"{k}={v}" for k, v in sorted(mode.items())))
if not all_pinned:
    print("WARN worktree fallback: " + ", ".join(f"{n}={m}" for n, m in sorted(MODES.items())), file=sys.stderr)
    if head != EXPECTED_HEAD:
        print(f"FAIL worktree HEAD {head} != expected {EXPECTED_HEAD}", file=sys.stderr); sys.exit(2)
sys.exit(1 if unresolved else 0)
