"""Record the two normal Cargo live scopes; leave all unexecuted goal scopes pending."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,x):Path(p).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('--debug',type=Path,required=True);ap.add_argument('--release',type=Path,required=True);a=ap.parse_args()
queue=read(ROOT/'工作佇列.json');assert not queue['active_workloads']and not queue['source_and_corresponding_QA_frozen']
source=queue['current_product_identity']['source_manifest_sha256'];profiles={};cohort=None
for profile,path in [('Debug',a.debug),('Release',a.release)]:
    path=path.resolve(strict=True);assert path.is_relative_to(V/'native-ability-normal-live');q=read(path)
    assert q['status']=='normal_current75_ability_self_and_nonclan_scope_qualified'and q['profile']==profile and q['source_manifest_sha256']==source
    assert q['actual_sessions']==q['actual_cast_requests']==4 and q['successful_self_casts']==2 and q['nonclan_target_rejections']==q['MP_rejections']==1
    assert q['recorded_original_owned_processes_terminal']and q['recorded_identities']==4 and q['observed_terminal_exit_code']==0 and q['wrapper_terminal_tool_observed']
    assert not q['source_overlays_used']and not q['normal_product_modified']and q['normal_source_tree_used']and sha(path.parent/'raw.zip')==q['raw_archive_sha256']
    receipt=read(path.parent.parent/'receipt.json');current=Path(receipt['compile']);assert sha(current/'cohort.json')==receipt['cohort_sha256']
    if cohort is None:cohort=current
    else:assert cohort==current
    profiles[profile]=dict(qualification=str(path),qualification_sha256=sha(path),sessions=4,cast_requests=4,self_casts=2,nonclan_no_response=1,MP_rejections=1,recovery_walk=q['MP_refusal_recovery_walk'],original_PNGs=q['original_PNGs'])
normal=read(cohort/'cohort.json');completion=read(cohort/'completion-observation.json');assert completion['observed_terminal_exit_code']==0 and completion['recorded_original_owned_processes_all_terminal']and completion['cohort_sha256']==sha(cohort/'cohort.json')
for n,h in read(cohort/'compiled-inputs.json').items():assert sha(ROOT/n)==h,n
for profile,item in normal['profiles'].items():assert sha(item['normal']['path'])==item['normal']['sha256']==queue['current_product_identity'][profile+'_normal_sha256']
before=read(Path(queue['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(ROOT/n)==h for n,h in before.items())
out=V/'native-ability-normal-live'/('checkpoint-both-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in before:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
record=dict(status='both_normal_current75_ability_scopes_qualified_limited',source_manifest_sha256=source,normal_cohort=str(cohort),normal_completion_sha256=sha(cohort/'completion-observation.json'),profiles=profiles,actual_sessions=8,actual_cast_requests=8,successful_self_casts=4,nonclan_no_response_cases=2,MP_rejection_recovery_walks=2,recorded_live_original_identities_terminal=8,source_overlays_used=False,normal_source_tree_used=True,normal_source_modified_by_tests=False,same_clan_positive_target_verified=False,natural_expiry_verified=False,buff_GUI_verified=False,normal_GUI_verified=False,physical_audio_verified=False,current_broad_default_verified=False,current_full_library_verified=False,current_G06_verified=False,whole_goal_complete=False)
save(out/'evidence.json',record)
work='來源'+source[:8]+'正常兩版各31相關案與11原資源特效、正常Cargo雙玩家共8實際sessions／4自身施法／2非同盟無回覆／2MP拒絕後移動恢復，均已獨立核對來源／映像／封包與原程序終止。僅此能力強化與繪圖範圍通過；下一步當版G06控制器與廣域116目標、完整library及必要正常可見性。44FE舊廣域／library不移記新版本，同端點現行75伺服器契約阻礙保留原證據。開發期間不請使用者登入，8秒／60秒／Release／G11預算不變，相鄰只讀且登入器產物獨立，正常GUI／實體音效及完整G01–G12未完成。'
for n in before:
    path=ROOT/n
    if path.suffix=='.json':
        value=read(path);assert value['current_product_identity']['source_manifest_sha256']==source
        value.update(current_native_ability_live=record,active_workloads={},active_workload_sessions={},source_and_corresponding_QA_frozen=False,candidate_and_corresponding_QA_frozen=False,source_freeze_while_Cargo=False,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        value['current_focus']=dict(item='ability-current-G06-and-broad',work=work,applies_to_current_source=True,goal_complete=False)
        for item in value.get('items',[]):
            if item['id'].startswith('G03'):item.update(state='ability_self_and_rejection_scope_qualified_remaining_gameplay_pending',complete=False,current_source_manifest_sha256=source,current_ability_evidence=record)
        if n.endswith('next-validation-action.json'):value['next_action']=work
        save(path,value)
    else:
        first,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert sep;datetime.strptime(first[:10],'%Y-%m-%d');path.write_text((datetime.now(timezone.utc)+timedelta(hours=8)).strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
save(out/'readback.json',dict(before_files=before,after_files={n:sha(ROOT/n)for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json')))
save(out.parent/'current-continuation.json',dict(**record,latest_checkpoint=str(out),latest_readback=str(out/'readback.json'),next_action=work,must_poll_original_before_restart=False))
print(json.dumps(dict(checkpoint=str(out),actual_sessions=8,normal_scope_qualified=True,whole_goal_complete=False),ensure_ascii=False))
