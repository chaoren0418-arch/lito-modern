"""Independent source/test/normal-main qualification for the water producer."""
from pathlib import Path
import hashlib,json,re,sys,tomllib,zipfile
ROOT=Path(__file__).resolve().parents[1]
APP=ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
RELEASE=Path(sys.argv[1]).resolve(strict=True);assert RELEASE.is_relative_to(APP)
DEBUG=APP/'actual-tests-20261016-071047-820645'
LABELS=APP/'actual-tests-20261016-071433-673834'
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
digest=lambda b:hashlib.sha256(b).hexdigest().upper()
sha=lambda p:digest(p.read_bytes())
save=lambda p,v:p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
def pe(data):
    at=int.from_bytes(data[60:64],'little')
    assert data[:2]==b'MZ' and data[at:at+4]==b'PE\0\0'
    assert data[at+4:at+6]==b'\x64\x86' and data[at+24:at+26]==b'\x0b\x02'
pins=read(RELEASE/'compiled-inputs.json');assert len(pins)==999
for name,value in pins.items():assert sha(ROOT/name)==value,name
profile=tomllib.loads((ROOT/'LiTo381-Modern/Cargo.toml').read_text(encoding='utf-8-sig'))['profile']['release']
assert profile['opt-level']=='z' and profile['lto']=='thin' and profile['codegen-units']==16 and profile['strip']=='symbols'
entry=read(RELEASE/'entry-script-before.json')
assert entry['observed_before_normal_builds'] and entry['profile_environment']=={}
assert sha(ROOT/'啟動現代客戶端.ps1')==sha(RELEASE/'entry-script-before.ps1')==entry['sha256']
runs={};counts={};metadata={}
for name,paths,expected,targets in [('Debug',[DEBUG,LABELS],81,17),('Release',[RELEASE],65,12)]:
    total=0;names=set();runs[name]=[]
    for index,folder in enumerate(paths):
        receipt=read(folder/'receipt.json'); assert receipt['owned_process'] is None and receipt['source_inputs_unchanged']
        assert receipt['profile']==name and receipt['compile_exit_code']==0
        assert receipt['status'] in ['actual_applied_lighting_and_related_targets_passed','actual_lighting_tests_and_unique_entry_normal_builds_passed']
        assert read(folder/'compiled-inputs.json')==pins
        with zipfile.ZipFile(folder/'compiled-source.zip') as z:
            assert set(z.namelist())==set(pins)
            for n,h in pins.items():assert digest(z.read(n))==h,n
        products=read(folder/'products.json');assert not names.intersection(products);names.update(products)
        with zipfile.ZipFile(folder/'test-products.zip') as z:
            assert set(z.namelist())=={n+'.exe' for n in products}
            for n,p in products.items():
                data=z.read(n+'.exe');assert digest(data)==p['sha256'];pe(data)
                log=(folder/n/'stdout.log').read_bytes()
                found=re.findall(rb'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',log)
                assert len(found)==1;values=list(map(int,found[0]));assert values[1:]==[0,0,0,0]
                assert receipt['results'][n]==dict(exit_code=0,passed=values[0],failed=0,ignored=0);total+=values[0]
                for logname in ['stdout.log','stderr.log']:metadata[name+'/'+str(index)+'/'+n+'/'+logname]=folder/n/logname
        runs[name].append({'path':str(folder),'receipt_sha256':sha(folder/'receipt.json'),
            'compiled_source_sha256':sha(folder/'compiled-source.zip'),'test_products_sha256':sha(folder/'test-products.zip')})
        for n in ['receipt.json','products.json','compiled-inputs.json','runner.py']:
            metadata[name+'/'+str(index)+'/'+n]=folder/n
    assert total==expected and len(names)==targets
    counts[name]=dict(passed=total,failed=0,ignored=0,actual_test_executables=targets)
