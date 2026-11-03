from r491_common import *
import struct
def same(q):
 actual=pin(q['path']);assert (actual['bytes'],actual['sha256'])==(q['bytes'],q['sha256']),q['path'];return actual
production=read(L/'r506c-production-source-snapshot.json');consumer=read(L/'r506c-transfer-consumer-candidate.json');player=read(L/'r506c-player-candidate.json')
for q in production['source_pins']+consumer['source_pins']:
 same(q);assert pin(q['snapshot'])['sha256']==q['sha256']
old=read(L/'r505-production-source-snapshot.json')
for q in old['source_pins']:assert pin(q['snapshot'])['sha256']==q['sha256']
first=read(L/'r506-first-attempt/r506-transfer-consumer-candidate.json')
for q in first['source_pins']:assert pin(q['snapshot'])['sha256']==q['sha256']
for q in [first,first['production_library'],consumer,consumer['production_library'],player['executable'],player['production_library']]:same(q)
assert consumer['production_library']==player['production_library']
assert player['candidate_executions']==0 and player['GUI']==0
final_exit=read(L/'r506c-transfer-consumer-exit.json');assert final_exit['exit_code']==0 and final_exit['executions']==1 and final_exit['historical_PASS_reruns']==0
assert read(L/'r506-first-attempt/r506-transfer-consumer-exit.json')['exit_code']!=0
events=read(L/'r506c-event-details.json');assert len(events['flows'])==10 and all(f['passed']for f in events['flows']);assert events['actual_socket_TX_frames']==26 and events['IO_workers_joined']==5 and events['endpoint_threads_joined']==5 and events['Runtime_Closed']==3
assert events['GUI']==0 and events['server_Java_fixture_executed']==False
adoption=read(L/'r506-API-adoption.json');data=Path(adoption['originals'][1]['path']).read_bytes();asm=Path(adoption['originals'][0]['path']).read_text(encoding='utf-8-sig').splitlines()
for q in adoption['originals']:same(q)
asm_checks=[];pe_checks=[]
for root in [L/'r506-source-fragments',L/'r506-host-source-fragments']:
 for p in root.glob('*'):
  lines=p.read_text(encoding='utf-8').splitlines()
  if p.name.startswith('asm-'):
   for line in lines:n,s=line.split(': ',1);assert asm[int(n)-1]==s
   asm_checks.append(pin(p))
  else:
   va=int(lines[0].split()[1],16);raw=bytes.fromhex(lines[1]);assert data[va-0x400000:va-0x400000+len(raw)]==raw;pe_checks.append(pin(p))
#The earlier760F0 slice starts inside the previous MOV immediate. It is raw
#byte evidence, not an instruction boundary. Full475F40 gives the valid calls.
assert data[0x476118-0x400000:0x476118-0x400000+5]==bytes.fromhex('E83F9D0700')
assert data[0x476095-0x400000:0x476095-0x400000+4]==bytes.fromhex('3BC87F3B')
save(L/'r506-fragment-boundary-note.json',dict(raw_slice=pin(L/'r506-source-fragments/pe-004760F0.txt'),qualified_function=pin(L/'r506-source-fragments/pe-00475F40.txt'),note='760F0 starts inside MOV immediate; its linear disassembly must not be treated as instruction authority; actual gethostbyname CALL is476118, signed capacity CMP/JG is476095/476097; raw bytes independently matched',runtime_different_UIrow_host_lookup_adopted=False,GUI=0))
table=struct.unpack('<'+'I'*60,data[0x630B20-0x400000:0x630B20-0x400000+240]);channel=[]
for slot in range(4):
 row=table[slot*15:(slot+1)*15];assert row[0]==2 and row[1]==7581+slot*2 and 0x492FA0 in row and slot in row
 channel.append(list(row))
