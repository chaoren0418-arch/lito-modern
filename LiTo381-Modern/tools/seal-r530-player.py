import pathlib,json,hashlib,shutil,difflib,re,datetime,collections
P=pathlib.Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003';T=P/'target/r530-player-entry-candidate'
def read(p):return json.loads(pathlib.Path(p).read_text('utf-8-sig'))
def pin(p):
 p=pathlib.Path(p);h=hashlib.sha256()
 with p.open('rb')as f:
  for b in iter(lambda:f.read(8<<20),b''):h.update(b)
 return dict(path=str(p),bytes=p.stat().st_size,sha256=h.hexdigest().upper())
def save(p,v):
 with pathlib.Path(p).open('x',encoding='utf-8',newline='\n')as f:json.dump(v,f,ensure_ascii=False,indent=2);f.write('\n')
candidate=read(L/'r530-player-candidate.json');candidate['state']='R530_EXPLICIT_SUPERVISOR_READY_BACKEND485F_PLAYER_COMPILED_GUI_UNEXECUTED'
for key in ['executable','production_library','candidate_main','original_main','binding','skin_binding']:assert pin(candidate[key]['path'])==candidate[key]
for n in ['library-build','targeted-tests','player-build']:assert read(L/f'r530-{n}-accounting.json')['exit_code']==0
checks={}
for n,count in [('targeted-tests',2),('route-tests',1),('provider-check',1)]:
 s=(L/f'r530-{n}.stdout.log').read_text('utf-8-sig');assert f'{count} passed; 0 failed' in s;checks[n]=dict(passed=count,old_tests_executed=0,stdout=pin(L/f'r530-{n}.stdout.log'),stderr=pin(L/f'r530-{n}.stderr.log'))
panel=read(L/'r530-panel-check-attempt1/result.json');assert panel['first_error'] is None and panel['producer_error'] is None and panel['close_ok'];assert panel['IO_started']==panel['IO_joined']==2 and panel['queued_events']==panel['discarded_late_events']==0
wire=read(L/'r530-panel-check-attempt1/wire.json');commands=[v for v in wire if v.get('C',[None])[0]==210];assert [v['C']for v in commands]==[[210,1],[210,0]] and all(v['frame_bytes']==4 for v in commands)
assert not any(v.get('C',[None])[0]in [208,209,221] for v in wire)
assert 'R530 new positive Panel' in (L/'r530-panel-tests.stdout.log').read_text('utf-8-sig')
checks['new_finite_panel_case']=dict(passed=True,execution_attempts=1,normal_entry_epochs=1,synthetic_Rust_fixture=True,actual_backend=False,GUI=0,result=pin(L/'r530-panel-check-attempt1/result.json'),wire=pin(L/'r530-panel-check-attempt1/wire.json'),stdout=pin(L/'r530-panel-tests.stdout.log'),stderr=pin(L/'r530-panel-tests.stderr.log'),old_PASS_replays=0)
main=pathlib.Path(candidate['candidate_main']['path']).read_text('utf-8');route=(T/'r530-backend-route-check.rs').read_text('utf-8');assert main[main.index('fn route('):main.index('fn run()')]==route[route.index('fn route('):route.index('#[cfg(test)]mod r530_backend_route')]
provider=(P/'src/modern_lito/native_backend_485f_provider.rs').read_text('utf-8');entry=(P/'src/modern_lito/native_backend_485f_entry.rs').read_text('utf-8');window=(P/'src/modern_lito/native_city_window.rs').read_text('utf-8')
assert 'nw-c9afae2f2cd14da08208599b1bf066fb'in provider and '2B45D78545ADF12A0CF2FAD02FB1ABF1E9899E3F2AE5B40A8A9F68BBF900D5F4'in provider
assert '+180000' in provider and '+60000'in provider and 'code==259&&ticks.to_string()==creation_text'in provider
assert 'Accepted::load(&path,&sha)'in main and 'native_city_window::run_backend_485f(inputs,skin_inputs,accepted)'in main
assert 'b.invoke(r,&panel,operation)'in window and 'b.begin()'in window and 'b.poll_health(r)'in window
invoke=entry[entry.index('pub(super) fn invoke'):entry.index('pub(super) fn finish')];assert 'read_bound'not in invoke and 'fresh_status'not in invoke and 'sleep'not in invoke
reader=(P/'src/modern_lito/native_backend_485f_status_reader.rs').read_text('utf-8');old_reader=(P/'tools/r528-status-io-reader.rs').read_text('utf-8').split('#[cfg(test)]')[0];assert reader==old_reader
testerr=(L/'r530-targeted-tests.stderr.log').read_text('utf-8-sig');testfile=P/re.search(r'Running unittests .*? \(([^)]+)\)',testerr)[1];frozen_test=T/'r530-new-entry-tests.exe';shutil.copyfile(testfile,frozen_test)
old=read(L/'r529-production-source-snapshot.json');old_by={str(pathlib.Path(v['path']).relative_to(P)):v for v in old['source_pins']};sources=[];changes=[];diff='';source_root=T/'source';source_root.mkdir()
for source in sorted((P/'src').rglob('*.rs')):
 rel=source.relative_to(P);v=pin(source);previous=old_by.get(str(rel));before=None
 if previous:
  f=pin(previous['snapshot']);assert f['bytes']==previous['bytes']and f['sha256']==previous['sha256']
  if previous['sha256']!=v['sha256']:before=pathlib.Path(previous['snapshot']).read_text('utf-8');changes.append(dict(kind='modified',relative_path=str(rel),before=previous,after=v))
 else:before='';changes.append(dict(kind='added',relative_path=str(rel),after=v))
 if before is not None:diff+=''.join(difflib.unified_diff(before.splitlines(True),source.read_text('utf-8').splitlines(True),fromfile='R529-frozen/'+str(rel),tofile='R530/'+str(rel)))
 dest=source_root/rel;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(source,dest);sources.append(dict(**v,snapshot=str(dest)))
