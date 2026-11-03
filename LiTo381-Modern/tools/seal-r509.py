from r491_common import *
inputs=read(L/'r509b-java-inputs.json');runtime=Path(inputs['runtime']);execution=read(L/'r509-integration-exit.json');events=read(L/'r509-event-details.json');memory=read(L/'r509-memory-results.json')
assert execution['consumer_exit']==0 and execution['all_owned_retired'] and execution['all_owned_listeners_released']
assert execution['native_normal_stop']['forced_termination'] is False and execution['native_normal_stop']['client_sessions_quiescent'] and execution['SQL_retirement']['mysql_retired']
assert memory['passed']==5 and len(events)==4 and all(e['passed_new_finite_boundary'] for e in events)
consumer=read(L/'r509b-consumer-candidate.json');player=read(L/'r509-player-candidate.json');assert consumer['production_library']['sha256']==player['production_library']['sha256']
for v in [consumer,player]:
 for e in v.values():
  if isinstance(e,dict) and all(k in e for k in ['path','bytes','sha256']):assert pin(e['path'])==e
snapshot=read(L/'r509-production-source-snapshot.json')
for e in snapshot['source_pins']:assert pin(e['path'])['sha256']==e['sha256']==pin(e['snapshot'])['sha256']
for e in read(L/'r509-java-execution-inputs.json')['files']:assert pin(e['path'])==e
captures=list((runtime/'native-captures').rglob('frames.jsonl'));assert len(captures)==1
frames=[json.loads(s) for s in captures[0].read_text(encoding='utf-8').splitlines()];groups=[];S='server_to_client';C='client_to_server'
for frame in frames:
 if frame['direction']==S and frame['opcode']=='0x01':groups.append([])
 groups[-1].append(frame)
