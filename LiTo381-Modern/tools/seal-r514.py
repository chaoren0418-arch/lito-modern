from r491_common import *
first=read(L/'r514-production-source-snapshot.json');final=read(L/'r514b-production-source-snapshot.json')
assert len(final['source_pins'])==629 and len(final['changes'])==7
assert len(final['delta_from_first_R514'])==1 and final['delta_from_first_R514'][0].endswith('native_source_connection.rs')
checked={}
def verify(e,path=None):
 q=path or e['path'];a=pin(q);assert(a['bytes'],a['sha256'])==(e['bytes'],e['sha256']),q;checked[q]=a
for e in final['source_pins']:
 verify(e);verify(e,e['snapshot'])
for e in first['source_pins']:verify(e,e['snapshot'])
first_by={e['path']:e for e in first['source_pins']}
for e in final['source_pins']:
 if e['path'] not in final['delta_from_first_R514']:assert e['sha256']==first_by[e['path']]['sha256']
r=read(L/'r514-new-consumer-results.json');delta=read(L/'r514b-constructor-policy-results.json')
assert len(r['cases'])==7 and all(e['pass'] for e in r['cases']) and delta['pass']
assert r['socket_workers_started']==r['socket_workers_joined']==6
assert [(e['first_refused_opcode'],e['saved_frame_index']) for e in r['cases'][3:6]]==[('D0',69),('CD',72),('FE',None)]
c=read(L/'r514-consumer-candidate.json');fc=read(L/'r514b-consumer-candidate.json');player=read(L/'r514b-player-candidate.json')
assert fc['production_library']['sha256']==player['production_library']['sha256']
assert c['production_library']['sha256']!=fc['production_library']['sha256']
reports=['r514-input-adoption.json','r514-native-evidence-adoption.json','r514-result-design-adoption.json','r514-test-inputs.json','r514-production-source-snapshot.json','r514-source-difference.patch','r514b-production-source-snapshot.json','r514b-source-difference.patch','r514-consumer-candidate.json','r514b-consumer-candidate.json','r514-player-candidate.json','r514b-player-candidate.json','r514-entry-source-contract.json','r514b-entry-source-contract.json','r514-new-consumer-results.json','r514b-constructor-policy-results.json','r514-library-build.jsonl','r514-library-build.stderr.log','r514b-library-build.jsonl','r514b-library-build.stderr.log']
for prefix in ['r514','r514b']:
 for kind in ['consumer','consumer-build','player-build']:
  for stream in ['stdout','stderr']:reports.append(f'{prefix}-{kind}.{stream}.log')
tools=['adopt-r514-inputs.py','adopt-r514-native.py','prepare-r514.py','finalize-r514-source.py','r514-room-entry-consumer.rs','r514b-constructor-policy-consumer.rs','build-r514-consumer.ps1','build-r514-player.ps1','build-r514b-consumer.ps1','build-r514b-player.ps1','seal-r514.py']
# Validate current acceptance dependencies without recursing into unrelated
# historical reports or treating pre-change live paths as current evidence.
for name in ['r514-input-adoption.json','r514-native-evidence-adoption.json','r514-result-design-adoption.json','r514-consumer-candidate.json','r514b-consumer-candidate.json','r514-player-candidate.json','r514b-player-candidate.json']:
 def walk(v,location='root'):
  if isinstance(v,dict):
   if all(k in v for k in ('path','bytes','sha256')):verify(v)
   for k,x in v.items():
    if k=='historical':continue
    walk(x,location+'.'+k)
  elif isinstance(v,list):
   for x in v:walk(x,location)
 walk(read(L/name))
for e in [pin(L/n) for n in reports]+[pin(P/'tools'/n) for n in tools]:verify(e)
verification=L/'r514-sealed-link-verification.json'
save(verification,dict(status='ALL_CURRENT_ACCEPTANCE_PINS_VERIFIED',current_files=len(checked),pins=list(checked.values()),
 intentional_prior_source='Initial R514 live source path for native_source_connection.rs superseded by final R514b; first frozen629 source files and consumer/library retained and verified',
 historical_exclusions='Only four explicitly listed in Native adoption: two historical live Runtime comparisons resolved through R513 frozen file; two historical prebatch mutable aggregates excluded'))