assert len(sources)==643 and len(changes)==6
(L/'r530-production-source-difference.patch').write_text(diff,'utf-8',newline='\n')
save(L/'r530-production-source-snapshot.json',dict(source_count=643,source_pins=sources,changes=changes,preserved_R529_source_count=640,difference=pin(L/'r530-production-source-difference.patch'),library=candidate['production_library']))
preserved=[]
old_candidate=read(L/'r529-player-candidate.json')
for k in ['executable','production_library','candidate_main','original_main','binding','skin_binding']:assert pin(old_candidate[k]['path'])==old_candidate[k];preserved.append(old_candidate[k])
for v in read(L/'r529-prior-evidence-preservation.json')['pins']:
 assert pin(v['path'])==v;preserved.append(v)
for name in ['r529-room-ui-candidate-handoff.json','r529-production-source-snapshot.json','r529-R528-server-closure-reception.json','r528-status-io-real-backend-consumer-handoff.json','r528-status-io-consumer-ready-handoff.json','r527-production-source-snapshot.json']:
 preserved.append(pin(L/name))
f527=read(L/'r527-production-source-snapshot.json')
for v in f527['source_pins']:
 f=pin(v['snapshot']);assert f['bytes']==v['bytes']and f['sha256']==v['sha256']
save(L/'r530-prior-evidence-preservation.json',dict(pins=preserved,R529_frozen_source_verified=640,R527_frozen_source_verified=639,old_PASS_replays=0,old_live_or_stress_replays=0,old_fixed_artifacts_rebuilt=False))
def warnings(name):
 out=[]
 for line in (L/name).read_text('utf-8-sig').splitlines():
  try:v=json.loads(line)
  except json.JSONDecodeError:continue
  if v.get('reason')=='compiler-message'and v['message']['level']=='warning':out.append(v['message']['message'])
 return out
before=warnings('r529-library-build.jsonl');after=warnings('r530-library-build.jsonl');seen=collections.Counter(before);additional=[]
for message in after:
 if seen[message]:seen[message]-=1
 else:additional.append(message)