receipt=read(RELEASE/'receipt.json');assert receipt['status']=='actual_lighting_tests_and_unique_entry_normal_builds_passed'
normal=read(RELEASE/'normal-products.json');assert normal==receipt['normal_products'] and set(normal)=={'Debug','Release'}
normal_output_encodings={}
for name,p in normal.items():
    assert p['unique_entry_build_exit_code']==0 and sha(Path(p['path']))==p['sha256'];pe(Path(p['path']).read_bytes())
    folder=RELEASE/('normal-'+name)
    assert Path(p['path']).name=='LiToModern.exe' and Path(p['path']).parent.name==name.lower()
    raw=(folder/'stdout.log').read_bytes();assert len(raw)<131072
    try:
        output=raw.decode('utf-8-sig');normal_output_encodings[name]='utf-8-sig'
    except UnicodeDecodeError:
        output=raw.decode('cp950');assert output.encode('cp950')==raw
        normal_output_encodings[name]='cp950_strict_roundtrip'
    assert str(Path(p['path'])) in output
    assert b'Finished' in (folder/'entry-build.log').read_bytes()
    with zipfile.ZipFile(folder/'product.zip') as z:
        assert z.namelist()==['LiToModern.exe'];assert digest(z.read('LiToModern.exe'))==p['sha256']
    p.update(unique_entry=True,features=[],machine='AMD64',optional_header='PE32+',
        build_arguments=['pwsh.exe','-NoProfile','-File',str(ROOT/'啟動現代客戶端.ps1'),'-BuildOnly','-BuildProfile',name])
    for n in ['stdout.log','stderr.log','entry-build.log']:metadata['normal/'+name+'/'+n]=folder/n
for n in ['entry-script-before.ps1','entry-script-before.json','normal-products.json']:
    metadata['normal/'+n]=RELEASE/n
out=RELEASE/'normal-qualified';out.mkdir(exist_ok=False)
metadata['qualifier.py']=Path(__file__)
for name in ['qualifier-first-observer.py','qualifier-first-observer-failure.json']:
    if (RELEASE/name).exists():metadata['observer-history/'+name]=RELEASE/name
save(out/'source-inputs.json',pins)
normal_q={'status':'qualified_unique_entry_water_source_normal_builds','source_files':999,
 'source_manifest_includes_compile_fixtures':True,'source_manifest_sha256':sha(out/'source-inputs.json'),
 'features':[],'profiles':normal,'entry_script_sha256':entry['sha256'],
 'normal_output_encodings':normal_output_encodings,'normal_GUI_verified':False,'whole_goal_complete':False}
save(out/'qualified.json',normal_q)
with zipfile.ZipFile(out/'raw-metadata.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
    for n,p in metadata.items():z.write(p,n)
with zipfile.ZipFile(out/'raw-metadata.zip') as z:
    assert set(z.namelist())==set(metadata)
    for n,p in metadata.items():assert digest(z.read(n))==sha(p)
scope={'status':'qualified_water_source_producer_Debug81_Release65_and_normal_mains',
 'source_files':999,'source_manifest_sha256':sha(out/'source-inputs.json'),'counts':counts,'runs':runs,
 'normal_mains':{n:p['sha256'] for n,p in normal.items()},'all_archives_readback_verified':True,
 'water_source_producer_scene_integrated':True,'water_extra_effect_complete':False,
 'map303_gate_complete':False,'special_global_filters_complete':False,
 'full_default_regression_verified':False,'full_library_regression_verified':False,
 'full_current_TCP_regression_verified':False,'normal_GUI_verified':False,'actual_new_game_sessions':0,
 'whole_goal_complete':False,'raw_metadata_archive_sha256':sha(out/'raw-metadata.zip'),
 'raw_metadata_files':{n:sha(p) for n,p in metadata.items()},'qualifier_sha256':sha(Path(__file__))}
save(out/'scoped-qualified.json',scope)
assert read(out/'qualified.json')==normal_q and read(out/'scoped-qualified.json')==scope
print(json.dumps({'normal':str(out/'qualified.json'),'scope':str(out/'scoped-qualified.json'),'counts':counts,'mains':scope['normal_mains']}))
