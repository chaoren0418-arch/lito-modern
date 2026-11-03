"""Apply the independently reproduced owned-focus repair, preserving all prior scope."""
from pathlib import Path
from datetime import datetime, timezone, timedelta
import hashlib, json, subprocess, zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
OLD='2DAF4DE08EF396B1DE5B2264820E689859BFCD837015D71A6D74643A2DDEE9DE'
NAME='LiTo381-Modern/src/modern_lito/native_login_window.rs'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def encode(d):return(json.dumps(d,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
q=read(R/'工作佇列.json');assert q['current_product_identity']['source_manifest_sha256']==OLD
assert not q['active_workloads']and not q['source_and_corresponding_QA_frozen']and q['current_full_default_library_verified']
library=Path(q['current_native_catalog_library']['run']);inputs=read(library/'compiled-inputs.json');pins=read(library/'source-inputs.json')
assert len(inputs)==1048 and len(pins)==1047 and sha(library/'source-inputs.json')==OLD
for n,h in inputs.items():assert sha(R/n)==h,n
proof=V/'native-keyboard-focus-probe/20261019-130334-204342/receipt.json';probe=read(proof)
assert probe['status']=='isolated_owned_control_focus_loss_reproduced_candidate_API_checks_passed'
assert probe['normal_source_manifest_sha256']==OLD and probe['normal_UI_source_sha256']==inputs[NAME]
assert probe['normal_source_inputs_unchanged']and not probe['normal_product_modified']and all(j['exit_code']==0 for j in probe['processes'])
assert sha(probe['actual_exe'])==probe['actual_exe_sha256']
ids=[probe['pipeline'],*probe['processes']];numeric=','.join(str(j['pid'])for j in ids)
command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+numeric+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-Command',command]);live=read_json=json.loads(raw.decode('utf-8-sig'))if raw.strip()else[]
live=[live]if isinstance(live,dict)else live
assert not any(x['ProcessId']==j['pid']and abs((datetime.fromisoformat(x['created'].replace('Z','+00:00'))-datetime.fromisoformat(j['creation_utc'])).total_seconds())<.001 for x in live for j in ids)
case=V/'native-catalog-sky-gui/case-20261019-125408-897618-debug-cross-endpoint'
c=read(case/'cohort.json');events=read(case/'Sky-events.json');assert c['status']=='failed_preserved'and c['recorded_owned_processes_terminal']
assert any(e['label']=='game-ready'and '14 按鈕 取得角色 ID: 105'in e['tree']for e in events)
assert any(e['label']=='focus-after-source'and e['focused_element']is None for e in events)
before=read(Path(q['last_goal_turn_evidence']).with_name('readback.json'))['after_files']
assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
original=(R/NAME).read_bytes();body=original.decode('utf-8-sig');nl='\r\n'if'\r\n'in body else'\n'
old_import='Input::KeyboardAndMouse::{EnableWindow, GetFocus, SetFocus}'
assert body.count(old_import)==1;body=body.replace(old_import,'Input::KeyboardAndMouse::{EnableWindow, GetFocus, IsWindowEnabled, SetFocus}')
anchor=nl.join(['        self.refresh_rental();','    }','    unsafe fn action(&mut self, id: u16) -> Result<()> {'])
replacement=nl.join(['        self.refresh_rental();','        self.restore_keyboard_focus_after_update();','    }',
    '    unsafe fn restore_keyboard_focus_after_update(&self) {',
    '        if self.closing || GetForegroundWindow() != self.parent { return; }',
    '        let focused = GetFocus();',
    '        let retired = focused.0.is_null() || self.controls.iter().any(|(_, control)|',
    '            *control == focused && !IsWindowEnabled(*control).as_bool());',
    '        if retired { let _ = SetFocus(Some(self.parent)); }',
    '    }','    unsafe fn action(&mut self, id: u16) -> Result<()> {'])
assert body.count(anchor)==1;body=body.replace(anchor,replacement);payload=body.encode('utf-8')
changed=hashlib.sha256(payload).hexdigest().upper();new_pins=dict(pins);new_pins[NAME]=changed
source_bytes=encode(new_pins);source=hashlib.sha256(source_bytes).hexdigest().upper();assert source!=OLD
new_inputs=dict(inputs);new_inputs[NAME]=changed
out=V/'native-keyboard-focus-normal'/('application-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
record=dict(status='applied_keyboard_focus_pending_normal_validation',before_source_manifest_sha256=OLD,source_manifest_sha256=source,
    changed_files=[NAME],after_files={NAME:changed},canonical_before=before,source_files=1047,compiled_input_files=1048,
    normal_validation_complete=False,normal_product_source_modified=True,source_overlays_used=False,diagnostic_API_extension=False,
    isolated_focus_API_proof=str(proof),isolated_focus_API_proof_sha256=sha(proof),original_GUI_case=str(case),
    original_GUI_case_outer_exit_code=1,original_GUI_case_terminal_chunk='10d46a',GUI_verified=False,whole_goal_complete=False)
work='來源'+source[:8]+'只修正常GUI控制項停用後的鍵盤焦點：僅自有前景視窗、非關閉中、焦點空缺或屬於已停用自有控制項時回到父視窗；不搶有效控制項或背景焦點。隔離API已實際重現原EnableWindow失焦及四項修正條件；正常兩版建置／相關冒煙、當版廣域／完整library與正常GUI尚待。2DAF兩版library各1969過0敗484忽略、廣域544／540及236服務sessions只保留原版本資格。Sky擷取仍逾時，原GUI兩批失敗與探針紀錄保留；先自行測試才交人工，不請使用者登入，相鄰只讀，G01–G12未完成。'
date=datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d');staged={NAME:payload}
for n in before:
    p=R/n
    if p.suffix=='.json':
        j=read(p);identity=j['current_product_identity'];assert identity['source_manifest_sha256']==OLD
        identity.update(source_manifest_sha256=source,source_qualification_pending=True,Debug_normal_qualified_for_current_source=False,Release_normal_qualified_for_current_source=False,GUI_verified=False,whole_goal_complete=False)
        j['qualification_before_keyboard_focus_change']=dict(source_manifest_sha256=OLD,library=j.get('current_native_catalog_library'),broad=j.get('current_native_catalog_broad_qualification'),normal=j.get('current_native_catalog_normal_qualification'))
        j['current_full_default_library_verified']=False;j['current_native_keyboard_focus_application']=record
        for key,value in j['current_source_goal_validation'].items():
            value.update(complete=False,current_source_manifest_sha256=source,prior_evidence_source_retained=OLD,prior_whole_source_qualification_applies_to_current_source=False)
        for item in j.get('items',[]):
            item.update(complete=False,current_source_manifest_sha256=source,current_evidence_applies_to_current_source=False)
            if item['id']=='G01-current-baseline':item['state']='keyboard_focus_normal_two_profile_build_and_baseline_pending'
            if item['id']=='G03-current-regression':item['state']='keyboard_focus_current_source_related_broad_and_full_library_pending'
            if item['id']=='G08-player-gui':item['state']='keyboard_focus_repair_applied_normal_GUI_and_capture_pending'
        j.update(active_workloads={},active_workload_sessions={},source_and_corresponding_QA_frozen=False,source_freeze_while_Cargo=False,
            next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        j['current_focus']=dict(item='keyboard_focus_normal_build_and_GUI_regression',work=work,applies_to_current_source=True,goal_complete=False)
        if n.endswith('next-validation-action.json'):j['next_action']=work
        if'updated_date'in j:j['updated_date']=date
        staged[n]=encode(j)
    else:
        first,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert sep;datetime.strptime(first[:10],'%Y-%m-%d');staged[n]=(date+'：'+work+'\n\n'+rest).encode('utf-8')
assert len(staged)==9 and all(sha(R/n)==h for n,h in before.items())and(R/NAME).read_bytes()==original
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in staged:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert z.read(NAME)==original and all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
with zipfile.ZipFile(out/'applied-payload.zip','x',zipfile.ZIP_DEFLATED)as z:z.writestr(NAME,payload)
for file,data in [('before-source-inputs.json',(library/'source-inputs.json').read_bytes()),('before-compiled-inputs.json',(library/'compiled-inputs.json').read_bytes()),('applied-source-inputs.json',source_bytes),('applied-compiled-inputs.json',encode(new_inputs))]:(out/file).write_bytes(data)
(out/'application.py').write_bytes(Path(__file__).read_bytes());(out/'original-probe-process-CIM.json').write_bytes(raw)
for n,data in staged.items():(R/n).write_bytes(data)
for n,h in new_inputs.items():assert sha(R/n)==h,n
after={n:sha(R/n)for n in before};record.update(before_archive_sha256=sha(out/'before.zip'),payload_archive_sha256=sha(out/'applied-payload.zip'),before_files={NAME:inputs[NAME],**before})
(out/'application.json').write_bytes(encode(record));(out/'evidence.json').write_bytes(encode(record))
(out/'readback.json').write_bytes(encode(dict(before_files=before,after_files=after,before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'),source_after_sha256=changed)))
print(json.dumps(dict(application=str(out),source_manifest_sha256=source,changed_files=[NAME],normal_validation_complete=False),ensure_ascii=False))
