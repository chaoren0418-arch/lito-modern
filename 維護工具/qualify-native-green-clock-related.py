"""Independent readback of scene tests and both unique-entry normal products."""
from pathlib import Path
from datetime import datetime, timezone
import argparse, hashlib, json, re, tomllib, zipfile

ROOT=Path(__file__).resolve().parents[1]
PROJECT=ROOT/'LiTo381-Modern'
ap=argparse.ArgumentParser();ap.add_argument('directory',type=Path);args=ap.parse_args()
out=args.directory.resolve(strict=True)
assert out.is_relative_to(ROOT/'驗證紀錄/current/native-actor-green-clock')
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def entry(z,n):
    with z.open(n) as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def archive(p,record):
    assert sha(p)==record['sha256']
    with zipfile.ZipFile(p) as z:
        assert set(z.namelist())==set(record['files']) and len(z.namelist())==len(record['files'])
        for n,h in record['files'].items():assert entry(z,n)==h,n
def pe(p):
    with Path(p).open('rb') as f:
        h=f.read(64);assert h[:2]==b'MZ';f.seek(int.from_bytes(h[60:64],'little'));v=f.read(26)
    assert v[:4]==b'PE\0\0' and v[4:6]==b'\x64\x86' and v[24:26]==b'\x0b\x02'
