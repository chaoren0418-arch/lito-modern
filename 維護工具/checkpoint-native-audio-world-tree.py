"""Publish qualified reconnect scope separately from C72 and whole-goal acceptance."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
ap=argparse.ArgumentParser();ap.add_argument('qualifications',nargs='+',type=Path);a=ap.parse_args();records={}
for path in a.qualifications:
    path=path.resolve(strict=True);assert path.is_relative_to(ROOT/'驗證紀錄/current/native-audio-world-tree')
    q=read(path);assert q['status']=='independently_qualified_same_connection_arena_WORLD_audio_and_authority' and q['original_exit_code']==0 and q['actual_sessions']==4
    assert all(j['original_terminal'] for j in q['recorded_identities'])
    assert not any(q[n] for n in ['GUI_verified','physical_audio_verified','C72_audio_handoff_verified','whole_G06_complete','whole_goal_complete'])
    assert q['profile'] not in records
    records[q['profile']]=dict(qualification=str(path),sha256=sha(path),original_session_id=q['original_session_id'],original_exit_code=0,terminal_tool_chunk=q['terminal_tool_chunk'],source_manifest_sha256=q['source_manifest_sha256'],product_sha256=q['product_sha256'],actual_sessions=4,old_cue_first_submit='Stale',old_cue_repeat='AlreadyConsumed',old_cue_deliveries=0,new_own_cue_deliveries_per_player=1)
sources={j['source_manifest_sha256'] for j in records.values()};assert len(sources)==1;source=next(iter(sources))
names=['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json','驗證紀錄/current/native-terrain-residence/continuation.json','驗證紀錄/current/native-presentation-source-owner/application-holy-list-20261017-060417-273807/next-validation-action.json','README.md','docs/ARCHITECTURE.zh-TW.md','docs/HISTORY.md']
out=ROOT/'驗證紀錄/current/native-audio-world-tree'/('tracker-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
before={n:sha(ROOT/n) for n in names}
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in names:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
gui_failure=ROOT/'驗證紀錄/current/native1030-interactive-teleport/20261017-213850-821601/failure-readback.json'
gui=read(gui_failure);assert gui['original_exit_code']==1 and gui['capture_frames']==0 and not gui['successful_input_claimed'] and not gui['GUI_visual_verified']
work='來源'+source[:8]+'既有正常兩主、G06控制器96sessions、非同步跨端點成功12／失敗84及重新登入音效16sessions維持原範圍。同連線技能131由map564→普通WORLD4，以'+('Debug／Release兩版' if len(records)==2 else 'Debug')+'完成'+str(len(records)*4)+' LOGIN/GAME sessions並獨立讀回：真實C14→S5A／C08、角色與物品全欄接續、generation加一且epoch不變、舊房間／頻道清除；未播放heal1／sound109舊cue首次Stale、重複AlreadyConsumed，WORLD新自身cue各一次，兩人真實移動與對端可見。原PNG逐檔與像素核對；這是函式庫控制器／持續Presentation診斷。正常Release視窗操作另案98918因Windows擷取兩次逾時及coordinate input geometry unavailable失敗，僅讀到輔助功能樹，零網路封包，兩個確切自有客戶端與伺服器已清理；不得宣稱GUI或OS輸入通過。正式同端點與一般頻道C9成功包完全相同，不能以該包可靠早退；產品未改。C72來源音效、同端點成功、正常GUI／實體輸入音效及完整G06／G01–G12仍待。所有修改產物限重建版，登入器與素材唯讀且產物獨立；時限及Release設定不變。'

record=dict(scope='same GAME connection arena map564→WORLD map4, skill131, real held heal cue and inventory continuity only',profiles=records,actual_sessions=len(records)*4,C72_audio_handoff_verified=False,authority_continuity_verified=True,GUI_verified=False,physical_audio_verified=False,whole_G06_complete=False,whole_goal_complete=False)
for n in names[:5]:
    p=ROOT/n;j=read(p);assert j['current_product_identity']['source_manifest_sha256']==source
    assert not j.get('active_workloads') and not j.get('active_workload_sessions'),'do not overwrite live work'
    j.update(current_native_interactive_GUI_failure=dict(readback=str(gui_failure),sha256=sha(gui_failure),original_session_id=98918,original_exit_code=1,successful_input_claimed=False,GUI_visual_verified=False),current_native_audio_world_tree=record,current_native_audio_world_tree_active=None,source_and_corresponding_QA_frozen=False,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'readback.json'),goal_complete=False,whole_goal_complete=False)
    if n==names[4]:j['next_action']=work
    else:j['current_focus']=dict(item='G06-same-connection-WORLD-audio-and-authority',work=work,applies_to_current_source=True,goal_complete=False)
    save(p,j)
for n in names[5:]:
    p=ROOT/n;first,sep,rest=p.read_text(encoding='utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18') and sep
    p.write_text('2026-10-18：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes());save(out/'readback.json',dict(record=record,before=before,after={n:sha(ROOT/n) for n in names},whole_goal_complete=False))
print(json.dumps(dict(checkpoint=str(out),profiles=list(records),actual_sessions=len(records)*4,whole_goal_complete=False),ensure_ascii=False))
