import pathlib,json,hashlib,difflib,datetime
P=pathlib.Path(__file__).resolve().parent.parent
L=P/'logs/continuation-20261003'
T=P/'target/r528-status-io-backend-consumer'
def read(p):return json.loads(pathlib.Path(p).read_text('utf-8-sig'))
def pin(p):
 p=pathlib.Path(p);raw=p.read_bytes();return dict(path=str(p),bytes=len(raw),sha256=hashlib.sha256(raw).hexdigest().upper())
def save(p,v):
 with pathlib.Path(p).open('x',encoding='utf-8',newline='\n')as f:json.dump(v,f,ensure_ascii=False,indent=2);f.write('\n')
def check(v):assert pin(v['path'])==v;return v
now=datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).isoformat()
candidate=read(L/'r528-status-io-candidate.json')
for v in candidate.values():
 if isinstance(v,dict)and {'path','bytes','sha256'}<=v.keys():check(v)
assert candidate['status_reader']['sha256']=='EB80740516AF815C2EAAA35E1024A2E02E281140B11F5A45CB289D898CB4207E'
assert candidate['countdown_module']['sha256']=='9AE57C12F033ED87F7F1FAA8BA98741B690B581166465C8763CA4E89D520E03A'
assert candidate['production_library']['sha256']=='5C9DFD442B6F107B1A44DE1FA882A4517D449BE017F99F4B87E5F56BFA006502'
old=read(L/'r528-final-candidate.json');old_pins=[]
for v in old.values():
 if isinstance(v,dict)and {'path','bytes','sha256'}<=v.keys():old_pins.append(check(v))
assert old['executable']['sha256']=='AB3B9ABEE0A3212D5BECEEB25E3A65B5CAEB4BA274E3AEDF0791D89775A9AF24'
history=[]
for name,sha in [
 ('r528-countdown-cancel-consumer-ready-handoff.json','28BC666D4512DA1BFF5D9584BD1C4C7A9F85FC7DDF02F13C5648B2C6F84B9E9F'),
 ('r528-status-reader-compatibility-handoff.json','664E47D5EB6D129AD7623361DEDC0900B43A159BA75427DAC9503B751A32EC9E'),
 ('r528-countdown-cancel-draft-preparation-handoff.json','73F9EA92A2D1EF0D9C9CF3FB2D7EDB869A7A5568C628AE8761E4991D23CED511')]:
 v=pin(L/name);assert v['sha256']==sha;history.append(v)
failure=pin(L/'r528-status-reader-review/first-provider-failure-result-observed.json')
assert failure['sha256']=='3CBD80CDF077FD536FDF20E398046A419CD70F519FD4FF160BE473760AD530FA'
source_snapshot=read(L/'r527-production-source-snapshot.json');sources=[]
assert len(source_snapshot['source_pins'])==639
for v in source_snapshot['source_pins']:
 current=pin(v['path']);assert current['sha256']==v['sha256']and current['bytes']==v['bytes']
 frozen=pin(v['snapshot']);assert frozen['sha256']==v['sha256']and frozen['bytes']==v['bytes'];sources.append(dict(current=current,frozen=frozen))
save(L/'r528-status-io-production-immutability.json',dict(status='639_CURRENT_AND_FROZEN_SOURCES_UNCHANGED_LIBRARY_UNCHANGED',source_count=639,production_library=candidate['production_library'],source_pairs=sources,source_changes=0,production_builds=0,old_candidate_preserved=old_pins,historical_handoffs_preserved=history,first_failed_provider_result_preserved=failure))
accounting={}
for label in ['build','tests-build','tests','prepared']:
 a=read(L/f'r528-status-io-{label}-accounting.json');assert a['attempts']==1 and a['exit_code']==0
 assert not (L/f'r528-status-io-{label}.stderr.log').read_text('utf-8-sig').strip()
 accounting[label]=a
assert '5 passed; 0 failed' in (L/'r528-status-io-tests.stdout.log').read_text('utf-8-sig')
assert '9 filtered out' in (L/'r528-status-io-tests.stdout.log').read_text('utf-8-sig')
adoption=read(L/'r528-status-io-cold-adoption.json')
assert adoption['payload_count']==122 and adoption['launch_count']==26
assert adoption['root']['sha256']=='6C2C82073BB3DB18FC2121A7C74FAEF4E7A5DF28376874C171F4888BE99ED4EE'
# Already verified cold server pins: preserve that read-only evidence; no Java I/O retest.
for v in adoption['copies']:check(v)
diff=''
for previous,new in [('r528-backend-consumer.rs','r528-status-io-consumer.rs'),('r528-provider.rs','r528-status-io-provider.rs')]:
 diff+=''.join(difflib.unified_diff((P/'tools'/previous).read_text('utf-8').splitlines(True),(T/new).read_text('utf-8').splitlines(True),fromfile='preserved/'+previous,tofile='new/'+new))
