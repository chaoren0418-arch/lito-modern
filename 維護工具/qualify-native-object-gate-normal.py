"""Bind source1019 Debug/Release floor scopes and unique-entry normal mains."""
from pathlib import Path
import hashlib,json,sys,tomllib,zipfile
ROOT=Path(__file__).resolve().parents[1]
APP=ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
DEBUG=Path(sys.argv[2]).resolve(strict=True);assert DEBUG.is_relative_to(APP)
RELEASE=Path(sys.argv[1]).resolve(strict=True);assert RELEASE.is_relative_to(APP)
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
digest=lambda b:hashlib.sha256(b).hexdigest().upper()
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
save=lambda p,v:p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
def pe(b):
    e=int.from_bytes(b[60:64],'little');assert b[:2]==b'MZ' and b[e:e+4]==b'PE\0\0' and b[e+4:e+6]==b'\x64\x86' and b[e+24:e+26]==b'\x0b\x02'
pins=read(RELEASE/'compiled-inputs.json');assert len(pins)==1019
for n,h in pins.items():assert sha(ROOT/n)==h,n
profile=tomllib.loads((ROOT/'LiTo381-Modern/Cargo.toml').read_text(encoding='utf-8-sig'))['profile']['release']
assert profile['opt-level']=='z' and profile['lto']=='thin' and profile['codegen-units']==16 and profile['strip']=='symbols'
counts={};runs={};metadata={};ignored_names={}
OLD=APP/'actual-tests-20261016-084451-903086/normal-qualified'
assert sha(OLD/'source-inputs.json')==read(OLD/'qualified.json')['source_manifest_sha256']
with zipfile.ZipFile(OLD.parent/'compiled-source.zip') as z:
    old_scene_bytes=z.read('LiTo381-Modern/src/modern_lito/world_scene.rs')
assert digest(old_scene_bytes)==read(OLD/'source-inputs.json')['LiTo381-Modern/src/modern_lito/world_scene.rs']
old_scene=old_scene_bytes.decode('utf-8-sig')
import re
preexisting_ignored=set(re.findall(r'#\[ignore(?:\s*=\s*"[^"]*")?\]\s*(?:#\[[^\]]+\]\s*)*(?:pub\s+)?fn\s+(\w+)',old_scene));assert len(preexisting_ignored)==6
for name,folder in [('Debug',DEBUG),('Release',RELEASE)]:
    q=read(folder/'profile-qualified.json');r=read(folder/'receipt.json')
    assert q['status']=='independently_qualified_object_gate_profile_and_selected_lib_groups' and q['profile']==r['profile']==name
    assert r['compile_exit_code']==0 and r['owned_process'] is None and r['source_inputs_unchanged']
    assert read(folder/'compiled-inputs.json')==pins and q['source_manifest_sha256']==sha(folder/'compiled-inputs.json')
    assert (q['passed'],q['failed'],q['ignored'],q['actual_test_executables'],q['runtime_groups'])==(135,0,6,23,24)
    assert q['integration_targets']==22 and q['selected_library_groups']==['world_raster::tests::','world_scene::tests::']
    ignored_names[name]=re.findall(r'^test (\S+) \.\.\. ignored',(folder/'lib-world-scene/stdout.log').read_text(encoding='utf-8-sig'),re.M)
    assert len(ignored_names[name])==6 and {n.rsplit('::',1)[-1] for n in ignored_names[name]}==preexisting_ignored
    for archive,key in [('compiled-source.zip','source_archive_sha256'),('test-products.zip','test_products_archive_sha256'),('profile-raw-metadata.zip','raw_metadata_archive_sha256')]:assert sha(folder/archive)==q[key]
    with zipfile.ZipFile(folder/'compiled-source.zip') as z:
        assert set(z.namelist())==set(pins)
        for n,h in pins.items():assert digest(z.read(n))==h,n
    products=read(folder/'products.json');assert len(products)==23
    with zipfile.ZipFile(folder/'test-products.zip') as z:
        assert set(z.namelist())=={n+'.exe' for n in products}
        for n,p in products.items():b=z.read(n+'.exe');assert digest(b)==p['sha256'];pe(b)
    with zipfile.ZipFile(folder/'profile-raw-metadata.zip') as z:
        assert set(z.namelist())==set(q['raw_metadata_files'])
        for n,h in q['raw_metadata_files'].items():assert digest(z.read(n))==h,n
    counts[name]={'passed':135,'failed':0,'ignored':6,'actual_test_executables':23}
    runs[name]=[{'path':str(folder),'receipt_sha256':sha(folder/'receipt.json'),'compiled_source_sha256':sha(folder/'compiled-source.zip'),'test_products_sha256':sha(folder/'test-products.zip'),'profile_qualification_sha256':sha(folder/'profile-qualified.json')}]
    for n in ['receipt.json','profile-qualified.json','compiled-inputs.json','products.json','runner.py']:metadata[name+'/'+n]=folder/n
