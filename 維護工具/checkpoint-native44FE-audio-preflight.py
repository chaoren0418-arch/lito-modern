"""Register one qualified Debug audio branch; no full G06/GUI or Release promotion."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(p.read_text('utf-8-sig'))
def sha(p):
    with p.open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
run=V/'native44FE-teleport-audio/run-20261018-164507-209091';qp=next(run.glob('qualified-*/qualified.json'));q=read(qp);r=read(run/'receipt.json')
assert q['profile']=='Debug'and q['scenario']=='refused'and q['actual_sessions']==8 and q['original_exit_code']==0 and q['recorded_owned_all_terminal']and q['unplayed_source_audio_rejection_verified']and q['new_scene_own_sound_events_delivered_once_each']
assert q['normal_source_bodies_unchanged']and not q['source_overlays_used']and q['diagnostic_request_API_extension']and not q['normal_product_source_modified']and not q['normal_GUI_API_permission_inferred']
assert len(q['diagnostic_PNGs'])==4
for p in q['diagnostic_PNGs']:assert sha(Path(p['path']))==p['sha256']
failure=V/'native44FE-teleport-audio/run-20261018-163721-922344/failed-preflight/failure.json';f=read(failure);assert f['no_client_LOGIN_started']and f['owned_processes_all_terminal']and f['Rust_exit_code']==101 and f['server_exit_code_recorded']is None
names=['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json','驗證紀錄/current/native-terrain-residence/continuation.json','驗證紀錄/current/native-presentation-source-owner/application-holy-list-20261017-060417-273807/next-validation-action.json','README.md','docs/ARCHITECTURE.zh-TW.md','docs/HISTORY.md']
for n in names[:5]:assert read(ROOT/n)['current_product_identity']['source_manifest_sha256']==q['source_manifest_sha256']
out=run/('checkpoint-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir();before={n:sha(ROOT/n)for n in names}
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in names:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
record=dict(status='Debug_refused_nonempty_audio_case_independently_qualified_only',run=str(run),qualification=str(qp),qualification_sha256=sha(qp),source_manifest_sha256=q['source_manifest_sha256'],actual_sessions=8,old_unplayed_source_cues_rejected=2,new_own_sound_events_delivered_once=2,original_session_id=9123,original_exit_code=0,terminal_tool_chunk='bcb1dc',recorded_owned_identities=q['recorded_owned_identities'],recorded_owned_all_terminal=True,normal_source_bodies_unchanged=True,diagnostic_API_extension_only=True,normal_GUI_permission_inferred=False,diagnostic_PNGs=q['diagnostic_PNGs'],diagnostic_PNGs_visually_inspected=True,visual_scope='four unmodified native diagnostic map500 frames: detailed terrain/NPC/player/minimap present; no black image; source generation1 to recovery generation2; no normal GUI acceptance',retained_failed_probe_version_guard=str(failure),retained_failed_probe_sha256=sha(failure),Release_diagnostic_compile_session_id=40888,Release_case_qualified=False,GUI_verified=False,physical_input_verified=False,physical_audio_verified=False,whole_G06_complete=False,whole_goal_complete=False)
save(out/'evidence.json',record);save(out/'visual-readback.json',dict(images=q['diagnostic_PNGs'],scope=record['visual_scope'],not_edited=True,normal_GUI_verified=False));work='來源44FEA738正常兩版各23場景及G06控制器12案96sessions已核對；當版Debug拒絕分支的非同步Presentation／非空音效另8sessions通過，兩舊待播cue零交付、恢復兩新事件各一次、四張原PNG已目視，僅此限定範圍。舊探針C02硬編碼檢查造成登入前101與server退出碼缺項的失敗已封存，未計遊戲通過；修正為嚴格75DA後本案已獨立核對。Release診斷編譯沿原40888續讀，之後完成同來源其餘11分支，不能移用舊版Release結果。正式源碼與此批QA凍結，8／60秒、Release及G11界限不變；相鄰只讀、產物獨立，正常GUI／實體音效、同端點與完整G06／G01–G12未完成。'
for n in names:
    p=ROOT/n
    if p.suffix=='.json':
        j=read(p);j.update(current_native44FE_nonempty_audio_preflight=record,source_and_corresponding_QA_frozen=True,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False);j['current_focus']=dict(item='current-nonempty-audio-remaining-branches',work=work,goal_complete=False)
        if n.endswith('next-validation-action.json'):j['next_action']=work
        save(p,j)
    else:
        first,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert datetime.strptime(first[:10],'%Y-%m-%d')and sep;p.write_text((datetime.now(timezone.utc)+timedelta(hours=8)).strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
save(out/'readback.json',dict(before_files=before,after_files={n:sha(ROOT/n)for n in names},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json')));(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes())
save(run.parent/'current-continuation.json',dict(source_manifest_sha256=q['source_manifest_sha256'],qualified_Debug_refused=record,Release_compile=str(V/'native44FE-teleport-audio/compile-20261018-164822-068790-release-885e9e04'),original_session_id=40888,original_terminal_exit_code=None,must_poll_original_session_before_restarting=True,latest_checkpoint=str(out),latest_readback=str(out/'readback.json'),next_action=work,whole_goal_complete=False));print(json.dumps(dict(checkpoint=str(out),Debug_sessions=8,Release_case_qualified=False),ensure_ascii=False))
