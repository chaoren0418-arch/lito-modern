"""Seal existing R527 evidence. No tests, sockets, credentials or process operations."""
from r491_common import *
import shutil

RUN_ID = 'nw-6fdb7420a2bc4b8492dd7cb90ff270ed'
own = L / ('r527-backend-' + RUN_ID)
peer = own / 'accepted-server-closure'
peer.mkdir(exist_ok=True)
closure_path = P.parent.parent / 'lito伺服器/server-modern-485F-live-supply.json'
closure_pin = pin(closure_path)
assert closure_pin['bytes'] == 7530
assert closure_pin['sha256'] == 'F16697CEDDDFCD5DAEB9DCFDB6407EDF5267A5A268301F0112A4BFE0D7341903'
closure = read(closure_path)
assert closure['run_identity'] == RUN_ID and not closure['currently_running']
assert closure['Guest_C_D2'] == [2, 1, 0] and closure['Guest_ready'] == [False, True, False]
assert closure['Host_notready'] and not closure['owner_start_sent']
assert closure['E1_WORLD_start_result'] == 0 and closure['retries'] == 0
assert not closure['credentials_contents_or_hash_in_report']
server_root = Path(closure['evidence_root'])

# All top-level pinned references are metadata, approved artifacts or code/binaries.
# No private properties, SQL datadir or raw capture files are traversed.
root_checked = []
for v in closure.values():
    if isinstance(v, dict) and {'path', 'bytes', 'sha256'} <= v.keys():
        a = pin(v['path'])
        assert a == v, (v['path'], 'closure root reference changed')
        root_checked.append(a)
assert len(root_checked) == 16
manifest = read(closure['manifest']['path'])
assert len(manifest['files']) == 23
manifest_by_name = {e['path']: e for e in manifest['files']}

# Only explicitly sanitized wire files and nonsecret lifecycle/GET artifacts.
selected = [
    'evidence/final-CaptureLog-auth-masked.json',
    'evidence/final-Host-frames-auth-ciphertext-omitted.json',
    'evidence/final-write-boundaries.json',
    'evidence/Guest-finite-wire-getter-counter-proof.json',
    'evidence/final-retirement-identity.json',
    'evidence/final-result.json',
    'evidence/final-status-owned-retired.json',
    'history/accepted-live-supply.json',
]
selected_checked = []
for name in selected:
    original = server_root / name
    a = pin(original)
    expected = manifest_by_name[name]
    assert a['bytes'] == expected['bytes'] and a['sha256'] == expected['sha256']
    dest = peer / Path(name).name
    if dest.exists():
        assert pin(dest)['sha256'] == a['sha256']
    else:
        shutil.copyfile(original, dest)
    selected_checked.append(dict(original=a, own_copy=pin(dest)))
shutil.copyfile(closure_path, peer / 'accepted-server-closure-root.json')
shutil.copyfile(closure['manifest']['path'], peer / 'accepted-server-manifest.json')

result_pin = pin(own / 'result.json')
assert result_pin['sha256'] == 'BC4F586D180A93CC447FECCD8B89C2C385E2A5521F34D188FD990993972C9559'
result = read(own / 'result.json')
assert result['success'] and result['case_error'] is None
assert result['actual_backend_attempts'] == 1 and not result['synthetic_replay']
assert result['IO_started'] == result['IO_joined'] == 2
assert result['queued_events'] == result['discarded_late_events'] == 0 and result['Close_ok']
assert not result['credentials_contents_or_hash_recorded'] and not result['Modern_starts_or_stops_JavaSQL']
assert result['GUI'] == 0 and not result['owner_start_WORLD_allowed']
assert pin(peer / 'accepted-live-supply.json')['sha256'] == pin(own / 'accepted-server-live-supply.json')['sha256'] == '1D5E2A837F7F78980DCE0F354A2EEB50709983D0E55157C4B6948B6471EB03A5'
assert pin(own / 'accepted-ready-contract.json')['sha256'] == '667B69B3FF198085D1B1B98AFDF4AC1F6A99FEE0D974D1B54397F54D2890F635'

