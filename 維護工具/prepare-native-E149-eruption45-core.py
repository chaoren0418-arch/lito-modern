"""Prepare two unintegrated runtime modules; preserve the current 30-file candidate."""
from pathlib import Path
from datetime import datetime, timezone
import json
from native_E149_recovery_contract import R, SOURCE, FORMAL, read, sha, encoded, require, check_pins

T=R/'維護工具';V=R/'驗證紀錄/current'
C=T/'candidates/native-E149-eruption45-core-20261024-085557'
INSPECTION=V/'native-E149-eruption45-core-inspection/20261024-085557-421299/inspection.json'

def prepare():
 w=read(R/'工作佇列.json');require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'],'development-only idle phase expected')
 previous=w['current_E149_zombie41_development'];require(sha(previous['qualification'])==previous['qualification_sha256'],'predecessor differs');q=read(previous['qualification']);base=read(q['candidate'])
 require(sha(INSPECTION)=='BDBA7C06B6FAB44B836D973CB4D05DC1AF5C889DA0A02B4BFE343AB0F853EEFE','primary inspection differs');proof=read(INSPECTION);check_pins({p:h for p,h in proof['files'].items()});require(all(sha(p)==h for p,h in proof['original_pins'].items()),'original source differs')
 old=T/'candidates/native-eruption-assets/eruption_assets.rs';s=old.read_text('utf-8-sig')
 def replace(a,b):
  nonlocal s
  require(s.count(a)==1,'asset predecessor anchor differs '+a[:60]);s=s.replace(a,b)
 replace('//! Candidate current-TW eruption assets; no world, renderer, request or audio device.','//! Strict shared-Text #45 assets; loading and runtime use remain unverified.\n//! No TW context, world, renderer, request or audio device.')
 replace('use lito_modern::modern_lito::{','use super::{')
 replace('tw_action_catalog::SharedTwActionCatalog,','shared_action_catalog::SharedActionCatalog,')
 replace('    catalog: SharedTwActionCatalog,\n    sprites:', '    catalog_sha256: [u8; 32],\n    sprites:')
 start=s.index('    /// Explicit current TW asset-data namespace.');end=s.index('        qualify(catalog.catalog())?;',start)
 s=s[:start]+'''    /// Borrow the Model-owned shared Text context; never read launcher/TW.
    /// SPR/WAV digests below are strict expected inputs, not an executed claim.
    pub(crate) fn load_with_shared(root: &Path, catalog: &SharedActionCatalog,
        cancel: &AtomicBool) -> Result<Self> {
        active(cancel)?;
        ensure!(root.canonicalize()? == catalog.root(), "eruption catalog root differs");
        ensure!(catalog.sha256() == SHARED_TEXT, "unreviewed eruption shared Text");
'''+s[end:]
 replace('            catalog,','            catalog_sha256: catalog.sha256(),')
 replace('    pub fn catalog(&self) -> &SharedTwActionCatalog {\n        &self.catalog\n    }','    pub(crate) fn catalog_sha256(&self) -> [u8; 32] { self.catalog_sha256 }')
 start=s.index('const CURRENT_PACK:');end=s.index('// Shadow SPRs inherit',start)
 digest=bytes.fromhex(proof['original_pins'][str(Path('D:/lineage開發/3-lito開發/lito脫殼資源/resources/decoded/Text/row15937-list.spr'))])
 s=s[:start]+'const SHARED_TEXT: [u8; 32] = ['+', '.join(str(x) for x in digest)+'];\n\n'+s[end:]
 replace('    ensure!(catalog.sprites.len() == 4, "eruption exact selection");','    // A full shared catalog contains other IDs; qualify these four without aliases.')
 require('SharedTw' not in s and 'load_current_tw' not in s and 'CURRENT_PACK' not in s,'TW loader retained')
 asset=C/'native_eruption45_assets.rs'
 if asset.exists():require(asset.read_text('utf-8-sig')==s,'prepared asset source changed; preserve it')
 else:asset.write_text(s,encoding='utf-8',newline='\n')
 core=C/'native_eruption45.rs';require(core.exists(),'core source missing')
 modules={p.stem:dict(path=str(p),sha256=sha(p)) for p in [core,asset]}
 contract=dict(schema='E149-eruption45-core-source-v1',base_source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,
  predecessor_qualification=previous['qualification'],predecessor_qualification_sha256=previous['qualification_sha256'],
  predecessor_candidate=q['candidate'],predecessor_candidate_sha256=sha(q['candidate']),predecessor_metadata_cohort=str(Path(q['compile_run'])/'cohort.json'),predecessor_metadata_cohort_sha256=sha(Path(q['compile_run'])/'cohort.json'),
  inspection=str(INSPECTION),inspection_sha256=sha(INSPECTION),original_pins=proof['original_pins'],
  asset_predecessor=dict(path=str(old),sha256=sha(old),qualification_transferred=False),modules=modules,
  retained_effective_overlay_count=len(base['effective_overlay_files']),new_supplementary_runtime_modules=2,
  modules_integrated_into_scene=False,request_registered=False,current_source_coverage=q['current_source_coverage'],
  written_deferred_cases=10,previous_written_deferred_cases=155,tests_executed=0,PEs_linked=False,outputs_executed=False,
  pending=['Integrate class7/class6 graph into current S23 admission, history, owner/generation retirement, draw batching and typed audio.',
   'Bind #45 exact formal per-skill route and register its request/selection only with a complete renderer.',
   'Bind child hurt exclusions, original graphics initialization and relative LT family; no invented graphics base.',
   'After explicit test resumption, run ten new regressions, shared Text/10 SPR/3 WAV qualification and end-to-end casts.'],
  whole_goal_complete=False)
 cp=C/'contract.json'
 if cp.exists():
  history=C/'contract-history';history.mkdir(exist_ok=True);backup=history/(sha(cp)+'.json')
  if not backup.exists():backup.write_bytes(cp.read_bytes())
 cp.write_bytes(encoded(contract));print(json.dumps(dict(contract=str(cp),sha256=sha(cp),new_modules=2,new_deferred_cases=10,scene_integrated=False),ensure_ascii=False))

if __name__=='__main__':prepare()
