"""Archive the first component tooling failure and add its missing sha2 extern."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def data(x):return (json.dumps(x,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
run=V/'native-haste43-components/20261020-105819-950936';c=read(run/'cohort.json');w=read(R/'工作佇列.json')
assert c['status']=='failed_preserved' and c['owned_process'] is None and len(c['process_history'])==1
assert c['process_history'][0]['exit_code']==1 and not c['normal_source_tree_modified'] and not c['candidate_applied_to_product']
assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
source=c['base_source_manifest_sha256'];assert source==w['current_product_identity']['source_manifest_sha256']
errors=(run/'Debug/baseline-compile/stderr.log').read_text('utf-8-sig')
assert errors.count('error[E0432]: unresolved import `sha2`')==4 and 'aborting due to 4 previous errors' in errors
rows=[c['pipeline'],*c['process_history']];ids=','.join(str(x['pid']) for x in rows)
cmd="@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
obs=subprocess.run(['pwsh','-NoProfile','-Command',cmd],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert obs.returncode==0 and not obs.stderr
live=json.loads(obs.stdout.decode('utf-8-sig')) if obs.stdout.strip() else [];live=[live] if isinstance(live,dict) else live
assert not any(x['ProcessId']==y['pid'] and abs((datetime.fromisoformat(x['created'].replace('Z','+00:00'))-datetime.fromisoformat(y['creation_utc'])).total_seconds())<.001 for x in live for y in rows)
prep_path=Path(c['preparation']);prep=read(prep_path);assert sha(prep_path)==c['preparation_sha256'] and 'sha2' not in prep['required_extern_crates']
normal=Path(w['current_native_installed_resource_profile_normal_qualification']['run']);assert all(sha(R/n)==h for n,h in read(normal/'compiled-inputs.json').items())
runner=R/'維護工具/run-current-native-haste43-components.py';reader=R/'維護工具/checkpoint-current-native-haste43-components.py'
assert sha(run/'runner.py')==sha(runner)
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out=run/('dependency-repair-'+stamp);out.mkdir()
files={x.relative_to(run).as_posix():x for x in run.rglob('*') if x.is_file() and out not in x.parents}
files.update({'preparation-original.json':prep_path,'reader-original.py':reader,'repair.py':Path(__file__)})
hashes={n:sha(x) for n,x in files.items()}
with zipfile.ZipFile(out/'original.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n,x in files.items():z.write(x,n)
with zipfile.ZipFile(out/'original.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in hashes.items())
new_dir=prep_path.parent.parent/('component-preparation-'+stamp);new_dir.mkdir();new_prep=dict(prep)
new_prep.update(required_extern_crates=[*prep['required_extern_crates'],'sha2'],supersedes_preparation=str(prep_path),supersedes_sha256=sha(prep_path),repair='Only add sha2 to existing exact broad-artifact extern resolution; probe/test/fixture bytes unchanged.')
new_path=new_dir/'preparation.json';new_path.write_bytes(data(new_prep))
old_runner=runner.read_text('utf-8-sig');old_relative=prep_path.relative_to(R).as_posix();new_relative=new_path.relative_to(R).as_posix()
assert old_runner.count(old_relative)==1
new_runner=old_runner.replace(old_relative,new_relative);runner.write_text(new_runner,encoding='utf-8',newline='\n')
assert all(sha(p['path'])==p['sha256'] for p in new_prep['probe_sources'].values())
record=dict(status='component_tooling_sha2_dependency_failure_preserved_and_repaired',source_manifest_sha256=source,original_session=26263,terminal_tool_chunk='11b8eb',observed_exit_code=1,failed_active_checkpoint_chunk='5f8a18',failed_active_checkpoint_reason='Original pipeline already terminal; no active checkpoint was written.',original_identities_terminal=True,recorded_identities=len(rows),rust_tests_executed=0,normal_product_source_modified=False,candidate_applied=False,GUI_verified=False,whole_goal_complete=False,original_archive=str(out/'original.zip'),original_archive_sha256=sha(out/'original.zip'),raw_files=hashes,old_preparation=str(prep_path),old_preparation_sha256=sha(prep_path),new_preparation=str(new_path),new_preparation_sha256=sha(new_path),runner_sha256=sha(runner))
(out/'evidence.json').write_bytes(data(record));(out/'runner-after.py').write_bytes(runner.read_bytes());(out/'owned-CIM.json').write_bytes(obs.stdout)
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in before:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
work='兩版#43／54效果資源各2項已核對；互斥資源85／488實跑，餘403。加速術元件原26263因工具漏sha2 extern編譯1，尚未執行Rust測試；原失敗與兩身分終止已封存。僅修正隔離工具依賴，新批次先原版3個預期反例再候選20項，產品未套用／正式施放0。Debug普通房間期限失敗仍待新驗，保留其餘資格，不交人工。'
staged={}
for n in before:
    path=R/n
    if path.suffix=='.json':
        j=read(path);j['current_native_haste43_component_tooling_failure']=record;j.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
        if n.endswith('next-validation-action.json'):j['next_action']=work
        staged[n]=data(j)
    else:
        first,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert sep;staged[n]=('2026-10-20：'+work+'\n\n'+rest).encode('utf-8')
assert all(sha(R/n)==h for n,h in before.items())
for n,payload in staged.items():(R/n).write_bytes(payload)
(out/'readback.json').write_bytes(data(dict(before_files=before,after_files={n:sha(R/n) for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
print(json.dumps(dict(evidence=str(out/'evidence.json'),new_preparation=str(new_path),normal_product_source_modified=False),ensure_ascii=False))
