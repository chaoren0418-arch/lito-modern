"""Preserve the initial 18 metadata scope; correct late-bookmark and queued native input edges."""
from pathlib import Path
import argparse,copy,json
from native_E149_recovery_contract import R,read,sha,encoded,require
T=R/'維護工具';V=R/'驗證紀錄/current';B=T/'candidates/native-E149-mass69-integration-20261025-004000';C=T/'candidates/native-E149-mass69-input-20261025-004000'
def main(chunk):
 original=V/'native-E149-mass69-integration-metadata/20261024-163304-618808/cohort.json';first=read(original);require(first['status']=='eighteen_metadata_compilers_complete_no_tests' and first['owned_process']is None and [r['exit_code']for r in first['compiler_runs']]==[0]*18 and first['tests_executed']==0,'initial compiler scope incomplete');require(not C.exists(),'refinement exists');k=copy.deepcopy(read(B/'contract.json'));prefix='LiTo381-Modern/src/modern_lito/'
 deltas={
 'native_mass69.rs':[("        let owner=Character::current(s,epoch).context(\"bookmark without selected GAME character\")?;","        let Some(owner)=Character::current(s,epoch)else{return Ok(true);};")],
 'native_login.rs':[("native summon owner retired or eight-second portal timeout","傳送等待逾時或角色／場景已更新，請重新連線")],
 'native_login_window.rs':[
  ('                    ui.held.clear();ui.held_attack.clear();ui.mass_choice=None;','                    if ui.mass_menu.command_id()!=Some((wparam.0&0xffff)as u16){return LRESULT(0);}\n                    ui.held.clear();ui.held_attack.clear();ui.mass_choice=None;'),
  ('let focus=GetFocus();ui.mass_token.as_ref().and_then','let focus=GetFocus();if focus==ui.control(CANCEL)&&IsWindowEnabled(focus).as_bool(){Some((CANCEL,focus))}else{ui.mass_token.as_ref().and_then'),
  ('.map(|(_,yes)|(t.command_id(yes),focus)))};','.map(|(_,yes)|(t.command_id(yes),focus)))}};'),
  ('self.text(MASS_LABEL,"等待伺服器傳送結果。");','self.text(MASS_LABEL,"等待伺服器傳送結果；可用「取消」結束連線。");')]
 }
 for short,changes in deltas.items():
  name=prefix+short;row=k['runtime_overrides'][name];s=Path(row['path']).read_text('utf-8-sig')
  for a,b in changes:require(s.count(a)==1,'source refinement anchor differs '+short+': '+a[:90]);s=s.replace(a,b)
  row['changes']+=changes
  if short=='native_mass69.rs':
   test=T/'templates/native_mass69_late_bookmark_tests.rs';s+='\n'+test.read_text('utf-8-sig');row['supplements'].append(dict(path=str(test),sha256=sha(test)));k['preparation_pins'][str(test)]=sha(test)
  p=C/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_text(s,encoding='utf-8',newline='\n');row['path']=str(p);row['sha256']=sha(p);k['effective_overlay_paths'][name]=str(p);k['effective_overlay_files'][name]=sha(p)
 k.update(new_tests_written=10,source_review_precursor=str(B/'contract.json'),source_review_precursor_sha256=sha(B/'contract.json'),source_review_precursor_cohort=str(original),source_review_precursor_cohort_sha256=sha(original),source_review_precursor_observed_original_tool_chunk=chunk,source_review_precursor_observed_actual_exit_code=0,source_review_precursor_scope='initial18 metadata before no-owner S30 ignore / fully ignored stale combo / explicit Cancel Enter source corrections; not final scope',late_bookmark_without_owner_ignored=True,stale_combo_notification_does_not_mutate_current_choice=True,explicit_Cancel_Enter_during_portal_wait=True)
 compiler=T/'compile-native-E149-mass69-input.py';require(not compiler.exists(),'refined compiler exists');source=(T/'compile-native-E149-mass69-integration.py').read_text('utf-8-sig').replace('candidates/native-E149-mass69-integration-20261025-004000','candidates/native-E149-mass69-input-20261025-004000').replace('native-E149-mass69-integration-metadata','native-E149-mass69-input-metadata');needle=" inputs=read(old['base_compiled_inputs'])";require(source.count(needle)==1,'compiler source anchor differs');source=source.replace(needle," first=read(contract['source_review_precursor_cohort']);pins.update(first['input_pins']);pins[contract['source_review_precursor_cohort']]=contract['source_review_precursor_cohort_sha256'];pins[contract['source_review_precursor']]=contract['source_review_precursor_sha256'];pins[str(T/'compile-native-E149-mass69-integration.py')]=sha(T/'compile-native-E149-mass69-integration.py')\n"+needle);compiler.write_text(source,encoding='utf-8',newline='\n');k['preparation_pins'][str(Path(__file__).resolve())]=sha(Path(__file__).resolve());(C/'contract.json').write_bytes(encoded(k));print(json.dumps(dict(contract=str(C/'contract.json'),sha256=sha(C/'contract.json'),new_cases=10,tests_executed=0),ensure_ascii=False))
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--observed-original-tool-chunk',required=True);a=p.parse_args();main(a.observed_original_tool_chunk)
