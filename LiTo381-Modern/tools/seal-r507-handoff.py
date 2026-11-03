from r491_common import *
import re
assert pin(L/'r506-native-transfer-channel-handoff.json')['sha256']=='40DC622136D898E691D31091CA031F8BE47F75D3A4EDA55634A3F11BC3248C5F'
prod=read(L/'r507c-production-source-snapshot.json');checks=0
for e in prod['source_pins']:
 assert pin(e['path'])['sha256']==e['sha256']==pin(e['snapshot'])['sha256'];checks+=2
consumers={}
for label in ['r507g','r507v','r507n']:
 c=read(L/(label+'-consumer-candidate.json'))
 for e in c.values():
  if isinstance(e,dict) and 'sha256' in e:assert pin(e['path'])['sha256']==e['sha256'];checks+=1
 consumers[label]=pin(L/(label+'-consumer-candidate.json'))
player=read(L/'r507c-player-candidate.json');assert player['candidate_executions']==0
for e in [player['executable'],player['production_library'],player['candidate_main']]:assert pin(e['path'])['sha256']==e['sha256'];checks+=1
assert all(read(L/(n+'-consumer-candidate.json'))['production_library']['sha256']==player['production_library']['sha256'] for n in consumers)
for label in ['r507g','r507v']:assert (L/(label+'-consumer.exit.txt')).read_text(encoding='utf-8-sig').strip()=='0'
first=read(L/'r507-second-attempt/r507-event-partial.json');host=read(L/'r507g-event-details.json');timer=read(L/'r507v-event-details.json');native=read(L/'r507n-integration-exit.json');boundary=read(L/'r507-actual-java-boundary.json')
assert len(first)==4 and len(host['trace'])==2 and len(timer['trace'])==2
assert all(e['passed'] for e in first+host['trace']+timer['trace'])
assert host['actual_synthetic_frames']==21 and timer['actual_socket_frames']==9
assert all(e['details']['workers_started']==e['details']['workers_joined'] for e in host['trace'])
assert timer['trace'][-1]['details']['workers_started']==timer['trace'][-1]['details']['workers_joined']==4
exits=[L/'r507-first-attempt/r507-integration-exit.json',L/'r507c-integration-exit.json',L/'r507d-integration-exit.json',L/'r507n-integration-exit.json']
retirement=[]
for p in exits:
 v=read(p);assert v['all_owned_retired'] and v['all_owned_listeners_released'] and v['SQL_retirement']['mysql_retired'];assert not v['native_normal_stop']['forced_termination'];retirement.append(pin(p))
assert native['consumer_exit']==0 and boundary['java_version_and_synthetic_account_login_accepted'] and not boundary['full_native_login_channel_path']
references=[L/'r507c-production-source-snapshot.json',L/'r507c-source-difference.patch',L/'r507c-player-candidate.json',L/'r507-contract-boundaries.json',L/'r507-actual-java-boundary.json',L/'r507-second-attempt/r507-event-partial.json',L/'r507g-event-details.json',L/'r507v-event-details.json',L/'r507n-event-details.json',L/'r507n-integration-exit.json',L/'r507n-java-execution-inputs.json',L/'r507-source-oracle.json',L/'r507-timeout-oracle.json',L/'r507-first-attempt/preservation.json',L/'r507-second-attempt/preservation.json',L/'r507-third-attempt/preservation.json',L/'r507-fourth-attempt-preserved.json',L/'r507-fifth-attempt-preserved.json']
files=[pin(p) for p in references]
details=read(L/'r507-contract-boundaries.json')
handoff=dict(id='R507-modern-owned-Java-directory-and-synthetic-startup-host-Back-timeout',state='FINITE_NEW_CHECKS_PASSED_ACTUAL_JAVA_SOURCE0_DEPENDENCY_OPEN',functional_checks=8,boundary_observation_checks=1,historical_R506_PASS_reruns=0,unchanged_completed_modes='16/40',GUI=0,player_EXE_executions=0,original_Native_GUI=0,existing_provider_operations=0,server_namespace_writes=0,
 results={'Java_generated_S_FB':[dict(case=e['case'],rooms=e['details']['rooms'],servers=e['details']['servers'],body_bytes=e['details']['body_bytes']) for e in first[:3]],'new_bounds':first[3]['details'],'host_Back':host['trace'],'timeout':timer['trace'],'synthetic_actual_socket_TX':30,'actual_Java_native_socket_TX':2,'Java_native_auth':'actual separate pinned Java Main accepted synthetic version/account in isolated SQL; masked LOGIN capture','full_Java_login_channel_path':False},
 input_scope={'root_parent':'D:/lito-dev-runtime/20261003-supervision/modern','unique_SQL_runtimes':4,'temporary_directories_and_child_processes':'checked main + explicit tmpdir + minimal TEMP/TMP + owned datadir/logs/profile/state/captures; all physical datadir IDs observed','all_four_native_servers_stopped_normally':True,'all_four_mysql_stopped':True,'all_owned_listeners_released':True,'successful_deletion_disabled':'fixtures deliberately omit successful(); retained SQL data, no recursive delete or refused-cleanup retry'},
 production={'source_count':len(prod['source_pins']),'changed_from_R506':prod['changed'],'snapshot':pin(L/'r507c-production-source-snapshot.json'),'difference':pin(L/'r507c-source-difference.patch'),'library':player['production_library'],'separate_player':pin(L/'r507c-player-candidate.json'),'new_entry_flag':'--candidate-synthetic-startup <absolute fixture profile> <absolute separate startup host>','fixed_consumers':consumers},
 Native_proof_vs_Modern_adaptation=pin(L/'r507-contract-boundaries.json'),
 precise_remaining={'first_actual_java_blocker':'source0 S_LOGIN_CHECK accepted body26 has no qualified Native receiver/admission in Rust; exact Java producer bytes retained','next_bootstrap':'source59 remains unsupported; C8 directory selection/GAME fresh auth/NEWS E2 not attempted','actual_64AD74':'private050104CD runtime return UNKNOWN; separate explicit synthetic host input does not qualify historical Native/T73/provider initialization','GUI':'0; copied source buttons/shared commands compiled and headless exercised; real rendering/clicks not accepted','timer':'only Modern monotonic strict >3s/>10s qualified; Native QPC/u32 wrap, hostname DNS and no-host fallback remain OPEN','full_gameplay':'16/40 unchanged; full room/battle/actor/world paths not added'},
 failed_attempts={'first':'bounded RX expansion originally created inline65533byte stack array causing0xC00000FD before flow; fixed as heap box','later':'preserved fixed harness failures: Expect before causal Send and stale previous write_complete before polling new C8. Source library unchanged after heap fix; only remaining unpassed host tests resumed','untested_compiles':['r507 fixed pre-Window hook','r507b stack-array candidate','r507t/r507u timeout preparation candidates']},
 preserved_R506=pin(L/'r506-native-transfer-channel-handoff.json'),owned_retirement=retirement,evidence=files)