r=read(RELEASE/'receipt.json');assert r['status']=='actual_lighting_tests_and_unique_entry_normal_builds_passed'
entry=read(RELEASE/'entry-script-before.json');assert entry['profile_environment']=={} and entry['observed_before_normal_builds']
assert sha(ROOT/'啟動現代客戶端.ps1')==sha(RELEASE/'entry-script-before.ps1')==entry['sha256']
normal=read(RELEASE/'normal-products.json');assert normal==r['normal_products'] and set(normal)=={'Debug','Release'}
encodings={}
for name,p in normal.items():
    path=Path(p['path']);assert path.resolve()==ROOT/'LiTo381-Modern/target/current/x86_64-pc-windows-msvc'/name.lower()/'LiToModern.exe'
    assert p['unique_entry_build_exit_code']==0 and sha(path)==p['sha256'] and path.name=='LiToModern.exe';pe(path.read_bytes())
    folder=RELEASE/('normal-'+name);raw=(folder/'stdout.log').read_bytes();assert len(raw)<131072
    try:output=raw.decode('utf-8-sig');encodings[name]='utf-8-sig'
    except UnicodeDecodeError:output=raw.decode('cp950');assert output.encode('cp950')==raw;encodings[name]='cp950_strict_roundtrip'
    assert str(path) in output and b'Finished' in (folder/'entry-build.log').read_bytes()
    with zipfile.ZipFile(folder/'product.zip') as z:assert z.namelist()==['LiToModern.exe'] and digest(z.read('LiToModern.exe'))==p['sha256']
    p.update(unique_entry=True,features=[],machine='AMD64',optional_header='PE32+',build_arguments=['pwsh.exe','-NoProfile','-File',str(ROOT/'啟動現代客戶端.ps1'),'-BuildOnly','-BuildProfile',name])
    for n in ['stdout.log','stderr.log','entry-build.log']:metadata['normal/'+name+'/'+n]=folder/n
for n in ['entry-script-before.ps1','entry-script-before.json','normal-products.json']:metadata['normal/'+n]=RELEASE/n
metadata['normal-independent-qualifier.py']=Path(__file__)
out=RELEASE/'normal-qualified';out.mkdir(exist_ok=False);(out/'source-inputs.json').write_bytes((RELEASE/'compiled-inputs.json').read_bytes())
(out/'preexisting-ignored-world-scene.rs').write_bytes(old_scene_bytes)
metadata['ignored-scope/original-world-scene.rs']=out/'preexisting-ignored-world-scene.rs'
metadata['ignored-scope/original-source-inputs.json']=OLD/'source-inputs.json'
metadata['ignored-scope/original-normal-qualified.json']=OLD/'qualified.json'
nq={'status':'qualified_unique_entry_object_gate_normal_builds','source_files':1019,'source_manifest_includes_compile_fixtures':True,'source_manifest_sha256':sha(out/'source-inputs.json'),'features':[],'profiles':normal,'entry_script_sha256':entry['sha256'],'normal_output_encodings':encodings,'normal_GUI_verified':False,'whole_goal_complete':False}
with zipfile.ZipFile(out/'raw-metadata.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
    for n,p in metadata.items():z.write(p,n)
with zipfile.ZipFile(out/'raw-metadata.zip') as z:
    assert set(z.namelist())==set(metadata)
    for n,p in metadata.items():assert digest(z.read(n))==sha(p)
scope={'status':'qualified_object_gate_Debug135_Release135_and_normal_mains','source_files':1019,'source_manifest_sha256':sha(out/'source-inputs.json'),'counts':counts,'runs':runs,'normal_mains':{n:p['sha256'] for n,p in normal.items()},'all_archives_readback_verified':True,'integration_targets':22,'selected_library_groups':['world_raster::tests::','world_scene::tests::'],'runtime_groups_per_profile':24,'ignored_scope':'6 existing opt-in world_scene tests, explicitly unexecuted.','floor_RGB555_parity_and_gate64_cases_verified':True,'floor_gate_only':False,'supported_first_pass_object_gate_integrated':True,'full_object_dispatch_original_cases':263423,'map303_gate_complete':False,'map303_overlay_and_background_complete':False,'special_global_filters_complete':False,'water_extra_effect_complete':False,'full_default_regression_verified':False,'full_library_regression_verified':False,'full_current_TCP_regression_verified':False,'normal_GUI_verified':False,'whole_goal_complete':False,'raw_metadata_archive_sha256':sha(out/'raw-metadata.zip'),'raw_metadata_files':{n:sha(p) for n,p in metadata.items()}}
scope['ignored_test_names']=ignored_names
save(out/'qualified.json',nq);save(out/'scoped-qualified.json',scope);assert read(out/'qualified.json')==nq and read(out/'scoped-qualified.json')==scope
print(json.dumps({'normal':str(out/'qualified.json'),'scope':str(out/'scoped-qualified.json'),'counts':counts,'mains':scope['normal_mains']}))
