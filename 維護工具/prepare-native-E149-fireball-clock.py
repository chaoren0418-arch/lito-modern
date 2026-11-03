"""Stage class5 fireball callback/asset APIs; no request/S39 integration or execution."""
from pathlib import Path
from datetime import datetime,timezone
import json,re
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current';NAME='LiTo381-Modern/src/modern_lito/light_arrow_effect.rs'
def prepare():
 w=read(R/'工作佇列.json');require(not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'] and w['current_user_requested_goal']['test_execution_deferred_by_user'],'development-only idle scope required');require(sha(w['current_formal_reference_identity']['formal'])==FORMAL,'formal changed')
 ref=w['current_E149_area2230_development'];require(sha(ref['qualification'])==ref['qualification_sha256'],'previous source changed');q=read(ref['qualification']);check_pins(q['source_pins']);require(sha(q['candidate'])==q['candidate_sha256'],'candidate changed');base=read(q['candidate']);require(sha(base['contract'])==base['contract_sha256'],'previous contract changed');prior=read(base['contract'])
 files=dict(base['effective_overlay_files']);paths=dict(base['effective_overlay_paths']);require(len(files)==26,'source projection differs');check_pins({paths[n]:h for n,h in files.items()});source=Path(paths[NAME]);text=source.read_text('utf-8-sig')
 native_path=V/'native-E149-fireball25-contract/20261024-051952-268325/contract.json';native=read(native_path);require(sha(native['PE'])==native['PE_sha256'] and not native['PE_executed'],'native source changed')
 core=T/'templates/native_fireball_clock_core.rs';assets=T/'templates/native_fireball_clock_assets.rs';qa=T/'templates/native_fireball_clock_tests.rs'
 changes=[
  ('        let dx = i64::from(target.x) - i64::from(from.x);', '''        Self::build_native_path(from,target)
    }
    /// The S39 fallback is one heading step beyond the wire u16 origin. Only
    /// this class5 entry admits that edge cell; old public profiles stay strict.
    pub(crate) fn new_area_fireball(from:WorldCell,target:WorldCell)->Result<Self> {
        if [from.x,from.y].iter().any(|v|!(0..=65535).contains(v))
            ||[target.x,target.y].iter().any(|v|!(-1..=65536).contains(v)) {
            return Err(LightArrowError::InvalidInput("area fireball world coordinates"));
        }
        Self::build_native_path(from,target)
    }
    fn build_native_path(from:WorldCell,target:WorldCell)->Result<Self> {
        let dx = i64::from(target.x) - i64::from(from.x);'''),
  ('        let (source_frame, finished) = bound.sequence.sample(elapsed)?;\n        let asset = bound', '''        let (source_frame, finished) = bound.sequence.sample(elapsed)?;
        self.frame_source(bound,heading,source_frame,finished)
    }
    fn frame_source<'a>(&'a self,bound:&BoundSequence,heading:u8,source_frame:usize,finished:bool)->Result<VisualSample<'a>> {
        if heading>7 {return Err(LightArrowError::InvalidInput("heading"));}
        let asset = bound'''),
  ('    pub fn flight_frame(&self,path:&LightArrowPath,elapsed:Duration)',assets.read_text('utf-8-sig')+'    pub fn flight_frame(&self,path:&LightArrowPath,elapsed:Duration)'),
  ('// Literal threshold branches from4B9210 after converting native dx/2 to world.',core.read_text('utf-8-sig')+'\n// Literal threshold branches from4B9210 after converting native dx/2 to world.')]
 for before,after in changes:require(text.count(before)==1,'source anchor differs: '+before);text=text.replace(before,after)
 tests=qa.read_text('utf-8-sig');require(len(re.findall(r'#\[test\]',tests))==7 and tests.count('#[ignore=')==1,'QA scope differs');text+=tests
 out=owned(T/'candidates'/('native-E149-fireball-clock-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')));out.mkdir(parents=True)
 (out/'base-overlay.json').write_bytes(encoded(dict(candidate_files=files,candidate_paths=paths,previous_skill_qualification=ref,current_fixture_binding=w['current_E149_fixture_binding'])))
 target=out/NAME;target.parent.mkdir(parents=True);target.write_text(text,encoding='utf-8',newline='\n');files[NAME]=sha(target);paths[NAME]=str(target)
 contract=dict(schema='E149-fireball-native-clock-source-v1',formal_target_sha256=FORMAL,source_changes={NAME:changes},QA_template=str(qa),QA_template_sha256=sha(qa),native_snapshot_contract=str(native_path),native_snapshot_contract_sha256=sha(native_path),original_pins={**prior['original_pins'],str(native_path):sha(native_path)},movement_ms=60,flight_graphics=171,secondary_graphics=218,secondary_before_flight_cues=True,secondary_terminal_zero_dwell=True,request_S39_world_integration=False,tests_executed=0)
 (out/'contract.json').write_bytes(encoded(contract));record=dict(base_source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,candidate_files={NAME:sha(target)},base_files={NAME:sha(source)},base_overlay=str(out/'base-overlay.json'),base_overlay_sha256=sha(out/'base-overlay.json'),previous_skill_qualification=ref,current_fixture_binding=w['current_E149_fixture_binding'],effective_overlay_files=files,effective_overlay_paths=paths,original_metadata_cohort=str(Path(q['compile_run'])/'cohort.json'),changed_runtime_files=[NAME],contract=str(out/'contract.json'),contract_sha256=sha(out/'contract.json'),new_tests_written=7,new_resource_tests_require_original_root=1,retained_skill_tests_written=74,tests_executed=0,production_applied=False,canonical_registry_selected=False,whole_goal_complete=False)
 (out/'candidate.json').write_bytes(encoded(record));print(json.dumps(dict(candidate=str(out/'candidate.json'),changed_files=1,effective_files=26,tests_executed=0),ensure_ascii=False))
if __name__=='__main__':prepare()
