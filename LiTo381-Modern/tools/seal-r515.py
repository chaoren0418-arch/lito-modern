from r491_common import *
source=read(L/'r515-production-source-snapshot.json');result=read(L/'r515-new-consumer-results.json')
assert len(source['source_pins'])==630 and len(source['changes'])==5
assert len(result['cases'])==7 and all(v['pass'] for v in result['cases'])
assert result['socket_fixture_workers']==result['new_fixture']==result['Java_SQL']==result['old_PASS_replays']==0
consumer=read(L/'r515-consumer-candidate.json');player=read(L/'r515-player-candidate.json')
assert consumer['production_library']['sha256']==player['production_library']['sha256']
checked={}
def verify(e,path=None):
 q=path or e['path'];a=pin(q);assert(a['bytes'],a['sha256'])==(e['bytes'],e['sha256']),q;checked[q]=a
for e in source['source_pins']:verify(e);verify(e,e['snapshot'])
def walk(v):
 if isinstance(v,dict):
  if all(k in v for k in ('path','bytes','sha256')):verify(v)
  for k,x in v.items():
   if k in ('historical','explicit_historical_exclusions'):continue
   walk(x)
 elif isinstance(v,list):
  for x in v:walk(x)
for name in ['r515-input-adoption.json','r515-native-evidence-adoption.json','r515-consumer-candidate.json','r515-player-candidate.json']:walk(read(L/name))
native=read(L/'r515-native-evidence-adoption.json');assert(native['matching_reference_occurrences'],native['root_raw_witness_occurrences'],native['unique_root_raw_witnesses'])==(170,66,58)
assert len(native['new_raw_slices'])==26 and len(native['inherited_raw_slices'])==12
reports=['r515-input-adoption.json','r515-native-evidence-adoption.json','r515-test-inputs.json','r515-production-source-snapshot.json','r515-source-difference.patch','r515-consumer-candidate.json','r515-player-candidate.json','r515-entry-source-contract.json','r515-new-consumer-results.json','r515-library-build.jsonl','r515-library-build.stderr.log','r515-seal-historical-metadata-drift.json']
for kind in ['consumer','consumer-build','player-build']:
 for stream in ['stdout','stderr']:reports.append(f'r515-{kind}.{stream}.log')
