"""Qualify compiled read-only paths and static resource closure, not a case."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,ast,hashlib,json,re,subprocess,zipfile
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
def sha(path):
 with Path(path).open('rb') as file:return hashlib.file_digest(file,'sha256').hexdigest().upper()
data=lambda v:(json.dumps(v,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--terminal-chunk',required=True);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native-E149-recovery-bootstrap-compile')
cohort=read(run/'cohort.json');cp=Path(cohort['candidate']);c=read(cp)
assert sha(cp)==cohort['candidate_sha256'] and cohort['status']=='integrated_fixture_compilation_only_complete'
assert len(cohort['process_history'])==1 and cohort['process_history'][0]['name']=='Java' and cohort['process_history'][0]['exit_code']==0
assert cohort['metadata']=={} and len(cohort['classes'])==6
assert all(sha(p)==h for p,h in cohort['classes'].items()) and all(sha(p)==h for p,h in cohort['input_pins'].items())
assert all(sha(p)==h for p,h in cohort['process_history'][0]['logs'].items())
assert '-proc:none' in cohort['process_history'][0]['command']
assert cohort['tests_executed']==cohort['classes_instantiated']==cohort['desktop_inputs']==0 and not cohort['SQL_executed']
assert c['Runtime_files_changed']==0 and c['all_Rust_inputs_byte_identical']
old_path=Path(c['predecessor_candidate']);old=read(old_path);assert sha(old_path)==c['predecessor_candidate_sha256']
assert all(sha(cp.parent/n)==h for n,h in c['candidate_files'].items())
changed={n for n,h in c['candidate_files'].items() if old['candidate_files'][n]!=h};assert changed==set(c['changed_files'])
assert len(changed)==2 and all(n.endswith('.java') for n in changed)
inspection_path=next(run.glob('nativefields-bytecode-*/inspection.json'));inspection=read(inspection_path)
assert sha(run/'cohort.json')==inspection['compile_cohort_sha256'] and all(sha(p)==h for p,h in inspection['output_files'].items())
directory=inspection_path.parent
observer=(directory/'NativeRecoveryReadonlySnapshot.stdout.txt').read_text('utf-8-sig');harness=(directory/'NativeTeleportFailureHarness.stdout.txt').read_text('utf-8-sig')
assert not re.search(r'\bputfield\b',observer)
static=re.findall(r'putstatic\s+#[0-9]+\s+// Field ([^:]+)',observer);assert set(static)=={'observedIds','observedFields'}
assert not re.search(r'java/lang/reflect/Field\.(?:set:|setInt:|setLong:|setBoolean:)',observer)
assert 'java/lang/reflect/Method.invoke:' not in observer and 'java/lang/reflect/Proxy' not in observer
assert 'RuntimeInventory.items:' not in observer
for word in ['java/util/EnumMap.clone:','java/sql/Timestamp.clone:','java/util/concurrent/atomic/AtomicLong.get:',
 'java/util/concurrent/atomic/AtomicInteger.get:','java/lang/reflect/Field.get:','java/lang/Class.getDeclaredFields:']:
 assert word in observer,word
calls=lambda lines:{m.group(1) for line in lines if (m:=re.search(r'// (?:InterfaceMethod|Method) (.+)',line))}
def original_calls(lines):
 return {name for name in calls(lines) if name.split(':',1)[0].startswith('local/lito381/server/') and 'NativePeerHarness$Session.owner:' not in name}
w=read(R/'工作佇列.json');assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
assert w['current_user_requested_goal']['test_execution_deferred_by_user']
original_review=read(w['current_E149_recovery_development']['readonly_observer_review'])
assert original_calls(observer.splitlines())==original_calls(original_review['external_method_calls']) and len(original_calls(observer.splitlines()))==18
def body(name,needle):
 text=(directory/(name+'.stdout.txt')).read_text('utf-8-sig');start=text.index(needle);next_method=text.find('\n  ',start+len(needle))
 return text[start:next_method if next_method!=-1 else len(text)]
for name,needle,opcode in [('AtomicLong','public final long get();','lreturn'),('AtomicInteger','public final int get();','ireturn')]:
 text=(directory/(name+'.stdout.txt')).read_text('utf-8-sig');part=text[text.index(needle):];part=part[:part.index('\n  public ',1)]
 instructions=re.findall(r'(?m)^\s*\d+:\s+(\w+)',part);assert instructions==['aload_0','getfield',opcode]
date=(directory/'Date.stdout.txt').read_text('utf-8-sig');date=date[date.index('public java.lang.Object clone();'):];date=date[:date.index('\n  public ',1)]
assert 'java/lang/Object.clone:' in date and 'BaseCalendar$Date.clone:' in date and 'astore_1' in date
enum=(directory/'EnumMap.stdout.txt').read_text('utf-8-sig');enum=enum[enum.index('public java.util.EnumMap<K, V> clone();'):];enum=enum[:enum.index('\n  private ',1)]
assert 'java/util/AbstractMap.clone:' in enum and 'Field vals:' in enum and 'Field entrySet:' in enum
assert 'SharedClientResources.initialize:' in harness
source=(cp.parent/'LiTo381-Modern/tests/oracle/NativeTeleportFailureHarness.java').read_text('utf-8-sig')
assert source.index('  initializeSharedProfile();')<source.index('new ServerSocket(')<source.index('ServerConfig config=ServerConfig.load(')
assert source.count('\\"shared_client_resources_initialized\\":true')==2
assert 'G06_E149_native_item_fields_v1' in source
fragment=Path(c['fragment']);assert sha(fragment)==c['fragment_sha256']
item_fields=set(re.findall(r'field\(actor,"([^"]+)"\)',fragment.read_text('utf-8')))
declarations=V/'native-formal-reference-refresh/20261023-192435-953238/G06-closure-20261023-193307-103834/NativeDeWorldKinds$Item.stdout.txt'
decl=declarations.read_text('utf-8-sig');assert all(re.search(r'\s'+re.escape(name)+r';',decl) for name in item_fields)
parser=T/'native_E149_item_attributes.py';tree=ast.parse(parser.read_text('utf-8'));compile(tree,str(parser),'exec')
assert not any(isinstance(n,(ast.Import,ast.ImportFrom)) for n in ast.walk(tree))
assigned={target.id:node.value for node in tree.body if isinstance(node,ast.Assign) for target in node.targets if isinstance(target,ast.Name)}
assert len(ast.literal_eval(assigned['PERMANENT_FIELDS']))==8 and len(ast.literal_eval(assigned['DIAGNOSTIC_FIELDS']))==14
resource=read(c['resource_closure']);assert resource['rows']==60303 and sha(resource['catalog'])==resource['catalog_sha256']
assert not resource['original_initializer_invoked'] and not resource['SQL_or_network_calls_in_SharedClientResources']
assert sha(w['current_formal_reference_identity']['formal'])==c['formal_target_sha256']
inputs=read(cohort['base_compiled_inputs']);assert len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items())
assert all(sha(p)==h for p,h in cohort['normal_product_pins'].items())
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
failure=V/'native-E149-recovery-bootstrap-compile/20261023-221036-476525';failed=read(failure/'cohort.json')
assert failed['status']=='compile_failure_preserved' and failed['process_history'][0]['exit_code']==1
assert 'is a subtype of pattern type Enum' in (failure/'Java/stderr.log').read_text('utf-8-sig')
identities=[cohort['pipeline'],*cohort['process_history'],failed['pipeline'],*failed['process_history']]
ids=','.join(str(row['pid']) for row in identities)
query="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
observed=subprocess.check_output(['pwsh','-NoProfile','-Command',query]);actual=json.loads(observed.decode('utf-8-sig')) if observed.strip() else [];actual=[actual] if isinstance(actual,dict) else actual
for row in identities:assert not any(x['ProcessId']==row['pid'] and abs((datetime.fromisoformat(x['created'].replace('Z','+00:00'))-datetime.fromisoformat(row['creation_utc'])).total_seconds())<.001 for x in actual)
out=V/'native-E149-item-observer-development'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
(out/'owned-CIM.json').write_bytes(observed)
record=dict(status='extended_native_item_observer_and_original_shared_init_compilation_only',candidate=str(cp),candidate_sha256=sha(cp),
 compile_run=str(run),compile_cohort_sha256=sha(run/'cohort.json'),observed_original_tool_chunk=a.terminal_chunk,observed_actual_exit_code=0,
 Java_compiler_exit_code=0,Java_classes=cohort['classes'],Rust_compilers_invoked=0,Rust_files_changed=0,
 formal_target_sha256=c['formal_target_sha256'],source_manifest_sha256=c['base_source_manifest_sha256'],
 bytecode_inspection=str(inspection_path),bytecode_inspection_sha256=sha(inspection_path),resource_closure=c['resource_closure'],resource_closure_sha256=sha(c['resource_closure']),
 common_initializer_call_explicitly_retained=True,common_initializer_before_sockets_and_config=True,embedded_duration_rows=60303,
 original_game_method_call_set_unchanged_18_record_accessors=True,observer_game_field_writes=False,gameplay_item_view_refresh=False,
 observer_owned_static_caches=['observedIds','observedFields'],timestamp_and_ability_clones_reviewed=True,atomic_getters_read_only=True,
 native_actor_private_fields_observed=sorted(item_fields),permanent_fields=8,diagnostic_fields=14,cached_ability_values_observed_without_recompute=True,
 Python_parser=str(parser),Python_parser_sha256=sha(parser),Python_AST_only=True,Python_parser_executed=False,
 failed_compile=str(failure),failed_compile_original_tool_chunk='c2ec71',failed_compile_exit_code=1,original_compiler_identities_terminal=True,
 integrated_into_active_fixture=False,integrated_into_producer_reader=False,client_image_bindings_requalified_for_new_fixture=False,
 tests_executed=0,classes_instantiated=0,original_initializer_invoked=False,SQL_executed=False,database_started=False,desktop_inputs=0,
 normal_sources_unchanged=True,normal_PEs_unchanged=True,production_applied=False,
 full_Main_process_qualified=False,full_native_owner_qualified=False,full_attribute_matrix_qualified=False,whole_goal_complete=False,
 limitations=['Bytecode/source and data readback only; initializer, observer, parser, Job, SQL/JDBC and cases remain unexecuted',
  'New candidate is not yet the active fixture; existing client-image/producer/reader qualifications retain their preceding fixture context',
  'Eight permanent fields, cached ability map and fourteen diagnostic fields are separate; diagnostic timers/generations are not invariants',
  'Repeated stable reads are not atomic authority; no complete nondefault/expiry/mutation-denial matrix is inferred',
  'Dark-item payload/timer lease internals and standard JDK collection-view lazy caches are outside zero-cache-write claims'],
 test_execution_deferred_by_user=True,recorded_utc=datetime.now(timezone.utc).isoformat())
(out/'qualified-development.json').write_bytes(data(record))
raw={'candidate.json':cp,'candidate.zip':cp.parent/'candidate.zip','cohort.json':run/'cohort.json','compiler.py':run/'compiler.py',
 'resource-closure.json':Path(c['resource_closure']),'reader.py':Path(__file__),'item-fields.fragment.java':fragment,'item-attributes.py':parser,
 'inspection.json':inspection_path,'documentation.md':T/'native-E149-extended-item-observation.md','qualified-development.json':out/'qualified-development.json',
 'failed-cohort.json':failure/'cohort.json','failed-stderr.log':failure/'Java/stderr.log','failed-compiler.py':failure/'compiler.py',
 'failed-candidate.zip':Path(failed['candidate']).parent/'candidate.zip','owned-CIM.json':out/'owned-CIM.json',
 'report-updater-before.py':T/'update-master-report-text-index-order.py'}
for p in cohort['classes']:raw['classes/'+Path(p).name]=Path(p)
for p in cohort['process_history'][0]['logs']:raw['logs/'+Path(p).name]=Path(p)
for p in inspection['output_files']:raw['bytecode/'+Path(p).name]=Path(p)
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
 for name,path in raw.items():z.write(path,name)
with zipfile.ZipFile(out/'raw.zip') as z:assert all(hashlib.sha256(z.read(name)).hexdigest().upper()==sha(path) for name,path in raw.items())
record.update(qualification=str(out/'qualified-development.json'),qualification_sha256=sha(out/'qualified-development.json'),raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'),previous_goal_turn_evidence=w['last_goal_turn_evidence'])
(out/'evidence.json').write_bytes(data(record))
work='E149新Java候選明確保留SharedClientResources原初始化：catalog／60303項duration資料已靜態核對，未呼叫initializer。新增物品native sidecar：8項永久屬性（元素／鑲卡／魔石／覺醒）、cached ability與14項診斷，原17欄保留；Timestamp／EnumMap先複製，反射僅取欄位，未呼叫遊戲getter／recompute。原'+a.terminal_chunk+' actual exit0編譯6class，原記錄身分退出、18 record accessor與JDK複製／atomic.get路徑已核對；首輪Enum pattern型別錯誤exit1保留。Rust來源／正常1049輸入及兩EXE不變，未重編Rust或執行測試；Python比較器僅AST、非零覆蓋不得當完整矩陣。新候選尚未接入active fixture／producer／reader及映像context。下一步分開核對相同Rust位元組與新Java資格，完成reader／計畫接續，保留使用者暫緩全部測試；整體未完成。'
staged={}
for name in before:
 path=R/name
 if path.suffix=='.json':
  j=read(path);j['current_E149_item_attribute_observer_development']=record
  j.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
  j['current_focus']=dict(item='G06_3d_E149_extended_native_item_observation_development',work=work,applies_to_normal_EXE=False,goal_complete=False)
  if name.endswith('next-validation-action.json'):j['next_action']=work
  staged[name]=data(j)
 else:
  _,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert sep
  staged[name]=(datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest).encode('utf-8')
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
 for name in before:z.write(R/name,name)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(name)).hexdigest().upper()==h for name,h in before.items())
assert all(sha(R/n)==h for n,h in before.items()) and all(sha(R/n)==h for n,h in inputs.items())
for name,blob in staged.items():(R/name).write_bytes(blob)
assert all((R/name).read_bytes()==blob for name,blob in staged.items())
(out/'readback.json').write_bytes(data(dict(before_files=before,after_files={n:sha(R/n) for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
print(json.dumps(dict(evidence=str(out/'evidence.json'),Java_classes=6,Rust_compilers=0,permanent_fields=8,diagnostic_fields=14,tests_executed=0,active_fixture_adopted=False,eight_files_read_back=True),ensure_ascii=False))
