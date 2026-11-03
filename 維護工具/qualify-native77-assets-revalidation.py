"""Combine unchanged-source test evidence with independently revalidated archives."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,re,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--terminal-tool-chunk',required=True);a=ap.parse_args();run=a.run.resolve(strict=True)
assert run.is_relative_to(ROOT/'驗證紀錄/current/native77-assets-revalidation');r=read(run/'receipt.json')
assert r['status']=='executed_current_archive_native77_revalidation' and r['owned_process'] is None and r['archive_inputs_unchanged']
old=Path(r['original_cohort']);assert sha(old/'cohort.json')==r['original_cohort_sha256'];c=read(old/'cohort.json')
assert c['source_manifest_sha256']==r['source_manifest_sha256']==sha(old/'source-inputs.json')
inputs=read(old/'compiled-inputs.json');assert len(inputs)==1031
for n,h in inputs.items():assert sha(ROOT/n)==h
assert read(old/'terminal-only-observation.json')['recorded_owned_all_terminal'] and read(old/'terminal-only-observation.json')['actual_terminal_tool_chunk']=='c82688'
assert len(r['process_history'])==30 and all(j['exit_code']==0 for j in r['process_history'])
pins=read(run/'asset-inputs-before.json');assert sha(run/'asset-inputs-before.json')==r['archive_inputs_sha256']
print('CHECKING_CURRENT_ARCHIVE_FINGERPRINTS='+str(len(pins['archives'])),flush=True)
for n,v in pins['archives'].items():
    f=Path(n);assert sha(f)==v['sha256'] and (f.stat().st_size,f.stat().st_mtime_ns)==(v['bytes'],v['mtime_ns']),n
for n,v in pins['sound_metadata'].items():
    f=Path(n);assert (f.stat().st_size,f.stat().st_mtime_ns)==(v['bytes'],v['mtime_ns']),n
for n,v in r['libraries'].items():assert sha(v['path'])==v['sha256']
assert sha(r['golden_image']['path'])==r['golden_image']['sha256']
report=read(run/'current-asset-report.json');assert sha(run/'current-asset-report.json')==r['current_asset_report_sha256']
assert report['frames']==72 and report['empty_frames']==10 and report['encoded_bytes']==716815 and report['sound_pcm_bytes']==188158 and not report['audio_device_opened']
assert len(report['members'])==4 and report['sound_ids']==[717] and report['retained_bytes']<=32*1024*1024
original=(ROOT/'LiTo381-Modern/src/modern_lito/disintegrate_assets.rs').read_text(encoding='utf-8-sig')
body=(run/'current-assets-probe.rs').read_text(encoding='utf-8-sig');assert '__ASSETS_PATH__' not in body and 'pub mod disintegrate_assets' not in body
pattern=r'\(\s*(\d+),\s*(\d+),\s*"([0-9a-f]{64})",\s*(\d+),\s*"([0-9a-f]{64})"\s*,?\s*\)'
goldens=re.findall(pattern,original);assert len(goldens)==4 and re.findall(pattern,body)==goldens
for row,expected in zip(report['members'],goldens):
    shard,size,source,frame_bytes,pixels=expected
    assert row['raw_sha256']==source and row['raw_bytes']==int(size) and row['frame_bytes_with_offsets']==int(frame_bytes) and row['frame_pixel_sha256']==pixels and row['frames']==18
def log(base,label):return (base/label/'stdout.log').read_text(encoding='utf-8-sig')
def summary(text):return re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',text)
assert set(re.findall(r'^(.+): test$',log(run,'golden-inventory'),re.M))=={'current_tw_assets_match_existing_independent_goldens'}
assert summary(log(run,'golden-current-resources'))==[('ok','1','0','0','0','0')]
profiles={}
for profile,info in r['profiles'].items():
    origin=c['profiles'][profile];assert len(info['integration_targets'])==12
    assert summary(log(run,profile+'/native77-actual-assets'))==[('ok','10','0','0','0','2416')]
    related_source_passed=0;related_source_ignored=0
    for prefix,v in origin['selections'].items():
        label=prefix.replace('modern_lito::','').replace('::','-').strip('-');counts=v['counts']
        assert summary(log(old,profile+'/'+label))==[('ok',*[str(counts[k]) for k in ['passed','failed','ignored','measured','filtered_out']])]
        rows=dict(re.findall(r'^test (.+?) \.\.\. (ok|FAILED|ignored)(?:,.*)?$',log(old,profile+'/'+label),re.M));assert set(rows)==set(v['test_names']) and 'FAILED' not in rows.values()
        related_source_passed+=counts['passed'];related_source_ignored+=counts['ignored']
    assert related_source_passed==55
    total=0;ignored=0
    for target,item in info['integration_targets'].items():
        assert sha(item['path'])==item['sha256']==origin['integration_targets'][target]['sha256'];counts=item['counts'];assert counts==origin['integration_targets'][target]['counts']
        rows=dict(re.findall(r'^test (.+?) \.\.\. (ok|FAILED|ignored)(?:,.*)?$',log(run,profile+'/'+target),re.M))
        assert set(rows)==set(origin['integration_targets'][target]['registered_tests']) and {n for n,v in rows.items() if v=='ignored'}==set(origin['integration_targets'][target]['registered_ignored']) and 'FAILED' not in rows.values()
        assert summary(log(run,profile+'/'+target))==[('ok',str(counts['passed']),'0',str(counts['ignored']),'0','0')]
        total+=counts['passed'];ignored+=counts['ignored']
    assert total==69 and ignored==0
    for kind in ['normal','test']:assert sha(info[kind]['path'])==info[kind]['sha256']==origin[kind]['sha256']
    profiles[profile]=dict(normal_sha256=info['normal']['sha256'],test_sha256=info['test']['sha256'],retained_same_source_library_passed=55,retained_same_source_library_ignored=related_source_ignored,revalidated_actual_asset_passed=10,revalidated_external_images=12,revalidated_external_passed=69,revalidated_external_ignored=0)
actual_jar=ROOT.parent/'lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar';assert sha(actual_jar)==c['reference_jar_sha256']=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
driver=read(run/'original-runner-process.json');identities=[dict(pid=driver['pid'],created=driver['creation_utc'])]+[dict(pid=j['pid'],created=j['creation_utc']) for j in r['process_history']]
ids=','.join(str(j['pid']) for j in identities);ps="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-Command',ps]);live=json.loads(raw.decode('utf-8-sig')) if raw.strip() else []
if isinstance(live,dict):live=[live]
for row in live:
    for item in identities:
        if row['ProcessId']==item['pid']:assert abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-datetime.fromisoformat(item['created'])).total_seconds())>.001
terminal=dict(original_session_id=42190,exit_code=0,terminal_tool_chunk=a.terminal_tool_chunk);save(run/'actual-tool-terminal-readback.json',terminal)
out=run/('qualified-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir();(out/'qualifier.py').write_bytes(Path(__file__).read_bytes())
files=[f for f in run.iterdir() if f.is_file()]
for job in r['process_history']:files.extend(f for f in (run/job['label']).iterdir() if f.is_file())
raw_pins={f.relative_to(run).as_posix():sha(f) for f in files}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for f in files:z.write(f,f.relative_to(run).as_posix())
with zipfile.ZipFile(out/'raw.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in raw_pins.items())
q=dict(status='independently_qualified_native77_normals_related_with_current_archive_revalidation',run=str(run),source_manifest_sha256=r['source_manifest_sha256'],source_files=1030,compiled_input_files=1031,original_cohort=str(old),original_cohort_sha256=sha(old/'cohort.json'),
    original_source_qualification='same-source normal builds/version0 and55 synthetic library bodies per profile retained; old changed-resource qualification is not reused',profiles=profiles,
    current_archives=str(run/'asset-inputs-before.json'),current_archives_sha256=sha(run/'asset-inputs-before.json'),current_selected_four_SPRs_and72_frames_match_independent_CSharp=True,
    current_asset_report_sha256=sha(run/'current-asset-report.json'),recorded_owned_identities=len(identities),recorded_owned_all_terminal=True,original_terminal=terminal,raw_files=raw_pins,raw_archive_sha256=sha(out/'raw.zip'),
    new_sessions=0,new_casts=0,GUI_verified=False,real_audio_device_verified=False,full_G06_verified=False,whole_goal_complete=False)
save(out/'qualified.json',q);print(json.dumps(dict(qualification=str(out/'qualified.json'),sha256=sha(out/'qualified.json'),profiles=profiles,recorded_owned_identities=len(identities),whole_goal_complete=False),ensure_ascii=False))
