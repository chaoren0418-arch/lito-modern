"""Record the targeted normal-source change without carrying old qualifications forward."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,x):Path(p).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
out=Path(read(V/'native-same-endpoint-catalog-normal/current-application.json')['path']);c=read(out/'application.json');assert not c['source_applied']
queue=read(ROOT/'工作佇列.json');assert not queue['active_workloads']and not queue['source_and_corresponding_QA_frozen']
old=c['before_source_manifest_sha256'];assert queue['current_product_identity']['source_manifest_sha256']==old
source=read(out/'before-source-inputs.json');inputs=read(out/'before-compiled-inputs.json');changed={n:sha(ROOT/n)for n,h in inputs.items()if sha(ROOT/n)!=h}
assert set(changed)==set(c['changed_files'])and len(changed)==3
before=c['canonical_before'];assert len(before)==8 and all(sha(ROOT/n)==h for n,h in before.items())
for n,h in changed.items():source[n]=inputs[n]=h
save(out/'applied-source-inputs.json',dict(sorted(source.items())));save(out/'applied-compiled-inputs.json',dict(sorted(inputs.items())))
with zipfile.ZipFile(out/'applied-payload.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in changed:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'applied-payload.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in changed.items())
new=sha(out/'applied-source-inputs.json');assert len(source)==1047 and len(inputs)==1048 and new!=old
c.update(status='applied_same_endpoint_catalog_pending_normal_validation',source_applied=True,source_manifest_sha256=new,after_files=changed,payload_archive_sha256=sha(out/'applied-payload.zip'),source_overlays_used=False,diagnostic_API_extension=False,normal_product_source_modified=True,new_behavior_tests=6,normal_validation_complete=False,whole_goal_complete=False)
save(out/'application.json',c);(out/'apply-recorder.py').write_bytes(Path(__file__).read_bytes())
record=dict(status=c['status'],application=str(out),application_sha256=sha(out/'application.json'),source_manifest_sha256=new,before_source_manifest_sha256=old,source_files=1047,compiled_input_files=1048,changed_files=changed,new_behavior_tests=6,source_overlays_used=False,normal_product_source_modified=True,diagnostic_API_extension=False,normal_related_qualification=False,actual_same_endpoint_sessions_verified=False,GUI_verified=False,whole_goal_complete=False)
save(out/'evidence.json',record)
work='來源'+new[:8]+'已加入同端點租裝路由目錄的一次性接續：僅同位址／連接埠、原角色與下一個GAME連線可重新綁定，舊世界／物品與可用狀態仍退役，新GAME服務目錄優先。新增六項行為測試，正常兩版建置與當版相關測試／實際雙玩家待驗。33A3兩版廣域、library及其他資格只留原來源；原33451失敗保留，物品接續須依正式租裝暫存範圍核對。開發期間不請使用者登入，8秒／60秒／Release與G11預算不變，相鄰只讀且登入器產物獨立，完整G01–G12未完成。'
date=(datetime.now(timezone.utc)+timedelta(hours=8)).strftime('%Y-%m-%d')
for n in before:
    p=ROOT/n
    if p.suffix=='.json':
        value=read(p);identity=value['current_product_identity'];assert identity['source_manifest_sha256']==old
        value['retained_33A3_product_identity']=identity;value['retained_33A3_goal_validation']=value.get('current_source_goal_validation')
        for key,item in value.items():
            if key.startswith('current_native')and isinstance(item,dict)and item.get('source_manifest_sha256')==old:item.update(applies_to_current_source=False,retained_source_version_only=True)
        value['current_product_identity']=dict(source_manifest_sha256=new,source_files=1047,compiled_input_files=1048,source_qualification_pending=True,Debug_normal_sha256=identity['Debug_normal_sha256'],Release_normal_sha256=identity['Release_normal_sha256'],existing_normal_source_manifest_sha256=old,Debug_normal_qualified_for_current_source=False,Release_normal_qualified_for_current_source=False,GUI_verified=False,whole_goal_complete=False)
        previous=value.get('current_source_goal_validation',{})
        value['current_source_goal_validation']={key:dict(state='same_endpoint_catalog_new_source_validation_pending',complete=False,source_manifest_sha256=new,pending_application=str(out))for key in previous}
        for item in value.get('items',[]):
            if item.get('id','').startswith('G'):item.update(complete=False,current_source_manifest_sha256=new,current_evidence_applies_to_current_source=False,pending_application=str(out))
        value.update(current_native_same_endpoint_catalog_application=record,current_full_default_library_verified=False,active_workloads={},active_workload_sessions={},source_and_corresponding_QA_frozen=False,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        value['current_focus']=dict(item='same-endpoint-catalog-normal-validation',work=work,applies_to_current_source=True,goal_complete=False)
        if n.endswith('next-validation-action.json'):value['next_action']=work
        if 'updated_date'in value:value['updated_date']=date
        save(p,value)
    else:
        first,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert sep;datetime.strptime(first[:10],'%Y-%m-%d');p.write_text(date+'：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
save(out/'readback.json',dict(before_files=before,after_files={n:sha(ROOT/n)for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json')))
save(out.parent/'current-application.json',dict(path=str(out),status=c['status'],source_manifest_sha256=new,application_sha256=sha(out/'application.json'),latest_readback=str(out/'readback.json')))
save(out.parent/'current-continuation.json',dict(**record,latest_readback=str(out/'readback.json'),next_action='Build both normal EXEs using the unique entry; run component/service regressions, then fresh ordinary75 same-endpoint cases with formal inventory scope. Retain old-source evidence and failed33451.'))
print(json.dumps(dict(application=str(out),source_manifest_sha256=new,source_files=1047,compiled_input_files=1048,new_behavior_tests=6,qualification_granted=False),ensure_ascii=False))
