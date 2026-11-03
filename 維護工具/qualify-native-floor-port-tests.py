"""Read back actual source1005 floor tests; selected lib groups are not full lib."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,re,sys,zipfile
ROOT=Path(__file__).resolve().parents[1]
APP=ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
RUN=Path(sys.argv[1]).resolve(strict=True);assert RUN.is_relative_to(APP)
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
digest=lambda b:hashlib.sha256(b).hexdigest().upper()
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
save=lambda p,v:p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
def pe(data):
    e=int.from_bytes(data[60:64],'little');assert data[:2]==b'MZ' and data[e:e+4]==b'PE\0\0' and data[e+4:e+6]==b'\x64\x86' and data[e+24:e+26]==b'\x0b\x02'
r=read(RUN/'receipt.json');assert r['status'] in ['actual_applied_lighting_and_related_targets_passed','actual_lighting_tests_and_unique_entry_normal_builds_passed']
assert r['compile_exit_code']==0 and r['owned_process'] is None and r['source_inputs_unchanged']
pins=read(RUN/'compiled-inputs.json');assert len(pins)==r['compiled_inputs']==1005
for n,h in pins.items():assert sha(ROOT/n)==h,n
assert pins['LiTo381-Modern/tests/fixtures/native-floor-gate-original.json']=='1B0AAD36AD82B594B7AC535EE86A1EBC536788B62A78F82732BF3BE333BBCFC3'
with zipfile.ZipFile(RUN/'compiled-source.zip') as z:
    assert set(z.namelist())==set(pins)
    for n,h in pins.items():assert digest(z.read(n))==h,n
required={'native_light_color','native_light_sources','native_render_environment','native_light_lifecycle','native_world_lighting','native_world','native_motion','native_actor_states','native_environment','native_world_lighting_scene','native_water_light','native_actor_relations','native_water_clock','native_terrain_pixels','native_world_layers','native_resource_loading','native_terrain_cache','native_terrain_residence'}
assert set(r['suites'])==required and r['library_group_filters']==['world_raster::tests::','world_scene::tests::']
products=read(RUN/'products.json');assert set(products)==required|{'lito_modern'}
messages=[json.loads(l) for l in (RUN/'build/stdout.log').read_text(encoding='utf-8-sig').splitlines() if l.startswith('{')]
artifacts={m['target']['name']:m for m in messages if m.get('reason')=='compiler-artifact' and m.get('executable') and m['profile']['test']};assert set(artifacts)==set(products)
with zipfile.ZipFile(RUN/'test-products.zip') as z:
    assert set(z.namelist())=={n+'.exe' for n in products}
    for n,p in products.items():
        data=z.read(n+'.exe');assert digest(data)==sha(Path(p['path']))==p['sha256'];pe(data)
        assert Path(artifacts[n]['executable']).resolve()==Path(p['path']).resolve() and artifacts[n]['features']==[]
incidental=read(RUN/'incidental-bin-products.json')
with zipfile.ZipFile(RUN/'incidental-bin-products.zip') as z:
    assert set(z.namelist())=={n+'.exe' for n in incidental}
    for n,p in incidental.items():assert digest(z.read(n+'.exe'))==p['sha256'] and p['normal_main_qualified'] is False
assert set(r['results'])==required|{'lib-world-raster','lib-world-scene'}
counts={'passed':0,'failed':0,'ignored':0};metadata={}
for name,result in r['results'].items():
    log=(RUN/name/'stdout.log').read_text(encoding='utf-8-sig');m=re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',log);assert len(m)==1
    terminal,p,f,i,measured,filtered=m[0];p,f,i,measured,filtered=map(int,[p,f,i,measured,filtered]);assert terminal=='ok' and result['exit_code']==f==measured==0
    assert (p,f,i,filtered)==tuple(result[k] for k in ['passed','failed','ignored','filtered'])
    if name.startswith('lib-'):
        expected='world_raster::tests::' if name=='lib-world-raster' else 'world_scene::tests::';assert result['filters']==[expected] and result['product_sha256']==products['lito_modern']['sha256'] and filtered>0
        assert all(expected in n for n in re.findall(r'^test (.+) \.\.\. ',log,re.M))
    else:assert result['filters']==[] and filtered==0 and p>0 and result['product_sha256']==products[name]['sha256']
    for k,v in zip(counts,[p,f,i]):counts[k]+=v
    for n in ['stdout.log','stderr.log']:metadata[name+'/'+n]=RUN/name/n
assert counts=={k:r[k] for k in counts} and counts['failed']==0
assert r['results']['native_terrain_pixels']['passed']==3 and r['results']['lib-world-raster']['passed']==7 and r['results']['lib-world-raster']['ignored']==0
assert r['results']['lib-world-scene']['passed']==19 and r['results']['lib-world-scene']['ignored']==6
for n in ['compiled-inputs.json','products.json','incidental-bin-products.json','runner.py','receipt.json','entry-script-before.ps1','entry-script-before.json']:
    metadata[n]=RUN/n
for n in ['stdout.log','stderr.log']:metadata['build/'+n]=RUN/'build'/n
metadata['independent-qualifier.py']=Path(__file__)
with zipfile.ZipFile(RUN/'profile-raw-metadata.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
    for n,p in metadata.items():z.write(p,n)
with zipfile.ZipFile(RUN/'profile-raw-metadata.zip') as z:
    assert set(z.namelist())==set(metadata)
    for n,p in metadata.items():assert digest(z.read(n))==sha(p),n
q={'status':'independently_qualified_floor_port_profile_and_selected_lib_groups','observed_utc':datetime.now(timezone.utc).isoformat(),'run':str(RUN),'profile':r['profile'],'source_files':1005,'source_manifest_sha256':sha(RUN/'compiled-inputs.json'),'integration_targets':18,'selected_library_groups':r['library_group_filters'],'actual_test_executables':19,'runtime_groups':20,**counts,
 'source_archive_sha256':sha(RUN/'compiled-source.zip'),'test_products_archive_sha256':sha(RUN/'test-products.zip'),'raw_metadata_archive_sha256':sha(RUN/'profile-raw-metadata.zip'),'raw_metadata_files':{n:sha(p) for n,p in metadata.items()},'floor_RGB555_geometry_and_gate64_original_cases_verified':True,'floor_gate_only':True,'map303_overlay_and_background_complete':False,'full_default_regression_verified':False,'full_library_unit_regression_verified':False,'full_current_TCP_regression_verified':False,'normal_main_scope':'Incidental Cargo bins retained separately; unique-entry qualification is separate.','normal_GUI_verified':False,'whole_goal_complete':False}
p=RUN/'profile-qualified.json';assert not p.exists();save(p,q);assert read(p)==q
print(json.dumps({k:q[k] for k in ['status','profile','source_files','actual_test_executables','passed','failed','ignored']}))
