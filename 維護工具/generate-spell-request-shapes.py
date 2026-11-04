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
srv = root / "lito伺服器/LiTo381-Server/src/main/java/local/lito381/server"
cs_lines = (srv / "ClientSession.java").read_text(encoding="utf-8").split("\n")
cat = (srv / "BattleSpellCatalog.java").read_text(encoding="utf-8")
illusion = (srv / "IllusionSkillRequests.java").read_text(encoding="utf-8")
dragon = (srv / "DragonSkillRequests.java").read_text(encoding="utf-8")

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

def handlers(rec):
    d = by_type.get(ALIAS.get(rec, rec), {})
    return {k: v for k, v in d.items() if not k[0].startswith("useClassicWorld")}  # BATTLE 路徑

def decide(rec, const, sid, f):
    if sid in wrapper:
        return "attackSpellRequest", wrapper[sid] + "（線上必須 11 bytes）"
    hs = handlers(rec)
    shapes = set().union(*hs.values()) if hs else set()
    where = ", ".join(f"{k[0]} CS:{k[1]}" for k in sorted(hs, key=lambda k: k[1]))
    if len(shapes) == 1:
        return next(iter(shapes)), where
    rs = (f.get("requestShape") or "").split(".")[-1]
    if rec == "ShieldSpell":
        return ("objectSpellRequest" if f.get("targeting", "").endswith("PLAYER_OBJECT") else "selectorOnlySpellRequest"), f"useShieldSpell，targeting={f.get('targeting')}"
    if rec in ("AbilityBuffSpell", "InvisibilitySpell"):
        p = {"OBJECT_TARGET": "objectSpellRequest", "SELF_TICK": "selfSpellRequest", "SELECTOR_ONLY": "selectorOnlySpellRequest"}.get(rs)
        return p, f"{where}，requestShape={rs}"
    if rec == "FreezeAttackSpell":
        centered, line = int(f.get("areaRadius", "0")) > 0, f.get("lineTargeted") == "true"
        return ("selectorOnlySpellRequest" if centered or line else "attackSpellRequest"), f"freeze areaRadius={f.get('areaRadius')} lineTargeted={f.get('lineTargeted')}（CS:33607–33617）"
    if rec == "AreaAttackSpell":
        rng = int(f.get("maximumRange", "-1"))
        selector_only = rng == 0 or const == "MAGMA_BREATH"
        return ("selectorOnlySpellRequest" if selector_only else "attackSpellRequest"), f"area maximumRange={rng}{'，MAGMA_BREATH 固定 SELECTOR_ONLY' if const == 'MAGMA_BREATH' else ''}（CS:33683–33697）"
    if rec == "AttackSpell":
        return "attackSpellRequest", "useAttackSpell（selector-only 變體的 6 個技能都在外層 wrapper 內）"
    return None, where or "no handler"

rows, unresolved = [], []
for rec, const, sid, r, c, f in sorted(instances, key=lambda x: (x[2] is None, x[2] or 0)):
    p, why = decide(rec, const, sid, f)
    if sid != (r << 3) + c + 1 or p is None:
        unresolved.append((sid, const, rec, why))
    rows.append((sid, const, r, c, rec, SHAPE.get(p, "?"), why))
rows.append((5, "TELEPORT", 0, 4, "（dispatcher 特例）", SHAPE["teleportSpellRequest"], "useSpellBeforeDragonTargets → useTeleportSpell CS:27905"))
rows.sort(key=lambda x: (x[0] is None, x[0] or 0))
wrapper_missing = sorted(s for s in wrapper if s not in {x[0] for x in rows})

# WORLD（經典世界）路徑：useClassicWorld* handler 的形狀若與 BATTLE 不同就列出
CONDITIONAL = {"ShieldSpell", "AbilityBuffSpell", "InvisibilitySpell", "FreezeAttackSpell", "AreaAttackSpell", "AttackSpell"}
world_diff = []
world_compared = 0
for rec in sorted({x[0] for x in instances}):
    d = by_type.get(ALIAS.get(rec, rec), {})
    world = {k: v for k, v in d.items() if k[0].startswith("useClassicWorld")}
    if not world:
        continue
    world_compared += 1
    wshapes = set().union(*world.values())
    bshapes = set().union(*handlers(rec).values()) if handlers(rec) else set()
    if rec in CONDITIONAL:
        continue  # WORLD 分支條件與 BATTLE 相同（CS:33056–33069、33205–33219），逐筆形狀一致
    if wshapes != bshapes:
        world_diff.append((rec, sorted(bshapes), sorted(wshapes), ", ".join(f"{k[0]} CS:{k[1]}" for k in world)))

head = subprocess.run(["git", "-C", str(root), "rev-parse", "HEAD"], capture_output=True, text=True).stdout.strip()
if markdown:
    print("| skillId | 常數 | row | col | 記錄類型 | 尾段形狀 | 判定依據 |\n|---:|---|---:|---:|---|---|---|")
    for x in rows:
        print("| %d | `%s` | %d | %d | %s | %s | %s |" % x)
else:
    cnt = collections.Counter(x[5] for x in rows)
    print(f"source HEAD {head}; skills={len(rows)} unresolved={len(unresolved)} wrapper-not-in-catalog={wrapper_missing}")
    print("shapes: " + "; ".join(f"{k}={v}" for k, v in sorted(cnt.items())))
    for u in unresolved:
        print("UNRESOLVED", u)
    print(f"world record types with classic-world handlers={world_compared}; world-shape-differs={len(world_diff)}")
    for w in world_diff:
        print("WORLD-DIFF", w)
if head != EXPECTED_HEAD:
    print(f"FAIL source HEAD {head} != expected {EXPECTED_HEAD}", file=sys.stderr); sys.exit(2)
sys.exit(1 if unresolved else 0)
