"""Extract formal75 stat/handoff getters from already sealed bytecode; no JVM."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,re,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
base=V/'native-source-handoff-expiry/contract-20261019-174028-047309';proof=read(base/'readback.json');raw=base/'ClientSession.stdout.txt'
assert sha(raw)==proof['raw_files']['ClientSession.stdout.txt']=='964DC949F7EA6C575A8E725D025EA67B94B73ABF81C200BBE2BA190D72248CF1'
assert proof['formal_jar_sha256']=='75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
lines=raw.read_text('utf-8-sig').splitlines();starts=[i for i,l in enumerate(lines)if re.match(r'^  \S.*\(.*\).*;$',l)]
names=['handleRentalTownTeleportAction','sendRentalTownTransferDirectory','selectedCharacterSummary',
       'equipmentAdjustedBaseMaxHp','equipmentAdjustedBaseMaxMp','sourceBaseMaximumHp','sourceBaseMaximumMp','effectiveCharisma','sendAbilityScoresStatus']
selected={};relations={}
for name in names:
    found=[n for n,i in enumerate(starts)if re.search(r'\s'+re.escape(name)+r'\(',lines[i])];assert len(found)==1,name
    n=found[0];start=starts[n];end=starts[n+1]if n+1<len(starts)else len(lines)
    text='\n'.join(lines[start:end])+'\n';selected[name]=text
    relations[name]=[l.strip()for l in text.splitlines()if'// Method'in l and re.search(r'Character|Stat|strength|dexterity|constitution|wisdom|charisma|intelligence|maxHp|maxMp|handoff|Handoff|Teleport|equipment|Equipment|Ability|rental',l)]
out=V/'native-nondefault-stat-contract'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
(out/'selected-methods.json').write_text(json.dumps(selected,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');(out/'inspector.py').write_bytes(Path(__file__).read_bytes())
record=dict(status='formal75_stat_and_handoff_static_contract_only',formal_jar_sha256=proof['formal_jar_sha256'],
    original_disassembly=str(raw),original_disassembly_sha256=sha(raw),selected_methods=names,method_relations=relations,
    selected_methods_sha256=sha(out/'selected-methods.json'),JVM_started=False,network_sessions=0,product_source_modified=False,
    stat_continuity_verified=False,GUI_verified=False,whole_goal_complete=False)
(out/'readback.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
files=['selected-methods.json','inspector.py','readback.json']
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in files:z.write(out/n,n)
with zipfile.ZipFile(out/'raw.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==sha(out/n)for n in files)
print(json.dumps(dict(contract=str(out),methods=len(selected),relations=relations,stat_continuity_verified=False),ensure_ascii=False))