# Refresh own unsealed draft difference with the final compiled provider/test inclusion.
(L/'r528-status-io-source-difference.patch').write_text(diff,'utf-8',newline='\n')
provider=(T/'r528-status-io-provider.rs').read_text('utf-8');old_provider=(P/'tools/r528-provider.rs').read_text('utf-8')
def section(s,start,end):return s[s.index(start):s.index(end,s.index(start))]
assert section(provider,"fn armed_getter<'a>",'pub fn synchronized_host')==section(old_provider,"fn armed_getter<'a>",'pub fn synchronized_host')
assert section(provider,'fn sync_common','fn static_ready')==section(old_provider,'fn sync_common','fn static_ready')
assert section(provider,'fn template','fn fresh_status')==section(old_provider,'fn template','fn fresh_status')
consumer=(T/'r528-status-io-consumer.rs').read_text('utf-8');prior=(P/'tools/r528-backend-consumer.rs').read_text('utf-8')
assert section(consumer,'fn run_case','fn save_new')==section(prior,'fn run_case','fn save_new')
assert consumer[consumer.index('fn main'):]==prior[prior.index('fn main'):]
for name in ['r528-status-io-consumer.rs','r528-status-io-countdown-case.rs','r528-status-io-provider.rs','r528-status-io-reader.rs','r528-status-io-interface-tests.rs']:
 assert (P/'tools'/name).read_bytes()==(T/name).read_bytes()
checks=dict(wrapper_compiles=1,targeted_interface_harness_compiles=1,new_interface_tests=5,old_tests_filtered=9,compile_warnings=0,compile_failures=0,test_failures=0,prepared_check_runs=1,all_first_attempt=True,old_PASS_replays=0,production_rebuilds=0,Runtime_synthetic_replays=0,preflight=0,connect=0,provider_starts=0,JavaSQL=0,GUI=0,credentials_reads=0,PIDqueries=0,server_writes=0,retired_READY_reads=0,read_only_inspection_tool_failures=[dict(kind='FileNotFoundError',scope='Initial debug used Path(.) parent to inspect shared progress; corrected to cwd absolute. No build/test/runtime or server mutation.')])
save(L/'r528-status-io-build-and-scope-accounting.json',dict(updated_at=now,checks=checks,initial_accounting=accounting,own_scratch_retained=candidate['scratch'],no_cleanup_requested=True,no_tool_approval_rejection=True))
limits=read(L/'r528-status-io-provider-supply-request.json')['limits']
notes=dict(
 accepted_server_root=adoption['root'],accepted_bundle=adoption['bundle'],cold_payload_pins=122,cold_launch_pins=26,cold_unique_pins=adoption['unique_checked'],
 frozen_reader=candidate['status_reader'],
 same_handle='Single default-shared File handle metadata/bytes/metadata, <=65536bytes, mtime<=5s on same opened generation; drop handle before parse, error immediately propagated. No override/retry/lock.',
 throttle='At most one status read start per200ms while waiting Host gate. Initial validation records start too; 2ms Runtime polling continues, skipped callback returns None without file I/O/sleep/cached READY. No catchup. Frequency choice is conservative, not a Java atomic move compatibility guarantee.',
 diagnostic='status_publication schema accepted and recorded only with actual Host gate; counters/first_error do not admit/reject readiness. Existing header, mtime, lease, two-peer/normal-source/baseline/Host actual wire and qualified private Ready guards retained. Snapshot precedes current server publication move.',
 publisher='status.json only, one content write then atomic replace; AccessDenied only max6 moves/5x20ms sleeps, initial AccessDenied on exhaustion. 100ms requested sleeps is not strict wallclock. No ownership request retry/fallback/truncate/lease extension/provider restart.',
 unchanged_countdown_flow='Baseline0 -> actual Host1 + two-peer WAITING armed -> GuestReady1/fullwrite/ownD1one -> actualE103 -> immediate typedCancel0 -> fullwrite/ownD1zero/E100 ->4s actual Runtime poll watch -> Close/join/queue/late. Countdown source byte-identical 9AE57C12; no disk read/wait/sleep between E103 and typedCancel0.',
 historical_context='AB3 executable/28BC ready handoff and failed nw975 safe result remain immutable. Failed nw975 READY not read or consumed, now explicitly refused. R527 retired nw6f and old nw97 still refused.',
 test_scope='5 new targeted checks once, 9 old tests filtered out. One new wrapper build and one new --test harness compile, fixed regular5C9 library reused. No actual Runtime, Java fixture, provider, SQL or socket execution.',
 limitations=limits,
 source_changes=0,production_sources_verified=639,scope=checks)
