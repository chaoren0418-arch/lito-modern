"""Prepare baseline/candidate library graphs and exact cases; no compiler/test."""
from pathlib import Path
from datetime import datetime,timezone
import re,json
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,check_pins,owned
T=R/'維護工具';V=R/'驗證紀錄/current'
w=read(R/'工作佇列.json');assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
assert w['current_user_requested_goal']['test_execution_deferred_by_user']
assert sha(w['current_formal_reference_identity']['formal'])==FORMAL
ref=w['current_native_cancel_recovery_candidate'];cp=Path(ref['candidate']);c=read(cp);assert sha(cp)==ref['candidate_sha256']
assert c['base_source_manifest_sha256']==SOURCE and c['new_tests_written']==14 and c['baseline_additional_cases']==6
check_pins({str(cp.parent/n):h for n,h in c['candidate_files'].items()})
qa=w['current_development_typecheck_qualification'];assert sha(qa['qualification'])==qa['qualification_sha256']
q=read(qa['qualification']);original=Path(q['run']);cohort=read(original/'cohort.json')
inputs=read(original/'base-compiled-inputs.json');assert len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items())
check_pins(q['normal_product_pins'])
current_registry=w['current_isolated_development']['candidates'];assert set(cohort['candidates'])==set(current_registry)
assert all(all(old[k]==current_registry[name][k] for k in ['path','sha256','files','tests_written']) for name,old in cohort['candidates'].items())
out=T/'candidates'/('native-E149-component-matrix-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
baseline=out/'baseline';baseline.mkdir();rewrites=[]
names={'pure':'LiTo381-Modern/src/modern_lito/native_rental_return_tests.rs',
 'wire':'LiTo381-Modern/src/modern_lito/native_teleport_controller_tests.rs'}
baseline_sources={'pure':Path(c['baseline_regression_tests']),'wire':Path(c['baseline_wire_regression_tests'])}
assert sha(baseline_sources['pure'])==c['baseline_regression_tests_sha256']
assert sha(baseline_sources['wire'])==c['baseline_wire_regression_tests_sha256']

def tests(path):
 text=path.read_text('utf-8-sig')
 pattern=r'(?m)^#\[(?:test|cfg_attr\(test, test\))\]\s*\n(?:pub\s+)?fn\s+(\w+)\s*\('
 rows=re.findall(pattern,text)
 assert len(rows)==len(set(rows))==text.count('#[test]')+text.count('#[cfg_attr(test, test)]')
 return rows

def rewrite(origin,semantic,target,redirects):
 text=origin.read_text('utf-8-sig');changes=[]
 def path_attr(m):
  original_path=owned((semantic.parent/m[1]).resolve(strict=True))
  destination=redirects.get(str(original_path),original_path)
  after='#[path = r#"'+str(destination)+'"#]';changes.append(dict(before=m[0],after=after));return after
 text=re.sub(r'#\[path\s*=\s*"([^"\n]+)"\]',path_attr,text)
 def include(m):
  original_path=owned((semantic.parent/m[2]).resolve(strict=True))
  after=m[1]+'!(r#"'+str(original_path)+'"#)';changes.append(dict(before=m[0],after=after));return after
 text=re.sub(r'(include_bytes|include_str|include)!\("([^"\n]+)"\)',include,text)
 target.write_text(text,encoding='utf-8',newline='\n')
 rewrites.append(dict(origin=str(origin),semantic_origin=str(semantic),generated=str(target),replacements=changes))

prefix={'pure':'modern_lito::native_login::rental_return_tests::','wire':'modern_lito::native_login::teleport_wire_tests::'}
cases={'baseline':[],'candidate':[]};baseline_qa={}
red_conditions={
 'repeated_cancel_keeps_only_the_unused_recovery_correlation':['unused correlation survived cancel'],
 'failure_then_cancel_does_not_erase_or_restore_retired_authority':['assertion failed: c.teleport_recovery.is_some()'],
 'expired_recovery_send_failure_never_publishes_the_unconfirmed_directory':['expired recovery source refresh'],
 'expired_recovery_confirms_source_before_publishing_a_new_directory':['left: Failed','right: LoadingCharacters'],
 'cancelling_the_expired_recovery_refresh_cannot_start_another_transport':['left: Failed','right: LoadingCharacters'],
 'recovery_character_absent_from_first_directory_confirms_source_before_manual_choice':['controller Failed:','teleport character absent from destination directory'],
}
for kind,name in names.items():
 normal=tests(R/name);candidate=tests(cp.parent/name);base=tests(baseline_sources[kind])
 assert set(normal)<=set(base)<=set(candidate)
 base_extra=set(base)-set(normal);assert base_extra<=set(red_conditions)
 for variant,listing in [('baseline',base),('candidate',candidate)]:
  for test in listing:
   red=variant=='baseline' and test in base_extra
   cases[variant].append(dict(test_name=prefix[kind]+test,module=kind,
    expected='specific_assertion_failure' if red else 'pass',
    panic_contains=red_conditions[test] if red else [],existing_control=test in normal,
    provenance='synthetic_controller_state_or_encrypted_loopback',formal_runtime_qualified=False))
 path=baseline/(Path(name).name);rewrite(baseline_sources[kind],R/name,path,{})
 baseline_qa[str(R/name)]=path
assert len(cases['baseline'])==23 and sum(x['expected']=='specific_assertion_failure' for x in cases['baseline'])==6
assert len(cases['candidate'])==28 and sum(x['existing_control'] for x in cases['candidate'])==17
assert sum(not x['existing_control'] for x in cases['candidate'])==11
assert {x['test_name'] for x in cases['baseline']}<={x['test_name'] for x in cases['candidate']}
controller=P/'src/modern_lito/native_login.rs';baseline_controller=baseline/'native_login.rs'
rewrite(controller,controller,baseline_controller,baseline_qa)
old_controller=original/'generated/LiTo381-Modern/src/modern_lito/native_login.rs'
modern=(original/'generated/modern_lito.rs').read_text('utf-8')
before='#[path = r#"'+str(old_controller)+'"#]';after='#[path = r#"'+str(baseline_controller)+'"#]'
assert modern.count(before)==1
(baseline/'modern_lito.rs').write_text(modern.replace(before,after),encoding='utf-8',newline='\n')
lib=(original/'generated/lib.rs').read_text('utf-8');before=str(original/'generated/modern_lito.rs');assert lib.count(before)==1
(baseline/'lib.rs').write_text(lib.replace(before,str(baseline/'modern_lito.rs')),encoding='utf-8',newline='\n')
module_map=read(original/'module-map.json');assert len(module_map)==362
baseline_map=[dict(row,path=str(baseline_controller) if Path(row['path'])==old_controller else row['path']) for row in module_map]
(baseline/'module-map.json').write_bytes(encoded(baseline_map))
candidate_root=original/'generated/lib.rs';baseline_root=baseline/'lib.rs'
commands={};external={};link_search={}
for profile in ['Debug','Release']:
 rows=[x for x in cohort['process_history'] if Path(x['output'])==original/profile/'library-test-code-compile'];assert len(rows)==1
 command=rows[0]['command'];assert rows[0]['exit_code']==0 and '--test' in command and '--emit=metadata' in command
 assert command.count(str(candidate_root))==1
 commands[profile]=command
 linked=read(w['current_E149_recovery_linked_image_qualifications'][profile]['qualification']);build=read(linked['compile_cohort'])
 assert sha(linked['compile_cohort'])==linked['compile_cohort_sha256']
 external.update(build['external_toolchain_dependency_pins']);link_search[profile]=build['native_link_search_paths']
pins={n:h for n,h in cohort['input_pins'].items() if Path(n).is_relative_to(R)}
pins.update({str(p):sha(p) for p in [cp,Path(qa['qualification']),original/'cohort.json',original/'module-map.json',
 original/'base-compiled-inputs.json',*baseline_sources.values(),controller,Path(__file__),T/'compile-native-E149-component-matrix.py']})
pins.update({str(p):sha(p) for p in baseline.iterdir() if p.is_file()})
assert all(sha(p)==h for p,h in external.items())
record=dict(schema='E149-recovery-component-matrix-plan-v1',status='development_plan_only',
 source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,cancel_candidate=str(cp),cancel_candidate_sha256=sha(cp),
 original_metadata_qualification=qa['qualification'],original_metadata_qualification_sha256=qa['qualification_sha256'],
 original_compiler_cohort=str(original/'cohort.json'),original_compiler_cohort_sha256=sha(original/'cohort.json'),
 roots=dict(baseline=str(baseline_root),candidate=str(candidate_root)),module_maps=dict(baseline=str(baseline/'module-map.json'),candidate=str(original/'module-map.json')),
 base_compiled_inputs=str(original/'base-compiled-inputs.json'),normal_product_pins=q['normal_product_pins'],
 cases=cases,source_rewrites=rewrites,root_replacement_only=dict(original_native_login=str(old_controller),baseline_native_login=str(baseline_controller)),
 input_pins=pins,external_toolchain_dependency_pins=external,compiler_templates=commands,native_link_search_paths=link_search,
 metadata_mode='baseline two profiles only; candidate existing full-graph metadata retained',
 link_mode='two profiles x baseline/candidate library-test PE; no test execution',
 expected_per_profile=dict(baseline_existing_controls=17,baseline_specific_assertion_failures=6,candidate_existing_controls=17,candidate_new_components=11),
 new_formal_cases_separate=3,tests_executed=0,compilers_invoked=0,PEs_executed=False,GUI_inputs=0,
 baseline_failures_observed=False,candidate_tests_passed=False,whole_goal_complete=False,
 limitations=['Source-derived names are not actual libtest --list inventory; obtain inventory after tests resume',
  'Baseline changes only native_login Runtime back to normal 3E, with matched baseline QA; other independent candidate modules stay identical',
  'Expected-red requires the named case, ordinary panic exit101, expected assertion evidence and passing original controls; build errors/crashes do not count',
  'Eleven new component cases and three new formal cases make fourteen; the formal cases belong to the separate 16-case suite'],
 recorded_utc=datetime.now(timezone.utc).isoformat())
(out/'plan.json').write_bytes(encoded(record));(out/'preparer.py').write_bytes(Path(__file__).read_bytes())
assert all(sha(R/n)==h for n,h in inputs.items());check_pins(q['normal_product_pins'])
print(json.dumps(dict(plan=str(out/'plan.json'),baseline_cases_per_profile=23,candidate_cases_per_profile=28,
 baseline_expected_red=6,candidate_new_components=11,new_formal_cases_separate=3,tests_executed=0,compilers_invoked=0),ensure_ascii=False))