capture = read(peer / 'final-CaptureLog-auth-masked.json')
assert len(capture) == 81
auth = [i for i, e in enumerate(capture) if e['direction'] == 'client_to_server' and e['opcode'] == '0x01']
assert auth == [3, 14, 38, 48]
assert all(capture[i]['plain'] == 'MASKED_C_LOGIN' for i in auth)
host_frames = read(peer / 'final-Host-frames-auth-ciphertext-omitted.json')
host_auth = [e for e in host_frames if e.get('direction') == 'C' and e.get('opcode') == 1]
assert len(host_auth) == 2
assert all(e['body_hex'] == 'MASKED_OWN_FIXTURE_LOGIN' and e['authentication_ciphertext_omitted'] and 'encrypted_body_hex' not in e for e in host_auth)
def body(index, direction, opcode):
    e = capture[index]
    assert e['direction'] == direction and e['opcode'] == f'0x{opcode:02X}'
    b = bytes.fromhex(e['plain'])
    assert b[0] == opcode and e['wire_length'] == len(b) + 2
    return b
C, S = 'client_to_server', 'server_to_client'
alias = b'Fx485Guest(Fx485Moon)\0'
assert body(43, C, 0xC8) == b'\xc8Fx485Moon\0'
assert body(49, S, 0xE6) == b'\xe6' and body(50, C, 0xE2) == b'\xe2'
assert body(51, S, 3) == b'\x03\x01'
cb = body(52, S, 0xCB)
assert len(cb) == 103 and cb[1:1+len(alias)] == alias
assert body(53, C, 0xC9) == b'\xc9' + alias
assert body(54, S, 0xC8) == b'\xc8\0'
assert body(56, C, 0xCA) == b'\xca\0'
assert body(57, S, 0xC9) == b'\xc9' + bytes(12)
assert body(65, C, 0xCF) == b'\xcf\x01\0\0\0\0'
ce = body(66, S, 0xCE)
assert ce[:6] == b'\xce\0\x01\0\0\0'
assert not any(e['opcode'] in ['0xE1', '0x28', '0xD3'] for e in capture)

boundaries = read(peer / 'final-write-boundaries.json')
rows = [e for e in boundaries if e['body_hex'].startswith('d1')]
assert len(rows) == 6
proof = read(peer / 'Guest-finite-wire-getter-counter-proof.json')
assert proof['Modern_result']['sha256'] == result_pin['sha256']
assert proof['completed_write_GET_samples'] == 6
assert proof['server_D1_per_peer'] == {'Host-GAME': [2,1,0], 'Rust-GAME-2': [2,1,0]}
assert proof['E1_WORLD_start_result_frames'] == 0 and proof['Match'] == 'WAITING'
correlated = []
for rec, state, index, wire in zip(result['records'], [2,1,0], [71,74,77], proof['actual_C_D2']):
    assert body(index, C, 0xD2) == bytes([210, state])
    assert wire['capture_index'] == index and wire['state'] == state
    for key in ['body_hex', 'cipher_frame_hex', 'counter_before', 'counter_after']:
        assert wire[key] == rec[key]
    assert rec['written_bytes'] == rec['frame_bytes'] == 4 and rec['body_bytes'] == 2
    assert rec['counter_after'] == rec['counter_before'] + 2
    frame = bytes.fromhex(rec['cipher_frame_hex'])
    assert frame[:2] == b'\x04\0' and len(frame) == 4
    key = rec['counter_before'].to_bytes(8, 'little')
    decoded = bytes([frame[2] ^ key[0], frame[3] ^ key[1] ^ frame[2]])
    assert decoded == bytes([210, state])
    d1 = b'\xd1' + alias + bytes([state])
    assert rec['D1_body_hex'] == d1.hex()
    assert body(index+1, S, 0xD1) == body(index+2, S, 0xD1) == d1
    per_peer = [e for e in rows if e['body_hex'] == d1.hex()]
    assert len(per_peer) == 2 and {e['epoch'] for e in per_peer} == {'Host-GAME','Rust-GAME-2'}
    for e in per_peer:
        guest, host = e['sessions']['Rust-GAME-2'], e['sessions']['Host-GAME']
        assert e['frame_bytes'] == 26 and e['room_id'] == 1 and e['room_exists']
        assert e['match_phase'] == 'WAITING' and not e['battle_started'] and e['countdown_value'] == 0
        assert guest['ready'] == (state == 1) and not host['ready']
        assert e['hub_ready_members'] == (['Rust-GAME-2'] if state == 1 else [])
        assert guest['phase'] == host['phase'] == 'ROOM'
        assert guest['selectedCharacter'] == 'Fx485GuestChar' and guest['selectedClassicServerName'] == 'Fx485Moon'
        assert guest['native_source_id'] == 72 and guest['physical_character_id'] == 41002
        assert guest['currentChannelId'] == guest['hubChannelId'] == 0 and guest['currentRoomId'] == 1
        assert guest['litoCharacterSelectionConsumed'] and guest['authenticatedGameAccounts_owns_session'] and guest['selectedCharacters_owns_session']
    correlated.append(dict(Modern=rec, server_C_capture_index=index, server_D1_capture_indices=[index+1,index+2], completed_write_GET=per_peer, independent_cipher_decryption_matches=True))
