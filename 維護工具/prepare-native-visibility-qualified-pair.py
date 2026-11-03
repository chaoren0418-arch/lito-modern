"""Bind two actual preflights and sixteen live cases as a limited prerequisite.

This is four related test targets per profile, not a broad/default/lib claim.
"""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,shutil,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
def archive(path,h,files):
    assert sha(path)==h
    with zipfile.ZipFile(path) as z:
        assert set(z.namelist())==set(files) and len(z.namelist())==len(files)
        for n,h in files.items():
            with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
ap=argparse.ArgumentParser()
for p in ['debug-preflight','release-preflight','debug-cases','release-cases']:ap.add_argument('--'+p,required=True,type=Path)
a=ap.parse_args();observations={};sources=None;compiled=None;entry_sha=sha(ROOT/'啟動現代客戶端.ps1')
for profile in ['Debug','Release']:
    run=getattr(a,profile.lower()+'_preflight').resolve(strict=True);cases=getattr(a,profile.lower()+'_cases').resolve(strict=True)
    assert run.is_relative_to(ROOT/'驗證紀錄/current/native-detection-invisibility') and cases.is_relative_to(run.parent)
    c=read(run/'cohort.json');q=read(run/'qualified/qualified.json');lq=read(cases/'qualified/qualified.json');lc=read(cases/'cohort.json')
    assert c['profile']==profile and q.get('profile','Debug')==profile and lq.get('profile','Debug')==profile
    assert q['status']==f'independently_qualified_product_visibility_{profile}_preflight_and_unique_normal'
    assert lq['status']==f'independently_qualified_normal_{profile}_visibility_eight_cases'
    assert q['observed_terminal_exit_code']==lq['observed_terminal_exit_code']==0
    assert q['recorded_owned_processes_all_terminal'] and lq['recorded_owned_all_terminal']
    assert q['recorded_owned_identities']==7 and lq['recorded_owned_identities']==32
    assert q['actual_test_executables']==4 and q['counts']['failed']==q['counts']['ignored']==0
    assert lq['actual_client_sessions']==32 and lq['actual_cast_requests']==14 and lq['accepted_casts']==8 and lq['negative_cases']==6
    assert q['source_manifest_sha256']==lq['source_manifest_sha256']==c['source_manifest_sha256']
    assert q['normal_sha256']==lq['normal_sha256']==sha(c['normal']['path'])
    normal_archive=c['normal']['archive'];assert normal_archive['sha256']==q['normal_product_archive_sha256']
    archive(Path(normal_archive['path']),normal_archive['sha256'],{'LiToModern.exe':q['normal_sha256']})
    assert entry_sha==c['entry_sha256'] and c['normal']['unique_entry_exit_code']==0
    pins=read(run/'source-inputs.json');inputs=read(run/'compiled-inputs.json')
    assert len(pins)==1024 and len(inputs)==1025 and sha(run/'source-inputs.json')==q['source_manifest_sha256']
    if sources is None:sources=pins;compiled=inputs;source_sha=q['source_manifest_sha256']
    else:assert pins==sources and inputs==compiled and source_sha==q['source_manifest_sha256']
    archive(Path(c['compiled_source_archive']['path']),c['compiled_source_archive']['sha256'],inputs)
    archive(run/'qualified/raw.zip',q['raw_archive_sha256'],q['raw_files'])
    archive(cases/'qualified/raw.zip',lq['raw_archive_sha256'],lq['raw_files'])
    observations[profile]=dict(run=run,c=c,q=q,cases=cases,lq=lq,lc=lc)
