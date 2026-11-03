"""Read back the delivery ZIP, exact executable, smoke evidence and asset manifest."""
from pathlib import Path,PurePosixPath
from datetime import datetime,timezone
import argparse,hashlib,importlib.util,json,os,re,zipfile
ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('case',type=Path);ap.add_argument('--session-id',type=int,required=True);ap.add_argument('--terminal-exit-code',type=int,required=True);a=ap.parse_args()
case=a.case.resolve(strict=True);assert case.is_relative_to(ROOT/'驗證紀錄/current/native-release-package') and a.terminal_exit_code==0
r=read(case/'receipt.json');assert r['status']=='created_current_native_Release_development_candidate_and_verified_archive'
package=Path(r['package']).resolve(strict=True);archive=Path(r['archive']).resolve(strict=True)
assert package==ROOT/'交付/Windows-x64-候選' and archive==ROOT/'交付/LiToModern-Windows-x64-candidate.zip'
assert len(r['package_files'])==6 and set(r['package_files'])=={p.name for p in package.iterdir()}
for name,digest in r['package_files'].items():assert sha(package/name)==digest
assert sha(archive)==r['archive_sha256']
with zipfile.ZipFile(archive) as z:
    assert len(z.namelist())==6 and set(z.namelist())=={'LiToModern-Windows-x64/'+name for name in r['package_files']}
    for name,digest in r['package_files'].items():
        with z.open('LiToModern-Windows-x64/'+name) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==digest
m=read(package/'manifest.json');startup=read(r['startup_qualification'])
assert sha(r['startup_qualification'])==r['startup_qualification_sha256']
assert m['source']['source_manifest_sha256']==startup['source_manifest_sha256'] and m['executable']['sha256']==sha(package/'LiToModern.exe')==startup['profiles']['Release']['normal_sha256']
assert sha(package/'build-inputs.json')==m['source']['compiled_input_list_sha256']
inputs=read(package/'build-inputs.json');assert len(inputs)==1025
for name,digest in inputs.items():assert sha(ROOT/name)==digest
assert m['source']['declared_inputs']==1024 and m['source']['compiled_inputs']==1025
assert m['validation']['startup_scoped_tests']=={p:q['counts'] for p,q in startup['profiles'].items()}
assert not any(m['validation'][key] for key in ['GUI_verified','physical_input_verified','clean_machine_verified','portable_package_accepted','whole_goal_complete'])
assert m['protocol_reference']['server_jar_sha256']=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
assert not m['protocol_reference']['server_packaged'] and not m['protocol_reference']['launcher_deployment_required']
for label,expected in [('version',0),('config',0),('invalid-config',2)]:
    observed=read(case/(label+'-stdout.log'))
    assert r['checks'][label]['report']==observed and r['checks'][label]['exit_code']==expected
    assert not (case/(label+'-stderr.log')).read_bytes()
assert r['checks']['version']['report']['default_entry']=='native_login_setup'
assert r['checks']['config']['report']['status']=='valid_native_settings' and not r['checks']['config']['report']['network_started']
assert r['checks']['invalid-config']['report']['status']=='failed'
assert not (package/'LiToModern.native.json').exists()
spec=importlib.util.spec_from_file_location('candidate_pe_reader',ROOT/'維護工具/inspect-release-dependencies.py');module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
pe=module.inspect(package/'LiToModern.exe')
for key in ['ordinary_imports','delayed_imports']:pe[key]=sorted(pe[key],key=lambda name:(name.lower(),name))
pe['dependencies']=sorted(pe['dependencies'],key=lambda row:(row['name'].lower(),row['name']))
assert pe==read(case/'PE-dependencies.json') and pe['machine']=='AMD64' and pe['optional_header']=='PE32+'
assets=read(package/'resource-files.json');manifest=Path(r['external_resource_manifest']);report=read(manifest.parent/'progress.json')
assert sha(package/'resource-files.json')==sha(manifest)==r['external_resource_manifest_sha256']==report['manifest_sha256']
assert len(assets)==report['files']==13339 and sum(v['bytes'] for v in assets.values())==report['bytes']==10807804454
assert report['whole_file_SHA256'] and report['metadata_and_file_set_unchanged_before_after']
before=read(manifest.parent/'observed-before.json');assert set(before)==set(assets)
resource_root=Path(report['resource_root'])
for name,item in assets.items():
    relative=PurePosixPath(name);assert not relative.is_absolute() and '..' not in relative.parts
    assert re.fullmatch('[0-9A-F]{64}',item['sha256']) and item['bytes']==before[name][0]
    st=(resource_root/name).stat(follow_symlinks=False)
    assert not getattr(st,'st_file_attributes',0)&0x400 and (st.st_size,st.st_mtime_ns,st.st_ino)==tuple(before[name])
out=case/'qualified';assert not out.exists();out.mkdir()
files={p.name:p for p in case.iterdir() if p.is_file()};files['independent-qualifier.py']=Path(__file__)
hashes={name:sha(p) for name,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
    for name,p in files.items():z.write(p,name)
with zipfile.ZipFile(out/'raw.zip') as z:
    for name,digest in hashes.items():
        with z.open(name) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==digest
q=dict(status='independently_readback_qualified_native_Release_development_package_candidate',original_session_id=a.session_id,observed_terminal_exit_code=0,package=str(package),archive=str(archive),archive_sha256=r['archive_sha256'],package_files=r['package_files'],exe_sha256=m['executable']['sha256'],source_manifest_sha256=startup['source_manifest_sha256'],source_files=1024,compiled_input_files=1025,external_resource_manifest_sha256=report['manifest_sha256'],external_asset_files=13339,external_asset_bytes=10807804454,external_manifest_producer_hashed_complete_files=True,current_external_file_identity_metadata_unchanged=True,external_file_hashes_recomputed_by_this_reader=False,limited_system_PATH_smoke_evidence_verified=True,actual_formal_sessions=0,GUI_verified=False,physical_input_verified=False,clean_machine_verified=False,portable_package_accepted=False,whole_goal_complete=False,raw_archive_sha256=sha(out/'raw.zip'),raw_files=hashes,qualified_utc=datetime.now(timezone.utc).isoformat())
(out/'qualified.json').write_text(json.dumps(q,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
print(json.dumps(dict(qualification=str(out/'qualified.json'),archive=str(archive),assets=13339,clean_machine_verified=False),ensure_ascii=False))
