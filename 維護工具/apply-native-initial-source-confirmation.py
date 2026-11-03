"""Apply the evidenced initial GAME source fix; preserve source and all eight handoffs."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import hashlib,json,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
OLD='4F2F911B4B70BDA16C4EE8C3EDD60223F3E72A22C8A3B1B808C3F5EABA60E6A2'
NAME='LiTo381-Modern/src/modern_lito/native_login.rs'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
encode=lambda v:(json.dumps(v,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
q=read(R/'工作佇列.json');assert q['current_product_identity']['source_manifest_sha256']==OLD
assert not q['active_workloads']and not q['source_and_corresponding_QA_frozen']and q['current_full_default_library_verified']
failure=q['current_normal_GUI_source_handoff_failure'];assert failure['source_manifest_sha256']==OLD and not failure['qualification_granted']
assert sha(failure['evidence'])==failure['evidence_sha256']
assert failure['formal_classic_source_handoff_TTL_seconds']==30 and failure['selection_to_GAME_version_seconds']>30
contract=failure['current_protocol_C8_contract'];assert sha(contract['method'])==contract['sha256'] and contract['live_GAME_C8_rebind_supported_by_bytecode']
library=Path(q['current_native_catalog_library']['run']);inputs=read(library/'compiled-inputs.json');pins=read(library/'source-inputs.json')
assert len(inputs)==1048 and len(pins)==1047 and sha(library/'source-inputs.json')==OLD
for n,h in inputs.items():assert sha(R/n)==h,n
before=read(Path(q['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
original=(R/NAME).read_bytes();body=original.decode('utf-8-sig');nl='\r\n'if'\r\n'in body else'\n'
lines=lambda rows:nl.join(rows)
changes=[
 ('    readiness_acked: bool,','    readiness_acked: bool,'+nl+'    confirm_initial_source: bool,'),
 ('            readiness_acked: false,','            readiness_acked: false,'+nl+'            confirm_initial_source: false,'),
 ('        self.readiness_acked = false;','        self.readiness_acked = false;'+nl+'        self.confirm_initial_source = false;'),
 (lines(['        ensure!(self.credentials.is_some(), "missing handoff credentials");','        self.transport.cancel();']),
  lines(['        ensure!(self.credentials.is_some(), "missing handoff credentials");',
         '        // LOGIN source handoffs expire while a user waits at the redirect.',
         '        // Rebind an ordinary initial GAME through its public C8 before',
         '        // publishing any character directory. Staged rental continuations',
         '        // retain the formal bootstrap auto-selection and must not replay C8.',
         '        self.confirm_initial_source = self.teleport.is_none() && self.recovery_character.is_none();',
         '        self.transport.cancel();'])),
 (lines(['        if self.state.phase == LoginPhase::LoadingCharacters && self.directory.complete() {','            self.state.characters = self.directory.view()?;']),
  lines(['        if self.state.phase == LoginPhase::LoadingCharacters && self.directory.complete() {',
         '            if self.confirm_initial_source {',
         '                let source = self.state.selected_source.as_ref().context("missing initial GAME source")?;',
         '                self.transport.send(self.transport.snapshot().epoch, source_request(source)?)?;',
         '                self.confirm_initial_source = false;',
         '                self.directory = super::native_game::Directory::default();',
         '                self.deadline = Some(Instant::now() + Duration::from_secs(8));',
         '                return Ok(());',
         '            }',
         '            self.state.characters = self.directory.view()?;'])),
 ('    pub fn cancel(&mut self) {','    pub fn cancel(&mut self) {'+nl+'        self.confirm_initial_source = false;'),
 ('    fn fail(&mut self, error: String) {','    fn fail(&mut self, error: String) {'+nl+'        self.confirm_initial_source = false;'),
]
for old,new in changes:assert body.count(old)==1,old;body=body.replace(old,new)
payload=body.encode('utf-8');changed=hashlib.sha256(payload).hexdigest().upper()
new_pins=dict(pins);new_pins[NAME]=changed;source_bytes=encode(new_pins);source=hashlib.sha256(source_bytes).hexdigest().upper()
new_inputs=dict(inputs);new_inputs[NAME]=changed;assert source!=OLD
out=V/'native-initial-source-confirmation-normal'/('application-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
record=dict(status='applied_initial_source_confirmation_pending_validation',before_source_manifest_sha256=OLD,source_manifest_sha256=source,
    changed_files=[NAME],after_files={NAME:changed},canonical_before=before,source_files=1047,compiled_input_files=1048,
    normal_validation_complete=False,normal_product_source_modified=True,source_overlays_used=False,diagnostic_API_extension=False,
    original_GUI_source_handoff_failure=failure,initial_GAME_C8_runtime_verified=False,normal_GUI_verified=False,whole_goal_complete=False,
    change_scope='Ordinary initial GAME uses public C8 after its first complete directory; withhold that first directory and publish the confirmed source only. Staged rental teleport/recovery bypass this extra request. No JAR, packet format, server TTL, native 8s or peer 60s changes.')
work='來源'+source[:8]+'已套初次GAME來源確認修正：首份角色清單先以既有GAME C8重綁選定來源，完成確認清單才可選角；租裝目的地／恢復自動接續保留。尚未編譯或通過測試。4F2F兩版完整library／資源與Debug限定GUI保留原版，Release49194來源到期失敗不改記。先正常兩版與相關19案、完整library，再自然超30秒GUI、快速交接及傳送／恢復回歸；不改JAR或時限，不請使用者操作。'
date=datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d');staged={NAME:payload}
for n in before:
    path=R/n
    if path.suffix=='.json':
        j=read(path);identity=j['current_product_identity'];assert identity['source_manifest_sha256']==OLD
        identity.update(source_manifest_sha256=source,source_qualification_pending=True,Debug_normal_qualified_for_current_source=False,Release_normal_qualified_for_current_source=False,GUI_verified=False,whole_goal_complete=False)
        j['qualification_before_initial_source_confirmation']=dict(source_manifest_sha256=OLD,library=j.get('current_native_catalog_library'),normal=j.get('current_native_accessibility_labels_normal_qualification'),resource=j.get('current_native_library_resource_optin'),GUI_entry=j.get('current_normal_GUI_rental_entry_keyboard'),source_failure=failure)
        j['current_full_default_library_verified']=False;j['current_native_initial_source_confirmation_application']=record
        for info in j['current_source_goal_validation'].values():info.update(complete=False,current_source_manifest_sha256=source,prior_evidence_source_retained=OLD,prior_whole_source_qualification_applies_to_current_source=False)
        for item in j.get('items',[]):item.update(complete=False,current_source_manifest_sha256=source,current_evidence_applies_to_current_source=False)
        j.update(next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        j['current_focus']=dict(item='initial_GAME_source_confirmation_validation',work=work,applies_to_current_source=True,goal_complete=False)
        if n.endswith('next-validation-action.json'):j['next_action']=work
        if 'updated_date'in j:j['updated_date']=date
        staged[n]=encode(j)
    else:
        first,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert sep
        staged[n]=(date+'：'+work+'\n\n'+rest).encode('utf-8')
assert len(staged)==9 and all(sha(R/n)==h for n,h in before.items())and(R/NAME).read_bytes()==original
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in staged:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert z.read(NAME)==original and all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
with zipfile.ZipFile(out/'applied-payload.zip','x',zipfile.ZIP_DEFLATED)as z:z.writestr(NAME,payload)
for filename,data in [('before-source-inputs.json',(library/'source-inputs.json').read_bytes()),('before-compiled-inputs.json',(library/'compiled-inputs.json').read_bytes()),('applied-source-inputs.json',source_bytes),('applied-compiled-inputs.json',encode(new_inputs))]:(out/filename).write_bytes(data)
(out/'application.py').write_bytes(Path(__file__).read_bytes())
for n,data in staged.items():(R/n).write_bytes(data)
for n,h in new_inputs.items():assert sha(R/n)==h,n
record.update(before_archive_sha256=sha(out/'before.zip'),payload_archive_sha256=sha(out/'applied-payload.zip'),before_files={NAME:inputs[NAME],**before})
(out/'application.json').write_bytes(encode(record));(out/'evidence.json').write_bytes(encode(record))
(out/'readback.json').write_bytes(encode(dict(before_files=before,after_files={n:sha(R/n)for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'),source_after_sha256=changed)))
print(json.dumps(dict(application=str(out),source_manifest_sha256=source,changed_files=[NAME],normal_validation_complete=False),ensure_ascii=False))