for n,h in compiled.items():assert sha(ROOT/n)==h,n
out=ROOT/'驗證紀錄/current/native-detection-invisibility'/('qualified-pair-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
(out/'source-inputs.json').write_bytes((a.debug_preflight/'source-inputs.json').read_bytes())
(out/'compiled-inputs.json').write_bytes((a.debug_preflight/'compiled-inputs.json').read_bytes())
shutil.copyfile(a.debug_preflight/'compiled-source.zip',out/'compiled-source.zip')
assert sha(out/'compiled-source.zip')==observations['Debug']['c']['compiled_source_archive']['sha256']
profiles={};proof={};processes=[];identities=[]
for profile,v in observations.items():
    preflight=v['run'];cases=v['cases'];q=v['q'];lq=v['lq'];c=v['c']
    profiles[profile]=dict(normal=c['normal'],products=c['products'],related_counts=q['counts'],preflight=str(preflight),cases=str(cases))
    proof[profile]=dict(qualification=str(preflight/'qualified/qualified.json'),qualification_sha256=sha(preflight/'qualified/qualified.json'),
        normal_sha256=q['normal_sha256'],actual_test_executables=4,counts=q['counts'],
        live_qualification=str(cases/'qualified/qualified.json'),live_qualification_sha256=sha(cases/'qualified/qualified.json'),
        live_actual_client_sessions=32,accepted_casts=8,negative_cases=6)
    processes+=c['process_history']
    identities+=c['process_history']
    for i,case in enumerate(v['lc']['cases']):
        assert case['wrapper']['exit_code']==0;identities.append(case['wrapper'])
        folder=Path(case['case'])
        assert sha(folder/'receipt.json')==case['receipt_sha256']==lq['raw_files'][f'case{i}/output/receipt.json']
        receipt=read(folder/'receipt.json')
        assert receipt['jar_sha256']=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
        assert receipt['exe_sha256']==q['normal_sha256']
        for name in ['a-launch-identity.json','b-launch-identity.json','server-launch-identity.json']:
            assert sha(folder/name)==lq['raw_files'][f'case{i}/output/{name}']
            identities.append(read(folder/name))
assert len(identities)==78
save(out/'recorded-upstream-process-identities.json',identities)
state=dict(status='executed_native_visibility_related_scope_and_both_unique_entry_normal_builds',
    source_files=1024,compiled_input_files=1025,source_manifest_sha256=source_sha,entry_sha256=entry_sha,
    compiled_source_archive=dict(path=str(out/'compiled-source.zip'),sha256=sha(out/'compiled-source.zip'),files=compiled),
    profiles=profiles,process_history=processes,owned_process=None,related_four_targets_only=True,
    full_default_regression_verified=False,full_library_verified=False,GUI_verified=False,whole_goal_complete=False)
save(out/'cohort.json',state);qualified=out/'qualified';qualified.mkdir()
(qualified/'reader.py').write_bytes(Path(__file__).read_bytes())
q=dict(status='independently_qualified_native_visibility_settings_and_normal_entry_scope',
    source_files=1024,compiled_input_files=1025,source_manifest_sha256=source_sha,entry_sha256=entry_sha,
    compiled_source_archive_sha256=sha(out/'compiled-source.zip'),profiles=proof,observed_upstream_terminal_exit_code=0,
    reference_jar_sha256='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539',
    common_catalog_sha256='B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01',
    recorded_owned_processes_all_terminal=True,recorded_owned_identities=78,
    limited_to_four_targets_and_eight_visibility_cases_each=True,actual_client_sessions=64,accepted_casts=16,negative_cases=12,
    full_default_regression_verified=False,full_library_verified=False,GUI_verified=False,whole_goal_complete=False)
save(out/'pair-evidence.json',q)
files={'reader.py':qualified/'reader.py'}
for name in ['cohort.json','source-inputs.json','compiled-inputs.json','recorded-upstream-process-identities.json','pair-evidence.json']:files[name]=out/name
hashes={n:sha(p) for n,p in files.items()}
with zipfile.ZipFile(qualified/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n,p in files.items():z.write(p,n)
archive(qualified/'raw.zip',sha(qualified/'raw.zip'),hashes)
q.update(raw_archive_sha256=sha(qualified/'raw.zip'),raw_files=hashes)
save(qualified/'qualified.json',q)
save(out/'completion-observation.json',dict(status='independently_bound_visibility_both_profiles_prerequisite',
    source_manifest_sha256=source_sha,qualification=str(qualified/'qualified.json'),qualification_sha256=sha(qualified/'qualified.json'),
    recorded_owned_processes_all_terminal=True,recorded_owned_identities=78,whole_goal_complete=False))
print(json.dumps(dict(output=str(out),source_manifest_sha256=source_sha,limited_test_targets_per_profile=4,actual_client_sessions=64),ensure_ascii=False))
