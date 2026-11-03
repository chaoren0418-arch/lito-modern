"""Preserve a normal GUI source mismatch and inspect the pinned formal contract."""
from pathlib import Path
from datetime import datetime, timezone
import argparse, hashlib, json, os, re, subprocess, zipfile
R=Path(__file__).resolve().parents[1]; V=R/'驗證紀錄/current'; P=R/'LiTo381-Modern'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('case',type=Path);a=ap.parse_args()
case=a.case.resolve(strict=True);assert case.is_relative_to(V/'native-accessibility-labels-sky-gui')
c=read(case/'cohort.json');w=read(R/'工作佇列.json')
assert c['profile']=='Release'and c['scenario']=='rental-entry'
assert c['status']=='runtime_exited_original_zero_independent_GUI_qualification_pending'and c['recorded_owned_processes_terminal']
assert not w['active_workloads']and not w['source_and_corresponding_QA_frozen']
source=w['current_product_identity']['source_manifest_sha256'];assert source==c['source_manifest_sha256']
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files']
assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
events=read(case/'Sky-events.json');b=[row for row in events if row['role']=='b']
assert [row for row in b if row['label']=='source-choice'][0]['action']['args']['value']=='測試乙'
response=[row for row in b if row['label']=='character-response'][-1]
assert 'PeerFixture2(測試甲)'in response['tree']
assert not any(row['label']=='select-character'for row in b)
states=[json.loads(line)['state']for line in(case/'server-trace.jsonl').read_text('utf-8-sig').splitlines()]
b_states=[row for row in states if row['test_account']=='PeerFixture2']
assert b_states and all(row['source']=='測試甲'for row in b_states)
frames_path=next((case/'captures').glob('*/frames.jsonl'))
frames=[json.loads(line)for line in frames_path.read_text('utf-8-sig').splitlines()]
dt=lambda s:datetime.fromisoformat(s.replace('Z','+00:00'))
account_time=dt(next(row['observed_utc']for row in b if row['label']=='account'))
own=[row for row in frames if dt(row['time'])>=account_time]
source_selection=next(row for row in own if row['direction']=='client_to_server'and row['plain'].split()[0]=='C8')
assert bytes.fromhex(source_selection['plain'])[1:].rstrip(b'\0').decode('cp950')=='測試乙'
redirect=next(row for row in own if row['direction']=='server_to_client'and 'C_C8 selected-source=測試乙'in row['note'])
game_version=next(row for row in own if row['direction']=='client_to_server'and row['plain'].split()[0]=='00')
elapsed=(dt(game_version['time'])-dt(source_selection['time'])).total_seconds();assert elapsed>30
out=V/'native-source-handoff-expiry'/('contract-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
(out/'inspector.py').write_bytes(Path(__file__).read_bytes())
jar=P/'target/current/server-reference/formal-server-75DA69B0.jar'
expected='75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371';assert sha(jar)==expected
jdk=Path(r'C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin')
env=os.environ.copy();env.update(TEMP=str(P/'target/current/scratch'),TMP=str(P/'target/current/scratch'))
commands=[];headers={};methods={}
for name in ['GameHub','ClientSession']:
    command=[str(jdk/'javap.exe'),'-J-Dfile.encoding=UTF-8','-J-Djava.io.tmpdir='+env['TEMP'],'-p','-c','-constants','-classpath',str(jar),'local.lito381.server.'+name]
    run=subprocess.run(command,capture_output=True,env=env,creationflags=subprocess.CREATE_NO_WINDOW)
    (out/(name+'.stdout.txt')).write_bytes(run.stdout);(out/(name+'.stderr.txt')).write_bytes(run.stderr)
    assert run.returncode==0
    text=run.stdout.decode('utf-8-sig');lines=text.splitlines()
    starts=[i for i,line in enumerate(lines)if re.match(r'^  \S.*\(.*\).*;$',line)]
    selected={}
    for pos,start in enumerate(starts):
        header=lines[start]
        if any(key in header for key in ['ClassicSourceHandoff','selectClassicServer',' login(', ' authenticate(']):
            end=starts[pos+1]if pos+1<len(starts)else len(lines)
            selected[header]='\n'.join(lines[start:end])+'\n'
    methods[name]=selected;headers[name]=list(selected)
    commands.append(dict(command=command,exit_code=run.returncode))
(out/'selected-methods.json').write_text(json.dumps(methods,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
proof=dict(status='normal_GUI_source_mismatch_preserved_formal_contract_inspected_not_qualified',case=str(case),profile=c['profile'],source_manifest_sha256=source,
           requested_source='測試乙',actual_GAME_source='測試甲',actual_character_directory='PeerFixture2(測試甲)',
           source_selection_frame=source_selection,source_redirect_frame=redirect,GAME_version_frame=game_version,
           selection_to_GAME_version_seconds=elapsed,stopped_before_wrong_source_character_selection=True,
           original_session=49194,terminal_tool_chunk='ab7b64',original_exit_code=0,GUI_qualification_granted=False,
           formal_jar=str(jar),formal_jar_sha256=expected,commands=commands,selected_method_headers=headers,
           cause_requires_semantic_review=True,product_source_modified=False,whole_goal_complete=False)
raw={name:case/name for name in ['cohort.json','runner.py','Sky-events.json','ready.json','result.json','server-trace.jsonl','server.properties','server.stdout.log','server.stderr.log']}
raw['frames.jsonl']=frames_path
for f in out.iterdir():
    if f.is_file():raw[f.name]=f
pins={n:sha(p)for n,p in raw.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n,p in raw.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip')as z:assert set(z.namelist())==set(pins)and all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in pins.items())
proof.update(raw_files=pins,raw_archive_sha256=sha(out/'raw.zip'))
(out/'readback.json').write_text(json.dumps(proof,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
assert sha(jar)==expected and all(sha(R/n)==h for n,h in before.items())
print(json.dumps(dict(evidence=str(out/'readback.json'),selection_to_GAME_version_seconds=elapsed,headers=headers,GUI_qualification_granted=False),ensure_ascii=False))
