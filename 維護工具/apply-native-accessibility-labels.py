"""Apply the observed skill-label repair; stage all source and continuation writes."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import hashlib,json,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
OLD='AB3CF26C4227A3E9E171AEA2E7765C1BA29120681BCB226B36FC53BA2635AE2D'
NAME='LiTo381-Modern/src/modern_lito/native_login_window.rs'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def encode(v):return(json.dumps(v,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
q=read(R/'工作佇列.json');assert q['current_product_identity']['source_manifest_sha256']==OLD
assert not q['active_workloads']and not q['source_and_corresponding_QA_frozen']and q['current_full_default_library_verified']
library=Path(q['current_native_catalog_library']['run']);inputs=read(library/'compiled-inputs.json');pins=read(library/'source-inputs.json')
assert len(inputs)==1048 and len(pins)==1047 and sha(library/'source-inputs.json')==OLD
for n,h in inputs.items():assert sha(R/n)==h,n
proofs=[]
for profile in ['Debug','Release']:
    receipt=q['current_normal_GUI_hotel_entry_keyboard'][profile]
    path=Path(receipt['qualification']);qualification=read(path)
    assert sha(path)==receipt['qualification_sha256']and qualification['source_manifest_sha256']==OLD
    assert qualification['observed_exit_code']==0 and qualification['recorded_original_owned_processes_terminal']
    case=Path(qualification['case']);events=read(case/'Sky-events.json')
    assert any('新密碼'in e['tree']and'ID: 208'in e['tree']for e in events)
    proofs.append(dict(qualification=str(path),qualification_sha256=sha(path),events=str(case/'Sky-events.json'),events_sha256=sha(case/'Sky-events.json')))
before=read(Path(q['last_goal_turn_evidence']).with_name('readback.json'))['after_files']
assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
original=(R/NAME).read_bytes();body=original.decode('utf-8-sig');nl='\r\n'if'\r\n'in body else'\n'
replacements=[
    ('const SKILLS: u16 = 208;','const SKILLS: u16 = 208;'+nl+'const SKILL_LABEL: u16 = 318;'),
    (nl.join(['            (DISCARD_ROOM,w!("BUTTON"),"取消編輯".into(),0),','            (','                SKILLS,']),nl.join(['            (DISCARD_ROOM,w!("BUTTON"),"取消編輯".into(),0),','            (SKILL_LABEL, w!("STATIC"), "技能".into(), 0),','            (','                SKILLS,'])),
    ('                SKILLS | CAST_SKILL => self.world_view && !self.result_view && !self.inventory_view,','                SKILL_LABEL => self.world_view && !self.result_view && !self.inventory_view && !self.rental.open,'+nl+'                SKILLS | CAST_SKILL => self.world_view && !self.result_view && !self.inventory_view,'),
    (nl.join(['            let _ = MoveWindow(','                self.control(SKILLS),','                px(16),','                area.bottom - px(90),','                (area.right - px(160)).max(px(80)),']),nl.join(['            let _ = MoveWindow(','                self.control(SKILL_LABEL),','                px(16),','                area.bottom - px(86),','                px(40),','                px(25),','                true,','            );','            let _ = MoveWindow(','                self.control(SKILLS),','                px(64),','                area.bottom - px(90),','                (area.right - px(208)).max(px(80)),']))]
for old,new in replacements:assert body.count(old)==1;body=body.replace(old,new)
payload=body.encode('utf-8');changed=hashlib.sha256(payload).hexdigest().upper()
new_pins=dict(pins);new_pins[NAME]=changed;source_bytes=encode(new_pins);source=hashlib.sha256(source_bytes).hexdigest().upper()
new_inputs=dict(inputs);new_inputs[NAME]=changed;assert source!=OLD
out=V/'native-accessibility-labels-normal'/('application-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
record=dict(status='applied_accessibility_labels_pending_normal_validation',before_source_manifest_sha256=OLD,source_manifest_sha256=source,
    changed_files=[NAME],after_files={NAME:changed},canonical_before=before,source_files=1047,compiled_input_files=1048,
    normal_validation_complete=False,normal_product_source_modified=True,source_overlays_used=False,diagnostic_API_extension=False,
    original_GUI_label_proofs=proofs,normal_GUI_label_fix_verified=False,GUI_verified=False,whole_goal_complete=False,
    change_scope='Adjacent visible skill label; world-only layout, hidden in inventory/results/NPC panel. No protocol, resources, clocks, or gameplay change.')
work='來源'+source[:8]+'只修技能下拉誤讀為新密碼：新增相鄰技能標籤與版面，NPC／背包／結算隱藏標籤；尚待正常兩版建置、相關16案、當版完整library及正常GUI名稱／巡覽／場景切換核對。AB3C的兩版全庫1969過0敗484忽略、資源10、控制器96及GUI限定入村8／旅館8sessions保留原版資格，不轉記新來源。繼續剩餘開發與自動測試，全部必要自測後才交人工；不請使用者登入，相鄰專案只讀。'
date=datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d');staged={NAME:payload}
for n in before:
    path=R/n
    if path.suffix=='.json':
        value=read(path);identity=value['current_product_identity'];assert identity['source_manifest_sha256']==OLD
        identity.update(source_manifest_sha256=source,source_qualification_pending=True,Debug_normal_qualified_for_current_source=False,Release_normal_qualified_for_current_source=False,GUI_verified=False,whole_goal_complete=False)
        value['qualification_before_accessibility_labels_change']=dict(source_manifest_sha256=OLD,library=value.get('current_native_catalog_library'),broad=value.get('current_native_catalog_broad_qualification'),normal=value.get('current_native_keyboard_focus_normal_qualification'),GUI_entry=value.get('current_normal_GUI_rental_entry_keyboard'),GUI_hotel=value.get('current_normal_GUI_hotel_entry_keyboard'))
        value['current_full_default_library_verified']=False;value['current_native_accessibility_labels_application']=record
        for key,info in value['current_source_goal_validation'].items():info.update(complete=False,current_source_manifest_sha256=source,prior_evidence_source_retained=OLD,prior_whole_source_qualification_applies_to_current_source=False)
        for item in value.get('items',[]):item.update(complete=False,current_source_manifest_sha256=source,current_evidence_applies_to_current_source=False)
        value.update(next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        value['current_focus']=dict(item='accessibility_labels_normal_library_GUI_validation',work=work,applies_to_current_source=True,goal_complete=False)
        if n.endswith('next-validation-action.json'):value['next_action']=work
        if'updated_date'in value:value['updated_date']=date
        staged[n]=encode(value)
    else:
        first,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert sep
        staged[n]=(date+'：'+work+'\n\n'+rest).encode('utf-8')
assert len(staged)==9 and all(sha(R/n)==h for n,h in before.items())and(R/NAME).read_bytes()==original
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in staged:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert z.read(NAME)==original and all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
with zipfile.ZipFile(out/'applied-payload.zip','x',zipfile.ZIP_DEFLATED)as z:z.writestr(NAME,payload)
for file,data in [('before-source-inputs.json',(library/'source-inputs.json').read_bytes()),('before-compiled-inputs.json',(library/'compiled-inputs.json').read_bytes()),('applied-source-inputs.json',source_bytes),('applied-compiled-inputs.json',encode(new_inputs))]:(out/file).write_bytes(data)
(out/'application.py').write_bytes(Path(__file__).read_bytes())
for n,data in staged.items():(R/n).write_bytes(data)
for n,h in new_inputs.items():assert sha(R/n)==h,n
record.update(before_archive_sha256=sha(out/'before.zip'),payload_archive_sha256=sha(out/'applied-payload.zip'),before_files={NAME:inputs[NAME],**before})
(out/'application.json').write_bytes(encode(record));(out/'evidence.json').write_bytes(encode(record))
(out/'readback.json').write_bytes(encode(dict(before_files=before,after_files={n:sha(R/n)for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'),source_after_sha256=changed)))
print(json.dumps(dict(application=str(out),source_manifest_sha256=source,changed_files=[NAME],normal_validation_complete=False),ensure_ascii=False))