assert len(before)==288 and len(after)==290 and len(additional)==2
failed=pin(L/'r530-route-build.stderr.log');assert 'couldn\'t read' in pathlib.Path(failed['path']).read_text('utf-8-sig')
save(L/'r530-check-accounting.json',dict(checks=checks,regular_library_build_attempts=1,player_build_attempts=1,player_executions=0,library_filtered_old_tests=2340,route_compile_attempts=2,route_executions=1,first_route_compile_failure=failed,route_repair='Only new route harness extracted unchanged enum/function. Original candidate main/EXE/library unchanged; no old main tests executed.',provider_static_check_compile_attempts=1,provider_static_check_executions=1,panel_compile_attempts=1,panel_execution_attempts=1,regular_warnings=len(after),historical_regular_warnings=len(before),new_regular_warning_messages=additional,test_harness_warnings=int(re.search(r'\(lib test\) generated (\d+) warnings',testerr)[1]),R529_three_naming_warnings_fixed=True,player_warnings=0,panel_warnings=0,route_corrected_harness_warnings=0,frozen_entry_test_binary=pin(frozen_test),JavaSQL=0,real_provider_connections=0,credentials_file_reads=0,GUI=0,old_PASS_replays=0))
request=dict(schema='modern.r530.player-fresh-provider-request.v1',status='COLD_UI_ENTRY_AND_NEW_PANEL_CHECK_READY_NO_LIVE_AUTHORIZATION_USED',lifecycle_owner='server group only; supervisor coordinates supply',interface='Existing accepted R528 aa561 status IO provider interface unchanged',prior_supply_request=pin(L/'r528-status-io-provider-supply-request.json'),arguments=['--candidate-backend-485f','<absolute fresh run ready-contract.json>','<supervisor reviewed READY SHA256>'],requirements=['New lower-case nw UUID; no retired nw-c9af/2B45 or other blacklisted runs','Frozen aa561 bundle/485F jar and exact R528 26 launch pins, 122 manifest linkage','900second lease, attempts1, >=180seconds at entry and first OpenLogin; >=60seconds thereafter','Exactly owned helper/mysql metadata plus matching live PID creation FILETIME, no protected PID','Host support + host_ready_after_guest_notready; initial Host-only unready WAITING0/0 and empty witness sets','Fx485Moon raw source at advertised GAME127.0.0.2:sameport; actual normal singleton Guest alias, channel0, actual CC0 public room matching READY','Fresh same-opened-handle status <=5seconds with accepted publication diagnostic; no background_failure or unexpectedWORLD','Supervisor gives READY path+digest; human locally enters supplied Guest password, no secret contents/clipboard/file copied into Modern'],UI_steps=['Start explicit candidate only after separate supervisor supply authorization','Click Backend485F驗收登入; enter Fx485Guest and supplied password','Select Fx485Moon, Continue news, select actual singleton Guest, channel1(displayed slot0), select provided public room and Join','Once actual qualified room context appears click取消準備 to send Guest baseline0','Wait for server Host1/twopeer witness and actual Host D1one; Ready becomes enabled and is allowed once','Click準備; view actual own D1 and received E1; click取消準備 promptly if real countdown cancellation is part of authorization','Observe received D1zero/E100 clearing, then Close; server group captures evidence and retires its owned processes'],new_Java_interface_required=False,Modern_starts_stops_or_writes_server=False,remaining='Actual native window clicks, account focus/password clearing, scaling/enabling/close responsiveness, real backend D1/E1 and server wire/GET/retirement acceptance require future authorized supply and allowed GUI tools.',timing_limits='Status file reads occur only in existing40ms timer, starts throttled>=200ms; no status read/PID query/sleep in Ready/Cancel callback. OS read duration/UI latency remains unverified; cancellation timing not guaranteed. Separate GET fields are not atomic/wire freshness/ACK/world proof.')
save(L/'r530-fresh-provider-supply-request.json',request)
contract=dict(candidate=candidate,flag=request['arguments'],authority='Opaque Accepted load verifies READY digest/path/lease/owned executable-command-datadir/PID+FILETIME/pins before constructing real Backend485fProfile; first OpenLogin revalidates and is single attempt. Legacy synthetic route remains separate.',path='CandidateBackend485f -> Accepted::load -> candidate_backend_entry -> native_city_window::run_backend_485f -> Backend485fProfile/AD368 pinned bootstrap/room_player.configure -> normal owned LOGIN/source/GAME/news/singleton/character/channel/observedpublicJoin -> Panel private Ready1/NotReady0 -> Runtime admission',source_constraints=request['requirements'],health='Samehandle reader identical to R528 production portion; >=200ms start rate, <=5s generation age, current owned PID FILETIME/lease/status failure checks; cached health <=1s at callback. First timer health failure closes own Runtime only. Ready gate requires cached actual twopeer Host witness + actual Host D1one + current Runtime private token.',display='Actual own D1 only; actual unsigned E1 count only, E100 clears countdown independently; no optimistic/local timer or ownerstart/WORLD.',checks=checks,scope='Cold compiled real backend entry and positive CPU Panel callback to owned finite Rust fixture; no native HWND command/click or real backend callback executed.',GUI=0)
save(L/'r530-entry-and-ui-contract.json',contract)
human=P/'Native-R530_Backend485F玩家入口候選交接.txt'
with human.open('x',encoding='utf-8',newline='\n')as f:f.write('R530 顯式 Backend485F 玩家候選已編譯；本輪冷態，GUI0／真provider0。\n\n候選：'+candidate['executable']['path']+'\nSHA256：'+candidate['executable']['sha256']+'\n參數：--candidate-backend-485f <新READY絕對路徑> <監製提供的SHA256>\n\n入口先驗證，首次登入再次驗證 R528 pins／租期／owned PID+FILETIME；只允許新run的一次初始連線。nw-c9af／2B45已明確拒絕。\n登入 Fx485Guest → Fx485Moon → news → 實收單一Guest角色 → 頻道slot0 → READY指定的實收公開room → 先取消準備送baseline0 → 等Host1實收與twopeer witness → 可按一次準備 → 依實收倒數取消 → Close。\n密碼由人員在原表單輸入，Modern不讀或複製server properties。健康讀取≥200ms起始間距、同handle≤5秒；準備／取消callback無檔案讀取、PID查詢或sleep。UI對OS讀取的延遲仍待驗收。\n\n新增2 library guard checks、1 CLI route check、1 nw-c9af static check PASS。1個新有限Rust fixture的Panel公開Join、Ready/Cancel完整frame4、D1/E1顯示、舊callback失效PASS；2IO已join／queue0／late0。它只證明CPU接線，未操作視窗或連真backend。\n首個CLI test編譯因既有relative include路徑失敗，錯誤保留；抽取原樣route函式後只跑新測試PASS。已通過fixture不重跑。\n643來源封存，對R529修改3／新增3；R529640、R527639與AB3/FAD83/舊失敗保留。Regular build290警告＝288既有＋2未使用欄位／常數；player與新fixture無警告。修正R529三個測試名稱警告，未重跑那4個旧PASS。\n\n後續由監製協調server組供應新READY／owned lease／status與證據；Modern不啟停Java/SQL、不跨寫server、不消費退休供應。既有R528介面足夠，沒有新增Java程式需求。仍待允許的GUI工具與實際視窗流程、輸入焦點／縮放／Close、真backend回包及server退休驗收；不證明ACK、同epoch freshness、tick drain或WORLD不可達。\n')
refs=[candidate['executable'],candidate['production_library'],candidate['candidate_main'],candidate['original_main'],candidate['binding'],candidate['skin_binding'],pin(human)]
for n in ['r530-entry-and-ui-contract.json','r530-fresh-provider-supply-request.json','r530-check-accounting.json','r530-production-source-snapshot.json','r530-production-source-difference.patch','r530-prior-evidence-preservation.json','r530-player-candidate.json','r530-library-build.jsonl','r530-library-build.stderr.log','r530-library-build-accounting.json','r530-targeted-tests-accounting.json','r530-player-build-accounting.json','r530-player-build.stdout.log','r530-player-build.stderr.log','r530-route-build.stdout.log','r530-route-build.stderr.log','r530-route-build2.stdout.log','r530-route-build2.stderr.log','r530-provider-check-build.stdout.log','r530-provider-check-build.stderr.log','r530-panel-build.stdout.log','r530-panel-build.stderr.log','r530-panel-tests.stdout.log','r530-panel-tests.stderr.log']:
 refs.append(pin(L/n))