save(L/'r528-status-io-integration-findings.json',notes)
human=P/'Native-R528_statusIO修復整合就緒交接.txt'
human.write_text('R528 status I/O 修復整合冷態交接\n\n'
 '新的 Modern consumer 已完成，待監製審查與另行授權 fresh server run。本轮只在 modern 建立 Rust wrapper、必要檢查與結果，不代 server 執行 Java fixture，不寫 server、不啟動 provider/SQL、不執行 preflight/connect。\n\n'
 f"執行檔：{candidate['executable']['path']}\nSHA256：{candidate['executable']['sha256']}\n來源 SHA256：{candidate['source']['sha256']}\nProvider 模組 SHA256：{candidate['provider_module']['sha256']}\n\n"
 '接受 server cold root 6C2C8207、aa561cbbe3d74e1b80f3d8858c1e576f bundle、884F2FE6 handoff、214AF07D launch；唯讀核對122 payload／26 launch與124個不重複 cold pins。Runtime validator嚴格固定新 handoff／launch／manifest pins，拒絕舊 bundle及退休 nw975/nw6f/nw97。\n\n'
 'EB807405 reader原封整合：只開一個 File handle，metadata→有界 bytes→同 handle metadata，以同一已開版本mtime核5秒，65536byte上限，關閉handle後完整JSONparse，錯誤立即返回，不鎖檔或重試。Host同步檢查每200ms最多開始一次讀檔；Runtime仍照原2ms poll。沒有快取READY或回補讀取，這個頻率選擇不保證 Java atomic move 永遠成功。\n\n'
 '新status_publication只作schema／診斷觀察，保留有限重試及first_error，不用counter0/null換取Ready資格。5項新檢查首次全PASS／warning0：頻率邊界、新bundle／manifest pins、診斷與現有status守門接線、新自有status檔samehandle讀取、退休nw975與舊bundle拒絕。9項舊測試未執行。1次新wrapper編譯＋1次針對性test harness編譯，1次--check-prepared，0preflight/connect／0JavaSQL／0socket／0GUI／0credential／0PIDquery。\n\n'
 '倒數取消模組保持9AE57C12 byte-identical，EB807405 reader也未變。Baseline0→Host1雙peer armed→Ready1完整寫入／ownD1one→actualE103→立即typedCancel0→完整寫入／ownD1zero／E100→4秒實際pollwatch→Close策略不變。E103到Cancel0不加入讀檔、等待、sleep或timer。639份production及frozen source核對不變，固定5C9 regular library不重編，AB3/28BC及nw975首次failure保存。\n\n'
 'server僅對AccessDenied作最多6次move／5x20ms重試；100ms是要求sleep總量而非wallclock上限。此host上持有SHARE_DELETE的reader仍可阻擋JDK17 atomic move，Rust rename測試不是Java兼容證明；無節流壓力測試曾耗尽预算（value56），限頻5ms測試成功只證明有限緩解。歷史阻擋handle／ACL／filter／AV原因未知，不能證明是ROOT或Modern造成。GET非globalatomic、不保證sameepochfreshness／queuedtickdrain／WORLD排除；正式runtime與GUI驗收仍待做。\n\n'
 '供應請求：logs/continuation-20261003/r528-status-io-provider-supply-request.json\n'
 '機器交接：logs/continuation-20261003/r528-status-io-consumer-ready-handoff.json\n',encoding='utf-8',newline='\n')
refs=[pin(human),*history,failure]
refs += [pin(L/name)for name in ['r528-status-io-candidate.json','r528-status-io-cold-adoption.json','r528-status-io-production-immutability.json','r528-status-io-source-difference.patch','r528-status-io-build-and-scope-accounting.json','r528-status-io-integration-findings.json','r528-status-io-provider-supply-request.json']]
for label in ['build','tests-build','tests','prepared']:
 refs += [pin(L/f'r528-status-io-{label}{suffix}')for suffix in ['.stdout.log','.stderr.log','-accounting.json']]
