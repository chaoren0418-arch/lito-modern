"""Read back the explicit asset entry, unchanged old loader and real pixel goldens."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,ast,hashlib,json,re,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native77-current-assets')
r=read(run/'receipt.json');assert r['status']=='executed_current_TW_asset_entry_candidate' and r['owned_process'] is None
assert len(r['process_history'])==5 and all(j['exit_code']==0 for j in r['process_history'])
p=r['preparation'];assert sha(p['source'])==p['source_sha256'] and sha(p['candidate'])==p['candidate_sha256']==sha(run/'candidate-assets.rs')
prepared=Path(p['candidate']).parent;original=Path(p['source']).read_text(encoding='utf-8-sig');candidate=Path(p['candidate']).read_text(encoding='utf-8-sig')
tree=ast.parse((ROOT/'維護工具/prepare-native77-current-assets.py').read_text(encoding='utf-8-sig'))
strings={node.targets[0].id:node.value.value for node in tree.body if isinstance(node,ast.Assign) and isinstance(node.targets[0],ast.Name) and isinstance(node.value,ast.Constant) and isinstance(node.value.value,str)}
gate=(prepared/'profile-gate.rs').read_text(encoding='utf-8-sig');test=(prepared/'added-test.rs').read_text(encoding='utf-8-sig')
assert candidate.count(strings['new'])==candidate.count(gate)==candidate.count(test)==1
assert candidate.replace(test,'').replace(gate,'').replace(strings['new'],strings['old'])==original
assert '&root.join("launcher")' in strings['new'] and 'require_current_profile(' in strings['new']
normal=Path(r['normal_qualification']);assert sha(normal)==r['normal_qualification_sha256'];c=read(normal.parent/'cohort.json')
assert read(normal)['source_manifest_sha256']==r['source_manifest_sha256']
inputs=read(normal.parent/'compiled-inputs.json');assert len(inputs)==1029
for name,h in inputs.items():assert sha(ROOT/name)==h,name
for name,h in r['original_input_pins'].items():assert sha(name)==h,name
for name,m in r['original_PAK_metadata'].items():assert (Path(name).stat().st_size,Path(name).stat().st_mtime_ns)==(m['bytes'],m['mtime_ns'])
for name,info in r['libraries'].items():assert sha(info['path'])==info['sha256']
for profile,info in c['profiles'].items():assert sha(info['normal']['path'])==info['normal']['sha256']
assert sha(r['product']['path'])==r['product']['sha256']
def log(label):return (run/label/'stdout.log').read_text(encoding='utf-8-sig')
def inventory(label):return set(re.findall(r'^(.+): test$',log(label),re.M))
all_tests=inventory('inventory');ignored=inventory('ignored-inventory')
assert len(all_tests)==4 and len(ignored)==2 and ignored<=all_tests
new='modern_lito::disintegrate_assets::tests::current_tw_namespace_requires_exact_profile_and_cancels_before_io'
old='modern_lito::disintegrate_assets::tests::data_qualification_rejects_changed_semantics_instead_of_guessing'
rows=dict(re.findall(r'^test (.+?) \.\.\. (ok|FAILED|ignored)(?:,.*)?$',log('unit'),re.M))
assert {n for n,s in rows.items() if s=='ok'}=={new,old} and {n for n,s in rows.items() if s=='ignored'}==ignored
for label,expected in [('unit',('2','0','2','0','0')),('actual-assets',('1','0','0','0','3'))]:
    assert re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',log(label))==[('ok',*expected)]
report=read(run/'asset-report.json');assert sha(run/'asset-report.json')==r['asset_report_sha256']
assert report['status']=='loaded_explicit_current_TW_assets_with_preserved_root_failure' and report['old_root_error']
assert report['frames']==72 and report['empty_frames']==10 and report['encoded_bytes']==716815 and report['retained_bytes']<=32*1024*1024
assert bytes(report['pack_sha256']).hex().upper()==p['pack_sha256'] and bytes(report['text_sha256']).hex().upper()==p['text_sha256'] and report['decoded_bytes']==5974928
body=(prepared/'probe_body.rs').read_text(encoding='utf-8-sig')
pattern=r'\(\s*(\d+),\s*(\d+),\s*"([0-9a-f]{64})",\s*(\d+),\s*"([0-9a-f]{64})"\s*,?\s*\)'
goldens=re.findall(pattern,original);assert len(goldens)==4 and re.findall(pattern,body)==goldens
assert len(report['members'])==4
for row,expected in zip(report['members'],goldens):
    shard,size,source,frame_bytes,pixels=expected
    assert row['raw_sha256']==source and row['raw_bytes']==int(size) and row['frame_bytes_with_offsets']==int(frame_bytes) and row['frame_pixel_sha256']==pixels and row['frames']==18
assert report['sound_ids']==[717] and 0<report['sound_pcm_bytes']<=1024*1024 and not report['audio_device_opened']
sound=next(h for name,h in r['original_input_pins'].items() if name.endswith('717.wav'));assert bytes(report['sound_file_sha256']).hex().upper()==sound
ids=','.join(str(j['pid']) for j in r['process_history'])
ps="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-Command',ps]);live=json.loads(raw.decode('utf-8-sig')) if raw.strip() else []
if isinstance(live,dict):live=[live]
for row in live:
    for old in r['process_history']:
        if row['ProcessId']==old['pid']:assert abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-datetime.fromisoformat(old['creation_utc'])).total_seconds())>.001
out=run/('qualified-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir();(out/'CIM.json').write_bytes(raw)
files={x.relative_to(run).as_posix():x for x in run.rglob('*') if x.is_file() and not any(part.startswith('qualified-') for part in x.relative_to(run).parts)}
files.update({'CIM.json':out/'CIM.json','qualifier.py':Path(__file__),'source-before.rs':Path(p['source']),'preparer.py':ROOT/'維護工具/prepare-native77-current-assets.py','profile-gate.rs':prepared/'profile-gate.rs','added-test.rs':prepared/'added-test.rs','probe-body.rs':prepared/'probe_body.rs'})
hashes={n:sha(x) for n,x in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for name,x in files.items():z.write(x,name)
with zipfile.ZipFile(out/'raw.zip') as z:
    assert set(z.namelist())==set(hashes)
    for name,h in hashes.items():assert hashlib.sha256(z.read(name)).hexdigest().upper()==h
q=dict(status='independently_qualified_explicit_current_TW_disintegrate_asset_candidate',source_manifest_sha256=r['source_manifest_sha256'],candidate_source=p['candidate'],candidate_source_sha256=p['candidate_sha256'],
    original_root_entry_preserved=True,existing_loader_body_preserved=True,original_qualification_assertions_preserved=True,unit_passed=2,unit_failed=0,unchanged_original_ignored_test_preserved=True,
    explicit_actual_asset_test_passed=1,actual_SPRs=4,actual_frames=72,no_draw_frames=10,encoded_bytes=716815,retained_bytes=report['retained_bytes'],sound_ids=[717],sound_pcm_bytes=report['sound_pcm_bytes'],
    old_root_failure_preserved=True,current_TW_pack_sha256=p['pack_sha256'],decoded_action_sha256=p['text_sha256'],recorded_owned_child_identities=5,recorded_owned_children_all_terminal=True,
    production_source_modified=False,native_admission_or_lifecycle_verified=False,playback_verified=False,new_sessions=0,new_casts=0,audio_device_verified=False,GUI_verified=False,whole_goal_complete=False,
    raw_files=hashes,raw_archive_sha256=sha(out/'raw.zip'))
save(out/'qualified.json',q)
print(json.dumps(dict(qualification=str(out/'qualified.json'),unit_passed=2,actual_SPRs=4,frames=72,production_source_modified=False),ensure_ascii=False))
