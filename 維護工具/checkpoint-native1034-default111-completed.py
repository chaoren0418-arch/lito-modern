"""Record only the terminal, independently read current-source broad scope."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
def save(p,j):p.write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int,required=True);ap.add_argument('--terminal-chunk',required=True);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native1034-default111')
c=read(run/'cohort.json');op=run/'completion-observation.json';o=read(op)
assert c['status']=='independently_qualified_current_default111_both_profiles' and c['owned_process'] is None
assert o['status']=='independently_qualified_current1034_default111_both_profiles_and_terminal_processes'
assert o['observed_terminal_exit_code']==0 and o['recorded_original_owned_processes_all_terminal'] and o['cohort_sha256']==sha(run/'cohort.json')
assert a.terminal_chunk not in ['','pending','unknown']
profiles={}
for profile in ['Debug','Release']:
    info=c['profiles'][profile];qp=Path(info['qualification']);q=read(qp)
    assert info['qualified'] and info['actual_runner_exit_code']==0 and sha(qp)==info['qualification_sha256']
    assert q['source_manifest_sha256']==c['source_manifest_sha256']==o['source_manifest_sha256']
    assert q['counts']['failed']==0 and q['actual_test_executables']==112 and not q['full_library_scope_inferred']
    assert sha(qp.parent/'raw.zip')==q['raw_archive_sha256']
    with zipfile.ZipFile(qp.parent/'raw.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in q['raw_files'].items())
    profiles[profile]=dict(qualification=str(qp),qualification_sha256=sha(qp),counts=q['counts'],integration_targets=111,actual_test_executables=112,normal_mains=q['normal_mains'])
normal=Path(c['normal_qualification']).parent.parent;inputs=read(normal/'compiled-inputs.json');assert len(inputs)==1035
for n,h in inputs.items():assert sha(ROOT/n)==h
position_path=ROOT/'驗證紀錄/current/native-class2-position/20261018-005710-660316/binding.json'
position=read(position_path);assert position['status']=='primary_class2_entry_coordinate_capture_qualified'
for key in ['primary_PE','primary_ASM']:assert sha(position[key]['path'])==position[key]['sha256']
cold_path=ROOT/'驗證紀錄/current/native-cone-cold-binding/20261018-034221-586202/binding.json'
cold=read(cold_path);assert cold['status']=='primary_type5_cold_factory_and_retained_canonical_graph_bound_only' and cold['effect_type']==5 and cold['constructor']=='0x42F1F0' and cold['new_casts']==0 and not cold['renderer_qualified']
motion_path=ROOT/'驗證紀錄/current/native-cone-cold-motion/20261018-035026-100702/binding.json';motion=read(motion_path)
assert motion['status']=='primary_type5_motion_and_arrival_control_flow_bound_only' and motion['scheduler_argument']==60 and motion['new_casts']==0 and not motion['renderer_qualified']
cold_research_paths={
    'current_native_cone_cold_direction':'native-cone-cold-direction/20261018-040839-795290/binding.json',
    'current_native_cone_cold_scheduler':'native-cone-cold-scheduler/20261018-040413-691279/binding.json',
    'current_native_cone_cold_coordinates':'native-cone-cold-coordinates/20261018-041801-504497/binding.json',
    'current_native_cone_cold_signed_spatial_binding':'native-cone-cold-signed-spatial/20261018-041923-899485/binding.json',
    'current_native_cone_cold_wire_spatial_candidate':'native-cone-cold-wire-spatial-candidate/20261018-041940-672770/qualified-20261018-042006-939884/qualified.json',
    'current_native_cone_cold_fly_sampling':'native-cone-cold-fly-sampling/20261018-042329-570447/binding.json',
    'current_native_cone_cold_impact':'native-cone-cold-impact/20261018-042916-036300/binding.json',
    'current_native_cone_cold_asset_typecheck':'native-cone-cold-assets-typecheck/20261018-042642-923787/typecheck-observation/observation.json',
    'current_native_cone_cold_asset_bank':'native-cone-cold-assets-runtime/20261018-043145-772927/qualified-20261018-043307-343544/qualified.json',
}
cold_research={}
for key,name in cold_research_paths.items():
    path=ROOT/'驗證紀錄/current'/name;value=read(path)
    assert value['source_manifest_sha256']==c['source_manifest_sha256'] and value['new_casts']==0 and not value['whole_goal_complete']
    cold_research[key]=dict(path=str(path),sha256=sha(path),source_manifest_sha256=c['source_manifest_sha256'],status=value['status'],normal_product_registered=False,new_casts=0,renderer_qualified=False,whole_goal_complete=False)
    if key.endswith('candidate'):
        assert value['recorded_original_identities_all_terminal'] and value['observed_original_exit_code']==0 and all(v['passed']==18 and v['failed']==0 and v['ignored']==0 for v in value['profiles'].values())
        cold_research[key].update(profiles=value['profiles'],scope=value['scope'],signed32_shift_division_boundary_cases_qualified=True)
before=read(run/'active-checkpoint/readback.json')['after_files'];assert len(before)==8 and all(sha(ROOT/n)==h for n,h in before.items())
out=run/('checkpoint-completed-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in before:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
record=dict(status='current_source_default111_both_profiles_qualified_limited',run=str(run),source_manifest_sha256=c['source_manifest_sha256'],source_files=1034,compiled_input_files=1035,profiles=profiles,original_session_id=a.session,terminal_tool_chunk=a.terminal_chunk,observed_terminal_exit_code=0,completion=str(op),completion_sha256=sha(op),recorded_original_owned_identities=o['recorded_original_owned_identities'],recorded_original_owned_processes_all_terminal=True,full_library_qualified=False,ignored_bodies_executed=False,GUI_verified=False,whole_goal_complete=False)
save(out/'evidence.json',record)
work='來源86E65C7B原'+str(a.session)+'兩版廣域各111整合＋main／112實際映像已actual exit0並獨立逐名稱、raw、來源／映像與'+str(o['recorded_original_owned_identities'])+'組程序身分終止核對：Debug'+str(profiles['Debug']['counts']['passed'])+'／Release'+str(profiles['Release']['counts']['passed'])+'過，各0敗與'+str(profiles['Debug']['counts']['ignored'])+'既有忽略。只通過此default整合範圍，未執行完整library或忽略本文；下一步同來源完整library兩版與必要資源。正常兩版各57相關測試及當版G06控制器／非同步Presentation各12案／96sessions維持各自資格。正常GUI僅Space版本交換三封包，WGC逾時與使用者登入未完成之租約已到期封存，舊位址失效；GUI／實體輸入音效、40技能請求缺口與其他玩法、完整G06及G01–G12仍待。8／60秒、Release及G11不變，相鄰只讀且產物獨立。'
for n in before:
    p=ROOT/n
    if p.suffix=='.json':
        j=read(p);assert j['current_product_identity']['source_manifest_sha256']==c['source_manifest_sha256']
        j.setdefault('retained_native_class2_initial_binding_before_position_correction',j.get('current_native_class2_binding'))
        j['current_native_class2_binding']=dict(path=str(position_path),sha256=sha(position_path),scope=position['inference_scope'],source_manifest_sha256=c['source_manifest_sha256'],historical_initial_following_claim_superseded=True,target_position_captured_once_at_admitted_launch=True,timer_does_not_relocate_bolt=True,normal_product_registered=True,scoped_class2_rendering_and_protocol_qualified=True,pixel_brightness_verified=False,GUI_verified=False,whole_goal_complete=False)
        j['current_native_cone_cold_binding']=dict(path=str(cold_path),sha256=sha(cold_path),source_manifest_sha256=c['source_manifest_sha256'],effect_type=5,constructor='0x42F1F0',scope='primary PE factory/parser and retained canonical asset graph only',new_casts=0,renderer_qualified=False,request_support_registered=False,whole_goal_complete=False)
        j['current_native_cone_cold_motion']=dict(path=str(motion_path),sha256=sha(motion_path),source_manifest_sha256=c['source_manifest_sha256'],scope='primary PE movement/arrival control flow; signed wire conversion, direction and fly sampling have separate linked evidence; timer units and playback remain pending',scheduler_argument=60,new_casts=0,renderer_qualified=False,whole_goal_complete=False)
        j.update(cold_research)
        j['current_native1034_default111']=record;j['current_native1034_default111_active'].update(status='completed_qualified_default_scope',completion=str(op),completion_sha256=sha(op),terminal_tool_chunk=a.terminal_chunk,observed_terminal_exit_code=0)
        j.update(active_workloads={},active_workload_sessions={},source_and_corresponding_QA_frozen=False,source_freeze_while_Cargo=False,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        j['current_focus']=dict(item='current-source-full-library-default',work=work,applies_to_current_source=True,goal_complete=False)
        j['current_source_goal_validation']['G03-current-regression']=dict(state='86E6_default111_both_qualified_full_library_and_other_acceptance_pending',complete=False,evidence=record)
        for item in j.get('items',[]):
            if item['id']=='G03-current-regression':item.update(state='86E6_default111_qualified_full_library_and_other_acceptance_pending',complete=False,current_source_manifest_sha256=c['source_manifest_sha256'],current_evidence=record)
        if n.endswith('next-validation-action.json'):j['next_action']=work
        save(p,j)
    else:
        first,sep,rest=p.read_text(encoding='utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18') and sep;p.write_text('2026-10-18：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes());save(out/'readback.json',dict(before_files=before,after_files={n:sha(ROOT/n) for n in before},evidence_sha256=sha(out/'evidence.json')))
assert all(sha(ROOT/n)==h for n,h in read(out/'readback.json')['after_files'].items())
print(json.dumps(dict(checkpoint=str(out),profiles=profiles,whole_goal_complete=False),ensure_ascii=False))