for n in ['prepare-r530-player.py','prepare-r530-panel-check.py','prepare-r530-provider-check.py','build-r530-player.ps1','build-r530-panel-check.ps1','r530-panel-callback-check.rs','r530-provider-boundary-check.rs','seal-r530-player.py']:refs.append(pin(P/'tools'/n))
for n in ['r530-route-check2.exe','r530-backend-route-check.rs','r530-panel-check.exe','r530-provider-boundary-check.exe','r530-new-entry-tests.exe']:refs.append(pin(T/n))
for n in ['result.json','wire.json']:refs.append(pin(L/'r530-panel-check-attempt1'/n))
for v in checks.values():
 for k in ['stdout','stderr']:
  if k in v:refs.append(v[k])
unique={v['path']:v for v in refs};assert len(unique)==len({v['path']for v in refs})
handoff=L/'r530-backend-player-candidate-handoff.json';save(handoff,dict(schema='modern.r530.backend-player-candidate-handoff.v1',status='COLD_EXPLICIT_BACKEND485F_NATIVE_WINDOW_ENTRY_AND_NEW_POSITIVE_CPU_PANEL_CHECK_COMPLETE_GUI_LIVE_ACCEPTANCE_PENDING',created_at=datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).isoformat(),candidate=candidate,checks=checks,production_source_count=643,changes_from_R529=6,GUI=0,real_provider_connections=0,JavaSQL=0,credentials_file_reads=0,old_PASS_replays=0,retired_READY_consumed=False,first_route_compile_failure_retained=True,existing_system_intercepts_preserved=True,supply_request=pin(L/'r530-fresh-provider-supply-request.json'),remaining=request['remaining'],refs=list(unique.values())))
for v in read(handoff)['refs']:assert pin(v['path'])==v
progress_path=P.parent/'監製進度.json';progress=read(progress_path);progress['updated_at']=datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).isoformat();progress['phase']='R530_explicit_Backend485F_player_compiled_positive_CPU_Panel_PASS_GUI_live_pending';progress['current_task']='R530顯式fresh READY＋監製SHA入口與同native_city_window接線已編譯封存；R528 lease/owned PID+FILETIME/pins/退休拒絕沿用，健康讀取200ms節流且不插入Ready/Cancel callback。4新cold checks＋1新有限Rust Panel fixture PASS／2IO joined、queue0、late0；643sources/6變更。GUI0／真provider0，待監製新供應及允許GUI驗收。';progress['overall_status']='in_progress';progress['r530']=dict(status='COLD_ENTRY_AND_POSITIVE_CPU_PANEL_PASS_GUI_LIVE_PENDING',handoff=pin(handoff),candidate=candidate['executable'],production_source_count=643,production_changes_from_R529=6,new_cold_tests=4,new_finite_Rust_fixture_cases=1,IO_started=2,IO_joined=2,queue=0,late=0,GUI=0,real_provider_connections=0,JavaSQL=0,retired_READY_consumed=False,first_route_compile_failure_retained=True,fresh_provider_needed=True,provider_interface_change_needed=False,server_writes=0,old_PASS_replays=0,remaining=request['remaining'])
progress_path.write_text(json.dumps(progress,ensure_ascii=False,indent=2)+'\n','utf-8')
print(json.dumps(dict(handoff=pin(handoff),refs=len(unique),candidate=candidate['executable'],source_count=643,changes=6,GUI=0,actual_backend=0),ensure_ascii=False,indent=2))
