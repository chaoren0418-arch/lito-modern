from r491_common import *
import shutil
inputs=read(L/'r508c-java-inputs.json');runtime=Path(inputs['runtime']);execution=read(L/'r508c-integration-exit.json');assert execution['consumer_exit']==0 and execution['all_owned_retired'] and execution['all_owned_listeners_released']
assert execution['native_normal_stop']['forced_termination'] is False and execution['native_normal_stop']['client_sessions_quiescent'] is True
events=read(L/'r508c-event-details.json');memory=read(L/'r508-memory-results.json');assert len(events)==2 and all(e['passed'] for e in events) and memory['passed']==5
consumer=read(L/'r508c-consumer-candidate.json');player=read(L/'r508q-player-candidate.json');assert consumer['production_library']['sha256']==player['production_library']['sha256']
for v in [consumer,player]:
 for item in v.values():
  if isinstance(item,dict) and all(k in item for k in ['path','sha256','bytes']):assert pin(item['path'])==item,item
snapshot=read(L/'r508c-production-source-snapshot.json')
for item in snapshot['source_pins']:assert pin(item['path'])['sha256']==item['sha256']==pin(item['snapshot'])['sha256']
for item in read(L/'r508c-java-execution-inputs.json')['files']:assert pin(item['path'])==item,item
captures=list((runtime/'native-captures').rglob('frames.jsonl'));assert len(captures)==1
frames=[json.loads(s) for s in captures[0].read_text(encoding='utf-8').splitlines()];groups=[]
for frame in frames:
 if frame['direction']=='server_to_client' and frame['opcode']=='0x01':groups.append([])
 groups[-1].append(frame)