out=L/'r514-room-entry-consumer-handoff.json'
v=dict(schema='lito.modern.finite.AD368.room-entry.v1',updated_at=datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).isoformat(),
 state='FINITE_OWNED_C_CC_C_CF_S_CE_ROOM_RECEIPT_SCENES_PASSED_NATIVE_GUARD_RETAINED',overall_completed=False,
 previous_R513=pin(L/'r513-status-family-consumer-handoff.json'),report_references=[pin(L/n) for n in reports],tools=[pin(P/'tools'/n) for n in tools],verification=pin(verification),
 source=dict(final=pin(L/'r514b-production-source-snapshot.json'),files=629,changes=final['changes'],first=pin(L/'r514-production-source-snapshot.json'),final_delta=final['delta_from_first_R514'],all_other_first_R514_source_bytes_unchanged=True),
 first_consumer=c,final_policy_consumer=fc,final_player=player,
 evidence_levels=dict(static=dict(server_root_refs=16,server_payloads=9,saved_CE_instances=4,Native_matching_reference_occurrences=127,Native_explicit_historical_exclusions=4,Native_root_raw_witnesses=29,Native_new_fixed_PE_ranges=27,result_constraints_payloads=17),
 program=dict(new_groups=8,first_library_new_groups=7,final_library_new_delta_groups=1,
 cases=r['cases']+[delta],new_socket_workers_started=6,new_socket_workers_joined=6,
 final_candidate_validation='Final change only restores generic success Snapshot structural validation. Room adapter/Runtime/navigation/send/presentation/return source unchanged byte for byte; final constructor-policy check verifies four saved success shapes, legacy zero structural shape, strict AD368 identity separation, exact failure/truncation/stale guards. No seven-case replay.',
 new_Java_SQL_runs=0,old_PASS_or_fixture_replays=0),physical=dict(original_operations=0,Native_handler_runs=0,real_Java_SQL_runs=0,GUI_runs=0,player_executable_runs=0)),
 behavior=dict(adapter='Explicit synthetic AD368 producer+11D94834 contract pin and qualified channel receipt required; no automatic adapter activation.',
 controls='Private epoch/navigation/roster/room revisions and typed Create/Join/RefreshList intent. New request retires earlier controls; only current completed serial permits reply. Stale/foreign/unsolicited/duplicate/mismatched room ID/name rejects atomically in direct candidate API; stream refusal retires epoch and archives receipt checkpoints.',
 wire='C_CC compact or validated legacy/extended tail; ordinary capacity2..100. C_CF explicit nonzero roomID/password. Caller supplies preencoded raw text; Java MS950 roundtrip/admission remains server authority, no encoding validity guarantee added. No option0 mode from C_CC.',
 success='Variable S_CE body27+N+P+O+K, Modern cap155/count<=8/exactend. Returned roomID, owner display alias, summary and raw fields stored separately from character/source identity. RoomReceipts is a Modern receipt scene, not complete roster, Java ROOM commit or Native numericphase/owner/GUI readiness.',
 failure='Exact CE+nonzero result, no success tail. StateRefused preserves raw code, requires distinct explicit RefreshList to emit opcode-only C_CE. No automatic create/join retries and no fabricated room/list rollback. Unknown code stores raw bytes without assumed error-message mapping.',
 Native='Original active-navigation guard remains. SourceFlow numeric phase/dispatch/mode and character checkpoint are unchanged by adapter room receipts. Native helper virtual/private/UI/callback graph and protected scene effects remain unqualified. Generic success constructor continues existing Native structural policy; AD368 identity validation occurs only in explicit adapter.',
 retirement='Stream unknown/refusal resets live room/character state and retains last room/character checkpoint; current/new epochs cannot revive old controls or unsolicited CE. Complete-write and partial-write gating uses existing bounded Sender.',
 roster='CD is explicitly refused as an incremental AD368 room receipt, instead of misusing generic Native actor comparison. CE users is a summary, no row count or terminator; named ArenaBot D0 is optional existing-member event, not a fixed join tail.'),
 actual_next_refusals=[dict(opcode='D0',saved_5ED_frame=69,body_bytes=14,total_frame_bytes=16,reason=r['cases'][3]['reason'],roomID_not_physical=True),dict(opcode='CD',saved_5ED_frame=72,body_bytes=84,total_frame_bytes=86,reason=r['cases'][4]['reason'],incremental_adapter_missing=True)],
 synthetic_negative=dict(opcode='FE',body_bytes=1,actual_producer_claim=False,reason=r['cases'][5]['reason']),
 result_constraints='6D692ECE design only: RESULT D9/F000 retainsroom/RESULT; named D0 confirmation commitsCHANNEL. No result/battle/exit implementation here.',
 next_smallest_gap=dict(task='Adopt exact AD368 participant contract and implement room-scoped incremental, possibly incomplete CD/D1 receipts; keep Native comparison table refusal and physical/source/self authority separate.',
 supplied_contract=dict(path=r'D:\lineage開發\3-lito開發\lito伺服器\server-room-participant-consumer-handoff.json',sha256='EF3FF32941FF07DD0276FCB885A67560ECC47CBE0B660AB778EC5F60CA46C65F',status='SUPERVISOR_SUPPLIED_FOR_NEXT_STEP_NOT_YET_ADOPTED_BY_R514'),
 required='C_CD preview inCHANNEL is separate from owned S_CE entry; duplicate/alias/state policies explicit; no CD terminator or CEusers row count; D1 state2 is shop/list, no guessed readytrue; unknown aliases cannot create characters.'),
 boundaries=dict(owner_ready=False,player_ready=False,Native_scene_ready=False,Native_virtual_private_callbacks=False,GUI=False,incremental_CD_D1_implemented=False,room_exit_result_battle=False,room_buttons_GUI_wired=False,other_group_files_modified=False,new_wire_or_opcodes=0))