tools=['adopt-r515-inputs.py','adopt-r515-native.py','prepare-r515.py','r515-participant-consumer.rs','build-r515-consumer.ps1','build-r515-player.ps1','seal-r515.py']
for e in [pin(L/name) for name in reports]+[pin(P/'tools'/name) for name in tools]:verify(e)
verification=L/'r515-sealed-link-verification.json'
save(verification,dict(status='ALL_CURRENT_R515_ACCEPTANCE_PINS_VERIFIED',current_files=len(checked),pins=list(checked.values()),historical_exclusions='Two explicit prebatch mutable Native aggregates, including informational current pins captured at adoption; excluded as a whole from current core dependency traversal. Drift diagnostics preserved. Frozen R513/R514 source metadata not reinterpreted as live source authority.'))
trace=read(L/'r515-input-adoption.json')['saved_frame_sources'][0]
out=L/'r515-room-participant-consumer-handoff.json'
handoff=dict(schema='lito.modern.finite.AD368.incremental-room-receipts.v1',updated_at=datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).isoformat(),
 state='FINITE_CD_D1_RECEIPT_MODEL_PASSED_RUNTIME_JOINED_ROUTE_COMPILED_NOT_POSITIVE_EXECUTED',overall_completed=False,
 previous_R514=pin(L/'r514-room-entry-consumer-handoff.json'),report_references=[pin(L/name) for name in reports],tools=[pin(P/'tools'/name) for name in tools],verification=pin(verification),
 source=dict(manifest=pin(L/'r515-production-source-snapshot.json'),files=630,changes=source['changes'],live_and_frozen_verified=True,generic_Native_actor_unchanged=source['generic_Native_actor_unchanged'],Native_navigation_guard_retained=True),consumer=consumer,player=player,
 evidence_levels=dict(static=dict(AD368_contract='EF3FF32941FF07DD0276FCB885A67560ECC47CBE0B660AB778EC5F60CA46C65F',server_root_refs=16,server_payloads=9,existing_CD=8,existing_D1=8,all_vector_offsets_raw_type_value_end_verified=True,Native_contract='1C2519F9F1B88C9E68FDF963FFAFF85366CEC755254613C1926A68E7ED9BC192',Native_matching_reference_occurrences=170,Native_historical_exclusions=2,Native_root_raw_occurrences=66,Native_unique_root_raw=58,Native_new_raw_slices=26,Native_inherited_raw_slices=12,Native_81_instruction_assertions='Producer report retained; not independently replayed or executed'),
 program=dict(new_groups=7,cases=result['cases'],new_socket_fixtures=0,new_socket_workers=0,new_Java_SQL=0,old_PASS_replays=0,positive_stateful_observation_Book_executed=True,Runtime_idle_joined_guard_executed=True,Runtime_local_unknown_stream_retirement_executed=True,
 positive_owned_joined_Runtime_route_executed=False,limitation=result['positive_joined_Runtime_integration']),
 physical=dict(new_Native_handler_runs=0,new_real_Java_SQL=0,new_GUI_runs=0,player_runs=0)),
 behavior=dict(wire='Separate explicit AD368 CD dscccdddcscsdccdc +12 awards+5 editions+DWORD, exact end and max105; D1 rawaliasNUL+state0/1/2 max30. Preserve all20 CD fields and full raw body; generic Native constructor/version/domains remain unchanged.',
 scope='Standalone Book is ExplicitObservation, admitted by current Runtime Scope for both book epoch and event owner/serial. It cannot prove joining, self/source/physical authority. Production joined route additionally requires explicit bootstrap+room+participant adapters, current RoomReceipts scene, accepted CE room ID and matching CE revision. JoinedCeReceipt book is private, created by successful owned CE.',
 identity='Projected row key=epoch+roomID+kind(force)+exact raw alias. No alias splitting, source/physical ID creation, self detection, owner status or Native comparison-table initialization. Cross kind and byte-case aliases remain distinct observations.',
 duplicates='Same kind/raw alias repeated CD retains existing whole member row and newer D1 state; each complete incoming packet is separately preserved in receipt history. Supervisor correction applied before first compile/test. No row overwrite or original Native runtime comparator claim.',
 D1='Apply per-socket arrival to a unique known raw alias. State enum NotReady/Ready/ShopList; Ready text is not state and state2 is not boolean true. Unknown alias or alias present across multiple kinds is recorded/ignored; no row creation, identity merge, buffering or later retroactive state application.',
 budgets='256 displayed rows and1024 full receipts are independent Modern allocation budgets, not room capacity or completion signals. Overflow rejects atomically. Every prefix including zero/CE-summary-count match remains PossiblyIncomplete; no timeout, terminator or count-based completion inference.',
 lifecycle='Source boundary/Close clears current book and archives its snapshot. Source Scope still required for observation changes; retired/foreign/same-serial-other-runtime/wrong-room cannot update. Positive joined checkpoint branch compiled/source reviewed only; standalone retired-epoch and local stream tests executed.',
 Native='Original same-kind comparison-zero early return and D1 first-match update recorded. Our conservative raw alias projection does not execute/emulate Native lead834A60/fold834960 tables; their private05010B41/external4F727C initialization, epoch/locale generation, S3A rebuild and pre-room-gate self copy remain unqualified. Owned epoch/room gating occurs first; generic Native actor and active navigation guard remain.',
 scope_excluded='CHANNEL C_CD preview request/controls, ready commands, prestart/countdown/world/battle, S_D0 removal/own exit, Native UI owners and Native comparison tables.'),
 next_boundaries=dict(trace=trace,index_basis='One-based position in exact saved5ED evidence/frames.json; contracts global_frame_index are zero-based and converted+1. Not49A indices.',
 Host=dict(saved_frame=69,opcode='D0',body_bytes=14,total_frame_bytes=16,unchanged_current_runtime_refusal=True,classification='Prior R514 actual Host first boundary retained; no D0 route changed or reexecuted in R515'),
 Guest=dict(saved_frame=95,opcode='E1',body_hex='e103',body_bytes=2,total_frame_bytes=4,
 classification='Next saved Guest S body after CD72/73 and D1 75/94. Data policy exercised as explicit observations; re-encoded E1 body rejected in a new local candidate stream. Not a replay/positive full joined Runtime or Native countdown acceptance.',reason=result['cases'][-1]['reason'])),
 next_smallest_gap=dict(task='Next finite stage: adopt D0 routing supplement and close same-epoch/CE-room ArenaBot/other removal-if-present or absent no-op, retaining own ROOM; independent self alias plus completed C_D0 intent required before any own-exit transition.',
 supplied_dependency=dict(path=r'D:\lineage開發\3-lito開發\lito伺服器\server-room-D0-consumer-routing-supplement.json',sha256='7BF90D85A297E8AAA9D90D3BD4AF11DFD13992D685FF7211CBD1A9433CA898E6',status='SUPERVISOR_SUPPLIED_FOR_NEXT_FINITE_STAGE_NOT_ADOPTED_OR_IMPLEMENTED_IN_R515'),
 Native_dependency=dict(path=r'D:\lineage開發\3-lito開發\lito脫殼資源\SD0-receive-contract.json',sha256='50A26C6526BC3BE8365DA435AE50117A25265EDE1AB0DC1E2F170BAEA95006C4',status='SUPERVISOR_SUPPLIED_NEXT_STAGE_ONLY_NOT_ADOPTED_OR_IMPLEMENTED_IN_R515'),
 additional_exit_report='Wait for corrected main SHA. Supervisor reports a mislabeled same_self_and_peer_body field in EE31B4DA-prefix ordinary exit handoff; do not adopt that report until corrected.',
 following='Only after named D0 distinction/positive joined receipt ingress is qualified, consider next E1 ready/prestart boundary; no broad ready/start/world work here.'),
 future_ready_contract=dict(path=r'D:\lineage開發\3-lito開發\lito伺服器\server-room-ready-prestart-consumer-contract.json',sha256='57F324FE093C80A8DAE8B7E207A9A8A628AA508E04EBB38AFE17F3C33D6274F8',status='FUTURE_ONLY_NOT_ADOPTED_OR_IMPLEMENTED_IN_R515'),
 boundaries=dict(owner_ready=False,player_ready=False,Native_scene_ready=False,Native_comparison_tables=False,Native_self_copy=False,GUI=False,positive_joined_Runtime_run=False,preview_request=False,ready_command=False,D0_unlocked=False,ready_start_world=False,other_group_files_modified=False,new_wire_or_opcodes=0))
