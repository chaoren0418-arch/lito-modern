"""Read fixed formal bytecode for development; no game, fixture or tests run."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib,json,re,subprocess,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
data=lambda v:(json.dumps(v,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
w=read(R/'工作佇列.json');assert not w['active_workloads'] and w['current_user_requested_goal']['test_execution_deferred_by_user']
jar=R/'LiTo381-Modern/target/current/server-reference/formal-server-75DA69B0.jar'
assert sha(jar)=='75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
client=V/'native-source-handoff-expiry/contract-20261019-174028-047309/ClientSession.stdout.txt'
assert sha(client)=='964DC949F7EA6C575A8E725D025EA67B94B73ABF81C200BBE2BA190D72248CF1'
out=V/'native-blind-formal75-contract'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
javap=Path(r'C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin\javap.exe')
raw={}
for name in ['Protocol','Protocol$Reader','BattleSpellCatalog','BattleSpellCatalog$BlindSpell']:
    cmd=[str(javap),'-J-Dfile.encoding=UTF-8','-classpath',str(jar),'-c','-p','local.lito381.server.'+name]
    (out/(name+'.command.json')).write_bytes(data(dict(command=cmd,purpose='static bytecode read only')))
    p=subprocess.run(cmd,capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW,timeout=120)
    (out/(name+'.stdout.txt')).write_bytes(p.stdout);(out/(name+'.stderr.txt')).write_bytes(p.stderr)
    assert p.returncode==0 and not p.stderr,(name,p.returncode,p.stderr[:200])
    raw[name]=p.stdout.decode('utf-8')
def select(text,names):
    lines=text.splitlines();starts=[i for i,l in enumerate(lines) if re.match(r'^  \S.*\(.*\).*;$',l)]
    methods={}
    for k,start in enumerate(starts):
        if any(' '+name+'(' in lines[start] or ('lambda$'+name+'$') in lines[start] for name in names):
            end=starts[k+1] if k+1<len(starts) else len(lines)
            methods[lines[start].strip()]='\n'.join(lines[start:end])+'\n'
    return methods
methods={
 'Protocol':select(raw['Protocol'],['spellSelector','attackSpellRequest','blind']),
 'Reader':select(raw['Protocol$Reader'],['d','h']),
 'BattleSpellCatalog':select(raw['BattleSpellCatalog'],['blindSpell']),
 'BlindSpell':raw['BattleSpellCatalog$BlindSpell'],
 'ClientSession':select(client.read_text('utf-8-sig'),['useClassicWorldBlindSpell','useBlindSpell','useSpellBeforeDragonTargets'])}
assert all(methods.values())
assert any('Protocol.attackSpellRequest' in s for s in methods['ClientSession'].values())
catalog_lines=raw['BattleSpellCatalog'].splitlines();hits=[i for i,l in enumerate(catalog_lines) if 'String curse-blind' in l];assert len(hits)==1
methods['curse-blind-constructor']='\n'.join(catalog_lines[hits[0]-28:hits[0]+4])+'\n'
(out/'selected-methods.json').write_bytes(data(methods));(out/'inspector.py').write_bytes(Path(__file__).read_bytes())
record=dict(status='formal75_blind_selected_bytecode_static_development_only',source_manifest_sha256=w['current_product_identity']['source_manifest_sha256'],formal_jar=str(jar),formal_jar_sha256=sha(jar),original_client_disassembly=str(client),original_client_disassembly_sha256=sha(client),selected_methods=str(out/'selected-methods.json'),selected_methods_sha256=sha(out/'selected-methods.json'),
 game_or_fixture_executions=0,tests_executed=0,builds_executed=0,product_source_modified=False,request_semantics_review_complete=False,receiver_rendering_implemented=False,GUI_verified=False,whole_goal_complete=False)
files={p.name:p for p in out.iterdir() if p.is_file()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==sha(p) for n,p in files.items())
record.update(raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'),raw_files={n:sha(p) for n,p in files.items()})
(out/'readback.json').write_bytes(data(record))
print(json.dumps(dict(contract=str(out/'readback.json'),static_only=True,tests_executed=0),ensure_ascii=False))
