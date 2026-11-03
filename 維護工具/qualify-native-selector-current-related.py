"""Read one completed profile while the other original profile may remain live."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,re,subprocess,tomllib,zipfile

ROOT=Path(__file__).resolve().parents[1]
PROJECT=ROOT/'LiTo381-Modern'
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def archive(path,digest,files):
    assert sha(path)==digest
    with zipfile.ZipFile(path) as z:
        assert len(z.namelist())==len(files) and set(z.namelist())==set(files)
        for name,h in files.items():
            with z.open(name) as stream:assert hashlib.file_digest(stream,'sha256').hexdigest().upper()==h
def amd64(path):
    with Path(path).open('rb') as f:
        h=f.read(64);assert h[:2]==b'MZ';f.seek(int.from_bytes(h[60:64],'little'));h=f.read(26)
        assert h[:4]==b'PE\0\0' and h[4:6]==b'\x64\x86' and h[24:26]==b'\x0b\x02'

ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--profile',required=True,choices=['Debug','Release']);a=ap.parse_args();run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-selector-integration')
c=read(run/'cohort.json');d=run/a.profile;info=c['profiles'][a.profile];assert info['compile_exit_code']==0 and info['version_exit_code']==0
expected_targets=set(c['targets'])|{'LiToModern'};assert len(expected_targets)==29 and set(info['products'])==set(info['tests'])==expected_targets
jobs=[p for p in c['process_history'] if Path(p['output']).is_relative_to(d)];assert len(jobs)==32 and all(v['exit_code']==0 for v in jobs)
app=Path(c['application']);application=read(app/'application.json');assert sha(app/'application.json')==c['application_sha256'] and application['applied'] and application['status']=='applied_selector_integration_pending_current_product_validation'
pins=read(run/'source-inputs.json');inputs=read(run/'compiled-inputs.json');assert len(pins)==c['source_files']==1028 and len(inputs)==c['compiled_input_files']==1029
assert sha(run/'source-inputs.json')==c['source_manifest_sha256'] and pins==read(app/'applied-source-inputs.json') and inputs==read(app/'applied-compiled-inputs.json')
for n,h in inputs.items():assert sha(ROOT/n)==h,n
source=c['compiled_source_archive'];archive(source['path'],source['sha256'],inputs);assert sha(ROOT/'啟動現代客戶端.ps1')==c['entry_sha256']
before=read(app/'base-source-inputs.json');assert len(before)==1024 and set(pins)-set(before)==set(application['added_files']) and {n for n in before if pins[n]!=before[n]}==set(application['before'])
archive(app/'before.zip',application['before_archive_sha256'],application['before']);archive(app/'proposal.zip',application['proposal_archive_sha256'],application['after'])
for n,h in application['after'].items():assert sha(ROOT/n)==h
assert len(application['before'])==27 and len(application['added_files'])==4 and application['existing_test_assertions_preserved'] and application['new_registered_tests']==4
equivalence=read(run/'source-descriptor-equivalence.json');assert equivalence['entries_equal'] and not equivalence['source_bytes_changed'] and equivalence['entries']==1028
assert equivalence['application_descriptor_sha256']==application['source_manifest_sha256']==sha(app/'applied-source-inputs.json') and equivalence['compiled_descriptor_sha256']==c['source_manifest_sha256']
prior=c['old_qualified_inventory_only'][a.profile];assert sha(prior['path'])==prior['sha256'];old=read(prior['path']);assert old['source_manifest_sha256']==application['base_source_manifest_sha256']
messages=[json.loads(l) for l in (d/'build/stdout.log').read_text(encoding='utf-8-sig').splitlines() if l.startswith('{')];assert any(m.get('reason')=='build-finished' and m['success'] for m in messages)
artifacts={m['target']['name']:m for m in messages if m.get('reason')=='compiler-artifact' and m.get('executable') and m['profile']['test']};assert set(artifacts)==expected_targets
names={};totals=dict(passed=0,failed=0,ignored=0)
for name,product in info['products'].items():
 exe=Path(product['path']).resolve(strict=True);assert exe.is_relative_to(PROJECT/'target/current') and sha(exe)==product['sha256'] and Path(artifacts[name]['executable']).resolve()==exe;amd64(exe)
 assert artifacts[name]['features']==[] and artifacts[name]['profile']==product['compiler_profile'] and artifacts[name]['profile']['opt_level']==('z' if a.profile=='Release' else '0')
 text=(d/('test-'+name)/'stdout.log').read_text(encoding='utf-8-sig');rows=re.findall(r'^test (.+?) \.\.\. (ok|FAILED|ignored)(?:,.*)?$',text,re.M);result=dict(rows);expected=dict(all=c['expected_new_tests'],ignored=[]) if name=='native_selector_only' else old['all_test_names']['main_unit' if name=='LiToModern' else name]
 assert len(rows)==len(result) and set(result)==set(expected['all']) and {n for n,s in rows if s=='ignored'}==set(expected['ignored'])
 counts={k:sum(v==value for _,v in rows) for k,value in [('passed','ok'),('failed','FAILED'),('ignored','ignored')]};assert counts['failed']==0
 terminal=re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',text);assert terminal==[('ok',str(counts['passed']),'0',str(counts['ignored']),'0','0')]
 assert all(info['tests'][name][k]==v for k,v in counts.items()) and info['tests'][name]['exit_code']==0 and info['tests'][name]['rows']==result and info['tests'][name]['expected_names']==expected
 job=next(v for v in jobs if Path(v['output'])==d/('test-'+name));assert job['arguments']==[str(exe),'--test-threads=1']
 for k,v in counts.items():totals[k]+=v
 names[name]=result
assert len(names['native_selector_only'])==4 and all(s=='ok' for s in names['native_selector_only'].values()) and totals==info['counts']
for test in ['presentation_sources_survive_scene_retirement_without_retaining_old_input','presentation_root_change_retires_old_model_and_source_leases','presentation_cancel_does_not_recapture_sources_from_a_pending_render','presentation_job_rejects_reused_model_from_other_resource_root','presentation_resume_same_scene_rejects_retired_render_contexts']:assert names['native_world_lighting_scene'][test]=='ok'
products=info['test_products_archive'];archive(products['path'],products['sha256'],{n+'.exe':v['sha256'] for n,v in info['products'].items()})
normal=info['normal'];assert normal['unique_entry_exit_code']==0 and sha(normal['path'])==normal['sha256'] and normal['sha256']!=application['normal_products_before'][a.profile.lower()]['sha256'];amd64(normal['path']);archive(normal['archive']['path'],normal['archive']['sha256'],{'LiToModern.exe':normal['sha256']})
job=next(v for v in jobs if Path(v['output'])==d/'normal-build');assert job['arguments']==['pwsh.exe','-NoProfile','-File',str(ROOT/'啟動現代客戶端.ps1'),'-BuildOnly','-BuildProfile',a.profile]
assert b'LiToModern' in (d/'version/stdout.log').read_bytes() and not (d/'version/stderr.log').read_bytes().strip()
release=tomllib.loads((PROJECT/'Cargo.toml').read_text(encoding='utf-8-sig'))['profile']['release'];assert (release['opt-level'],release['lto'],release['codegen-units'],release['strip'])==('z','thin',16,'symbols')
compilers=[v for v in c['observed_release_compilers'] if Path(v['parent_job_output']).is_relative_to(d)]
if a.profile=='Release':
 build=next(v for v in jobs if Path(v['output'])==d/'build');assert compilers
 for v in compilers:assert v['ParentProcessId']==build['pid'] and datetime.fromisoformat(v['CreationUtc'].replace('Z','+00:00'))>=datetime.fromisoformat(build['creation_utc'])
 assert any(all(s in v['CommandLine'] for s in ['-C opt-level=z','-C lto=thin','-C codegen-units=16','-C strip=symbols','--target x86_64-pc-windows-msvc']) for v in compilers)
reference=ROOT.parent/'lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar';assert sha(reference)==c['reference_jar_sha256']==application['reference_jar_sha256']
ids=[(v['pid'],v['creation_utc']) for v in jobs]+[(v['ProcessId'],v['CreationUtc']) for v in compilers]
command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);$ids=@("+','.join(str(pid) for pid,_ in ids)+");@(Get-CimInstance Win32_Process|Where-Object {$ids -contains $_.ProcessId}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
p=subprocess.run(['pwsh','-NoProfile','-NonInteractive','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert p.returncode==0 and not p.stderr
live=json.loads(p.stdout.decode('utf-8-sig')) if p.stdout.strip() else [];live=[live] if isinstance(live,dict) else live
for row in live:
 for pid,created in ids:
  if row['ProcessId']==pid:assert abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-datetime.fromisoformat(created.replace('Z','+00:00'))).total_seconds())>.001
out=d/'profile-qualified';assert not out.exists();out.mkdir();(out/'qualifier.py').write_bytes(Path(__file__).read_bytes());(out/'cohort-observation.json').write_bytes((run/'cohort.json').read_bytes());(out/'owned-CIM.json').write_bytes(p.stdout)
files={'qualifier.py':out/'qualifier.py','cohort-observation.json':out/'cohort-observation.json','owned-CIM.json':out/'owned-CIM.json','source-descriptor-equivalence.json':run/'source-descriptor-equivalence.json','source-inputs.json':run/'source-inputs.json','compiled-inputs.json':run/'compiled-inputs.json','application.json':app/'application.json'}
for file in d.rglob('*'):
 if file.is_file() and out not in file.parents and file.suffix.lower() not in ['.exe','.zip']:files[a.profile+'/'+file.relative_to(d).as_posix()]=file
raw=out/'raw.zip';hashes={n:sha(p) for n,p in files.items()}
with zipfile.ZipFile(raw,'x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
 for n,p in files.items():z.write(p,n)
archive(raw,sha(raw),hashes)
q=dict(status='independently_qualified_current_normal_selector_related_profile',profile=a.profile,source_files=1028,compiled_input_files=1029,source_manifest_sha256=c['source_manifest_sha256'],application_descriptor_sha256=application['source_manifest_sha256'],descriptor_conversion_only=True,compiled_source_archive_sha256=source['sha256'],actual_test_executables=29,counts=totals,all_test_results=names,new_registered_selector_S39_tests_passed=4,old_registered_test_names_and_ignored_set_preserved=True,normal_sha256=normal['sha256'],normal_product_archive_sha256=normal['archive']['sha256'],unique_entry_build_verified=True,recorded_profile_top_level_identities=len(jobs),recorded_profile_top_level_all_exit_zero=True,recorded_profile_identities_terminal=True,observed_release_compiler_identities=len(compilers),release_optimization_flags_observed=a.profile=='Release',actual_formal_sessions=0,GUI_verified=False,physical_audio_verified=False,full_current_regression_verified=False,whole_goal_complete=False,raw_archive_sha256=sha(raw),raw_files=hashes,observed_utc=datetime.now(timezone.utc).isoformat())
(out/'qualified.json').write_text(json.dumps(q,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps({k:q[k] for k in ['status','profile','counts','normal_sha256']},ensure_ascii=False))