out=L/'r507-native-java-host-boundary-handoff.json';save(out,handoff)
(P/'project').mkdir(exist_ok=True)
textfile=P/'project/Native-R507_Java隔離與返回路徑交接.txt';textfile.write_text('R507：modern 自有 Java 隔離與返回路徑\n\n新增8項功能驗證與1項真Java依賴觀察；R506歷史10 PASS未重跑。GUI0，玩家EXE0，完整模式仍16/40。\n\nJava Protocol真實產生6列、10列、127房間+10server最大37136byte S_FB，production Rust解析與chained cipher/LE16 framing通過。155byte／5列不再錯拒整份目錄；五列只限制可見UI，完整接收snapshot保留所有server列。接收buffer改為有界heap，避免inline大array造成stack overflow。Native證據與Modern allocation/strict trailing/fit/name/scene守門分列。\n\n獨立、不可變、明示synthetic的startup host與FixtureProfile.endpoint及64AD54角色名稱分離。Channel Back與lobby Return B8two送完舊socket c15後才new epoch/host:2000/new seed，依真3/203只重送保存alias一次；C8/CA100/C9及正常Close通過。兩個strict10s timeout方向真socket/seed通過，BE保留，四IO owner正常join；不宣稱Native QPC/u32 wrap或private050104CD值。\n\n四個新的modern ASCII runtime均保留資料：physical @@datadir和Win32 file identity核對；自有Java正常Ctrl+C停止、MySQL停止、所有listener退休；沒有server namespace寫入、既有provider操作、遞迴清理或被拒cleanup重試。所有首次與後續失敗候選及實際退出證據保留。\n\n真實獨立Java Main確實接受version與synthetic帳號登入。Rust第一個未支援封包是opcode0 S_LOGIN_CHECK accepted（26byte body）；Java接著送source59。未翻譯LTM1／未丟包，未選source C8／未做GAME fresh auth或NEWS E2，完整Java登入到channel仍OPEN。private050104CD runtime return仍UNKNOWN，synthetic startup adapter不冒稱原版T73 host producer。\n\nJSON交接：'+str(out)+'\n',encoding='utf-8')
verification=L/'r507-sealed-link-verification.json';save(verification,dict(handoff=pin(out),human_handoff=pin(textfile),references_verified=len(files),source_and_artifact_checks=checks,source_count=len(prod['source_pins']),R506_preserved=True,GUI=0))
v=read(P.parent/'監製進度.json');v.update(updated_at=datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).isoformat(),phase='R507_new_finite_paths_verified_actual_Java_source0_dependency_open',current_task='R507新增8項S_FB／獨立host Back／timeout PASS；真Java版本與合成SQL登入接受，Rust source0 LOGIN_CHECK接線仍待原版資格；GUI0／整體16of40')
v['r506']['status']='finite_modern_verified_scope_pending_withdrawn_followup_R507';v['r507'].update(state=handoff['state'],scope_pending_withdrawn=True,functional_checks=8,boundary_observation_checks=1,synthetic_actual_TX=30,actual_Java_TX=2,Java_fixture_executions=4,actual_Native_auth_attempts=1,all_owned_retired=True,handoff=pin(out),GUI=0,complete_modes=16,total_modes=40)
save(P.parent/'監製進度.json',v)
print(json.dumps({'handoff':pin(out),'source_checks':checks,'functional_checks':8,'Java_source0_open':True},ensure_ascii=False))
