"""Record qualified controller/Presentation evidence while leaving full G06 open."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
def save(p,j):p.write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native1040-presentation-cohort')
c=read(run/'cohort.json');qp=run/'qualified/qualified.json';q=read(qp)
assert q['status']=='independently_qualified_current1040_Presentation_both_profiles_six_branches'
assert q['source_manifest_sha256']==c['source_manifest_sha256'] and q['actual_sessions']==96 and q['actual_cases']==12 and q['actual_diagnostic_executables']==4
assert q['original_exit_code']==0 and q['recorded_cohort_identities_all_terminal'] and q['actual_Presentation_executed']
assert not q['GUI_verified'] and not q['physical_audio_verified'] and not q['whole_G06_complete']
assert sha(qp.parent/'cohort-raw.zip')==q['raw_archive_sha256']
with zipfile.ZipFile(qp.parent/'cohort-raw.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in q['raw_files'].items())
for n,h in read(run/'compiled-inputs.json').items():assert sha(ROOT/n)==h
images_path=run/'diagnostic-image-observation.json';images=read(images_path);assert images['source_manifest_sha256']==c['source_manifest_sha256'] and len(images['images'])==8
assert all(p['visually_observed'] and sha(p['path'])==p['sha256'] for p in images['images'])
for case in q['cases'].values():
    p=Path(case['path']);j=read(p);assert sha(p)==case['sha256'] and j['recorded_owned_all_terminal'] and j['source_manifest_sha256']==c['source_manifest_sha256']
before=read(run/'active-checkpoint/readback.json')['after_files'];assert len(before)==8 and all(sha(ROOT/n)==h for n,h in before.items())
out=run/('tracker-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in before:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
record=dict(status='current_source_Presentation_both_profiles_six_branches_qualified_limited',run=str(run),qualification=str(qp),qualification_sha256=sha(qp),source_manifest_sha256=c['source_manifest_sha256'],normal_mains=c['normal_mains'],original_session_id=q['original_session_id'],original_exit_code=0,terminal_tool_chunk=q['terminal_tool_chunk'],actual_sessions=96,actual_cases=12,diagnostic_executables=4,actual_Presentation_executed=True,recorded_cohort_identities=q['recorded_cohort_identities'],recorded_cohort_identities_all_terminal=True,per_case_qualifications=q['cases'],unplayed_source_audio_rejection_verified=False,normal_EXE_runtime_inferred=False,GUI_verified=False,physical_input_verified=False,physical_audio_verified=False,same_endpoint_verified=False,whole_G06_complete=False,whole_goal_complete=False)
save(out/'evidence.json',record)
record.update(diagnostic_image_observation=str(images_path),diagnostic_image_observation_sha256=sha(images_path),diagnostic_images_visually_observed=8,pixel_brightness_verified=False)
save(out/'evidence.json',record)
work='來源4E1F7A4E當版G06控制器兩版六分支12案／96sessions已獨立核對；另原'+str(q['original_session_id'])+'之正常函式庫非同步Presentation同六分支兩版12案／96sessions已actual exit0並獨立封包／角色物品／退役與程序身分核對，四診斷映像、'+str(q['recorded_cohort_identities'])+'cohort記錄身分及各案子程序均終止。僅控制器與非同步繪圖診斷資格。下一步當版冰錐實際C02正向／拒絕／移動目標及場景退役音效回歸，再廣域／完整library及正常GUI。正常兩版各90相關檢查維持各自資格；技能書請求靜態辨識35／缺39不是玩法完成。GUI／實體輸入音效、未播放舊音效跨端點拒絕、同端點與其他服務、完整G06及G01–G12仍待。8／60秒、Release與G11預算不變，登入器／重建版產物獨立，相鄰只讀。'
for n in before:
    p=ROOT/n
    if p.suffix=='.json':
        j=read(p);assert j['current_product_identity']['source_manifest_sha256']==c['source_manifest_sha256']
        j['current_native1040_Presentation']=record
        j['current_native1040_Presentation_active'].update(status='completed_qualified_diagnostic_scope',qualification=str(qp),qualification_sha256=sha(qp),original_exit_code=0,terminal_tool_chunk=q['terminal_tool_chunk'])
        j.update(active_workloads={},active_workload_sessions={},source_and_corresponding_QA_frozen=False,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        j['current_focus']=dict(item='current-source-cone-cold-C02-runtime',work=work,applies_to_current_source=True,goal_complete=False)
        for key in ['current_native_G06_presentation','current_native_G06_failure_presentation']:
            if key in j:j[key].update(applies_to_current_source=False,retained_source_version_only=True)
        if 'G06-room-service-branches' in j['current_source_goal_validation']:j['current_source_goal_validation']['G06-room-service-branches'].update(state='current1040_controller_and_Presentation_six_branches_qualified_GUI_and_other_G06_pending',complete=False,evidence=str(out/'evidence.json'))
        for item in j.get('items',[]):
            if item['id']=='G06-room-service-branches':
                item.update(state='current1040_controller_and_Presentation_qualified_GUI_and_other_branches_pending',complete=False,current_Presentation_evidence=record)
                for step in item.get('rental_acceptance_steps',[]):
                    if step.get('id')=='G06.3d':step.update(state='current1040_controller_and_Presentation_qualified_GUI_and_other_branches_pending',current_Presentation_evidence=record,remaining='當版冰錐C02與退役音效回歸、正常GUI／實體輸入音效、未播放舊音效跨端點拒絕、同端點與其他服務及完整G06.3d仍待。')
        if n.endswith('next-validation-action.json'):j['next_action']=work
        save(p,j)
    else:
        first,sep,rest=p.read_text(encoding='utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18') and sep
        p.write_text('2026-10-18：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes());save(out/'readback.json',dict(before_files=before,after_files={n:sha(ROOT/n) for n in before},evidence_sha256=sha(out/'evidence.json'),before_archive_sha256=sha(out/'before.zip')))
assert all(sha(ROOT/n)==h for n,h in read(out/'readback.json')['after_files'].items())
print(json.dumps(dict(checkpoint=str(out),actual_sessions=96,actual_cases=12,whole_G06_complete=False,whole_goal_complete=False),ensure_ascii=False))