for epoch in ['Host-GAME','Rust-GAME-2']:
    assert [bytes.fromhex(e['body_hex'])[-1] for e in rows if e['epoch'] == epoch] == [2,1,0]

retirement = read(peer / 'final-retirement-identity.json')
final = read(peer / 'final-result.json')
assert pin(peer / 'final-retirement-identity.json')['sha256'] == 'E2444CE824BEFD944A05EFB53BDBF7E4B39FB1A20A1661F2658439914ED00B81'
assert retirement['state'] == 'RETIRED' and retirement['run_root'] == closure['run_root']
assert retirement['ports_released'] and retirement['datadir_retained'] and not retirement['forced_stop']
assert not retirement['unrelated_processes_queried'] and retirement['owned'] == closure['exact_own_PID_retired']
assert [(e['pid'], e['creation_FILETIME_100ns'], e['same_live_identity']) for e in retirement['owned']] == [(22088,'134357261583260298',False),(16044,'134357261630649117',False)]
assert final['all_run_threads_retired'] and final['mysql_retired'] and not final['unreleased_ports']
assert final['final_connected_sessions'] == final['final_authenticated_accounts'] == final['final_selected_characters'] == 0
assert not final['background_failure'] and not final['unexpected_WORLD_start_result'] and final['no_automatic_owner_start']

candidate = read(L / 'r527-consumer-candidate.json')
candidate_checks = []
for e in candidate.values():
    if isinstance(e,dict) and {'path','bytes','sha256'} <= e.keys():
        assert pin(e['path']) == e
        candidate_checks.append(e)
assert candidate['executable']['sha256'] == '3D3406840F74452614730B862DBEEFF41A7B195C677F4E4D00405990C634B831'
snapshot = read(L / 'r527-production-source-snapshot.json')
source_checks = []
for e in snapshot['source_pins']:
    for path in [e['path'],e['snapshot']]:
        a = pin(path)
        assert a['bytes'] == e['bytes'] and a['sha256'] == e['sha256']
        source_checks.append(a)
