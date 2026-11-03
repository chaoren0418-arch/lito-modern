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
    path=path.resolve(strict=True);assert path.is_relative_to(ROOT/'驗證紀錄/current/native-audio-reconnect')
    q=read(path);assert q['status']=='independently_qualified_explicit_battle_cancel_reconnect_audio_only' and q['original_exit_code']==0 and q['actual_sessions']==8
    assert all(j['original_terminal'] for j in q['recorded_identities'])
    assert not any(q[n] for n in ['GUI_verified','physical_audio_verified','C72_audio_handoff_verified','authority_continuity_verified','whole_G06_complete','whole_goal_complete'])
    assert q['profile'] not in records
    records[q['profile']]=dict(qualification=str(path),sha256=sha(path),original_session_id=q['original_session_id'],original_exit_code=0,terminal_tool_chunk=q['terminal_tool_chunk'],source_manifest_sha256=q['source_manifest_sha256'],product_sha256=q['product_sha256'],actual_sessions=8,old_cue_first_submit='Stale',old_cue_repeat='AlreadyConsumed',old_cue_deliveries=0,new_own_cue_deliveries_per_player=1)
sources={j['source_manifest_sha256'] for j in records.values()};assert len(sources)==1;source=next(iter(sources))
names=['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json','驗證紀錄/current/native-terrain-residence/continuation.json','驗證紀錄/current/native-presentation-source-owner/application-holy-list-20261017-060417-273807/next-validation-action.json','README.md','docs/ARCHITECTURE.zh-TW.md','docs/HISTORY.md']
out=ROOT/'驗證紀錄/current/native-audio-reconnect'/('tracker-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
before={n:sha(ROOT/n) for n in names}
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in names:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
work='來源'+source[:8]+'既有正常兩主／G06控制器96sessions／非同步成功12及失敗84sessions保留原範圍資格。普通戰鬥音效以'+('Debug／Release兩版' if len(records)==2 else 'Debug')+'實際獨立端點取消／重登入完成'+str(len(records)*8)+' LOGIN/GAME sessions並獨立讀回：兩玩家真實C14→S37 gfx11722／sound109，保留未播放cue在新epoch首次Stale、重複AlreadyConsumed、舊cue零送出、新自身cue各一次；舊輸入表面拒絕新場景、取消與退出工作者靜止。這是重登入音效生命週期診斷，不是C72傳送／角色權威接續或裝置音效驗收。首輪同音效跨玩家計數斷言及讀回目標寫死的工具失敗封存，產品來源未改。租裝村兩種來源音效嘗試均無資格；C72來源音效、同端點、正常GUI／實體輸入音效與完整G06／G01–G12仍待。重建版與登入器獨立，只讀參考線索，替換登入器JAR提案已撤回；修改與產物限重建版，原時限及Release設定不變。'
record=dict(scope='explicit cancel/reconnect ordinary battle audio only',profiles=records,actual_sessions=len(records)*8,C72_audio_handoff_verified=False,authority_continuity_verified=False,GUI_verified=False,physical_audio_verified=False,whole_G06_complete=False,whole_goal_complete=False)
for n in names[:5]:
    p=ROOT/n;j=read(p);assert j['current_product_identity']['source_manifest_sha256']==source
    assert not j.get('active_workloads') and not j.get('active_workload_sessions'),'do not overwrite live work'
    j.update(current_native_audio_reconnect=record,current_native_audio_reconnect_active=None,source_and_corresponding_QA_frozen=False,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'readback.json'),goal_complete=False,whole_goal_complete=False)
    if n==names[4]:j['next_action']=work
    else:j['current_focus']=dict(item='G06-audio-lifecycle-supported-source',work=work,applies_to_current_source=True,goal_complete=False)
    save(p,j)
for n in names[5:]:
    p=ROOT/n;first,sep,rest=p.read_text(encoding='utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18') and sep
    p.write_text('2026-10-18：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes());save(out/'readback.json',dict(record=record,before=before,after={n:sha(ROOT/n) for n in names},whole_goal_complete=False))
print(json.dumps(dict(checkpoint=str(out),profiles=list(records),actual_sessions=len(records)*8,whole_goal_complete=False),ensure_ascii=False))
