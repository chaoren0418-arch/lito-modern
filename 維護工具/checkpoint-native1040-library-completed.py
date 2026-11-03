"""Publish only terminal independently read current-source full library scope."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,j):p.write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int,required=True);ap.add_argument('--terminal-chunk',required=True);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native1040-library-default') and a.terminal_chunk not in ['','pending','unknown']
c=read(run/'cohort.json');op=run/'completion-observation.json';o=read(op)
assert c['status']=='executed_both_current_full_library_default_selections' and c['owned_process'] is None
assert o['status']=='independently_qualified_current1040_library_both_profiles_and_terminal_processes' and o['cohort_sha256']==sha(run/'cohort.json')
assert o['observed_terminal_exit_code']==0 and o['recorded_original_owned_processes_all_terminal'] and o['original_session_id']==a.session and o['terminal_tool_chunk']==a.terminal_chunk
assert c['source_manifest_sha256']==o['source_manifest_sha256']=='4E1F7A4E02DA1201F2254A959A4130D5038C6F0A9393449D4EB30F8AE2883ED4'
profiles={}
for profile in ['Debug','Release']:
    info=o['profiles'][profile];qp=Path(info['qualification']);q=read(qp)
    assert sha(qp)==info['qualification_sha256'] and q['source_manifest_sha256']==c['source_manifest_sha256']
    assert q['status']=='independently_qualified_current_full_library_default_selection' and q['failed']==q['filtered_out']==q['Rust_exit_code']==0
    assert q['passed']==c['profiles'][profile]['counts']['passed'] and q['ignored']==c['profiles'][profile]['counts']['ignored']
    assert not q['ignored_bodies_executed'] and not q['GUI_verified'] and not q['whole_goal_complete']
    assert sha(qp.parent/'raw.zip')==q['raw_archive_sha256']
    with zipfile.ZipFile(qp.parent/'raw.zip') as z:
        assert set(z.namelist())==set(q['raw_files'])
        for n,h in q['raw_files'].items():
            with z.open(n) as stream:assert hashlib.file_digest(stream,'sha256').hexdigest().upper()==h
    profiles[profile]=dict(qualification=str(qp),qualification_sha256=sha(qp),passed=q['passed'],failed=0,ignored=q['ignored'],inventory=q['inventory'],actual_test_sha256=q['actual_test_sha256'])
for n,h in read(run/'compiled-inputs.json').items():assert sha(ROOT/n)==h
before=read(run/'active-checkpoint/readback.json')['after_files'];assert len(before)==8 and all(sha(ROOT/n)==h for n,h in before.items())
out=run/('checkpoint-completed-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in before:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
record=dict(status='current_source_both_full_library_default_selections_qualified_limited',source_manifest_sha256=c['source_manifest_sha256'],source_files=1040,compiled_input_files=1041,profiles=profiles,run=str(run),original_session_id=a.session,terminal_tool_chunk=a.terminal_chunk,observed_actual_exit_code=0,completion=str(op),completion_sha256=sha(op),recorded_original_owned_identities=o['recorded_original_owned_identities'],recorded_original_owned_processes_all_terminal=True,ignored_bodies_executed=False,GUI_verified=False,whole_goal_complete=False)
save(out/'evidence.json',record)
broad=read(Path(c['input_cohort'])/'completion-observation.json')
assert broad['source_manifest_sha256']==c['source_manifest_sha256'] and broad['recorded_original_owned_processes_all_terminal']
work='來源4E1F7A4E原'+str(a.session)+'完整library default兩版已actual exit0並獨立核對：Debug'+str(profiles['Debug']['passed'])+'／Release'+str(profiles['Release']['passed'])+'過、各0敗，忽略'+str(profiles['Debug']['ignored'])+'／'+str(profiles['Release']['ignored'])+'本文未執行；來源、實際映像、raw與'+str(o['recorded_original_owned_identities'])+'組程序身分終止均符。當版廣域Debug'+str(broad['profiles']['Debug']['counts']['passed'])+'／Release'+str(broad['profiles']['Release']['counts']['passed'])+'過、各0敗，與正常兩版各90相關檢查、G06控制器／非同步Presentation各12案96sessions、冰錐來源本體六案24sessions維持各自限定資格。下一步同來源真實未播放音效跨端點退役及必要原資源驗證，再正常GUI；技能35辨識／39缺口仍是靜態請求範圍，45／46唯讀研究尚未實作。GUI／實體輸入音效、同端點與其他服務、必要玩法／可攜交付及完整G01–G12未完成。8／60秒、Release與G11不變，相鄰只讀且登入器與重建版產物獨立。'

for n in before:
    path=ROOT/n
    if path.suffix=='.json':
        j=read(path);assert j['current_product_identity']['source_manifest_sha256']==c['source_manifest_sha256']
        j['current_native1040_library']=record
        if 'current_native_retired_join_normal' in j:j['current_native_retired_join_normal'].update(current_source_full_library_default_verified=True,ignored_bodies_executed=False)
        j['current_native1040_library_active'].update(qualification_granted=True,status='completed_qualified_default_library_scope',completion=str(op),completion_sha256=sha(op),terminal_tool_chunk=a.terminal_chunk,observed_actual_exit_code=0)
        j.update(active_workloads={},active_workload_sessions={},source_and_corresponding_QA_frozen=False,source_freeze_while_Cargo=False,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        j['current_focus']=dict(item='current1040-audio-handoff-and-necessary-resource-qualification',work=work,applies_to_current_source=True,goal_complete=False)
        j['current_source_goal_validation']['G03-current-regression']=dict(state='current1040_broad_and_full_library_default_both_qualified_other_acceptance_pending',complete=False,evidence=record)
        for item in j.get('items',[]):
            if item['id']=='G03-current-regression':item.update(state='current1040_broad_and_full_library_default_both_qualified_other_acceptance_pending',complete=False,current_source_manifest_sha256=c['source_manifest_sha256'],current_evidence=record)
        if n.endswith('next-validation-action.json'):j['next_action']=work
        save(path,j)
    else:
        first,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18') and sep;path.write_text('2026-10-18：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes());save(out/'readback.json',dict(before_files=before,after_files={n:sha(ROOT/n)for n in before},evidence_sha256=sha(out/'evidence.json'),before_archive_sha256=sha(out/'before.zip')))
assert all(sha(ROOT/n)==h for n,h in read(out/'readback.json')['after_files'].items())
save(run.parent/'current-continuation.json',dict(**record,checkpoint=str(out),must_poll_original_session_before_restarting=False,next_action='Current-source authentic audio handoff and necessary-resource acceptance; no normal GUI or physical acceptance inferred.'))
print(json.dumps(dict(checkpoint=str(out),profiles=profiles,whole_goal_complete=False),ensure_ascii=False))