r=read(out/'receipt.json')
assert r['status']=='executed_green_clock_related_scope_and_unique_entry_normal_builds'
assert r['owned_process'] is None and r['sources_unchanged'] and r['profile_environment']=={} and r['features']==[]
assert r['process_history'] and all(p['exit_code']==0 for p in r['process_history'])
pins=read(out/'source-inputs.json')
assert len(pins)==r['source_files']==1021 and sha(out/'source-inputs.json')==r['source_manifest_sha256']=='180741A87501C13178B8521318365810B458324663F9D419758AF983099415E9'
for n,h in pins.items():assert sha(ROOT/n)==h,n
actual={str(p.relative_to(ROOT)).replace('\\','/') for folder in ['src','tests'] for p in (PROJECT/folder).rglob('*') if p.is_file()}
assert actual=={n for n in pins if n.startswith(('LiTo381-Modern/src/','LiTo381-Modern/tests/'))}
assert r['source_archive']['files']==pins;archive(out/'compiled-source.zip',r['source_archive'])
raw=read(out/'archive-readback.json');archive(out/'raw.zip',raw)
for n,h in raw['files'].items():assert sha(out/n)==h,n
assert sha(ROOT/'啟動現代客戶端.ps1')==sha(out/'entry-script-before.ps1')==r['entry_script_sha256']
release=tomllib.loads((PROJECT/'Cargo.toml').read_text(encoding='utf-8-sig'))['profile']['release']
assert release['opt-level']=='z' and release['lto']=='thin' and release['codegen-units']==16 and release['strip']=='symbols'
assert set(r['profiles'])==set(r['normal_products'])=={'Debug','Release'}
assert r['suites']==['native_world_lighting_scene','native_world','native_motion','native_world_layers','native_terrain_residence','native_light_lifecycle']
assert r['library_filters']=={'lib-world-raster':'world_raster::tests::','lib-world-scene':'world_scene::tests::'}
counts={};normal={};encodings={};original_scopes={}
for profile in ['Debug','Release']:
    prior=r['previous_scope_qualifications'][profile];q=read(Path(prior['path']))
    assert sha(prior['path'])==prior['sha256'] and q['profile']==profile and q['source_manifest_sha256']==r['source_manifest_sha256']
    assert q['status']=='independently_qualified_source1021_green_shared_clock_scope' and (q['passed'],q['failed'],q['ignored'])==(36,0,0)
    original_scopes[profile]=prior
    d=out/profile;p=r['profiles'][profile]
    assert p['compile_exit_code']==0
    products=read(d/'products.json');assert set(products)==set(r['suites'])|{'lito_modern'}
    assert set(p['test_archive']['files'])=={n+'.exe' for n in products}
    archive(d/'test-products.zip',p['test_archive'])
    build=[json.loads(x) for x in (d/'build/stdout.log').read_text(encoding='utf-8-sig').splitlines() if x.startswith('{')]
    assert any(m.get('reason')=='build-finished' and m['success'] for m in build)
    artifacts={m['target']['name']:m for m in build if m.get('reason')=='compiler-artifact' and m.get('executable') and m['profile']['test']}
    assert set(artifacts)==set(products)
    for name,item in products.items():
        path=Path(item['path']).resolve(strict=True);assert path.is_relative_to(PROJECT/'target/current')
        assert sha(path)==item['sha256']==p['test_archive']['files'][name+'.exe']
        assert artifacts[name]['features']==[] and Path(artifacts[name]['executable']).resolve()==path;pe(path)
    assert set(p['results'])==set(r['suites'])|set(r['library_filters'])
    totals=dict(passed=0,failed=0,ignored=0)
    for label,v in p['results'].items():
        assert v['filters']==([r['library_filters'][label]] if label in r['library_filters'] else [])
        assert v['product_sha256']==products[v['product']]['sha256'] and v['exit_code']==0
        inv=re.findall(r'^(.+): test$',(d/(label+'-inventory')/'stdout.log').read_text(encoding='utf-8-sig'),re.M)
        assert inv==v['inventory'] and len(inv)==len(set(inv))
        log=(d/(label+'-runtime')/'stdout.log').read_text(encoding='utf-8-sig')
        individual=re.findall(r'^test (.+?) \.\.\. (ok|FAILED|ignored)(?:,.*)?$',log,re.M)
        assert len(individual)==len(inv) and {n for n,_ in individual}==set(inv)
        found=re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',log)
        assert len(found)==1 and found[0][0]=='ok'
        passed,failed,ignored,measured,filtered=map(int,found[0][1:])
        assert failed==measured==0 and (v['filters'] or filtered==0)
        assert (passed,failed,ignored)==(sum(s=='ok' for _,s in individual),sum(s=='FAILED' for _,s in individual),sum(s=='ignored' for _,s in individual))
        for k,value in [('passed',passed),('failed',failed),('ignored',ignored)]:assert v[k]==value;totals[k]+=value
    assert all(p[k]==v for k,v in totals.items())
    assert p['actual_test_executables']==len(products)==7 and p['runtime_groups']==len(p['results'])==8
    counts[profile]=totals|dict(actual_test_executables=7,runtime_groups=8)
    n=r['normal_products'][profile];path=Path(n['path']).resolve(strict=True)
    assert path==PROJECT/'target/current/x86_64-pc-windows-msvc'/profile.lower()/'LiToModern.exe'
    assert n['unique_entry_build_exit_code']==0 and sha(path)==n['sha256'] and n['features']==[];pe(path)
    assert n['build_arguments']==['pwsh.exe','-NoProfile','-File',str(ROOT/'啟動現代客戶端.ps1'),'-BuildOnly','-BuildProfile',profile]
    folder=out/('normal-'+profile);archive(folder/'product.zip',n['archive'])
    assert n['archive']['files']=={'LiToModern.exe':n['sha256']}
    raw_output=(folder/'stdout.log').read_bytes()
    try:output=raw_output.decode('utf-8-sig');encodings[profile]='utf-8-sig'
    except UnicodeDecodeError:
        output=raw_output.decode('cp950');assert output.encode('cp950')==raw_output;encodings[profile]='cp950_strict_roundtrip'
    assert str(path) in output and b'Finished' in (folder/'entry-build.log').read_bytes()
    normal[profile]=n
qualification=dict(status='independently_qualified_green_clock_related_scope_and_unique_entry_normal_builds',source_files=1021,
    source_manifest_sha256=r['source_manifest_sha256'],features=[],profiles=normal,related_scope_counts=counts,
    preceding_green_scope_qualifications=original_scopes,normal_output_encodings=encodings,
    entry_script_sha256=r['entry_script_sha256'],compiled_source_sha256=r['source_archive']['sha256'],raw_archive_sha256=raw['sha256'],
    full_default_regression_verified=False,full_library_regression_verified=False,full_current_TCP_regression_verified=False,
    current_GUI_qualified=False,whole_goal_complete=False,qualified_utc=datetime.now(timezone.utc).isoformat(),qualifier_sha256=sha(Path(__file__)))
dest=out/'qualified.json';assert not dest.exists()
dest.write_text(json.dumps(qualification,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
(out/'independent-qualifier.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(qualification=str(dest),counts=counts,normal_mains={p:v['sha256'] for p,v in normal.items()}),ensure_ascii=False))
