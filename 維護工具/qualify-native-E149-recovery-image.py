"""Qualify actual isolated linking, not runtime behavior; release own freeze."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,hashlib,json,struct,subprocess,zipfile
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,owned,encoded,validate_plan,check_pins
V=R/'驗證紀錄/current';T=R/'維護工具'
def sha(path):
 with Path(path).open('rb') as file:return hashlib.file_digest(file,'sha256').hexdigest().upper()
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--terminal-chunk',required=True);ap.add_argument('--original-session',type=int,required=True);a=ap.parse_args()
run=owned(a.run.resolve(strict=True),V/'native-E149-recovery-linked-build');c=read(run/'cohort.json');p=validate_plan(c['plan'])
assert c['status']=='linked_compilers_exited' and c['owned_process'] is None and c['source_manifest_sha256']==SOURCE
assert c['formal_target_sha256']==FORMAL and c['effective_overlay_files']==p['effective_overlay_files']
assert len(c['compiler_runs'])==2 and [r['kind'] for r in c['compiler_runs']]==['library','formal_test']
assert all(r['exit_code']==0 and r['wait_confirmed'] and not r.get('exact_original_handle_cleanup',False) for r in c['compiler_runs'])
assert c['tests_executed']==c['game_or_fixture_executions']==c['desktop_inputs']==0
assert not c['outputs_executed'] and not c['SQL_executed'] and not c['database_started'] and not c['candidate_applied']
check_pins(c['input_pins']);assert all(sha(n)==h for n,h in c['external_toolchain_dependency_pins'].items())
assert all(sha(n)==h for n,h in c['normal_product_pins'].items()) and all(sha(n)==h for n,h in c['output_files'].items())
assert sha(c['plan'])==c['plan_sha256'] and sha(run/'builder.py')==c['input_pins'][str(T/'build-native-E149-recovery-image.py')]
qpath=Path(c['original_metadata_qualification']);q=read(qpath);assert sha(qpath)==c['original_metadata_qualification_sha256']
original=Path(q['run']);old=read(original/'cohort.json')
assert q['source_manifest_sha256']==SOURCE and q['generated_source_paths_only_verified']
assert len(read(old['module_map']))==q['full_module_declarations']==362 and sha(old['module_map'])==old['module_map_sha256']
assert all(sha(n)==h for n,h in old['input_pins'].items())
profile=c['profile'];assert profile==p['profile'] and profile in ['Debug','Release']
for row,label in zip(c['compiler_runs'],['library-compile','teleport-recovery-formal-code-compile']):
 prior=[r for r in old['process_history'] if Path(r['output'])==original/profile/label];assert len(prior)==1
 expected=list(prior[0]['command']);expected[expected.index('--emit=metadata')]='--emit=link'
 expected[expected.index('-o')+1]=row['output']
 if row['kind']=='formal_test':
  expected[expected.index(str(original/'generated/LiTo381-Modern/tests/native_teleport_failure.rs'))]=p['effective_overlay_paths']['LiTo381-Modern/tests/native_teleport_failure.rs']
  expected[expected.index('lito_modern='+str(original/profile/'library.rmeta'))]='lito_modern='+c['library']
 for flag in c['native_link_search_paths']:expected+=['-L',flag]
 assert expected==row['command'] and '--emit=link' in row['command'] and Path(row['command'][0]).name=='rustc.exe'
 assert sha(row['command'][0])==row['compiler_sha256'] and sha(row['output'])==row['output_sha256']
 assert Path(row['output']).stat().st_size==row['output_bytes']>0
 check_pins(row['logs'])
 if profile=='Release':
  for name in ['opt-level=z','lto=thin','codegen-units=16','strip=symbols']:assert name in row['command']
library=owned(c['library'],P/'target/current/verification/native-E149-recovery-linked-images')
with library.open('rb') as file:assert file.read(8)==b'!<arch>\n'
product=owned(c['product'],library.parent)
with product.open('rb') as file:
 header=file.read(64);assert header[:2]==b'MZ';file.seek(struct.unpack_from('<I',header,0x3c)[0]);pe=file.read(80)
assert pe[:4]==b'PE\0\0' and struct.unpack_from('<H',pe,4)[0]==0x8664
assert struct.unpack_from('<H',pe,24)[0]==0x20b and struct.unpack_from('<I',pe,40)[0]>0
characteristics=struct.unpack_from('<H',pe,22)[0];assert characteristics&2 and not characteristics&0x2000
assert sha(product)==c['product_sha256'] and sha(library)==c['library_sha256']
inputs=read(c['base_compiled_inputs']);assert len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items())
assert all(c['input_pins'][p['effective_overlay_paths'][n]]==h for n,h in p['effective_overlay_files'].items())
w=read(R/'工作佇列.json');assert w['current_user_requested_goal']['test_execution_deferred_by_user']
active=w['active_workloads'];assert not active or (set(active)=={'native_E149_recovery_link'} and active['native_E149_recovery_link']['run']==str(run))
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
identities=[c['pipeline'],*c['compiler_runs']];ids=','.join(str(x['pid']) for x in identities)
query="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
observed=subprocess.check_output(['pwsh','-NoProfile','-Command',query]);actual=json.loads(observed.decode('utf-8-sig')) if observed.strip() else []
actual=[actual] if isinstance(actual,dict) else actual
for row in identities:assert not any(x['ProcessId']==row['pid'] and abs((datetime.fromisoformat(x['created'].replace('Z','+00:00'))-datetime.fromisoformat(row['creation_utc'])).total_seconds())<.001 for x in actual)
out=run/('qualified-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
(out/'owned-CIM.json').write_bytes(observed);(out/'reader.py').write_bytes(Path(__file__).read_bytes())
# Compiler inputs stay bound to their real sources/artifacts. Mutable tooling
# and documentation were development guards, not Rust compiler inputs: retain
# their actual observed versions in this immutable qualification directory so
# later orchestration work need not relink an unchanged binary.
stable_pins={};control_snapshots={};snapshot_pins={}
for name,h in c['input_pins'].items():
 path=Path(name)
 if path.parent==T:
  snapshot=out/'control-inputs'/path.name;snapshot.parent.mkdir(exist_ok=True)
  snapshot.write_bytes(path.read_bytes());assert sha(snapshot)==h
  control_snapshots[name]=dict(snapshot=str(snapshot),sha256=h);snapshot_pins[str(snapshot)]=h
 else:stable_pins[name]=h
qualified_cohort=dict(c,input_pins=stable_pins|snapshot_pins,mutable_control_input_snapshots=control_snapshots,
 original_observed_cohort=str(run/'cohort.json'),original_observed_cohort_sha256=sha(run/'cohort.json'),
 receipt_kind='immutable_snapshot_of_actual_link_commands_and_inputs')
(out/'qualified-cohort.json').write_bytes(encoded(qualified_cohort))
record=dict(status='qualified_linked_isolated_E149_recovery_controller',profile=profile,source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,
 effective_overlay_files=p['effective_overlay_files'],effective_overlay_paths=p['effective_overlay_paths'],
 compile_cohort=str(out/'qualified-cohort.json'),compile_cohort_sha256=sha(out/'qualified-cohort.json'),plan=c['plan'],plan_sha256=c['plan_sha256'],
 original_observed_cohort=str(run/'cohort.json'),original_observed_cohort_sha256=sha(run/'cohort.json'),
 compiler_control_versions_archived=True,mutable_control_input_snapshots=control_snapshots,
 product=str(product),product_sha256=sha(product),product_bytes=product.stat().st_size,library=str(library),library_sha256=sha(library),library_bytes=library.stat().st_size,
 compiler_exit_codes=[0,0],full_source_and_overlay_pins_verified=True,recorded_original_compilers_terminal=True,original_identity_count=3,
 original_session_id=a.original_session,observed_terminal_tool_chunk=a.terminal_chunk,observed_actual_exit_code=0,
 reused_full_module_graph_qualification=str(qpath),reused_full_module_graph_qualification_sha256=sha(qpath),module_declarations=362,normal_compiled_inputs=1049,
 direct_dependencies=len(old['profiles'][profile]['extern']),native_link_search_paths=c['native_link_search_paths'],
 x64_PE32_plus_executable_verified=True,rlib_archive_verified=True,normal_source_unchanged=True,normal_PEs_unchanged=True,
 tests_executed=0,outputs_executed=False,SQL_executed=False,database_started=False,desktop_inputs=0,
 candidate_applied=False,functional_qualified=False,GUI_verified=False,full_Main_process_qualified=False,full_native_owner_qualified=False,
 limitations=['Actual linking only; neither PE, library tests nor fixtures were executed',
  'Library uses the identical previously reviewed complete generated module graph; the formal test source is the newer four-file integration candidate',
  'Selected link output does not qualify normal entry, full library, all opt-in targets, runtime, GUI, portability or complete G06',
  'Compiler/stdlib/transitive cached dependencies are hash-pinned in place; this is not a clean-host SDK qualification'],
 test_execution_deferred_by_user=True,whole_goal_complete=False,recorded_utc=datetime.now(timezone.utc).isoformat())
prior_image=w.get('current_E149_recovery_linked_image_qualifications',{}).get(profile)
if prior_image:record.update(supersedes_image_qualification=prior_image['qualification'],supersedes_image_qualification_sha256=prior_image['qualification_sha256'],
 supersedes_reason='Archive actual control/tool versions separately from compiler inputs; same original compilers and binaries, no relink/re-execution')
failure=V/'native-E149-recovery-linked-build-preparation-failures/20261023-211539-715380'
assert read(failure/'observed-format.json')['actual_type']=='int'
record.update(preparation_failure=str(failure/'failure.json'),preparation_failure_format_correction=str(failure/'observed-format.json'),
 preparation_failure_original_tool_chunk='dcff45',preparation_failure_compiler_started=False)
if (run/'checkpoint-transition-rejection-8d652d/evidence.json').exists():
 record['active_checkpoint_rejections']=[str(run/'checkpoint-transition-rejection-8d652d/evidence.json'),str(run/'checkpoint-after-completion-rejection-96b8ff.json')]
(out/'qualified.json').write_bytes(encoded(record))
raw={'qualified.json':out/'qualified.json','reader.py':Path(__file__),'builder.py':run/'builder.py','cohort.json':run/'cohort.json',
 'qualified-cohort.json':out/'qualified-cohort.json',
 'plan.json':Path(c['plan']),'module-map.json':Path(old['module_map']),'base-compiled-inputs.json':Path(c['base_compiled_inputs']),
 'owned-CIM.json':out/'owned-CIM.json','preparation-failure.json':failure/'failure.json','preparation-format-correction.json':failure/'observed-format.json',
 'failed-preparer.py':failure/'builder.py','report-updater-before.py':T/'update-master-report-text-index-order.py'}
for row in c['compiler_runs']:
 for name in row['logs']:raw[row['kind']+'/'+Path(name).name]=Path(name)
for name,path in p['effective_overlay_paths'].items():raw['effective-overlay/'+name]=Path(path)
for entry in control_snapshots.values():raw['control-inputs/'+Path(entry['snapshot']).name]=Path(entry['snapshot'])
for n,name in enumerate(record.get('active_checkpoint_rejections',[])):raw['checkpoint-rejection-'+str(n)+'.json']=Path(name)
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
 for n,path in raw.items():z.write(path,n)
with zipfile.ZipFile(out/'raw.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==sha(path) for n,path in raw.items())
record.update(qualification=str(out/'qualified.json'),qualification_sha256=sha(out/'qualified.json'),raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'),previous_goal_turn_evidence=w['last_goal_turn_evidence'])
(out/'evidence.json').write_bytes(encoded(record))
both=set(w.get('current_E149_recovery_linked_image_qualifications',{}))|{profile}
next_work='接續owned case orchestrator的程序身分、期限及失敗回收開發' if both=={'Debug','Release'} else '接續建立／獨立核對Release隔離映像，再開發owned case orchestrator'
work='E149隔離'+profile+'已真實連結候選rlib與正式案例x64 PE；原session '+str(a.original_session)+'／'+a.terminal_chunk+' actual exit0，兩命令各exit0、362模組完整graph、1049正常輸入、14檔有效覆蓋、13直接依賴／原生搜尋路徑、artifact及3個記錄身分終止皆核對。輸出未執行，正常兩EXE未變、候選未套用，不能當功能／library／GUI通過。初次數量欄誤讀準備失敗與active觀察跨階段退件保留；未因觀察逾時重啟。'+next_work+'；使用者暫緩DB／SQL／factory／功能／GUI測試，整體未完成。'
staged={}
for name in before:
 path=R/name
 if path.suffix=='.json':
  j=read(path);j.setdefault('current_E149_recovery_linked_image_qualifications',{})[profile]=record
  j['active_workloads'].pop('native_E149_recovery_link',None);j['source_and_corresponding_QA_frozen']=False
  j['current_E149_recovery_link_active']=None
  j['current_E149_recovery_case_reader_development'].update(linked_image_builder_pending=False,linked_profile_qualifications=sorted(both),
   both_profiles_linked=both=={'Debug','Release'},isolated_linked_image_qualified=both=={'Debug','Release'},
   linked_image_status_as_of=datetime.now(timezone.utc).isoformat(),runtime_producer_implementation_pending=True)
  j.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
  j['current_focus']=dict(item='G06_3d_E149_isolated_linked_images_development',work=work,applies_to_normal_EXE=False,goal_complete=False)
  if name.endswith('next-validation-action.json'):j['next_action']=work
  staged[name]=encoded(j)
 else:
  _,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert sep
  staged[name]=(datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest).encode('utf-8')
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
 for name in before:z.write(R/name,name)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(name)).hexdigest().upper()==h for name,h in before.items())
assert all(sha(R/n)==h for n,h in before.items()) and all(sha(n)==h for n,h in c['normal_product_pins'].items())
for name,blob in staged.items():(R/name).write_bytes(blob)
assert all((R/name).read_bytes()==blob for name,blob in staged.items())
(out/'readback.json').write_bytes(encoded(dict(before_files=before,after_files={n:sha(R/n) for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
print(json.dumps(dict(evidence=str(out/'evidence.json'),qualification=record['qualification'],profile=profile,compiler_exit_codes=[0,0],x64_PE=True,outputs_executed=False,tests_executed=0,both_profiles_linked=both=={'Debug','Release'}),ensure_ascii=False))
