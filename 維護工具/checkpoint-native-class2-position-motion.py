"""Publish shared thirteen-module candidate evidence; retain historical scopes."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
cases=[('Debug','motion','20261018-011426-892859'),('Debug','retirement','20261018-012450-982267'),('Release','motion','20261018-012622-171713'),('Release','retirement','20261018-012757-244960')]
records=[];core_pins=None
for profile,kind,name in cases:
    run=ROOT/'驗證紀錄/current/native-class2-world'/name;qp=next(run.glob('qualified-*/qualified.json'));q=read(qp);r=read(run/'receipt.json');b=read(Path(r['compile'])/'receipt.json')
    assert q['status']=='independently_qualified_class2_active_request_and_WORLD_retirement' and q['profile']==profile
    assert q['original_exit_code']==0 and q['recorded_owned_all_terminal'] and q['actual_sessions']==6
    assert q['motion_hurt_preflight_directions']==8 and q['actual_S5A']==q['actual_C08']==2
    assert q['target_motion_placement_verified']==(kind=='motion')
    if kind=='retirement':assert q['active_ambient_at_tree_request_verified']
    pins={name:pin['sha256'] for name,pin in q['candidate_inputs'].items() if name!='diagnostic_probe.rs'}
    assert len(pins)==13
    if core_pins is None:core_pins=pins
    assert core_pins==pins
    for name,pin in q['candidate_inputs'].items():
        if name!='diagnostic_probe.rs':assert sha(pin['path'])==pin['sha256']
    build=Path(r['compile']);assert sha(build/'diagnostic_probe.rs')==q['candidate_inputs']['diagnostic_probe.rs']['sha256']
    assert sha(r['product'])==q['product_sha256'] and sha(build/'receipt.json')==r['compile_receipt_sha256']
    for name,digest in read(build/'compiled-inputs.json').items():assert sha(ROOT/name)==digest
    assert sha(qp.parent/'raw.zip')==q['raw_archive_sha256']
    with zipfile.ZipFile(qp.parent/'raw.zip') as z:assert all(hashlib.sha256(z.read(name)).hexdigest().upper()==digest for name,digest in q['raw_files'].items())
    records.append(dict(profile=profile,kind=kind,run=str(run),qualification=str(qp),qualification_sha256=sha(qp),
        diagnostic_probe_sha256=q['candidate_inputs']['diagnostic_probe.rs']['sha256'],product_sha256=q['product_sha256'],
        original_session=q['original_session_id'],terminal_chunk=q['terminal_tool_chunk'],actual_sessions=6,
        target_motion_placement_verified=q['target_motion_placement_verified'],active_ambient_at_tree_request_verified=q['active_ambient_at_tree_request_verified'],
        roles=q['roles'],recorded_identities=q['recorded_identities']))
bindings=[ROOT/'驗證紀錄/current/native-class2-position/20261018-005710-660316/binding.json',ROOT/'驗證紀錄/current/native-class2-motion-queue/20261018-011043-292053/binding.json']
assert read(bindings[0])['status']=='primary_class2_entry_coordinate_capture_qualified' and read(bindings[1])['status']=='primary_running_walk_queues_hurt_binding'
failures=[]
for name in ['20261018-005916-575372','20261018-010153-847023','20261018-010924-214425','20261018-011641-976835','20261018-012039-543194']:
    run=ROOT/'驗證紀錄/current/native-class2-world'/name;path=run/'failure-preserved.json';f=read(path)
    assert f['actual_exit_code']==1 and f['all_owned_terminal'] and sha(run/'failed-runtime.zip')==f['archive_sha256']
    failures.append(dict(path=str(path),sha256=sha(path),status=f['status'],original_session=f['original_session'],terminal_chunk=f['terminal_chunk']))
previous=ROOT/'驗證紀錄/current/native-class2-world-checkpoint/20261018-005151-343675/readback.json';before=read(previous)['after_files']
assert len(before)==8 and all(sha(ROOT/name)==digest for name,digest in before.items())
out=ROOT/'驗證紀錄/current/native-class2-position-motion-checkpoint'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for name in before:z.write(ROOT/name,name)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(name)).hexdigest().upper()==digest for name,digest in before.items())
record=dict(status='shared13_modules_both_profiles_real_target_motion_and_active_WORLD_retirement_qualified_candidate_only',
    core_module_pins=core_pins,cases=records,actual_sessions=24,actual_lightning_casts=4,actual_S5A=8,actual_C08=8,
    target_capture_policy='Sample matching target once at actor-marker entry; keep original cell through subsequent movement.',
    motion_hurt_fix='Keep motion/camera advancing and queue hurt until the walk boundary; preserve water rewind guard.',
    primary_bindings=[dict(path=str(p),sha256=sha(p)) for p in bindings],primary_binding_is_static_PE_analysis=True,
    eight_direction_motion_hurt_preflight_both_profiles=True,actual_target_step_confirmed_by_peer_S12_and_server_trace_both_profiles=True,
    source34_audio_first_result='Stale',source34_audio_repeat='AlreadyConsumed',source34_sink_calls=0,
    actual_source_flash_at_tree_request_verified_both_profiles=True,destination_current_calendar_preserved=True,
    fresh_destination_heal109_once_per_primary_peer=True,natural_third_peer_battle_result_verified=True,
    diagnostic_versions_differ=True,diagnostic_version_note='Debug motion used the earlier strict current-frame audio reader; later cases bind deferred cues by immutable finite key and retained S23/S37. All share identical thirteen core modules; each has independently sealed inputs, raw packets and observations.',
    failed_attempts_preserved=failures,failed_attempts_counted_as_success=False,
    controlled_local_night_replay=True,real_C02_night_clock=False,pixel_brightness_verified=False,
    same_numeric_map_S5A_pending=True,normal_product_registered=False,normal_product_source_modified=False,
    GUI_verified=False,physical_audio_verified=False,C72_handoff_verified=False,full_G06_verified=False,whole_goal_complete=False)
save(out/'evidence.json',record)
work='來源E15907AC正常產品／1031編譯輸入未改，default及完整library舊資格維持。原版PE證明class2在啟動時取一次目標座標，已修正候選逐次追蹤；實際重現後另修受擊／步行銜接的鏡頭回退，保留回退保護並驗八方向與生命更換。相同13核心模組的Debug／Release各完成目標移動及活動亮度傳圖案，共24 LOGIN/GAME sessions、4次11736、8次S5A／C08；目標移動由其他玩家S12及伺服器座標確認，落雷維持原格。活動覆寫傳圖後計數歸零、保留目的地日曆，舊34音效Stale且播放0次，新109與移動正常。五輪失敗含預期紅測與診斷工具錯誤已封存，不計成功；診斷音效改以不可變事件鍵核對延後交付。下一步同圖號S5A：已只讀定位C02世界樹之WORLD分支，先實測再正常產品整合。畫面提亮消費端、正常GUI／實體輸入音效、C72同端點及完整G06／G01–G12仍待；8／60秒、G11與Release設定維持，登入器只讀且產物獨立。'
for name in before:
    path=ROOT/name
    if path.suffix=='.json':
        j=read(path);assert j['current_product_identity']['source_manifest_sha256']=='E15907AC8DF9A2C84A789034023EB520C600C298D6ED622693D560BFD3B48182'
        if 'current_native_class2_WORLD_retirement' in j:j.setdefault('retained_native_class2_WORLD_retirement_11_modules',j['current_native_class2_WORLD_retirement'])
        j.update(current_native_class2_WORLD_retirement=record,current_native_class2_position_motion=record,
            next_task=work,next_focus=work,source_and_corresponding_QA_frozen=True,last_goal_turn_classification='progress',
            last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        if 'current_native_class2_night_candidate' in j:j['current_native_class2_night_candidate'].update(applies_to_latest_candidate=False,retained_immutable_compile_scope_only=True)
        if name.endswith('next-validation-action.json'):j['next_action']=work
        else:j['current_focus']=dict(item='canonical34-same-map-S5A-and-normal-integration',work=work,applies_to_current_source=False,normal_product_registered=False,goal_complete=False)
        save(path,j)
    else:
        first,sep,rest=path.read_text(encoding='utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18') and sep
        path.write_text('2026-10-18：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes())
save(out/'readback.json',dict(before_files=before,after_files={name:sha(ROOT/name) for name in before},evidence_sha256=sha(out/'evidence.json')))
assert all(sha(ROOT/name)==digest for name,digest in read(out/'readback.json')['after_files'].items())
print(json.dumps(dict(checkpoint=str(out),shared_core_modules=13,actual_sessions=24,normal_source_modified=False,whole_goal_complete=False),ensure_ascii=False))