assert len(snapshot['source_pins']) == 639
prep_pin = pin(L / 'r527-485F-real-backend-preparation-handoff.json')
assert prep_pin['sha256'] == '4DA8234875FDB66B424589C920FC7D440C771BF4CC6FE8B6555B332C895CE201'
save(own / 'cross-peer-finite-verification.json',dict(
    schema='modern.r527.cross-peer-finite-verification.v1', result=result_pin,
    accepted_closure=pin(peer/'accepted-server-closure-root.json'),
    closure_root_pins_verified=root_checked, selected_server_artifacts_verified=selected_checked,
    supervisor_attestation='2026-10-06 10:18:44 independent exact-PID absence/ports free/datadir retained; 16root +23manifest pins/31unique accepted. Modern independently checks16root and8selected sanitized artifacts; no raw capture/credential/datadir reads or hashes.',
    masked_auth_frames=4, normal_entry_correlated=True,
    normal_entry_evidence='Actual sourceFx485Moon, news, singleton03/CB exact rawalias, C9 same bytes, acceptedC8, channel0, C_CF room1, acceptedS_CE; per-completedwrite GET confirms selected actualsource/character and privateROOM.',
    normal_entry_cipher_limit='Authentication payloads masked. Full LOGIN/GAME C cipher chronology is server finiteproof provenance; Modern independently decrypts only three recorded C_D2 frames. PhysicalID remains backend observation, not client selection authority.',
    correlated_requests=correlated, retirement=retirement,
    provider_final_result=pin(peer/'final-result.json'),
    limitations=['Finite single WAITING case; perwrite GET is separate field/hub observation, not allpeer atomicity or server acceptance ACK.','No COUNTDOWN/E1, ownerstart/WORLD, partialwrite/sendfailure/race/roomreuse/sameepoch freshness qualification.','Close EOF peerD0 is not committed ownleave ACK or ROOM→CHANNEL.','GUI/originalart/defaultmain and Native private UI authority remain open.'],
    actual_case_attempts=1, case_failures=0, retries=0, preflight_attempts=1,
    IO_started=2, IO_joined=2, queue=0, discarded_late=0,
    credentials_copied_or_hashed=False, raw_server_auth_capture_read=False,
    Modern_server_start_stop=0, provider_retired=True,
    production_source_files=639, source_changes_since_preparation=0))

human_path = P / 'Native-R527_485F真實準備狀態往返交接.txt'
human_path.write_text('''R527 真實 485F backend 有限交接

這次 fresh R2 run nw-6fdb7420a2bc4b8492dd7cb90ff270ed：Modern 一次 preflight、一次實接成功，無失敗重連。正常 LOGIN→source Fx485Moon→GAME/news→singleton角色CB→同raw alias C9→acceptedC8→channel0→加入room1→私有ROOM/self守門後，送出 d202／d201／d200。

三筆各 body2／frame4／written4，64bit counter 每次+2；加密 frame 04003379、04003178、0400377f。Modern 實收 D1 state2／1／0、member revision2→3→4→5；server 已遮蔽登入資料的 CaptureLog 第71／74／77筆與三筆請求一致，每次後接兩筆同raw alias D1。Host-GAME與Rust-GAME-2各有2／1／0，六筆完成write後GET顯示 Guest ready false／true／false、Host false、WAITING，hub ready只在state1含Guest。

正常入場的跨端明文序列已核對：實際source、news、03 count1、完整103-byte CB、C9相同alias、acceptedC8、channel0、C_CF room1、acceptedS_CE。GET另外確認source72／Fx485GuestChar／physical41002／ROOM／選角已消耗；physical ID只是backend觀測，不成為client選角權限。登入內容已遮蔽，完整auth密文鏈未由Modern獨立讀取；Modern獨立解密僅這三筆D2。

Modern Close成功，IO started2／joined2，queue0，discarded late0。當時result的provider_stop_required=true與停機協調記錄原樣保留，表示當時需要server收尾；如今server owner已正常stop，closure證據確認helper22088／mysql16044指定creation FILETIME已退休、62065／62042釋放、datadir保留、無forced stop。Modern未啟停Java／SQL、未讀raw auth capture、未複製或hash密碼。

accepted closure根 F16697CE；原READY供應1D5E2A83另保存在history且與Modern先前副本一致。667B69 READY已消耗且provider退休，不能當成可重用端點。16個closure root pins與8個指定安全artifact已獨立核對；監製另核對23個manifest files（31 unique）。639份production來源與凍結副本未再變動，consumer EXE 3D340684／rlib 5C9DFD44仍相符。

準備階段4DA82348交接原樣保留：新增2項profile測試通過、2334舊測試filtered、舊PASS未重播；常規lib288則既有warning、新warning0、consumer0。準備時路徑篩選／編譯／封存修復與失敗記錄仍在原準備交接；本次實接0失敗、0重試，封存未新增test／socket／程序查詢。

本次只驗證一個WAITING participant狀態往返與normal entry。D1／GET不是server acceptance ACK或全peer原子性。COUNTDOWN／E1、owner start、WORLD、GUI、同epoch freshness、partialwrite／sendfailure／race／roomreuse、committed ownleave與Native private UI／原版美術／default main仍未驗收。Close產生的peer D0不升格為完成離房。整體專案仍in_progress。

機器交接：logs/continuation-20261003/r527-485F-real-backend-consumer-handoff.json
跨端有限核對：logs/continuation-20261003/r527-backend-nw-6fdb7420a2bc4b8492dd7cb90ff270ed/cross-peer-finite-verification.json
''',encoding='utf-8')

