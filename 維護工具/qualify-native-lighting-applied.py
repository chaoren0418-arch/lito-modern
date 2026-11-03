"""Independent readback of actual lighting tests and unique-entry normal builds."""
from pathlib import Path
import hashlib,json,re,zipfile
ROOT=Path(__file__).resolve().parents[1]
APP=ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def digest(b):return hashlib.sha256(b).hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
def pe(data):
    at=int.from_bytes(data[60:64],'little')
    assert data[:2]==b'MZ' and data[at:at+4]==b'PE\0\0'
    assert data[at+4:at+6]==b'\x64\x86' and data[at+24:at+26]==b'\x0b\x02'
release=Path(read(APP/'actual-tests-current.json')['output']).resolve(strict=True)
assert release.is_relative_to(APP)
receipt=read(release/'receipt.json')
assert receipt['status']=='actual_lighting_tests_and_unique_entry_normal_builds_passed'
assert receipt['profile']=='Release' and receipt['owned_process'] is None
assert receipt['passed']==43 and receipt['failed']==receipt['ignored']==0 and receipt['source_inputs_unchanged']
pins=read(release/'compiled-inputs.json');assert len(pins)==994
for n,h in pins.items():assert sha(ROOT/n)==h,n
folders={'Debug':[APP/'actual-tests-20261016-052203-270622',APP/'actual-tests-20261016-052445-426655'], 'Release':[release]}
runs={};counts={};raw={}
for profile,paths in folders.items():
    runs[profile]=[];total=0;names=set()
    for index,folder in enumerate(paths):
        v=read(folder/'receipt.json');assert v['owned_process'] is None and v['source_inputs_unchanged']
        assert v['status'] in ['actual_applied_lighting_and_related_targets_passed','actual_lighting_tests_and_unique_entry_normal_builds_passed']
        local=read(folder/'compiled-inputs.json')
        assert len(local) in [993,994] and all(pins[n]==h for n,h in local.items())
        assert set(pins)-set(local) <= {'LiTo381-Modern/tests/native_world_lighting_scene.rs'}
        with zipfile.ZipFile(folder/'compiled-source.zip') as z:
            assert set(z.namelist())==set(local)
            for n,h in local.items():assert digest(z.read(n))==h,n
        products=read(folder/'products.json')
        assert not names.intersection(products);names.update(products)
        with zipfile.ZipFile(folder/'test-products.zip') as z:
            assert set(z.namelist())=={n+'.exe' for n in products}
            for n,p in products.items():
                data=z.read(n+'.exe');assert digest(data)==p['sha256'];pe(data)
                log=(folder/n/'stdout.log').read_text(encoding='utf-8-sig')
                m=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',log)
                assert len(m)==1 and tuple(map(int,m[0]))==(v['results'][n]['passed'],0,0,0,0)
                assert v['results'][n]['exit_code']==0;total+=int(m[0][0])
        runs[profile].append({'path':str(folder),'receipt_sha256':sha(folder/'receipt.json'),
            'compiled_source_sha256':sha(folder/'compiled-source.zip'),'test_products_sha256':sha(folder/'test-products.zip')})
        for f in folder.rglob('*'):
            if f.is_file() and f.suffix in ['.json','.log','.py']:
                assert f.resolve().is_relative_to(folder) and not f.is_symlink()
                raw[profile+'/'+str(index)+'/'+f.relative_to(folder).as_posix()]=f
    assert total==43 and len(names)==10
    counts[profile]={'passed':total,'failed':0,'ignored':0}
normal=read(release/'normal-products.json');assert set(normal)=={'Debug','Release'}
for profile,p in normal.items():
    assert p['unique_entry_build_exit_code']==0 and sha(p['path'])==p['sha256'];pe(Path(p['path']).read_bytes())
    with zipfile.ZipFile(release/('normal-'+profile)/'product.zip') as z:
        assert z.namelist()==['LiToModern.exe'];assert digest(z.read('LiToModern.exe'))==p['sha256']
    command=['pwsh.exe','-NoProfile','-File',str(ROOT/'啟動現代客戶端.ps1'),'-BuildOnly','-BuildProfile',profile]
    assert (release/('normal-'+profile)/'stdout.log').is_file()
    p.update(unique_entry=True,features=[],machine='AMD64',optional_header='PE32+',build_arguments=command)
output=APP/'normal-current';output.mkdir(exist_ok=False)
save(output/'source-inputs.json',pins)
manifest=sha(output/'source-inputs.json')
normal_q={'status':'qualified_unique_entry_lighting_normal_builds','source_files':994,
    'source_manifest_includes_compile_fixtures':True,'source_manifest_sha256':manifest,'features':[],
    'profiles':normal,'normal_GUI_verified':False,'whole_goal_complete':False}
save(output/'qualified.json',normal_q)
with zipfile.ZipFile(output/'raw-metadata.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
    for n,f in raw.items():z.write(f,n)
with zipfile.ZipFile(output/'raw-metadata.zip') as z:
    assert set(z.namelist())==set(raw)
    for n,f in raw.items():assert digest(z.read(n))==sha(f)
scope={'status':'qualified_applied_lighting_tests_both_profiles','source_files':994,
    'source_manifest_sha256':manifest,'counts':counts,'test_products_per_profile':10,'runs':runs,
    'all_archives_readback_verified':True,'normal_mains':{n:p['sha256'] for n,p in normal.items()},
    'full_default_regression_verified':False,'full_library_regression_verified':False,
    'full_current_TCP_regression_verified':False,'normal_GUI_verified':False,'whole_goal_complete':False,
    'raw_metadata_archive_sha256':sha(output/'raw-metadata.zip'),'raw_metadata_files':{n:sha(f) for n,f in raw.items()},
    'water_extra_effect_complete':False,'map303_gate_complete':False,'special_global_filters_complete':False,
    'qualifier_sha256':sha(Path(__file__))}
save(output/'scoped-qualified.json',scope)
print(json.dumps({'normal':str(output/'qualified.json'),'scope':str(output/'scoped-qualified.json'),'counts':counts}))