assert len(groups)==4
seeds=[];records=[]
for i,group in enumerate(groups):
 dirs=[(e['direction'],e['opcode']) for e in group]
 S='server_to_client';C='client_to_server'
 wanted=[(S,'0x01'),(C,'0x00'),(S,'0x3A'),(C,'0x01')]+([(S,'0x00'),(S,'0x3B'),(S,'0xFB'),(S,'0xF3'),(C,'0xC8'),(S,'0xF4')] if i%2==0 else [(S,'0xE6'),(C,'0xE2'),(S,'0x03'),(S,'0xCB')])
 assert dirs==wanted,dirs
 seed=bytes.fromhex(group[0]['plain'])[1:];seeds.append(seed.hex())
 for e in group:
  if e['plain']!='MASKED_C_LOGIN':assert e['wire_length']==2+len(bytes.fromhex(e['plain']))
  else:assert e['direction']==C and e['wire_length']==26
 if i%2==0:
  assert bytes.fromhex(group[4]['plain'])==bytes.fromhex('0000100e00004c69546f20332e383143204c6f63616c00000000')
  directory=bytes.fromhex(group[5]['plain']);assert directory[:10]==bytes.fromhex('3b025769726553756e00');assert directory[10:14]==bytes.fromhex('7f000002') and int.from_bytes(directory[14:16],'little')==execution['ready']['port']
  source=['WireMoon','WireSun'][i//2];assert bytes.fromhex(group[8]['plain'])==bytes([200])+source.encode()+b'\0'
  f4=bytes.fromhex(group[9]['plain']);assert f4==bytes.fromhex('f4007f000002')+execution['ready']['port'].to_bytes(2,'little')
 else:
  assert bytes.fromhex(group[4]['plain'])==b'\xe6' and bytes.fromhex(group[5]['plain'])==b'\xe2' and bytes.fromhex(group[6]['plain'])==b'\x03\x01'
  record=bytes.fromhex(group[7]['plain']);records.append(record[1:record.index(0)].decode('ascii'))
assert len(set(seeds))==4 and records==['WireSend(WireMoon)','WireRecv(WireSun)']
actual_tx=sum(e['direction']=='client_to_server' for e in frames);assert actual_tx==12
save(L/'r508-actual-capture-verification.json',dict(capture=pin(captures[0]),groups=4,new_seed_words_LE=seeds,actual_TX_frames=actual_tx,actual_RX_frames=len(frames)-actual_tx,exact_sequence_per_socket_verified=True,actual_LOGIN_CHECK_bytes_match_sealed_golden=True,current_ephemeral_directory_and_F4_port_verified=execution['ready']['port'],received_character_aliases=records,C_LOGIN_masking_preserved=True,capture_limitation='Server capture records decoded bodies and wire lengths, not ciphertext bytes. Java accepted VERSION/LOGIN and produced matching results; no new ciphertext byte-oracle claim.',GUI=0))
adoption=read(L/'r508-input-adoption.json');final=read(L/'r508-final-producer-pin-correction.json');assert pin(final['final_handoff']['path'])==final['final_handoff']
sources=[pin(p) for p in sorted((L/'r508-source-fragments').glob('*'))]
boundaries={
 'Native_source0':'Fixed dispatcher ordinary0 ->4903D0 cdsch. Version>=5226 reads additional DWORD; result0 returns without source phase or ACK. Runtime parses named fields and refuses missing extension by default; locale/private globals/modal failure branches are not emulated.',
 'Java_explicit_adapter':'Requires separately loaded kind plus exact AD368 JAR and final producer-contract SHA marker, configured before epoch with synthetic profile. The runner independently pins the actual copied JAR; marker alone is not Java binary verification. Adapter permits absent Native extension without inventing its value; no Native phase is inferred.',
 'Native_source59':'48FAA0 signed count, CString row <=63 encoded bytes, dLE IPv4 and hLE port,72-byte Native row. Zero count leaves current rows; negative unsafe allocation and trailing bytes refused by Modern. 48F580 uses selected received source name in cs C8. Native selected==count bug is refused. Private cached flags/C35 and BC/BD auto paths remain unqualified.',
 'namespaces':'3B source rows and raw selected source name are distinct from FB VS endpoints/room groups and selected-character64AD54 alias. Source selection does not create a selected character or overwrite that alias.',
 'GAME_redirect':'Actual AD368 F4 after completed source C8 reconnects to owned nonzero loopback target with fresh epoch, seed, VERSION and same synthetic-account LOGIN. This explicitly scoped Java bootstrap differs from Native selected-character F4 D9/c15/BC replay; that existing R506/R507 route is retained.',
 'credentials':'Adapter-only private Zeroizing lease is copied from staged input for one GAME LOGIN; original form is wiped at initial write completion. Lease cleared at GAME LOGIN write completion, failure/disconnect/Close; no credentials in public key or capture.',
 'NEWS':'Fixed48F500 private62C224 controls immediate E2 vs modal48F550. Private flag remains unqualified. Java adapter offers a current-epoch/revision continuation control after E6 and completed GAME LOGIN; shared command sends c E2 once, then requires actual3/count rows. No delay/fake ACK, guessed ready or skills-owner readiness.',
 'ownership':'Old source IO canceled and retired on new epoch; no old queued RX remainder transferred to GAME. Source/news tokens and pending writes reject foreign/stale/duplicate events. OpenLogin cannot overwrite active bootstrap. Four actual Rust IO owners all joined; two additional foreign-control runtimes were offline-only and closed.',
 'GUI':'Production Window source combo/submit and NEWS continue use the same opaque shared commands. R508Q candidate linked to identical library as tested consumer; zero player executions and GUI0, no visual/native GUI qualification.',
 'complete_scope':'Seven finite new checks: five Rust parser cases and two actual AD368+isolatedSQL LOGIN/source-choice/GAME/NEWS/one-character-roster flows. No C_C9 game entry, actor/world/room/battle/skills/provider/private startup/locale/cached-C35 completion. Existing16of40 remains unchanged.'}
save(L/'r508-contract-boundaries.json',boundaries)
refs=[pin(L/n) for n in ['r508-input-adoption.json','r508-final-producer-pin-correction.json','r508-source-study-initial.json','r508-contract-boundaries.json','r508c-production-source-snapshot.json','r508c-source-difference.patch','r508c-library-build.jsonl','r508c-library-build.stderr.log','r508q-player-candidate.json','r508q-player-build.stderr.log','r508c-consumer-candidate.json','r508c-consumer-build.stderr.log','r508-memory-results.json','r508c-java-inputs.json','r508c-java-execution-inputs.json','r508c-integration-exit.json','r508c-event-details.json','r508-actual-capture-verification.json','r508-static-player-first-failure.json']]
owned=[pin(p) for p in sorted(runtime.rglob('*')) if p.is_file() and (p.suffix in ['.json','.jsonl','.log','.properties','.request'] or p.name=='native.state') and 'lito-modern-mysql-fixtures' not in p.parts and 'db' not in p.parts]
tools=[pin(p) for p in sorted((P/'tools').glob('*r508*')) if p.is_file()]+[pin(P/'tools/R508NativeFixtureMain.java')]
handoff=dict(schema='modern.R508.login-source-GAME-finite-handoff.v1',updated_at=datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).isoformat(),state='FINITE_7_NEW_CHECKS_PASS_NATIVE_GUI_SKILLS_OPEN',new_passes=7,parser_checks=5,actual_java_flows=2,actual_TX_frames=12,actual_Rust_socket_owners_started=4,actual_Rust_socket_owners_joined=4,all_owned_Java_SQL_and_listeners_retired=True,owned_runtime=str(runtime),sealed_producer=final['final_handoff'],announced_old_pin_corrected=final,producer_manifest_payloads=127,previous_R507=pin(L/'r507-native-java-host-boundary-handoff.json'),previous_R506=pin(L/'r506-native-transfer-channel-handoff.json'),historical_R506_10_PASS_reruns=0,historical_R507_8_PASS_reruns=0,GUI=0,player_executions=0,overall_maps='16/40 unchanged',production_source_count=len(snapshot['source_pins']),changed_source_files=snapshot['changed'],same_player_consumer_library_SHA=consumer['production_library']['sha256'],consumer=consumer,player=player,boundaries=boundaries,references=refs,native_source_fragments=sources,owned_runtime_evidence=owned,tools=tools,pending_icon_handoff={'path':r'D:/lineage開發/3-lito開發/lito脫殼資源/native-icon-draw-contract.json','sha256':'9BA690374734FECFE86CF95651E864515FFFDC78D71FE58C556974625AD95803','adopted':False,'next':'Targeted icon draw consumption after this finite R508 handoff; no wire/Java changes'},next='Preserve R508 finite results; consume sealed icon draw differences separately. Native cached-source/UI/private startup and later C_C9/world/skills remain separate qualification tasks.')
save(L/'r508-login-source-game-handoff.json',handoff)
text=P/'project/Native-R508_Login來源GAME整合交接.txt';text.parent.mkdir(exist_ok=True);text.write_text('R508：新增7項PASS（Rust欄位5、真實Java來源→GAME流程2）。\nWireSend/WireMoon與WireRecv/WireSun均走完LOGIN→0→3B→FB→F3→C8→F4→fresh GAME seed→VERSION→LOGIN→E6→一次E2→S03+S_CB。實際TX12frames、Rust socket owners4/4joined；自有Java/MySQL正常退出與監聽埠釋放。\nAD368與Native版本>=5226額外DWORD、來源F4及NEWS private flag差異皆為明確synthetic adapter，不能稱Native全流程等同。GUI0、候選玩家執行0、16/40不变；C_C9/world/room/battle/skills/private startup/cached C35仍開放。\n詳見'+str(L/'r508-login-source-game-handoff.json')+'\n圖示交接已記錄待下一階段針對性採用；未改shared BIN或Java產品。\n',encoding='utf-8')
v=read(P.parent/'監製進度.json');v.update(updated_at=datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).isoformat(),phase='R508_finite_LOGIN_source_GAME_roster_7_new_PASS_sealed_GUI0',current_task='R508已封存：新Rust欄位5+真實AD368來源GAME名單2 PASS；12TX、4/4IOjoined，自有JavaSQL及埠正常退休；GUI0／16of40，待圖示針對性消費')
v['r508'].update(state='finite_7_new_checks_sealed',handoff=pin(L/'r508-login-source-game-handoff.json'),actual_java_flows=2,new_passes=7,actual_TX_frames=12,GUI=0,all_owned_retired=True,historical_PASS_reruns=0);save(P.parent/'監製進度.json',v)
print(json.dumps({'handoff':pin(L/'r508-login-source-game-handoff.json'),'sources':len(snapshot['source_pins']),'new_PASS':7,'actual_TX':12,'all_owned_retired':True},ensure_ascii=False))