roots = [prep_pin, pin(human_path), pin(L/'r527-consumer-candidate.json'),
         pin(L/'r527-production-source-snapshot.json'), result_pin]
for name in ['r527-live-preflight-accounting.json','r527-live-connect-accounting.json',
             'r527-live-preflight.stdout.log','r527-live-preflight.stderr.log',
             'r527-live-connect.stdout.log','r527-live-connect.stderr.log',
             'r527-live-stop-coordination.json','r527-consumer-build-repair.json',
             'r527-seal-repair.json','r527-compiler-warning-verification.json']:
    roots.append(pin(L/name))
for name in ['accepted-ready-contract.json','accepted-server-live-supply.json',
             'independent-Modern-wire-verification.json','post-Modern-close-status-projection.json',
             'cross-peer-finite-verification.json']:
    roots.append(pin(own/name))
roots += [pin(p) for p in sorted(peer.iterdir()) if p.is_file()]
roots.append(pin(Path(__file__)))
assert len({r['path'] for r in roots}) == len(roots)
verification_path = L/'r527-final-live-verification.json'
save(verification_path,dict(status='PASS_EXISTING_EVIDENCE_ONLY_NO_REPLAY',
    root_references_checked=roots, candidate_checked=candidate_checks,
    source_pins_checked=source_checks, source_files=639,
    production_source_changes_since_preparation=0,
    closure_root_reference_count=len(root_checked), selected_server_artifact_count=len(selected_checked),
    test_runs=0, backend_connections=0, credentials_reads=0, process_queries=0,
    server_start_stop=0, raw_capture_auth_reads=0))
