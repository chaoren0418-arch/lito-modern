"""Publish source/compiler scope only, after observing the original compiler terminal."""
from pathlib import Path
from datetime import datetime,timezone
import argparse, hashlib, json, re, zipfile
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins

T=R/'維護工具';V=R/'驗證紀錄/current'
CHECKPOINTS=['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json','驗證紀錄/current/native-terrain-residence/continuation.json','驗證紀錄/current/native-presentation-source-owner/application-holy-list-20261017-060417-273807/next-validation-action.json','README.md','docs/ARCHITECTURE.zh-TW.md','docs/HISTORY.md']
NOTE='#45地裂術核心與共享資源候選已寫：class7獨立節點／source2遞迴、u32 fixed16／方向fold、終點候選先刪、class6最終首幀提交與十秒terminal；音效128／129／可重疊3080及Hit來源保留。新增兩個尚未接場景module，在原30檔候選完整graph加至364module；兩版library／library-test四metadata退出0，新10＋前155案全未跑。正式per-skill請求／S23 admission、owner／history／draw／typed音效、hurt相對gfx與原scheduler同時限排序仍待整合核對；共享10 SPR／3 WAV只設嚴格預期pin，未載入或播放，不移記TW資格。請求識別仍59／剩121；正常1049輸入、PE、registry及G06 binding／16案640次不變。只做開發，桌面／測試暫緩。'
NEXT='先完成#45正式per-skill分支與S23場景整合：admission／generation／owner、獨立節點History、原scheduler同時限排序、draw batch與typed audio occurrence，再補child hurt原排除／相對gfx與請求／選單入口；不猜原gfx base，不以一般finite替代。測試與桌面須等使用者明確恢復。'

def validate(cohort):
 c=read(cohort);contract=read(c['contract']);require(sha(c['contract'])==c['contract_sha256'],'contract changed');check_pins(c['input_pins']);check_pins(c['normal_product_pins']);require(all(sha(p)==h for p,h in c['external_toolchain_dependency_pins'].items()),'compiler/original dependency changed')
 require(c['status']=='four_metadata_compilers_complete_no_tests' and len(c['compiler_runs'])==4 and [x['exit_code'] for x in c['compiler_runs']]==[0]*4 and c['owned_process'] is None,'metadata cohort incomplete')
 require([(x['profile'],x['kind']) for x in c['compiler_runs']]==[(p,k) for p in ['Debug','Release'] for k in ['library','library-test']],'compiler order differs')
 require(c['tests_executed']==0 and not any(c[x] for x in ['PEs_linked','outputs_executed','scene_integrated','whole_goal_complete']),'execution boundary differs')
 old=read(c['predecessor_cohort']);require(sha(c['predecessor_cohort'])==c['predecessor_cohort_sha256'],'predecessor changed')
 modern=Path(old['generated_mod']).read_text('utf-8-sig')
 for name,row in contract['modules'].items():
  require(sha(row['path'])==row['sha256'],'module changed');modern+='\n#[path = r#"'+Path(row['path']).as_posix()+'"#]\npub(crate) mod '+name+';\n'
 require(Path(c['generated_mod']).read_text('utf-8-sig')==modern and len(re.findall(r'#\[path = r#"',modern))==364,'module graph differs')
 oldroot=Path(old['generated_root']).read_text('utf-8-sig');require(Path(c['generated_root']).read_text('utf-8-sig')==oldroot.replace(Path(old['generated_mod']).as_posix(),Path(c['generated_mod']).as_posix()),'library root differs')
 require(c['original_nested_runtime_pairs']==old['paired_runtime_sources'],'nested pairing differs')
 for pair in c['original_nested_runtime_pairs'].values():
  require(sha(pair['path'])==pair['sha256'] and sha(pair['source'])==pair['source_sha256'],'nested runtime source differs')
 core=Path(contract['modules']['native_eruption45']['path']).read_text('utf-8-sig');require(len(re.findall(r'#\[test\]',core))==contract['written_deferred_cases']==10,'new written-case count differs')
 products={}
 for row in c['compiler_runs']:
  require(sha(row['product'])==row['product_sha256'] and Path(row['product']).stat().st_size==row['product_bytes'],'metadata product differs');check_pins(row['logs'])
  template=next(x['command'] for x in old['compiler_runs'] if x['profile']==row['profile'] and x['kind']=='library');expected=list(template);expected[expected.index(old['generated_root'])]=c['generated_root'];expected[expected.index('-o')+1]=row['product']
  if row['kind']=='library-test':expected.remove('--crate-type=rlib');expected+=['--test']
  require(row['command']==expected and '--emit=metadata' in expected,'compiler command differs')
  products[row['profile']+'-'+row['kind']]=dict(path=row['product'],sha256=row['product_sha256'],bytes=row['product_bytes'])
 return c,contract,products