assert len(groups)==8
seeds=[];sequences=[]
for i,g in enumerate(groups):
 seq=[(e['direction'],e['opcode']) for e in g];seed=bytes.fromhex(g[0]['plain'])[1:];seeds.append(seed.hex())
 wanted=[(S,'0x01'),(C,'0x00'),(S,'0x3A'),(C,'0x01')]
 if i%2==0:wanted += [(S,'0x00'),(S,'0x3B'),(S,'0xFB'),(S,'0xF3'),(C,'0xC8'),(S,'0xF4')]
 else:
  case=i//2;count=[0,2,1,1][case];wanted += [(S,'0xE6'),(C,'0xE2'),(S,'0x03')]+[(S,'0xCB')]*count
  assert bytes.fromhex(g[6]['plain'])==bytes([3,count])
  if count:
   wanted += [(C,'0xC9'),(S,'0xC8'),(S,'0x7C'),(C,'0xCA'),(S,'0xC9')]
   if case!=3:wanted += [(S,'0xCA'),(S,'0x0C'),(S,'0x4D'),(S,'0x50'),(S,'0x51'),(S,'0x52'),(S,'0x56')]
   start=7+count;sent=bytes.fromhex(g[start]['plain']);record=bytes.fromhex(g[7+(1 if case==1 else 0)]['plain']);raw=record[1:record.index(0)]
   assert sent==bytes([201])+raw+b'\0';assert bytes.fromhex(g[start+1]['plain'])==b'\xc8\0'
   channel=[None,2,1,0][case];assert bytes.fromhex(g[start+3]['plain'])==bytes([202,channel]);assert bytes.fromhex(g[start+4]['plain'])== (bytes([201,10,0]) if case==3 else bytes([201,0,channel])+bytes(10))
   if case==1:
    assert [bytes.fromhex(e['plain'])[1:bytes.fromhex(e['plain']).index(0)].decode('cp950') for e in g[7:9]]==['測試乙','測試甲'];assert raw=='測試甲'.encode('cp950')
    assert bytes.fromhex(g[7]['plain'])[1:7]==bytes.fromhex('b4fab8d5a441')
   if case!=3:
    assert bytes.fromhex(g[start+5]['plain'])==bytes([202,channel,1,0,100,0]);assert len(bytes.fromhex(g[start+6]['plain']))==37
    assert events[case]['details']['first_following_unsupported_opcode']==12 and events[case]['details']['first_following_body_bytes']==37
 else_source=None
 assert seq==wanted,seq
 for e in g:
  if e['plain']=='MASKED_C_LOGIN':assert e['direction']==C
  else:assert e['wire_length']==2+len(bytes.fromhex(e['plain']))
 assert all(e['opcode']!='0x1E' for e in g)
 assert all(not (e['direction']==C and e['opcode']=='0xCA' and bytes.fromhex(e['plain'])==bytes([202,100])) for e in g)
 sequences.append(dict(socket_index=i,role='LOGIN' if i%2==0 else 'GAME',case=events[i//2]['case'],sequence=seq,seed_word_LE=seed.hex()))
assert len(set(seeds))==8
tx=sum(e['direction']==C for e in frames);assert tx==30
io=sum(e['after_Close']['workers_started'] for e in events);joined=sum(e['after_Close']['workers_joined'] for e in events);assert io==joined==8
save(L/'r509-actual-capture-verification.json',dict(capture=pin(captures[0]),actual_TX_frames=tx,actual_RX_frames=len(frames)-tx,sequences=sequences,eight_distinct_seeds=True,actual_MS950_A441_乙_verified=True,selected_multiple_row='actual source order [測試乙,測試甲]; slot1 echoes 測試甲 A5D2, no name/order-to-objectID derivation',no_auto_CA100=True,no_initial_S1E=True,later_statuses='Java captures 0C/4D/50/51/52/56 after entry; Runtime stops at first 0C/body37, later packets are not admitted',capture_limit='Decoded bodies and wire lengths, not ciphertext byte oracle; C_LOGIN remains masked',GUI=0))
live=read(runtime/'r509-owned-live.json');assert len(live['owners'])==4 and len(live['listeners'])==3
save(L/'r509-owner-identity-summary.json',dict(owners=[{k:e[k] for k in ['ProcessId','ParentProcessId','ExecutablePath','CreationDate']} for e in live['owners']],exact_GetProcessTimes_FILETIME={'helper':execution['helper'],'native_main':execution['native_normal_stop']['creation_filetime_utc']},other_creation_precision='MySQL and Python launcher CreationDate came from CIM at microsecond precision; exact100ns FILETIME not additionally sampled for those two; no process replay to fill this diagnostic gap',datadir=execution['ready']['physical_datadir'],datadir_identity=pin(runtime/'physical-datadir-id.json'),owned_live=pin(runtime/'r509-owned-live.json'),owned_retirement=pin(runtime/'r509-owned-retirement.json'),IO_started=io,IO_joined=joined))
boundaries={
 'typed34':'S03 count plus34wire fields (opcode,two rawMS950 names and31typed numeric fields). Signed lawful/AC/movement retain original bits; five-rank order differs from Java constructor. Runtime keeps epoch+rosterRevision and raw record180; display conversion does not replace C9 alias bytes.',
 'count_policy':'Native4 controls/reset sentinels remain qualified bound. Explicit Java adapter permits8 roster rows via Modern combo; structural count8 is not proof of original Native8 storage/UI. Actual Java tests cover0/1/2 rows; no actual8-row Java execution claim.',
 'selection':'User opaque current-epoch/revision choice then Submit sends received rawname C_C9 once; current completed write -> S_C8 accepted -> S7C receipt -> enabled user C_CA0..3. No automatic CA100. Foreign/stale/duplicate/retired controls reject. Numeric/name/ordinal fields never become objectID/sourceID.',
 'inventory':'S7C exact unsignedcount and dcchcdcsc row layout, CString256 and byte detail bound validated. Item wireID stays item-only. Actual AD368 zero andone-paper inventories received; original648EBC inventory owner443D10/447580 not installed or qualified.',
 'channel':'User CA2/CA1 -> matching ordinary S_C9 result0 plus10zero reserved bytes -> matching S_CA6-byte cchh receipts. User CA0 lowlevel10 -> actual S_C9 result10, no S_CA/ready. S_C8 refusal1/7/8 implemented with epoch retirement but not newly executed against actualJava here.',
 'first_remaining_boundary':'Both successful channel receipts followed by actual opcode0C length37. Runtime preserves first unsupported reason and pre-retirement C9/inventory/channel checkpoint; it retires current IO rather than skips packets. No Native lobby state owner/actor205/world/skills readiness inferred from selection, sceneCHANNEL, E2,C9,S_C8 or S_CA.',
 'return_reconnect':'Reviewed contract2FBFC3... pinned; this milestone adopts ordinary manual entry only. CA100 LOBBY+NONE F5->C8->S03/CB, CHANNEL ignored; 99 needs remembered ordinary channel. C15=0F, D0 reconnect30s refresh retains deadline. Full return/reconnect work queued, not implemented by these new flows.',
 'Native_Java':'AD368 login-check longopcode0 missingNative4-byte tail remains explicit adapter, strictNative validator unchanged. Conditional original readers/private dispatch are not universally emulated. No native crash/cipher-desync inference.',
 'failure_preservation':'Initial frozen consumer failed: fixturehex A4B9 was允 although expectation乙. Failedcandidate and observed metadata retained; independent PythonCP950 plus real AD368 A441 and current WindowsMS950 conversion agree. Corrected fixture adds both independent vectors; productiondecoder unchanged. Initial unsealed partialchecks are not counted as historical PASS reruns.',
 'GUI':'ProductionWindow combo/details/sharedcommands compile. Player candidate and testedconsumer link exactsame productionrlib, playerexecutions0/GUI0. No screenshots/NativeGUI/liveprivate owners qualified.',
 'scope':'9finite new checks =5memory groups+4actualJavaSQL flows. LOGIN/GAME setup reused necessary contract, no historical acceptance suite replay. 623production files,8changed fromR508Z. Overall16/40 unchanged; retired runtime never reused.'}
save(L/'r509-contract-boundaries.json',boundaries)
refs=[pin(L/n) for n in ['r509-input-adoption.json','r509-inventory-channel-AD368-IL.json','r509-production-source-snapshot.json','r509-source-difference.patch','r509c-library-build.jsonl','r509c-library-build.stderr.log','r509-player-candidate.json','r509b-consumer-candidate.json','r509b-consumer-build.stderr.log','r509-memory-results.json','r509-ms950-failed-vector.json','r509-owned-fixture-adaptation.json','r509b-java-inputs.json','r509-java-execution-inputs.json','r509-integration-exit.json','r509-event-details.json','r509-actual-capture-verification.json','r509-owner-identity-summary.json','r509-contract-boundaries.json','r509-additional-dependencies.json']]
owned=[pin(p) for p in sorted(runtime.rglob('*')) if p.is_file() and (p.suffix in ['.json','.jsonl','.log','.properties','.request'] or p.name=='native.state') and 'lito-modern-mysql-fixtures' not in p.parts and 'db' not in p.parts]
toolpins=[pin(p) for p in sorted((P/'tools').glob('*r509*')) if p.is_file()]+[pin(P/'tools/R509NativeFixtureMain.java')]
handoff=dict(schema='modern.R509.typed-character-selection-channel-finite.v1',updated_at=datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).isoformat(),state='FINITE_9_NEW_CHECKS_PASS_FIRST_0C37_OWNER_BOUNDARY_PRESERVED',new_passes=9,memory_groups=5,actual_java_flows=4,actual_TX_frames=tx,actual_RX_frames=len(frames)-tx,actual_Rust_IO_started=io,actual_Rust_IO_joined=joined,all_owned_Java_SQL_and_listeners_retired=True,owned_runtime=str(runtime),production_source_count=len(snapshot['source_pins']),changed_source_files=snapshot['changes'],same_player_consumer_library_SHA=consumer['production_library']['sha256'],player=player,consumer=consumer,references=refs,boundaries=boundaries,owned_runtime_evidence=owned,tools=toolpins,previous_R508=pin(L/'r508-login-source-game-handoff.json'),previous_R508Z=pin(L/'r508-zero-directory-guard-addendum.json'),GUI=0,player_executions=0,historical_PASS_reruns=0,overall_maps='16/40 unchanged',next='Consume queued BIN icon/pane draw and color initialization batch in a separate finite milestone; full return/reconnect,0C owner,actor/world/skills andNativeGUI remain open.')
save(L/'r509-character-selection-channel-handoff.json',handoff)
report=P/'project/Native-R509_角色選取頻道交接.txt';report.write_text('R509有限封存：新增9項驗證通過（記憶體5組＋實際AD368 Java/SQL 4流程）。\n全部寫入Modern自有目錄；保留canonical/tmpdir/datadir/PID/建立時間/loopback/deadline/退場guard。原MS950 fixture錯寫A4B9(允)，乙應為A441；失敗候選保留，未改decoder。實際producer兩列順序為測試乙、測試甲；選slot1回送測試甲原bytes。\n零角色；雙中文角色C9→C8成功→S7C0→手選CA2→S_C9→S_CA；WireSend單別名S7C1→手選CA1；低等級CA0實際拒絕10。成功兩流程下一封包為0C/body37，保留第一未支援邊界；不能稱完整大廳或skillsready。\n實際TX30、RX'+str(len(frames)-tx)+'、Rust IO8/8 joined，自有Java/MySQL/launcher全部正常退休、3listeners釋放；未重播歷史PASS。固定623來源8變更，玩家與consumer同rlib，玩家執行0、GUI0、16/40不變。\n返回/重連列後續，接續有限BIN圖像批次。詳見'+str(L/'r509-character-selection-channel-handoff.json')+'\n',encoding='utf-8')
q=P.parent/'監製進度.json';v=read(q);v.update(updated_at=handoff['updated_at'],phase='R509_finite9_sealed_first0C37_owner_open_GUI0',current_task='R5099項新驗證封存：選角及手選頻道receipt、0C37邊界保留；30TX、8/8IOjoined、JavaSQL及埠已退休。接續有限BIN圖像批次；返回重連與技能owner仍開放。');v['r509'].update(state=handoff['state'],handoff=pin(L/'r509-character-selection-channel-handoff.json'),new_passes=9,actual_java_flows=4,actual_TX_frames=tx,IO_joined=joined,all_owned_retired=True,GUI=0,historical_PASS_reruns=0);save(q,v)
print(json.dumps(dict(handoff=pin(L/'r509-character-selection-channel-handoff.json'),new_PASS=9,actual_TX=tx,actual_RX=len(frames)-tx,IO_joined=joined,GUI=0),ensure_ascii=False))
