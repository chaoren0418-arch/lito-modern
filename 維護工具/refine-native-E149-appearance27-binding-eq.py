"""Preserve the original Eq compile failure and derive the comparison-only correction."""
from pathlib import Path
import json,copy
from native_E149_recovery_contract import R,read,sha,encoded,require
T=R/'維護工具';V=R/'驗證紀錄/current';C=T/'candidates/native-E149-appearance27-binding-admission-20261024-144000';OUT=T/'candidates/native-E149-appearance27-binding-eq-20261024-144000';PREFIX='LiTo381-Modern/src/modern_lito/'
def main():
 cp=C/'contract.json';require(sha(cp)=='88680F9714E905A9765007F273151D6377E007AFE864FED92EABABAC13E78B6C','failed candidate changed');cohort=V/'native-E149-appearance27-binding-metadata/20261024-144741-317912/cohort.json';c=read(cohort);require(c['status']=='original_compile_failure_preserved' and c['owned_process'] is None and [(r['profile'],r['kind'],r['exit_code'])for r in c['compiler_runs']]==[('Debug','library',1)],'original failure not terminal');k=copy.deepcopy(read(cp));require(not OUT.exists(),'corrected candidate exists')
 old=T/'templates/native_appearance27_binding_support.rs';new=T/'templates/native_appearance27_binding_eq_support.rs';require(not new.exists(),'corrected support exists');s=old.read_text('utf-8-sig');a='#[derive(Clone,Debug,serde::Serialize)]\npub(crate) struct Tape';b='#[derive(Clone,Debug,PartialEq,Eq,serde::Serialize)]\npub(crate) struct Tape';require(s.count(a)==1,'comparison derive anchor differs');new.write_text(s.replace(a,b),encoding='utf-8',newline='\n')
 name=PREFIX+'native_appearance27.rs';row=k['runtime_overrides'][name];p=Path(row['path']);require(sha(p)==row['sha256'],'failed module changed');s=p.read_text('utf-8-sig');require(s.count(a)==1,'failed source derive absent');s=s.replace(a,b);OUT.mkdir();target=OUT/name;target.parent.mkdir(parents=True);target.write_text(s,encoding='utf-8',newline='\n');row.update(path=str(target),sha256=sha(target));k['effective_overlay_paths'][name]=str(target);k['effective_overlay_files'][name]=sha(target)
 matches=[x for x in row['supplements'] if x['path']==str(old)];require(len(matches)==1,'support pin absent');matches[0].update(path=str(new),sha256=sha(new))
 post=[delta for delta in row['changes'] if delta[0].startswith((' pub(crate) fn record(&mut self,entry:Entry){','self.queue.prune(tape.dropped_through);'))];require(len(post)==2,'post-supplement corrections differ');row['changes']=[delta for delta in row['changes'] if delta not in post];row['post_supplement_changes']=post
 # Reconstruct every corrected override BEFORE another compiler is launched.
 for n,r in k['runtime_overrides'].items():
  require(sha(r['predecessor'])==r['predecessor_sha256'],'predecessor changed');text=Path(r['predecessor']).read_text('utf-8-sig')
  for x,y in r['changes']:
   count=r.get('repeated_derive_replacement_count',1) if x=='Debug,PartialEq,Eq)]' else 1;require(text.count(x)==count,'reconstruction anchor differs '+n+': '+x[:90]);text=text.replace(x,y)
  for extra in r['supplements']:require(sha(extra['path'])==extra['sha256'],'supplement changed');text+='\n'+Path(extra['path']).read_text('utf-8-sig')
  for x,y in r.get('post_supplement_changes',[]):require(text.count(x)==1,'post-supplement anchor differs');text=text.replace(x,y)
  require(text==Path(r['path']).read_text('utf-8-sig') and sha(r['path'])==r['sha256'],'corrected source reconstruction differs')
 k.update(original_failed_metadata_cohort=str(cohort),original_failed_metadata_cohort_sha256=sha(cohort),original_failed_candidate=str(cp),original_failed_candidate_sha256=sha(cp),observed_original_failure_tool_chunk='c4630f',observed_original_failure_actual_exit_code=1,original_failure_reason='E0369/E0277: Tape required PartialEq/Eq for Actor derives; only the Tape comparison derive changed in corrected runtime source',source_reconstruction_preflight_complete=True)
 target=OUT/'contract.json';target.write_bytes(encoded(k));print(json.dumps(dict(contract=str(target),sha256=sha(target),reconstructed_runtime_files=5,new_cases=10,tests_executed=0,original_failure_preserved=str(cohort)),ensure_ascii=False))
if __name__=='__main__':main()