save(out,v)
human=P/'Native-R514_房間入口與回覆場景交接.txt'
human.write_text('R514有限房間入口交接\n\n明確AD368 adapter加owned Create(C_CC)/Join(C_CF) intent，epoch/navigation/roster/room revision及partial-write ticket守門。完成當前要求後才能收S_CE。成功是變長27+N+P+O+K，保存roomID、owner顯示alias與原始欄位；與character/source/physical身分分離。Modern RoomReceipts僅回覆場景，非完整名單或Native owner/GUI ready。\n\n短失敗僅2bytes，保留raw result。禁止盲重送建立/加入；獨立explicit RefreshList發C_CE，列表後續未實作。成功缺CD也不視為房间不存在。\n\n7組初始新檢查通過，6socket worker均join；最後僅改泛用Native成功constructor，保留既有Snapshot結構資格、將非零roomID/非空name驗證留在AD368 adapter。另1組final library針對差異新檢查通過，無socket，前7組不重播；adapter/Runtime等其餘628來源與初始候選完全相同。原版/JavaSQL/舊fixture/GUI0。\n\n下一實際拒收：5ED Host frame69 namedS_D0(roomID1,ArenaBot)，body14/wire16；Guest frame72 incrementalS_CD，body84/wire86。FE僅合成未知負向，不是actual producer。退役前room/character checkpoint保存，晚到/foreign/duplicate/mismatched事件不復活。\n\nNative fallback/phase2helper/listreset/場景clear來源已採用，但virtual/private/UI graph/currentepoch未資格；既有導航mode guard保留，numeric flow不因AD368 room receipt變動。Native phase2不等於JavaROOM。\n\n629來源、7變更；兩個固定consumer/library及最後獨立LiToModern-R514b.exe指紋見JSON；player0執行。後續最小缺口為明確AD368 incremental CD/D1資料，保持Native比較表拒絕、同alias/preview/ready/shop政策及physical/source/self權限分界。\n','utf-8')
progress=read(P.parent/'監製進度.json');progress.update(updated_at=v['updated_at'],phase='R514_finite_owned_room_entry_handoff_sealed',current_task='R514有限房間入口封存：7初始新組+1final constructor差異新組、6socket worker join；629來源7變更，Native guard保留。實際下一拒收Host D0 frame69/Guest CD frame72；後續最小缺口為AD368 incremental CD/D1，未宣告owner/Native scene/GUI。')
progress['r514']=dict(state='finite_handoff_sealed',handoff=pin(out),human=pin(human),source_manifest=pin(L/'r514b-production-source-snapshot.json'),source_files=629,changed_files=7,new_groups=8,first_groups=7,final_delta_groups=1,mock_workers_joined=6,actual_next_refused=['S_D0 frame69','S_CD frame72'],verification=pin(verification),GUI=0,owner_ready=False,player_executions=0,overall_completed=False)
save(P.parent/'監製進度.json',progress)
print(pin(out));print('sealed629 sources;8 focused groups across two libraries;6 joined socket workers;no old PASS/fixture replay')
