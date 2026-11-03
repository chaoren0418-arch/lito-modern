"""Independent source/image/log/process readback for the isolated native77 core."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,re,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native77-lifecycle')
r=read(run/'receipt.json');assert r['status']=='executed_current_TW_lifecycle_candidate' and r['owned_process'] is None
assert [j['label'] for j in r['process_history']]==['compile','inventory','ignored-inventory','unit','actual-lifecycle']
assert all(j['exit_code']==0 for j in r['process_history'])
for name,h in r['core_pins'].items():assert sha(name)==h==sha(run/Path(name).name)
p=r['preparation'];assert sha(p['candidate'])==p['candidate_sha256']==sha(run/'candidate-assets.rs')
normal=Path(r['normal_qualification']);assert sha(normal)==r['normal_qualification_sha256'];n=read(normal.parent/'cohort.json')
assert r['source_manifest_sha256']==n['source_manifest_sha256']=='78A49D5D73122C137A647B7F014055AC2E8B11BB15A36C5175D3FD8777B4EF93'
inputs=read(normal.parent/'compiled-inputs.json');assert len(inputs)==1029
for name,h in inputs.items():assert sha(ROOT/name)==h,name
for info in n['profiles'].values():assert sha(info['normal']['path'])==info['normal']['sha256']
for info in r['libraries'].values():assert sha(info['path'])==info['sha256']
lo,hi=map(datetime.fromisoformat,r['current_native_library_emission_window']);assert lo<=datetime.fromtimestamp(Path(r['libraries']['lito_modern']['path']).stat().st_mtime,tz=timezone.utc)<=hi
for name,h in r['original_input_pins'].items():assert sha(name)==h
for name,m in r['original_PAK_metadata'].items():assert (Path(name).stat().st_size,Path(name).stat().st_mtime_ns)==(m['bytes'],m['mtime_ns'])
assert sha(r['oracle']['path'])==r['oracle']['sha256']
vectors=[json.loads(x) for x in Path(r['oracle']['path']).read_text(encoding='utf-8-sig').splitlines() if json.loads(x)['skill']==77]
assert vectors==r['oracle']['vectors'] and {v['canonical_gfx'] for v in vectors}=={11746,11748}
assert sha(r['product']['path'])==r['product']['sha256']
def log(label):return (run/label/'stdout.log').read_text(encoding='utf-8-sig')
def inventory(label):return set(re.findall(r'^(.+): test$',log(label),re.M))
all_tests=inventory('inventory');ignored=inventory('ignored-inventory');assert len(all_tests)==13 and len(ignored)==8 and ignored<=all_tests
for label,expected in [('unit',('5','0','8','0','0')),('actual-lifecycle',('7','0','0','0','6'))]:
    assert re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',log(label))==[('ok',*expected)]
passed=set(re.findall(r'^test (.+?) \.\.\. ok$',log('actual-lifecycle'),re.M))
assert passed=={n for n in ignored if n.startswith('native77_probe::actual_')}
unit_passed=set(re.findall(r'^test (.+?) \.\.\. ok$',log('unit'),re.M));assert unit_passed==all_tests-ignored
core=(run/'native_disintegrate.rs').read_text(encoding='utf-8-sig');probe=(run/'lifecycle_probe.rs').read_text(encoding='utf-8-sig')
assert 'Iterator<Item = &\'a Launch>' in core and 'Arc::ptr_eq(&e.owner, owner)' in core
assert 'e.generation == world.generation' in core and 'attacker_incarnation == actor.incarnation' in core
assert 'Arc::ptr_eq(&e.publication, &self.publication)' in core and 'self.publication = Arc::new(())' in core
assert core.index('blit_screen_rgb555')<core.index('blit_multiply_rgb555')
assert 'Sound(717)' in core and 'Clip::new(' in core and 'NativeFinite' not in probe
assert 'assert_eq!(serde_json::to_value(&w).unwrap(), original)' in probe
assert 'assert_eq!(serde_json::to_value(&w).unwrap(), authority)' in probe
ids=','.join(str(j['pid']) for j in r['process_history'])
ps="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-Command',ps]);live=json.loads(raw.decode('utf-8-sig')) if raw.strip() else []
if isinstance(live,dict):live=[live]
for row in live:
    for old in r['process_history']:
        if row['ProcessId']==old['pid']:assert abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-datetime.fromisoformat(old['creation_utc'])).total_seconds())>.001
terminal=read(run/'actual-tool-terminal-readback.json');assert terminal==dict(original_session_id=90592,exit_code=0,terminal_tool_chunk='582eaa')
out=run/('qualified-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
(out/'qualifier.py').write_bytes(Path(__file__).read_bytes())
names=['runner.py','receipt.json','probe.rs','candidate-assets.rs','native_disintegrate.rs','lifecycle_probe.rs','actual-tool-terminal-readback.json']
names += [f.relative_to(run).as_posix() for label in ['compile','inventory','ignored-inventory','unit','actual-lifecycle'] for f in (run/label).iterdir() if f.is_file()]
pins={name:sha(run/name) for name in names}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for name in names:z.write(run/name,name)
with zipfile.ZipFile(out/'raw.zip') as z:assert all(hashlib.sha256(z.read(name)).hexdigest().upper()==h for name,h in pins.items())
q=dict(status='independently_qualified_isolated_native77_lifecycle',run=str(run),source_manifest_sha256=r['source_manifest_sha256'],normal_qualification=str(normal),
    core_pins=r['core_pins'],product=r['product'],unit_passed=5,unit_ignored=8,explicit_actual_asset_lifecycle_passed=7,explicit_actual_asset_lifecycle_ignored=0,
    all_tests=sorted(all_tests),passed_actual_asset_lifecycle_bodies=sorted(passed),recorded_owned_processes=r['process_history'],recorded_owned_all_terminal=True,
    original_terminal=terminal,raw_files=pins,raw_archive_sha256=sha(out/'raw.zip'),production_source_modified=False,
    stamp_scope='captured synchronously at native actor marker emission; internal publication, source-owner, world-generation and actor-incarnation; cancel/failure then same-owner retry rejects old publication',
    timing_scope='NativeFinite integer per-frame dwell, dispatch-relative scheduling, one sound marker, one impact marker, terminal source0 dwell',
    canvas_scope='current qualified actual parent/layer assets; existing Screen555 then Multiply555 wiring; projection overflow preflight',
    authority_scope='read-only World verified unchanged by presentation; existing native World immediate-hurt path not yet integrated',
    sound_payload_delivery_verified=False,native_scene_interleaving_verified=False,actual_server_casts=0,new_sessions=0,GUI_verified=False,whole_goal_complete=False)
save(out/'qualified.json',q)
print(json.dumps(dict(qualification=str(out/'qualified.json'),sha256=sha(out/'qualified.json'),unit_passed=5,explicit_actual_asset_lifecycle_passed=7,production_source_modified=False,whole_goal_complete=False),ensure_ascii=False))
