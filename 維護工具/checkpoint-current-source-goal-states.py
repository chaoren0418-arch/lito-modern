"""Normalize current state labels without replacing or promoting historical evidence."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current';read=lambda p:json.loads(Path(p).read_text('utf-8-sig'));sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper();data=lambda x:(json.dumps(x,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
w=read(R/'工作佇列.json');SOURCE='3E58268A74B9908597221E71BDEC646C1DFEB82A8658444BDF1C24BFABE94628';assert w['current_product_identity']['source_manifest_sha256']==SOURCE
active=w['current_native_catalog_library_active'];assert active['source_manifest_sha256']==SOURCE and set(active['independently_qualified_profiles'])=={'Debug'} and w['active_workload_sessions']=={'full_library':17087}
proof=active['independently_qualified_profiles']['Debug'];assert sha(proof['qualification'])==proof['qualification_sha256'] and proof['passed']==1975 and proof['failed']==0
assert not w['current_full_default_library_verified'];before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
states={x['id']:'current_source_required_scope_incomplete_prior_source_evidence_retained' for x in w['items']}
states.update({'G01-current-baseline':'catalog_normal_two_profile_related_baseline_qualified_limited','G03-current-regression':'current_library_Debug1975_qualified_Release_original_running_broad_and_GUI_pending','G05-gameplay-matrix':'static_request38_of_book74_missing36_current_source_full_gameplay_qualification_pending','G06-room-service-branches':'current_source_API_controller_and_GUI_branches_pending_prior_source_evidence_retained','G08-player-gui':'current_source_normal_GUI_full_flow_pending','G12-final-audit':'pending_full_G01_G12_current_source_acceptance'})
out=V/'current-source-state-normalization'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True);record=dict(source_manifest_sha256=SOURCE,status='current_state_labels_normalized_prior_evidence_retained',active_sessions=w['active_workload_sessions'],states=states,new_test_sessions=0,new_qualifications_granted=False,historical_results_modified=False,whole_goal_complete=False)
(out/'evidence.json').write_bytes(data(record));(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes())
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in before:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
staged={}
for n in before:
    p=R/n
    if p.suffix=='.json':
        j=read(p);assert j['current_product_identity']['source_manifest_sha256']==SOURCE and j['active_workload_sessions']==w['active_workload_sessions']
        for item in j.get('items',[]):
            item.setdefault('state_retained_before_3E58268A_label_normalization',item.get('state'));item.update(state=states[item['id']],complete=False,current_source_manifest_sha256=SOURCE,current_source_state_label_authoritative=True)
        for key,item in j['current_source_goal_validation'].items():
            if key in states:item.setdefault('state_retained_before_3E58268A_label_normalization',item.get('state'));item.update(state=states[key],complete=False,current_source_manifest_sha256=SOURCE,current_source_state_label_authoritative=True)
        j.update(last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False);staged[n]=data(j)
    else:staged[n]=p.read_bytes()
assert all(sha(R/n)==h for n,h in before.items())
for n,payload in staged.items():(R/n).write_bytes(payload)
(out/'readback.json').write_bytes(data(dict(before_files=before,after_files={n:sha(R/n) for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))));print(json.dumps(dict(checkpoint=str(out),active_sessions=w['active_workload_sessions'],runtime_or_qualification_changed=False),ensure_ascii=False))
