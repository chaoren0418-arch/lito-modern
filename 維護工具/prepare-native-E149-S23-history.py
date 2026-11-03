"""Preserve S23 projectile admission/cursors across Model rebuilds; no execution."""
from pathlib import Path
from datetime import datetime,timezone
import json,re
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current';PREFIX='LiTo381-Modern/src/modern_lito/';SPELL=PREFIX+'native_spell_projectiles.rs';SCENE=PREFIX+'native_world_scene.rs'
def prepare():
 w=read(R/'工作佇列.json');require(not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'] and w['current_user_requested_goal']['test_execution_deferred_by_user'],'development-only idle scope required');require(sha(w['current_formal_reference_identity']['formal'])==FORMAL,'formal changed')
 ref=w['current_E149_fireball25_development'];require(sha(ref['qualification'])==ref['qualification_sha256'],'previous source changed');q=read(ref['qualification']);check_pins(q['source_pins']);require(sha(q['candidate'])==q['candidate_sha256'],'candidate changed');base=read(q['candidate']);require(sha(base['contract'])==base['contract_sha256'],'previous contract changed');prior=read(base['contract']);files=dict(base['effective_overlay_files']);paths=dict(base['effective_overlay_paths']);require(len(files)==27 and SPELL not in files,'projection differs');check_pins({paths[n]:h for n,h in files.items()})
 analysis=V/'native-E149-loader-base-reconstruction/20261024-061320-238679/contract.json';a=read(analysis);require(a['payload_bytes']==1189423 and a['reconstructed_checksum']==a['original_checksum']==18638 and a['marker_F9_count']==5894 and a['raw_instruction_split_count']==1 and not a['modern_graphics_binding_verified'] and not a['native_instructions_executed'],'static reconstruction scope differs');require(sha(a['analysis_source'])==a['analysis_source_sha256'],'analysis source changed')
 qa=T/'templates/native_S23_history_tests.rs';text=qa.read_text('utf-8-sig');require(len(re.findall(r'#\[test\]',text))==3 and text.count('#[ignore=')==1,'QA scope differs')
 methods='''    pub(crate) fn history(&self)->History {History{generation:self.generation,seen:self.seen.clone(),flights:self.flights.clone(),launched:self.launched,arrived:self.arrived,omitted:self.omitted}}
    pub(crate) fn restore_history(&mut self,h:History,same_source:bool) {
        self.generation=h.generation;self.seen=h.seen;self.flights=if same_source{h.flights}else{vec![]};
        self.launched=h.launched;self.arrived=h.arrived;self.omitted=h.omitted;
    }
'''
 changes={SPELL:[('struct Flight {','#[derive(Clone)]\nstruct Flight {'),('#[derive(Default)]\npub struct Projectiles {',(T/'templates/native_S23_history.rs').read_text('utf-8-sig')+'#[derive(Default)]\npub struct Projectiles {'),('impl Projectiles {','impl Projectiles {\n'+methods),('c.projectile.flight_type==0 && c.projectile.graphics==graphics)) {','c.projectile.flight_type==0 && c.projectile.graphics==graphics)||self.flights.iter().any(|f|f.cue.projectile.graphics==graphics)) {')],SCENE:[
 ('skills:[super::native_skill_effects::History;4],many:super::native_attack_many::History,','skills:[super::native_skill_effects::History;4],many:super::native_attack_many::History,spell_projectiles:super::native_spell_projectiles::History,'),
 ('self.cone_cold.merge(&other.cone_cold);self.animation=','self.cone_cold.merge(&other.cone_cold);self.spell_projectiles.merge_admission(&other.spell_projectiles);self.animation='),
 ('many:self.many_effects.history(),disintegrate:','many:self.many_effects.history(),spell_projectiles:self.spell_projectiles.history(),disintegrate:'),
 ('        self.animator.restore(history.animation);','        self.animator.restore(history.animation);\n        self.spell_projectiles.restore_history(history.spell_projectiles,history.restore_other_effects);')]}
 out=owned(T/'candidates'/('native-E149-S23-history-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')));out.mkdir(parents=True);(out/'base-overlay.json').write_bytes(encoded(dict(candidate_files=files,candidate_paths=paths,previous_skill_qualification=ref,current_fixture_binding=w['current_E149_fixture_binding'])))
 changed={};before={}
 for name,replacements in changes.items():
  path=Path(paths.get(name,str(R/name)));s=path.read_text('utf-8-sig');before[name]=sha(path)
  for old,new in replacements:require(s.count(old)==1,'source anchor differs: '+old);s=s.replace(old,new)
  if name==SPELL:s+=text
  target=out/name;target.parent.mkdir(parents=True,exist_ok=True);target.write_text(s,encoding='utf-8',newline='\n');changed[name]=sha(target);files[name]=sha(target);paths[name]=str(target)
 contract=dict(schema='E149-S23-history-source-v1',formal_target_sha256=FORMAL,source_changes=changes,QA_template=str(qa),QA_template_sha256=sha(qa),native_snapshot_contract=prior['native_snapshot_contract'],native_snapshot_contract_sha256=prior['native_snapshot_contract_sha256'],original_pins={**prior['original_pins'],str(analysis):sha(analysis)},base_reconstruction=str(analysis),base_reconstruction_sha256=sha(analysis),same_source_restores_flights=True,retired_source_preserves_admission_only=True,merged_admission_requires_same_generation=True,active_flights_prepare_assets_after_ring_eviction=True,existing_projectile_clock_unchanged=True,tests_executed=0)
 (out/'contract.json').write_bytes(encoded(contract));record=dict(base_source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,candidate_files=changed,base_files=before,base_overlay=str(out/'base-overlay.json'),base_overlay_sha256=sha(out/'base-overlay.json'),previous_skill_qualification=ref,current_fixture_binding=w['current_E149_fixture_binding'],effective_overlay_files=files,effective_overlay_paths=paths,original_metadata_cohort=str(Path(q['compile_run'])/'cohort.json'),changed_runtime_files=list(changes),contract=str(out/'contract.json'),contract_sha256=sha(out/'contract.json'),new_tests_written=3,new_resource_tests_require_original_root=1,retained_skill_tests_written=94,tests_executed=0,production_applied=False,canonical_registry_selected=False,whole_goal_complete=False)
 (out/'candidate.json').write_bytes(encoded(record));print(json.dumps(dict(candidate=str(out/'candidate.json'),changed_files=2,effective_files=28,tests_executed=0),ensure_ascii=False))
if __name__=='__main__':prepare()