def publish(cohort,chunk):
 c,contract,products=validate(cohort);w=read(R/'工作佇列.json');require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'],'phase changed')
 prior=w['current_E149_zombie41_development'];require(prior['qualification']==contract['predecessor_qualification'] and prior['qualification_sha256']==contract['predecessor_qualification_sha256'],'current candidate predecessor differs')
 require('current_E149_eruption45_core_development' not in w,'stage already published')
 previous_checkpoint=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];require(len(previous_checkpoint)==8 and all(sha(R/n)==h for n,h in previous_checkpoint.items()),'checkpoint changed')
 out=owned(V/'native-E149-eruption45-core-development'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
 before={n:sha(R/n) for n in CHECKPOINTS}
 with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
  for n in CHECKPOINTS:z.write(R/n,n)
 pins=dict(c['input_pins']);pins[str(cohort)]=sha(cohort)
 for p in [Path(__file__).resolve(),T/'readback-native-E149-eruption45-core.py']:pins[str(p)]=sha(p)
 predecessor=read(prior['qualification']);source=dict(schema='E149-eruption45-core-development-v1',status='unselected_eruption45_core_and_shared_assets_metadata_only',
  source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,contract=c['contract'],contract_sha256=c['contract_sha256'],source_pins=pins,
  compile_run=str(cohort.parent),compile_cohort_sha256=sha(cohort),observed_original_tool_chunk=chunk,observed_actual_exit_code=0,compiler_exit_codes=[0]*4,metadata_products=products,
  predecessor_qualification=prior['qualification'],predecessor_qualification_sha256=prior['qualification_sha256'],retained_effective_overlay_files=30,new_supplementary_modules=2,resolved_module_path_attributes=364,original_nested_paths_preserved=6,
  new_tests_written=10,retained_skill_tests_written=155,current_source_coverage=contract['current_source_coverage'],
  current_fixture_binding_unchanged=predecessor['current_fixture_binding_unchanged'],current_formal_suite_unchanged=predecessor['current_formal_suite_unchanged'],canonical_registry_unchanged=w['current_isolated_development']['candidates'],
  modules_integrated_into_scene=False,request_registered=False,resource_digests_are_unrun_expectations=True,native_scheduler_tie_order_qualified=False,
  tests_executed=0,new_sessions=0,desktop_inputs=0,WAV_playbacks=0,PEs_linked=False,outputs_executed=False,production_applied=False,canonical_registry_selected=False,GUI_verified=False,old_source_success_transferred=False,whole_goal_complete=False,
  pending=contract['pending'],normal_product_pins=c['normal_product_pins'],recorded_utc=datetime.now(timezone.utc).isoformat())
 raw_files={str(Path(c['contract'])):sha(c['contract']),str(cohort):sha(cohort),str(Path(contract['inspection'])):sha(contract['inspection'])}
 raw_files.update(read(contract['inspection'])['files'])
 for module in contract['modules'].values():raw_files[module['path']]=module['sha256']
 for row in c['compiler_runs']:raw_files.update(row['logs'])
 for p in [Path(__file__).resolve(),T/'readback-native-E149-eruption45-core.py',T/'compile-native-E149-eruption45-core.py',T/'prepare-native-E149-eruption45-core.py',T/'native-E149-eruption45-core.md']:raw_files[str(p)]=sha(p)
 failures=[V/'native-E149-eruption45-core-inspection/20261024-085531-331408/failed-inspector.py',V/'native-E149-eruption45-core-inspection/20261024-085531-331408/original-failure.txt',V/'native-E149-eruption45-core-tooling-notes/20261024-090400/failed-prepare.py',V/'native-E149-eruption45-core-tooling-notes/20261024-090400/original-failure.txt']
 for p in failures:raw_files[str(p)]=sha(p)
 with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
  for p,h in raw_files.items():require(sha(p)==h,'archive input changed');z.write(p,Path(p).relative_to(R).as_posix())
 with zipfile.ZipFile(out/'raw.zip') as z:
  require(len(z.namelist())==len(raw_files) and all(hashlib.sha256(z.read(Path(p).relative_to(R).as_posix())).hexdigest().upper()==h for p,h in raw_files.items()),'raw archive differs')
 source.update(raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'),raw_archive_files=raw_files,original_tooling_failures_preserved=[str(p) for p in failures])
 qp=out/'qualified-development.json';qp.write_bytes(encoded(source))
 ref=dict(qualification=str(qp),qualification_sha256=sha(qp),status=source['status'],compile_run=source['compile_run'],compiler_exit_codes=[0]*4,new_supplementary_modules=2,scene_integrated=False,new_tests_written=10,tests_executed=0,request_recognition_count=59,missing_request_count=121,whole_goal_complete=False)
 ep=out/'evidence.json';ep.write_bytes(encoded(dict(qualification=ref,previous_goal_turn_evidence=w['last_goal_turn_evidence'],actual_progress='two source modules plus metadata, not tests or a complete skill',whole_goal_complete=False)))
 for n in CHECKPOINTS:
  require(sha(R/n)==before[n],'concurrent checkpoint writer')
  p=R/n
  if n.endswith('.json'):
   j=read(p);j['current_E149_eruption45_core_development']=ref;j['last_goal_turn_evidence']=str(ep);j['last_goal_turn_classification']='progress';j['next_development_action']=NEXT
   if 'whole_goal_complete' in j:j['whole_goal_complete']=False
   if 'goal_complete' in j:j['goal_complete']=False
   if n in ['工作佇列.json','監製進度.json']:
    j['next_task']=NEXT;j['current_focus']=dict(item='eruption45_core_and_shared_assets_unintegrated',work=NOTE,next=NEXT,evidence=str(qp));j['progress_updated_utc']=source['recorded_utc']
   p.write_bytes(encoded(j))
  else:p.write_text('2026-10-24：'+NOTE+' '+NEXT+' [來源與編譯](<'+qp.as_posix()+'>)。\n\n'+p.read_text('utf-8-sig'),encoding='utf-8',newline='\n')
 after={n:sha(R/n) for n in CHECKPOINTS};require(all(sha(R/n)==h for n,h in after.items()),'checkpoint readback differs')
 with zipfile.ZipFile(out/'before.zip') as z:require(all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items()),'before archive differs')
 (out/'readback.json').write_bytes(encoded(dict(before_files=before,after_files=after,before_archive_sha256=sha(out/'before.zip'),qualification=str(qp),qualification_sha256=sha(qp),tests_executed=0,whole_goal_complete=False)))
 print(json.dumps(dict(qualification=str(qp),sha256=sha(qp),checkpoint=str(out/'readback.json'),compiler_exit_codes=[0]*4,tests_executed=0,scene_integrated=False),ensure_ascii=False))

if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('cohort',type=Path);p.add_argument('--observed-original-tool-chunk',required=True);a=p.parse_args();publish(a.cohort,a.observed_original_tool_chunk)
