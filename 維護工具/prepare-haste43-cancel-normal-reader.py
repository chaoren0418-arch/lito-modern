"""Keep all normal build/test/PE assertions; replace only the repair provenance block."""
from pathlib import Path
from datetime import datetime,timezone
import ast,difflib,hashlib,json
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current'
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
old_path=T/'qualify-native-installed-resource-profile-normal.py';old=old_path.read_text('utf-8-sig');start=old.index("candidate=read(application['candidate'])");end=old.index('release=tomllib.loads',start)
block='''candidate=read(application['candidate']);assert sha(application['candidate'])==application['candidate_sha256']
assert manifest=='E0E6409E0F436D705CA55486A9EB86AEE3F483C247201EC3AAD7596F8C49517D'
assert candidate['before_source_manifest_sha256']==application['before_source_manifest_sha256']=='A695C12799E2D7A508223ABE55CE5B5BDC81A15A85CE05451DF60672FC8823C6'
assert candidate['candidate_source_manifest_sha256']==manifest and not candidate['game_or_protocol_clocks_changed']
review=read(application['candidate_review']);assert sha(application['candidate_review'])==application['candidate_review_sha256']
assert review['candidate_sha256']==application['candidate_sha256'] and review['exact_six_files'] and review['original_assertions_preserved'] and review['cancel_recovery_saved_before_scene_retirement'] and review['formal_positive_negative_and_cancel_components_independently_qualified']
old=read(app/'before-compiled-inputs.json');assert set(old)==set(inputs) and len(old)==1049
expected_changes={'LiTo381-Modern/'+n for n in ['src/modern_lito/native_skills.rs','src/modern_lito/native_skill_selection.rs','src/modern_lito/native_login.rs','src/modern_lito/native_rental_return_tests.rs','tests/native_skills.rs','tests/native_skill_selection.rs']}
changed={n for n,h in inputs.items() if old[n]!=h};assert changed==set(application['changed_files'])==set(candidate['after_files'])==expected_changes
assert application['created_files']==[] and application['after_files']==candidate['after_files'] and application['normal_product_source_modified'] and not application['diagnostic_API_extension'] and not application['clocks_changed']
verify_archive(app/'before.zip',application['before_files']);assert sha(app/'before.zip')==application['before_archive_sha256']
verify_archive(app/'applied-payload.zip',application['after_files']);assert sha(app/'applied-payload.zip')==application['payload_archive_sha256']
with zipfile.ZipFile(app/'before.zip') as z:before_sources={n:z.read(n) for n in expected_changes}
for n in expected_changes:assert hashlib.sha256(before_sources[n]).hexdigest().upper()==old[n]==candidate['before_files'][n] and sha(ROOT/n)==candidate['after_files'][n]
for name,proof in candidate['proofs'].items():
 assert sha(proof['path'])==proof['sha256'];proof_data=read(proof['path']);assert proof_data['source_manifest_sha256']==application['before_source_manifest_sha256'] and proof_data['qualified'] and not proof_data.get('candidate_applied',False)
name='LiTo381-Modern/src/modern_lito/native_login.rs';base=before_sources[name].decode('utf-8').replace('\\r\\n','\\n');current=(ROOT/name).read_text('utf-8-sig')
at=base.index('    pub fn cancel(&mut self) {');finish=base.index('    fn accept(&mut self, packet: Packet)',at);body=base[at:finish]
conditional='        if self.teleport.is_some()||self.recovery_character.is_some(){self.retire_teleport_scene();}\\n';anchor='        self.teleport_recovery=self.teleport_recovery_candidate();\\n';assert body.count(conditional)==body.count(anchor)==1
assert current==base[:at]+body.replace(conditional,'').replace(anchor,anchor+'        self.retire_teleport_scene();\\n')+base[finish:]
name='LiTo381-Modern/src/modern_lito/native_skills.rs';base=before_sources[name].decode('utf-8').replace('\\r\\n','\\n');expected=base.replace('1 | 8 | 9 | 19 | 26 | 35 | 42 | 57 =>','1 | 8 | 9 | 19 | 26 | 35 | 42 | 43 | 57 =>').replace('matches!(skill, 8 | 9 | 26 | 42)','matches!(skill, 8 | 9 | 26 | 42 | 43)');assert (ROOT/name).read_text('utf-8-sig')==expected
name='LiTo381-Modern/src/modern_lito/native_skill_selection.rs';base=before_sources[name].decode('utf-8').replace('\\r\\n','\\n');anchor='                        42 => "#42 力量強化（選取目標／自己）".into(),\\n';assert base.count(anchor)==1
assert (ROOT/name).read_text('utf-8-sig')==base.replace(anchor,anchor+'                        43 => "#43 加速術（選取目標／自己）".into(),\\n')
for name in ['LiTo381-Modern/tests/native_skills.rs','LiTo381-Modern/tests/native_skill_selection.rs']:
 base=before_sources[name].decode('utf-8').replace('\\r\\n','\\n').splitlines();current=(ROOT/name).read_text('utf-8-sig').splitlines();index=0
 for line in base:
  if 'assert' in line:
   while index<len(current) and current[index]!=line:index+=1
   assert index<len(current),(name,line);index+=1
name='LiTo381-Modern/src/modern_lito/native_rental_return_tests.rs';base=before_sources[name].decode('utf-8').replace('\\r\\n','\\n');current=(ROOT/name).read_text('utf-8-sig');assert current.startswith(base) and len(re.findall(r'^#\\[test\\]',current,re.M))==len(re.findall(r'^#\\[test\\]',base,re.M))+1
prior=read(application['before_normal_qualification']);assert sha(application['before_normal_qualification'])==application['before_normal_qualification_sha256'] and prior['source_manifest_sha256']==application['before_source_manifest_sha256'] and prior['passed_per_profile']==19 and prior['recorded_owned_processes_all_terminal']
history=application['qualification_before_repair'];assert sha(history['state'])==history['state_sha256'];prior_state=read(history['state']);negative=prior_state['current_native_cancel_retirement_counterexample'];assert negative['qualified_counterexample'] and sha(negative['qualification'])==negative['qualification_sha256']
'''
new=old[:start]+block+old[end:];new=new.replace("'negative-baseline.json':Path(negative['qualification'])","'pre-apply-cancel-counterexample.json':Path(negative['qualification'])")
ast.parse(new);target=T/'qualify-haste43-cancel-normal.py';assert not target.exists();out=V/'native-haste43-cancel-tooling'/('normal-reader-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
(out/'reader-before.py').write_bytes(old_path.read_bytes());target.write_text(new,encoding='utf-8',newline='\n');(out/'reader-after.py').write_bytes(target.read_bytes());(out/'review.diff').write_text(''.join(difflib.unified_diff(old.splitlines(True),new.splitlines(True),fromfile='old-reader',tofile='new-reader')),encoding='utf-8',newline='')
record=dict(source_manifest_sha256='E0E6409E0F436D705CA55486A9EB86AEE3F483C247201EC3AAD7596F8C49517D',old_reader_sha256=sha(old_path),new_reader=str(target),new_reader_sha256=sha(target),unchanged_build_PE_test_names_count_archive_and_process_assertions=True,only_repair_provenance_replaced=True,executed=False)
(out/'preparation.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(record,ensure_ascii=False))
