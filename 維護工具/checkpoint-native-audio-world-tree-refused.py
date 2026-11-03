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
    path=path.resolve(strict=True);assert path.is_relative_to(ROOT/'驗證紀錄/current/native-audio-world-tree-refused')
    q=read(path);assert q['status']=='independently_qualified_world_tree_refusal_retains_arena_audio_and_authority' and q['original_exit_code']==0 and q['actual_sessions']==4
    assert all(j['original_terminal'] for j in q['recorded_identities'])
    assert not any(q[n] for n in ['GUI_verified','physical_audio_verified','C72_audio_handoff_verified','whole_G06_complete','whole_goal_complete'])
    assert q['profile'] not in records
    records[q['profile']]=dict(qualification=str(path),sha256=sha(path),original_session_id=q['original_session_id'],original_exit_code=0,terminal_tool_chunk=q['terminal_tool_chunk'],source_manifest_sha256=q['source_manifest_sha256'],product_sha256=q['product_sha256'],actual_sessions=4,retained_cue_first_submit='Played109',retained_cue_repeat='AlreadyConsumed',retained_cue_deliveries_per_player=1,fresh_own_cue_deliveries_per_player=1,explicit_refused_requests_per_player=2)
sources={j['source_manifest_sha256'] for j in records.values()};assert len(sources)==1;source=next(iter(sources))
names=['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json','驗證紀錄/current/native-terrain-residence/continuation.json','驗證紀錄/current/native-presentation-source-owner/application-holy-list-20261017-060417-273807/next-validation-action.json','README.md','docs/ARCHITECTURE.zh-TW.md','docs/HISTORY.md']
out=ROOT/'驗證紀錄/current/native-audio-world-tree-refused'/('tracker-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
before={n:sha(ROOT/n) for n in names}
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in names:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
gui_failure=ROOT/'驗證紀錄/current/native1030-interactive-teleport/20261017-213850-821601/failure-readback.json'
gui=read(gui_failure);assert gui['original_exit_code']==1 and gui['capture_frames']==0 and not gui['successful_input_claimed'] and not gui['GUI_visual_verified']
work='來源'+source[:8]+'既有正常兩主、G06控制器96sessions、跨端點非同步成功12／失敗84、重新登入音效16及同連線WORLD成功8sessions維持原資格。WORLD拒絕分支'+('Debug／Release兩版' if len(records)==2 else 'Debug')+'完成'+str(len(records)*4)+' LOGIN/GAME sessions並獨立讀回：每玩家兩次明確技能131要求均回拒絕276，零S5A／C08；epoch／generation／map564與原房間、HP／MP、物品保持一致；真實未播放sound109仍可首次Played且重複AlreadyConsumed，兩人可移動及新施法各送一次自身音效。來源／映像／封包／PNG像素與工作者終止已核對。讀回列舉snake_case及變數遮蔽的工具失敗均封存，未重跑原runtime或改產品。此為函式庫控制器／持續Presentation診斷；正常GUI擷取／輸入工具失敗仍未通過。下一步核對當版廣域與library驗證缺口，按實際版本執行必要回歸；C72來源音效、同端點成功、GUI／實體輸入音效與完整G06／G01–G12仍待。修改產物限重建版，登入器與素材唯讀、產物獨立；時限與Release設定不變。'

record=dict(scope='two explicit world-tree-disabled refusals276 retain arena map564 and authentic unconsumed sound109; no endpoint handoff',profiles=records,actual_sessions=len(records)*4,C72_audio_handoff_verified=False,authority_continuity_verified=True,GUI_verified=False,physical_audio_verified=False,whole_G06_complete=False,whole_goal_complete=False)
for n in names[:5]:
    p=ROOT/n;j=read(p);assert j['current_product_identity']['source_manifest_sha256']==source
    assert not j.get('active_workloads') and not j.get('active_workload_sessions'),'do not overwrite live work'
    j.update(current_native_interactive_GUI_failure=dict(readback=str(gui_failure),sha256=sha(gui_failure),original_session_id=98918,original_exit_code=1,successful_input_claimed=False,GUI_visual_verified=False),current_native_audio_world_tree_refused=record,current_native_audio_world_tree_refused_active=None,source_and_corresponding_QA_frozen=False,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'readback.json'),goal_complete=False,whole_goal_complete=False)
    if n==names[4]:j['next_action']=work
    else:j['current_focus']=dict(item='G06-WORLD-refusal-audio-and-authority-retention',work=work,applies_to_current_source=True,goal_complete=False)
    save(p,j)
for n in names[5:]:
    p=ROOT/n;first,sep,rest=p.read_text(encoding='utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18') and sep
    p.write_text('2026-10-18：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes());save(out/'readback.json',dict(record=record,before=before,after={n:sha(ROOT/n) for n in names},whole_goal_complete=False))
print(json.dumps(dict(checkpoint=str(out),profiles=list(records),actual_sessions=len(records)*4,whole_goal_complete=False),ensure_ascii=False))