resources=read(L/'r506-channel-resources.json')
for m in resources['members']:
 for field in ['raw','reference','reference_png']:same(m[field])
 raw=Path(m['raw']['path']).read_bytes();w,h,flag,key=struct.unpack('<hhHH',raw[:8]);rgba=bytearray()
 for(v,)in struct.iter_unpack('<H',raw[8:]):
  if m['id']!=7580 and v==key:rgba.extend([0,0,0,0])
  else:rgba.extend([((x<<3)|(x>>2))for x in [(v>>10)&31,(v>>5)&31,v&31]]+[255])
 assert bytes(rgba)==Path(m['reference']['path']).read_bytes() and (w,h)==(m['width'],m['height'])
host=read(L/'r506-host-alias-adoption.json');same(host['host_contract']);same(host['conversion_vectors'])
for va,dest in [(0x480BC1,0x64B7E4),(0x480C77,0x64B7E4),(0x481861,0x64B7E4),(0x481DDC,0x64B7E4),(0x47C9D0,0x64AD74),(0x492F42,0x64AD54)]:assert data[va-0x400000:va-0x400000+5]==b'\xbf'+dest.to_bytes(4,'little')
report=dict(state='R506_MODERN_FINITE_NEW_CHANNEL_PATH_VERIFIED_NOT_OVERALL_ACCEPTANCE',production_pairs=len(production['source_pins']),consumer_pairs=len(consumer['source_pins']),baseline_frozen_sources_checked=len(old['source_pins']),failed_attempt_frozen_pairs_checked=len(first['source_pins']),source_changed=production['changed'],source_unchanged=production['unchanged'],asm_fragments_checked=len(asm_checks),PE_fragments_checked=len(pe_checks),asm_fragments=asm_checks,PE_fragments=pe_checks,channel_table_rows=channel,resources_checked=len(resources['members']),event_details=pin(L/'r506c-event-details.json'),exit=pin(L/'r506c-transfer-consumer-exit.json'),elapsed_seconds=events['elapsed_seconds'],max_poll_us=max(f['details'].get('max_poll_us',0)for f in events['flows']),source_snapshot=pin(L/'r506c-production-source-snapshot.json'),player=pin(L/'r506c-player-candidate.json'),consumer=pin(L/'r506c-transfer-consumer-candidate.json'),historical_PASS_reruns=0,GUI=0,server_Java_fixture_executions=0)
save(L/'r506-independent-contract.json',report)
textfile=P/'Native-R506_頻道切換與角色名稱重送交接.txt'
textfile.write_text('R506 modern 有限路徑交接，整體未完成。\n\n已核對 API 身分及六個 host/alias 複製位置；歷史 DLL bytes、實際 T73 host、私有前置呼叫仍未證實。64AD54 名稱容量28 bytes；64AD74 startup host 分離，未使用 fixture endpoint 代填。\n\n固定 R506C production library + 未執行 player candidate + 單次 headless consumer：10新情境，26 actual socket TX，5IO/5endpoint join，3 RuntimeClosed；same F4、received S_FB 第二列、新 socket F4/alias replay、channel/lobby、signed capacity、timeout/Close。資料與來源 pin 见 logs/continuation-20261003/r506-native-transfer-channel-handoff.json。\n\n第一失敗 consumer 原始source/exe/library/logs/partial紀錄保留於 target/r506-fixed-transfer-consumer 及 logs/continuation-20261003/r506-first-attempt。失敗原因是測試等待target compatibility時未poll；最終 R506C 重修後成功。\n\n限制：server Java fixture尚未執行，需先解決其server ASCII暫存目錄與只寫modern範圍；玩家/原版GUI=0；B8two/不同址UIrow startup-host Back、provider、room、Native絕對QPC/u32wrap/fullUI parity與真實auth仍開放。新 timeout是明確Modern保守斷線政策。總進度仍16/40，未宣布全體驗收。\n',encoding='utf-8')
limitations=['server Java fixture未執行；server ASCII暫存寫入例外尚待使用者答覆','startup host實際值/私有050104CD/完整可達執行鏈未證實；不得用fixture endpoint代填','不同址UIrow前置hostlookup/B8two及host fallback Back/room/provider未開放','Modern monotonic strict3s/10s；不宣稱Native絕對QPC/u32wrap；unresolved-host timeout明確Modern保守disconnect','GUI0；無原版啟動、玩家candidate執行、全Native UI parity或真實認證驗收']
handoff=dict(state='R506_MODERN_FINITE_NEW_PATH_COMPLETE_JAVA_SCOPE_PENDING_OVERALL_IN_PROGRESS',previous_accepted_R505=pin(L/'r505-native-transfer-prefix-address-gates-handoff.json'),human_handoff=pin(textfile),independent_contract=pin(L/'r506-independent-contract.json'),production_snapshot=pin(L/'r506c-production-source-snapshot.json'),source_difference=pin(L/'r506c-source-difference.patch'),API_adoption=pin(L/'r506-API-adoption.json'),host_alias_adoption=pin(L/'r506-host-alias-adoption.json'),source_study=pin(L/'r506-transfer-source-study.json'),fragment_boundary_note=pin(L/'r506-fragment-boundary-note.json'),new_signed_gate=pin(L/'r506c-new-gates.json'),source_oracle=pin(L/'r506c-source-oracle.json'),skin_binding=pin(L/'r506-native-menu-skin-binding.json'),resource_mapping=pin(L/'r506-channel-resources.json'),player_candidate=pin(L/'r506c-player-candidate.json'),consumer_candidate=pin(L/'r506c-transfer-consumer-candidate.json'),consumer_exit=pin(L/'r506c-transfer-consumer-exit.json'),event_details=pin(L/'r506c-event-details.json'),failed_first_attempt_preserved=pin(L/'r506-first-attempt/preservation.json'),flows=10,actual_socket_TX=26,setup_TX=8,selected_character_TX=3,saved_alias_replay_TX=2,other_navigation_TX=13,IO_joined=5,endpoint_joined=5,Runtime_Closed=3,minimum_channel_lobby_path_verified=True,player_candidate_executions=0,server_Java_fixture_executions=0,functional_failures_final=0,failed_attempts_preserved=1,historical_PASS_reruns=0,limitations=limitations,GUI=0,complete_modes=16,total_modes=40)
path=L/'r506-native-transfer-channel-handoff.json';save(path,handoff)
for value in handoff.values():
 if isinstance(value,dict) and all(k in value for k in ['path','bytes','sha256']):same(value)