roots.append(pin(verification_path))
handoff_path = L/'r527-485F-real-backend-consumer-handoff.json'
handoff = dict(schema='modern.r527.real-backend-consumer-handoff.v1',
    updated_at=datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).isoformat(),
    state='FINITE_TRUE_485F_WAITING_PARTICIPANT_NORMAL_ENTRY_PASSED_PROVIDER_RETIRED_OVERALL_OPEN',
    root_references=roots, previous_preparation=prep_pin, human=pin(human_path),
    source_files=639, changed_files_in_R527_preparation=1, source_changes_after_preparation=0,
    consumer=candidate, run_id=RUN_ID, actual_backend_cases=1, actual_backend_attempts=1,
    preflight_attempts=1, actual_case_failures=0, retries=0, synthetic_replay=False,
    result=result_pin, requests=result['records'], cross_peer=pin(own/'cross-peer-finite-verification.json'),
    normal_entry_correlated=True, peer_D1={'Host-GAME':[2,1,0],'Rust-GAME-2':[2,1,0]},
    completed_write_GET_count=6, Guest_ready=[False,True,False], Host_ready=[False,False,False],
    match_phase='WAITING', IO_started=2, IO_joined=2, queue=0, discarded_late=0, Close_ok=True,
    server_closure=pin(peer/'accepted-server-closure-root.json'),
    historical_accepted_READY=pin(own/'accepted-ready-contract.json'), READY_reusable=False,
    historical_stop_coordination=pin(L/'r527-live-stop-coordination.json'),
    result_stop_flag_preserved_as_historical=True, server_stop_required=False, provider_retired=True,
    retirement=pin(peer/'final-retirement-identity.json'), exact_retired_PIDs=retirement['owned'],
    ports_released=True, datadir_retained=True, forced_stop=False,
    regular_library_warning_count=288, new_warning_count=0, final_consumer_warning_count=0,
    preparation_new_unit_tests_passed=2, old_tests_filtered=2334, old_tests_run=0,
    preparation_failure_records=pin(L/'r527-consumer-build-repair.json'),
    historical_seal_repair=pin(L/'r527-seal-repair.json'),
    historical_preparation_and_result_rewritten=False,
    verification=pin(verification_path),
    boundaries=dict(Modern_JavaSQL_start_stop=0, server_namespace_writes=0,
        secret_contents_copied_or_hashed=False, raw_server_auth_capture_read=False,
        seal_credentials_reads=0, seal_process_queries=0, seal_test_runs=0, seal_connections=0,
        player_runs=0, GUI=0, owner_start_sent=False, WORLD=False, old_PASS_replays=0,
        password_guard_expanded=False, native_guard_retained=True),
    remaining=read(own/'cross-peer-finite-verification.json')['limitations'],
    awaiting_supervisor_finite_acceptance=True, overall_completed=False)
save(handoff_path,handoff)
for e in read(handoff_path)['root_references']:
    assert pin(e['path']) == e
assert pin(own/'result.json') == result_pin and pin(prep_pin['path']) == prep_pin
# Read immediately before merge and retain all other groups' existing fields.
progress_path = P.parent/'監製進度.json'
progress_data = read(progress_path)
progress_data['updated_at'] = handoff['updated_at']
progress_data['phase'] = 'R527_true485F_finite_case_sealed_provider_retired_overall_open'
progress_data['overall_status'] = 'in_progress'
progress_data['current_task'] = 'R527一次真實485F normalentry與WAITING participant d202/d201/d200有限PASS封存；雙peer D1与6GET一致，Modern2IOjoin／queue0／late0；server owner正常stop／指定PID退休／ports釋放／datadir保留。未新增連線或test；COUNTDOWN/E1、WORLD、GUI、sameepochfreshness等未驗收，待監製依完整handoff續派。'
r527 = progress_data.setdefault('r527',{})
r527['preparation_handoff'] = prep_pin
r527['preparation_human'] = r527.get('human')
r527['preparation_verification'] = r527.get('verification')
r527.update(state=handoff['state'], handoff=pin(handoff_path), human=pin(human_path),
    verification=pin(verification_path), actual_backend_executions=1,
    awaiting_supervisor_fresh_provider_acceptance=False,
    awaiting_supervisor_finite_acceptance=True, server_stop_required=False,
    provider_retired=True, server_closure=handoff['server_closure'], retirement=handoff['retirement'],
    READY_reusable=False, normal_entry_correlated=True,
    completed_write_GET_count=6, IO_started=2, IO_joined=2, queue=0, late=0,
    overall_completed=False)
save(progress_path,progress_data)
print(json.dumps(dict(status='SEALED_NO_REPLAY',handoff=pin(handoff_path),human=pin(human_path),
    root_reference_count=len(roots),source_files=639,closure_root_pins=16,
    selected_sanitized_server_artifacts=8,actual_backend_attempts=1,
    IO_joined=2,queue=0,late=0,provider_retired=True),ensure_ascii=False))
