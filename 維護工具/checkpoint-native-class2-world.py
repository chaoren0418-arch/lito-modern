"""Publish both-profile real WORLD retirement evidence without promoting product scope."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
records=[];candidate_pins=None
for profile,name in [('Debug','20261018-004704-901618'),('Release','20261018-004922-404624')]:
    run=ROOT/'驗證紀錄/current/native-class2-world'/name
    qp=next(run.glob('qualified-*/qualified.json'));q=read(qp);r=read(run/'receipt.json');build=Path(r['compile']);b=read(build/'receipt.json')
    assert q['status']=='independently_qualified_class2_active_request_and_WORLD_retirement' and q['profile']==profile
    assert q['original_exit_code']==0 and q['recorded_owned_all_terminal'] and q['actual_sessions']==6
    assert q['actual_lightning_casts']==1 and q['actual_tree_requests']==q['actual_S5A']==q['actual_C08']==2
    assert q['active_ambient_at_tree_request_verified'] and not q['normal_product_registered']
    assert not q['GUI_verified'] and not q['physical_audio_verified'] and not q['C72_handoff_verified']
    pins={name:p['sha256'] for name,p in q['candidate_inputs'].items()}
    if candidate_pins is None:candidate_pins=pins
    assert candidate_pins==pins
    for pin in q['candidate_inputs'].values():assert sha(pin['path'])==pin['sha256']
    for name,digest in read(build/'compiled-inputs.json').items():assert sha(ROOT/name)==digest
    assert sha(r['product'])==r['product_sha256']==q['product_sha256']
    assert sha(build/'receipt.json')==r['compile_receipt_sha256']
    assert sha(qp.parent/'raw.zip')==q['raw_archive_sha256']
    with zipfile.ZipFile(qp.parent/'raw.zip') as z:assert all(hashlib.sha256(z.read(name)).hexdigest().upper()==digest for name,digest in q['raw_files'].items())
    records.append(dict(profile=profile,run=str(run),qualification=str(qp),qualification_sha256=sha(qp),product_sha256=q['product_sha256'],
        original_session=q['original_session_id'],terminal_chunk=q['terminal_tool_chunk'],actual_sessions=6,actual_lightning_casts=1,actual_tree_requests=2,roles=q['roles'],recorded_identities=q['recorded_identities']))
failure_records=[]
for name in ['20261018-003153-746416','20261018-003401-905392','20261018-003908-966588','20261018-004248-604240']:
    run=ROOT/'驗證紀錄/current/native-class2-world'/name;p=run/'failure-preserved.json';f=read(p)
    assert f['actual_exit_code']==1 and f['all_owned_terminal'] and sha(run/'failed-runtime.zip')==f['archive_sha256']
    failure_records.append(dict(path=str(p),sha256=sha(p),status=f['status'],original_session=f['original_session'],actual_exit_code=1,terminal_chunk=f['terminal_chunk']))
previous=ROOT/'驗證紀錄/current/native-class2-night-checkpoint/20261018-002624-107049/readback.json'
before=read(previous)['after_files'];assert len(before)==8 and all(sha(ROOT/name)==digest for name,digest in before.items())
out=ROOT/'驗證紀錄/current/native-class2-world-checkpoint'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for name in before:z.write(ROOT/name,name)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(name)).hexdigest().upper()==digest for name,digest in before.items())
record=dict(status='both_profiles_real_C02_active_class2_and_WORLD_retirement_qualified_candidate_only',cases=records,candidate_pins=candidate_pins,
    actual_sessions=12,ordinary_protocol_peers_per_profile=3,actual_lightning_casts=2,actual_S5A=4,actual_C08=4,
    source34_audio_first_result='Stale',repeat='AlreadyConsumed',source34_sink_calls=0,fresh_destination_heal109_once_per_primary_peer=True,
    active_ambient_at_tree_request_both_profiles=True,destination_ambient_counter=0,destination_current_calendar_preserved=True,
    current_character_and_item_authority_continuity=True,server_WORLD_vitals_and_display_identity_independently_matched=True,
    natural_third_peer_battle_result_verified=True,battle_completion_rule_disabled=False,
    controlled_local_night_replay=True,real_C02_night_clock=False,pixel_brightness_verified=False,
    primary_target_follow_refresh_cadence_unresolved=True,real_same_numeric_map_S5A_during_flash_pending=True,
    failed_attempts_preserved=failure_records,failed_attempts_counted_as_success=False,
    normal_product_registered=False,normal_product_source_modified=False,GUI_verified=False,physical_audio_verified=False,C72_audio_handoff_verified=False,
    full_G06_verified=False,whole_goal_complete=False)
save(out/'evidence.json',record)
work='來源E15907AC正常產品／1031編譯輸入未改，既有default與完整library兩版資格維持。技能34同一隔離候選已新增Debug／Release實際三人C02流程，共12 LOGIN/GAME sessions、2次11736施法、4次S5A／C08；兩版皆有亮度覆寫仍活動時送出傳圖，目的地計數歸零且保留當前日曆。四份保留Sound34皆Stale、重送AlreadyConsumed且播放0次；目的地新heal109各一次、角色／物品／伺服器HPMP對應與實際移動已核對，第三人依正式規則自然結算。8張Model診斷PNG目視，非正常GUI或實體音效。四輪早期失敗原封存，不算成功；修正測試音效分組／暫存時機與等待順序，並以第三名正常玩家維持對戰，未修改正式JAR或弱化結算規則。下一步釐清原版class2目標位置更新時機，補移動／生命週期及同圖號S5A邊界，再完成正常產品整合。畫面提亮消費端、正常GUI／實體輸入音效、C72同端點及完整G06／G01–G12仍待；8／60秒、G11與Release設定維持，登入器只讀且產物獨立。'
for name in before:
    path=ROOT/name
    if path.suffix=='.json':
        j=read(path);assert j['current_product_identity']['source_manifest_sha256']=='E15907AC8DF9A2C84A789034023EB520C600C298D6ED622693D560BFD3B48182'
        j.update(current_native_class2_WORLD_retirement=record,next_task=work,next_focus=work,source_and_corresponding_QA_frozen=True,
            last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        if name.endswith('next-validation-action.json'):j['next_action']=work
        else:j['current_focus']=dict(item='canonical34-original-target-binding-and-scene-boundaries',work=work,applies_to_current_source=False,normal_product_registered=False,goal_complete=False)
        save(path,j)
    else:
        first,sep,rest=path.read_text(encoding='utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18') and sep
        path.write_text('2026-10-18：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes())
save(out/'readback.json',dict(before_files=before,after_files={name:sha(ROOT/name) for name in before},evidence_sha256=sha(out/'evidence.json')))
assert all(sha(ROOT/name)==digest for name,digest in read(out/'readback.json')['after_files'].items())
print(json.dumps(dict(checkpoint=str(out),actual_sessions=12,both_profiles_qualified=True,normal_source_modified=False,whole_goal_complete=False),ensure_ascii=False))