save(L/'r506-sealed-link-verification.json',dict(handoff=pin(path),direct_pins_verified=sum(isinstance(v,dict)and all(k in v for k in ['path','bytes','sha256'])for v in handoff.values()),production_current_frozen_pairs=619,consumer_current_frozen_pairs=223,GUI=0))
v=read(P.parent/'監製進度.json');v['phase']='R506_modern_finite_channel_path_verified_JAVA_fixture_scope_pending';v['current_task']='R506 modern同址/異址socket/角色名稱重送/來源列/channel-lobby有限路徑已驗證；server Java ASCII暫存scope待答覆；host Back/GUI/整體仍開放';v['overall_status']='in_progress';v['updated_at']=datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).isoformat();v['r506'].update(status='modern_finite_path_verified_java_scope_pending',handoff=pin(path),minimum_channel_lobby_path_verified=True,flows=10,actual_socket_TX=26,GUI=0,server_Java_fixture_executions=0,complete_modes=16,total_modes=40);save(P.parent/'監製進度.json',v)
print(json.dumps(dict(handoff=pin(path),production_pairs=619,consumer_pairs=223,flows=10,TX=26,asm=len(asm_checks),PE=len(pe_checks),max_poll_us=report['max_poll_us'],elapsed=events['elapsed_seconds'],GUI=0),ensure_ascii=True))
