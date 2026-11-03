"""Seal exact current broad scope without relabeling historical research."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(path):return json.loads(Path(path).read_text('utf-8-sig'))
def sha(path):
    with Path(path).open('rb')as stream:return hashlib.file_digest(stream,'sha256').hexdigest().upper()
def save(path,value):Path(path).write_text(json.dumps(value,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
parser=argparse.ArgumentParser();parser.add_argument('run',type=Path);parser.add_argument('--session',type=int,required=True);parser.add_argument('--terminal-chunk',required=True);args=parser.parse_args()
run=args.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native1040-default112')
c=read(run/'cohort.json');op=run/'completion-observation.json';o=read(op)
assert c['status']=='independently_qualified_current_default112_both_profiles'and c['owned_process']is None
assert o['status']=='independently_qualified_current1040_default112_both_profiles_and_terminal_processes'
assert o['observed_terminal_exit_code']==0 and o['recorded_original_owned_processes_all_terminal']and o['cohort_sha256']==sha(run/'cohort.json')
assert o['original_session_id']==args.session and o['terminal_tool_chunk']==args.terminal_chunk and args.terminal_chunk not in ['','pending','unknown']
profiles={}
for profile in ['Debug','Release']:
    info=c['profiles'][profile];qp=Path(info['qualification']);q=read(qp)
    assert info['qualified']and info['actual_runner_exit_code']==0 and sha(qp)==info['qualification_sha256']
    assert q['source_manifest_sha256']==c['source_manifest_sha256']==o['source_manifest_sha256']
    assert q['counts']['failed']==0 and q['actual_test_executables']==113 and q['integration_targets']==112 and not q['full_library_scope_inferred']
    assert sha(qp.parent/'raw.zip')==q['raw_archive_sha256']
    with zipfile.ZipFile(qp.parent/'raw.zip')as z:assert all(hashlib.sha256(z.read(name)).hexdigest().upper()==digest for name,digest in q['raw_files'].items())
    profiles[profile]=dict(qualification=str(qp),qualification_sha256=sha(qp),counts=q['counts'],integration_targets=112,actual_test_executables=113,normal_mains=q['normal_mains'])
normal=Path(c['normal_qualification']).parent.parent;inputs=read(normal/'compiled-inputs.json');assert len(inputs)==1041
for name,digest in inputs.items():assert sha(ROOT/name)==digest
before=read(run/'active-checkpoint/readback.json')['after_files'];assert len(before)==8 and all(sha(ROOT/name)==digest for name,digest in before.items())
for name in before:
    if name.endswith('.json'):assert read(ROOT/name)['current_product_identity']['source_manifest_sha256']==c['source_manifest_sha256']
out=run/('checkpoint-completed-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for name in before:z.write(ROOT/name,name)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(name)).hexdigest().upper()==digest for name,digest in before.items())
record=dict(status='current_source_default112_both_profiles_qualified_limited',run=str(run),source_manifest_sha256=c['source_manifest_sha256'],source_files=1040,compiled_input_files=1041,profiles=profiles,original_session_id=args.session,terminal_tool_chunk=args.terminal_chunk,observed_terminal_exit_code=0,completion=str(op),completion_sha256=sha(op),recorded_original_owned_identities=o['recorded_original_owned_identities'],recorded_original_owned_processes_all_terminal=True,full_library_qualified=False,ignored_bodies_executed=False,GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False)
save(out/'evidence.json',record)
work='來源4E1F7A4E原'+str(args.session)+'兩版廣域各112整合＋main／113實際映像已actual exit0並獨立逐名稱、raw、來源／映像與'+str(o['recorded_original_owned_identities'])+'組程序身分終止核對：Debug'+str(profiles['Debug']['counts']['passed'])+'／Release'+str(profiles['Release']['counts']['passed'])+'過，各0敗，既有忽略分別'+str(profiles['Debug']['counts']['ignored'])+'/'+str(profiles['Release']['counts']['ignored'])+'。只通過default整合與main範圍，完整library及忽略本文尚未執行；下一步同來源完整library兩版，再必要資源／正常GUI。正常兩版各90相關檢查、當版G06控制器／非同步Presentation各12案／96sessions、冰錐來源本體六案24sessions維持各自範圍。35辨識／39缺口仍是靜態請求範圍；正常GUI／實體輸入音效、未播放舊音效跨端點拒絕、同端點與其他服務及完整G06／G01–G12仍待。8／60秒、Release及G11不變，相鄰只讀且登入器與重建版產物獨立。'
for name in before:
    p=ROOT/name
    if p.suffix=='.json':
        value=read(p);value['current_native1040_default112']=record
        value['current_native1040_broad_active'].update(status='completed_qualified_default_scope',completion=str(op),completion_sha256=sha(op),terminal_tool_chunk=args.terminal_chunk,observed_terminal_exit_code=0,qualification_granted=True)
        value.update(active_workloads={},active_workload_sessions={},source_and_corresponding_QA_frozen=False,source_freeze_while_Cargo=False,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        value['current_focus']=dict(item='current1040-full-library-default',work=work,applies_to_current_source=True,goal_complete=False)
        if 'current_native_retired_join_normal'in value:value['current_native_retired_join_normal']['current_broad_default_verified']=True
        if 'current_source_goal_validation'in value:value['current_source_goal_validation']['G03-current-regression']=dict(state='current1040_default112_both_qualified_full_library_and_other_acceptance_pending',complete=False,evidence=record)
        for item in value.get('items',[]):
            if item['id']=='G03-current-regression':item.update(state='current1040_default112_qualified_full_library_and_other_acceptance_pending',complete=False,current_source_manifest_sha256=c['source_manifest_sha256'],current_evidence=record)
        if name.endswith('next-validation-action.json'):value['next_action']=work
        save(p,value)
    else:
        first,separator,rest=p.read_text('utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18')and separator
        p.write_text('2026-10-18：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes());save(out/'readback.json',dict(before_files=before,after_files={name:sha(ROOT/name)for name in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json')))
assert all(sha(ROOT/name)==digest for name,digest in read(out/'readback.json')['after_files'].items())
save(run.parent/'current-continuation.json',dict(**record,checkpoint=str(out),must_poll_original_session_before_restarting=False,next_action='Run current1040 full-library default selections after this terminal checkpoint. No historical evidence promotion.'))
print(json.dumps(dict(checkpoint=str(out),profiles=profiles,whole_goal_complete=False),ensure_ascii=False))