refs += [pin(T/name)for name in ['r528-status-io-consumer.rs','r528-status-io-countdown-case.rs','r528-status-io-provider.rs','r528-status-io-reader.rs','r528-status-io-interface-tests.rs','accepted-status-io-handoff.json','accepted-status-io-launch.json','accepted-status-io-manifest.json']]
refs += [pin(P/'tools'/name)for name in ['prepare-r528-status-io-integration.py','build-r528-status-io.ps1','seal-r528-status-io-integration.py','r528-status-io-consumer.rs','r528-status-io-countdown-case.rs','r528-status-io-provider.rs','r528-status-io-reader.rs','r528-status-io-interface-tests.rs']]
refs += adoption['copies']+[candidate['executable'],candidate['tests']]
fixture=pathlib.Path(candidate['scratch'])/'new-interface-status-only.json';refs.append(pin(fixture))
for v in refs:check(v)
verification=L/'r528-status-io-consumer-ready-verification.json'
save(verification,dict(status='PASS_COLD_NEW_INTERFACES_ONLY_NO_FRESH_RUNTIME',root_references_checked=refs,checks=checks,flow_unchanged=True,armed_getter_and_sync_and_process_qualification_source_unchanged=True,main_and_normal_entry_source_unchanged=True,source_count_unchanged=639,old_handoffs_preserved=history,first_failed_provider_result_preserved=failure))
refs.append(pin(verification))
handoff=L/'r528-status-io-consumer-ready-handoff.json'
save(handoff,dict(schema='modern.r528.status-io-consumer-ready-handoff.v1',updated_at=now,state='COLD_STATUS_IO_CONSUMER_READY_WAITING_SUPERVISOR_REVIEW_AND_SEPARATELY_AUTHORIZED_FRESH_PROVIDER',root_references=refs,candidate=candidate,accepted_cold_root=adoption['root'],accepted_bundle=adoption['bundle'],supply_request=pin(L/'r528-status-io-provider-supply-request.json'),human=pin(human),verification=pin(verification),historical_handoffs=history,provider_first_failure=failure,failed_READY_consumed=False,publisher_repair_integrated=True,checks=checks,limits=limits,fresh_READY_accepted=False,fresh_runtime_authorized=False,overall_complete=False))
# Read latest shared progress at the final write to preserve all other groups.
shared=P.parent/'監製進度.json';progress=read(shared)
progress['updated_at']=now;progress['phase']='R528_status_io_consumer_sealed_waiting_supervisor_review_fresh_provider'
progress['current_task']='R528冷態statusIO整合已封存：FAD83BE0新consumer，EB807405同handle讀者、200ms Host讀頻限制；新5介面首次PASS/warning0、舊9filtered。AB3/28BC與nw975failure、639production/rlib不變；0preflight/connect/provider/JavaSQL/GUI，待監製審查及另授權fresh供應。'
progress['overall_status']='in_progress'
r=progress.setdefault('r528',{})
r['previous_ready_handoff']=r.get('handoff');r['previous_candidate']=r.get('candidate')
r.update(state='COLD_STATUS_IO_CONSUMER_SEALED_FRESH_RUNTIME_NOT_AUTHORIZED',handoff=pin(handoff),human=pin(human),verification=pin(verification),candidate=pin(L/'r528-status-io-candidate.json'),supply_request=pin(L/'r528-status-io-provider-supply-request.json'),new_status_io_tests=5,new_status_io_old_tests_filtered=9,status_reader_proposal_pending_integration=False,publisher_repair_contract_pending=False,publisher_repair_accepted=True,status_poll_start_minimum_interval_ms=200,status_io_checks=checks,new_interface_pending=False,preflight_runs=0,connect_runs=0,fresh_provider_execution_authorized=False,fresh_READY_accepted=False,overall_completed=False,awaiting_supervisor_consumer_ready_acceptance=True,source_files_changed=0,server_writes=0,actual_backend_executions=0,JavaSQL=0,GUI=0,old_PASS_replays=0)
progress.setdefault('completed',[]).append('R528 statusIO新wrapper冷整合：124新cold pins／122payload／26launch，EB807405 samehandle＋200ms gate限頻；5新介面首次PASS、舊9filtered、warning0，639production/rlib不改；FAD83BE0 executable與新交接/supplyrequest封存，0fresh runtime。')
with shared.open('w',encoding='utf-8',newline='\n')as f:json.dump(progress,f,ensure_ascii=False,indent=2);f.write('\n')
print(json.dumps(dict(handoff=pin(handoff),human=pin(human),candidate_executable=candidate['executable'],checks=checks),ensure_ascii=False))
