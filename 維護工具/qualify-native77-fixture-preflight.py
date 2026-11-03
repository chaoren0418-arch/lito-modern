"""Read back only initial memory fixtures, not a network/gameplay qualification."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);a=ap.parse_args();run=a.run.resolve(strict=True)
assert run.is_relative_to(ROOT/'驗證紀錄/current/native77-live-fixture');r=read(run/'receipt.json')
assert r['status']=='executed_native77_initial_repository_profiles_only' and r['owned_process'] is None and r['repository_only'] and not r['game_server_started']
assert r['resource_profile']=='tw13081901-v1' and r['extended_item_details'] and r['new_sessions']==r['new_casts']==0
assert r['reference_jar_sha256']==sha(r['reference_jar'])=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
assert sha(r['common_catalog']['path'])==r['common_catalog']['sha256']=='B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'
p=r['preparation'];assert sha(p['base_harness'])==p['base_harness_sha256'] and sha(p['candidate_harness'])==p['candidate_harness_sha256']==sha(run/'NativeSelectorHarness.java')
source=(run/'NativeSelectorHarness.java').read_text(encoding='utf-8-sig');assert source.index('args[2].equals("--verify-only")')<source.index('try(ServerSocket login=new ServerSocket')
branch=source[source.index('if(args.length==3'):source.index('int peerCount=')];assert 'initialDisintegrateFixture(root,profile,accounts)' in branch and 'return;' in branch
assert 'if(name.equals("maxHp"))value=32767;' in source and 'if(mage&&name.equals("maxMp"))' in source and 'if(mage&&name.equals("lawful"))' in source
assert 'disintegrateFixture(profile))initialDisintegrateFixture(root,profile,second);' in source
assert len(r['class_pins'])==2
for n,h in r['class_pins'].items():assert Path(n).resolve(strict=True).is_relative_to(ROOT) and sha(n)==h
normal=ROOT/'驗證紀錄/current/native77-scene-integration/validation-20261017-183727-278843';inputs=read(normal/'compiled-inputs.json');assert len(inputs)==1031
for n,h in inputs.items():assert sha(ROOT/n)==h
assert r['source_manifest_sha256']==sha(normal/'source-inputs.json')
profiles=['disintegrate','disintegrate-refused','disintegrate-lawful'];assert list(r['profiles'])==profiles
assert [j['label'] for j in r['process_history']]==['compile',*profiles] and all(j['exit_code']==0 for j in r['process_history'])
reports={}
for name in profiles:
    info=r['profiles'][name];report=Path(info['report']);assert report.resolve(strict=True).is_relative_to(ROOT)
    assert sha(report)==info['sha256']==sha(run/(name+'.json'));data=read(report)
    assert data['profile']==name and data['before_all_sessions'] and data['runtime_mutations']==0 and data['natural_special_critical']
    rows=data['rows'];assert rows==info['rows'];assert all(row['max_hp']==32767 for row in rows)
    mage=next(row for row in rows if row['object_id']==3);target=next(row for row in rows if row['object_id']==1)
    assert mage['learned77'] and mage['max_mp']==(2 if name=='disintegrate-refused' else 5000) and mage['lawful']==(499 if name=='disintegrate-lawful' else 1000)
    assert set(f.name for f in report.parent.iterdir())=={'initial-disintegrate-fixture.json'}
    job=next(j for j in r['process_history'] if j['label']==name);args=job['command']
    assert args[-2:]==[name,'--verify-only'] and '-Dlito.resourceProfile=tw13081901-v1' in args and '-Dlito.extendedItemDetails=true' in args
    assert '-Dlito.commonCatalog='+r['common_catalog']['path'] in args
    assert (run/name/'stdout.log').read_text(encoding='utf-8-sig').strip()=='PRESESSION_FIXTURE_VERIFIED='+name
    reports[name]=dict(source_report=str(report),report_sha256=sha(report),rows=rows)
terminal=read(run/'actual-tool-terminal-readback.json');assert terminal==dict(exit_code=0,terminal_tool_chunk='5ee72d')
ids=','.join(str(j['pid']) for j in r['process_history']);ps="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-Command',ps]);live=json.loads(raw.decode('utf-8-sig')) if raw.strip() else []
if isinstance(live,dict):live=[live]
for row in live:
    for job in r['process_history']:
        if row['ProcessId']==job['pid']:assert abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-datetime.fromisoformat(job['creation_utc'])).total_seconds())>.001
out=run/('qualified-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir();(out/'qualifier.py').write_bytes(Path(__file__).read_bytes())
files=[f for f in run.iterdir() if f.is_file()]+[f for label in ['compile',*profiles] for f in (run/label).iterdir() if f.is_file()];pins={f.relative_to(run).as_posix():sha(f) for f in files}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for f in files:z.write(f,f.relative_to(run).as_posix())
with zipfile.ZipFile(out/'raw.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in pins.items())
q=dict(status='independently_qualified_native77_initial_memory_fixtures_only',run=str(run),source_manifest_sha256=r['source_manifest_sha256'],fixture_profiles=reports,
    resource_profile=r['resource_profile'],extended_item_details=True,reference_jar_sha256=r['reference_jar_sha256'],common_catalog=r['common_catalog'],candidate_harness=p['candidate_harness'],candidate_harness_sha256=p['candidate_harness_sha256'],
    class_pins=r['class_pins'],original_exit_code=0,original_terminal_tool_chunk='5ee72d',recorded_owned_processes=r['process_history'],recorded_owned_all_terminal=True,raw_files=pins,raw_archive_sha256=sha(out/'raw.zip'),
    qualification_scope='initial memory repository rows and unmodified book77 under TW/B64 configuration, before any socket/server/session',
    game_server_started=False,server_permissions_executed=False,natural_critical_observed=False,full_ClientSession_executed=False,new_sessions=0,new_casts=0,production_source_modified=False,GUI_verified=False,whole_goal_complete=False)
save(out/'qualified.json',q);print(json.dumps(dict(qualification=str(out/'qualified.json'),sha256=sha(out/'qualified.json'),fixture_profiles=3,new_sessions=0,new_casts=0,whole_goal_complete=False),ensure_ascii=False))