save(out,handoff)
human=P/'Native-R515_房間成員增量資料交接.txt'
human.write_text('R515有限CD/D1 receipt資料候選\n\n明確AD368 EF3FF329 schema：完整CD20欄位與rawbody、D1rawalias與state0/1/2。投影key為epoch+roomID+kind(force)+rawalias。same-kind/rawalias重複CD保留既有列及較新D1 state，每筆完整receipt另外保存；跨kind不合併。D1只更新唯一已知rawalias，未知/跨kind歧義記錄後忽略，不製造角色、不緩衝後補。Ready字串非state，state2=ShopList。\n\n名單始終PossiblyIncomplete；CEusers/timeout/列數吻合/EOF都不能宣告完整。256列、1024receipt是Modern記憶體預算，超限原子拒收。alias不拆成physical/source/self/owner身分。\n\n來源：16refs/9payload、8CD+8D1全部offset/raw/type/value/end核對。Native Batch15 1C2519F9：170引用相符，2明示歷史aggregate排除；66根raw出現(58去重)、26新/12繼承切片核對固定PE；81指令斷言只保留產出組結果，不冒稱重跑。Native重複列提前return及D1第一匹配已記錄；834960/834A60私有初始化/epoch/locale/S3A重建未資格，原版actor與navigation guard保留。\n\n程式：7組新pure receipt檢查全部PASS，含16savedvectors、非零原始欄位、完整截斷、per-peer sequence、duplicate/ShopList/unknown/ambiguity、epoch/room/容量及local未知stream。0socketfixture/worker、0JavaSQL/舊PASS/原版/GUI。Production joined route及CE建立book/checkpoint已編譯/來源review，沒有正向owned joined Runtime執行；正向資料策略由當前owned Scope的ExplicitObservation book驗證，不能稱完整進房或Native驗收。\n\n精確trace：5ED battle-skill-network-f0bd3a9dfbf741628b39792b065ff7ba/evidence/frames.json，一基索引。GuestCD72/73、D1 75/94之後的實際保存下一S body為frame95 E1/03(body2/wire4)，新local候選stream仍拒收；不是正向full joined Runtime重播。Host frame69 D0(body14/wire16)仍沿用R514拒收，R515未解鎖/重測。勿混用49A零基索引。\n\n630來源5變更、固定library/consumer/獨立LiToModern-R515.exe指紋见JSON，player0執行。下一最小有限段為7BF90D85 D0補件：同epoch/CEroom的ArenaBot/他人remove-if-present或absent no-op，保留自身ROOM；own-D0要另有selfalias資格與完成C_D0 intent，不能泛化離房。本輪僅列待採用，ready/start/world不擴張。\n','utf-8')
progress=read(P.parent/'監製進度.json');progress.update(updated_at=handoff['updated_at'],phase='R515_finite_incremental_room_receipt_handoff_sealed',current_task='R515 CD/D1有限純receipt候選封存：7新組/16vectors/630來源5變更，0fixture/舊PASS/JavaSQL；same-kind重複列保留D1state、raw receipts有界另存、跨kind/未知/歧義不合併；Native guard保留。正向joined Runtime僅編譯review未執行，Host5ED一基69 D0拒收不變，Guest資料後保存95 E1/03仍拒收。下一有限段為7BF90D85 D0補件，非ready/start/world。')
progress['r515']=dict(state=handoff['state'],handoff=pin(out),human=pin(human),source_manifest=pin(L/'r515-production-source-snapshot.json'),source_files=630,changed_files=5,new_groups=7,saved_CD=8,saved_D1=8,new_fixtures=0,mock_workers=0,old_PASS_replays=0,positive_joined_Runtime_run=False,Native_guard_retained=True,GUI=0,owner_ready=False,player_executions=0,next='D0 supplement7BF90D85 next finite stage;Guest saved E1/03 frame95',verification=pin(verification),overall_completed=False)
save(P.parent/'監製進度.json',progress)
print(pin(out));print('sealed630 sources;7 new pure receipt groups;zero fixtures/old PASS/JavaSQL;positive joined Runtime path remains unexecuted')
