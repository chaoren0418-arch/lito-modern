"""Future independent component readback. Never starts a test executable."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,re,subprocess,zipfile
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins,write_new
from native_E149_libtest_readback import inventory,selected_result

def assertion_site(p,case):
 directory=Path(p['roots']['baseline']).parent
 path=directory/('native_rental_return_tests.rs' if case['module']=='pure' else 'native_teleport_controller_tests.rs')
 source=path.read_text('utf-8');name=case['test_name'].rsplit('::',1)[1]
 if name=='recovery_character_absent_from_first_directory_confirms_source_before_manual_choice':
  marker='assert!(Instant::now()<until,"controller'
 else:
  start=re.search(r'(?m)^(?:pub\s+)?fn\s+'+re.escape(name)+r'\s*\(',source)
  require(start is not None,'counterexample source function absent')
  rest=source[start.start():];following=re.search(r'(?m)^(?:pub\s+)?fn\s+\w+\s*\(',rest[1:])
  body=rest[:following.start()+1] if following else rest
  markers={
   'repeated_cancel_keeps_only_the_unused_recovery_correlation':'expect("unused correlation survived cancel")',
   'failure_then_cancel_does_not_erase_or_restore_retired_authority':'assert!(c.teleport_recovery.is_some())',
   'expired_recovery_send_failure_never_publishes_the_unconfirmed_directory':'assert!(c.state.error.as_deref().unwrap().contains("expired recovery source refresh"))',
   'expired_recovery_confirms_source_before_publishing_a_new_directory':'assert_eq!(c.state.phase,LoginPhase::LoadingCharacters)',
   'cancelling_the_expired_recovery_refresh_cannot_start_another_transport':'assert_eq!(c.state.phase,LoginPhase::LoadingCharacters)',
  }
  marker=markers.get(name);require(marker is not None and body.count(marker)==1,'counterexample assertion site differs')
  at=start.start()+body.index(marker)
  return dict(file=str(path),line=source.count('\n',0,at)+1)
 require(source.count(marker)==1,'encrypted peer timeout assertion site differs')
 return dict(file=str(path),line=source.count('\n',0,source.index(marker))+1)

def terminal_observation(rows):
 require(rows and all(row['wait_confirmed'] is True and type(row['exit_code']) is int for row in rows),'original process exit receipts missing')
 require(len({(x['pid'],x['creation_filetime_ticks']) for x in rows})==len(rows),'duplicate original process identities')
 ids=','.join(str(int(row['pid'])) for row in rows)
 query="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);$ErrorActionPreference='Stop';@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
 actual=subprocess.run(['pwsh','-NoProfile','-NonInteractive','-Command',query],capture_output=True,timeout=15,creationflags=subprocess.CREATE_NO_WINDOW)
 require(actual.returncode==0,'independent terminal OS observation failed')
 live=json.loads(actual.stdout.decode('utf-8-sig')) if actual.stdout.strip() else [];live=[live] if isinstance(live,dict) else live
 epoch=datetime(1970,1,1,tzinfo=timezone.utc)
 for row in rows:
  millis=(row['creation_filetime_ticks']-116444736000000000)//10000
  require(row['start_epoch_millis']==millis,'process birth precision differs')
  for item in live:
   if item['ProcessId']==row['pid']:
    delta=datetime.fromisoformat(item['created'].replace('Z','+00:00'))-epoch
    current=(delta.days*86400+delta.seconds)*1000+delta.microseconds//1000
    require(current!=millis,'same original case/inventory process is still live')
 return actual.stdout.decode('utf-8-sig')

def verify(execution):
 execution=owned(execution,P/'target/current/verification/native-E149-component-matrix-execution');e=read(execution)
 require(e['schema']=='E149-component-matrix-execution-v1' and e['status']=='original_matrix_processes_terminal_results_unreviewed','not a terminal raw component matrix')
 require(e['source_manifest_sha256']==SOURCE and e['formal_target_sha256']==FORMAL
  and e['automatic_retries']==0 and e['desktop_inputs']==0,'raw execution target or scope differs')
 require(e['source_inputs_unchanged'] is True and e['normal_products_unchanged'] is True and e['forced_cleanup_used'] is False,'inputs changed or cleanup forced')
 require(e['owned_job_kill_on_close'] is True and e['job_accounting_final']['active_processes']==0
  and e['job_accounting_final']['total_processes']==106,'original matrix Job accounting differs')
 check_pins(e['evidence_pins']);check_pins(e['tool_source_pins'])
 require(sha(owned(e['tool_source_review']))==e['tool_source_review_sha256'],'component tool source review changed')
 review=read(e['tool_source_review']);require(review['component_producer_source_complete'] is True
  and review['diagnostic_preservation_source_complete'] is True,'component producer source review incomplete')
 require(review['source_pins']==e['tool_source_pins'],'executed tool source pin set differs')
 require(sha(owned(e['images_qualification']))==e['images_qualification_sha256'],'component image qualification changed')
 iq=read(e['images_qualification']);require(iq['schema']=='E149-component-images-qualified-v1'
  and iq['status']=='qualified_linked_selected_component_matrix_images','actual four-image qualification required')
 require(iq['source_manifest_sha256']==SOURCE and iq['formal_target_sha256']==FORMAL and iq['original_compiler_exit_codes']==[0,0,0,0]
  and iq['original_compilers_terminal'] is True and iq['compiler_controls_archived'] is True,'actual linked build identity differs')
 require(e['plan']==iq['plan'] and e['plan_sha256']==iq['plan_sha256']==sha(iq['plan']),'component plan differs from linked images')
 p=read(e['plan']);require(p['schema']=='E149-recovery-component-matrix-plan-v1','component plan schema differs')
 build=read(iq['compile_cohort']);require(sha(iq['compile_cohort'])==iq['compile_cohort_sha256'],'sealed compiler cohort changed')
 check_pins(build['input_pins']);check_pins(build['normal_product_pins'])
 inputs=read(p['base_compiled_inputs']);require(len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items()),'normal compiled inputs changed')
 require(set(iq['images'])=={profile+'-'+variant for profile in ['Debug','Release'] for variant in ['baseline','candidate']},'image matrix differs')
 rows=[];inventories={};inventory_checks=[]
 def snapshot(proof,expected_path):
  archive=owned(proof['archive'],execution.parent)
  require(archive==expected_path and e['evidence_pins'].get(str(archive))==proof['archive_sha256']==sha(archive),'diagnostic archive not preserved/pinned')
  files=proof['files'];metadata=proof['metadata']
  require(len(files)<=512 and set(metadata)==set(files)
   and all(re.fullmatch('[a-z0-9-]+[.]json',name) for name in files),'unreviewed diagnostic archive entries')
  with zipfile.ZipFile(archive) as z:
   entries=z.infolist()
   require(len(entries)==len(files) and {item.filename for item in entries}==set(files),'diagnostic archive has missing/duplicate entries')
   require(all(not item.is_dir() and item.file_size<=8_388_608 for item in entries)
    and sum(item.file_size for item in entries)<=33_554_432,'diagnostic archive exceeds capture bounds')
   payload={item.filename:z.read(item) for item in entries}
  require(all(hashlib.sha256(blob).hexdigest().upper()==files[name] and len(blob)==metadata[name]['size']
   and type(metadata[name]['mtime_ns']) is int for name,blob in payload.items()),'diagnostic archive bytes/metadata differ')
  return payload
 before_payload={}
 require(set(e['diagnostics_before'])=={'Debug','Release'},'both original diagnostic profiles required')
 for profile in ['Debug','Release']:
  before=e['diagnostics_before'][profile]
  require(owned(before['root'],P/'target/current/verification/native-teleport-wire')==P/'target/current/verification/native-teleport-wire'/profile,'diagnostic root escaped profile')
  before_payload[profile]=snapshot(before,execution.parent/(profile+'-diagnostics-before.zip'))
 def diagnostics(record,profile,is_inventory=False):
  require(record['diagnostic_restore_verified'] is True,'original diagnostic bytes/timestamps not restored after process')
  folder=owned(Path(record['stdout']).parent,execution.parent);before=e['diagnostics_before'][profile]
  proof=record['diagnostic_after_snapshot'];payload=snapshot(proof,folder/'diagnostics-after.zip')
  require(set(before['files'])<=set(proof['files']),'prior diagnostic deleted during process')
  expected={name for name,h in proof['files'].items() if name not in before['files'] or h!=before['files'][name]
   or proof['metadata'][name]['mtime_ns']!=before['metadata'][name]['mtime_ns']}
  changes=record['diagnostic_changes'];names=[item['name'] for item in changes]
  require(len(names)==len(set(names)) and set(names)==expected,'changed diagnostic artifacts differ from captured delta')
  if is_inventory:require(not changes,'--list changed fixed diagnostic outputs')
  for artifact in changes:
   file=owned(artifact['artifact'],folder/'diagnostics')
   require(file==folder/'diagnostics'/artifact['name'] and e['evidence_pins'].get(str(file))==artifact['sha256']==proof['files'][artifact['name']]==sha(file)
    and file.read_bytes()==payload[artifact['name']],'changed diagnostic artifact not preserved/pinned')
   diagnostic=read(file);require(diagnostic['synthetic_encrypted_loopback'] is True and diagnostic['authenticated_formal_sessions']==0
    and diagnostic['seeded_scope_from_C02'] is True,'component diagnostic asserted formal authentication')
  manifest=owned(record['diagnostic_restoration_manifest'],folder)
  require(manifest==folder/'diagnostic-restoration.json' and e['evidence_pins'].get(str(manifest))==sha(manifest),'restoration receipt not preserved/pinned')
  require(read(manifest)==dict(profile=profile,after_snapshot=proof,changed_artifacts=changes,original_bytes_and_mtimes_restored=True),'restoration receipt differs from original record')
 def process(record,image,command):
  row=record['process'];require(row['command']==command and Path(row['executable'])==Path(image['product']).resolve()
   and row['executable_sha256']==image['sha256']==sha(owned(image['product'])),'original selected process image/command differs')
  require(row['created_suspended'] is True and row['job_assigned_before_resume'] is True and not row.get('forced_cleanup',False),'process ran before private Job enrollment or was terminated')
  require(record['environment']=={'RUST_BACKTRACE':'0','RUST_LIB_BACKTRACE':'0','RUST_TEST_THREADS':'1'}
   and record['inherited_test_options_removed'] is True,'unreviewed inherited test environment')
  for key in ['stdout','stderr']:
   file=owned(record[key],execution.parent);require(e['evidence_pins'].get(str(file))==sha(file),'consumed original log not pinned')
  rows.append(row)
  return Path(record['stdout']).read_text('utf-8-sig'),Path(record['stderr']).read_text('utf-8-sig'),row['exit_code']
 require(len(e['inventories'])==4,'four actual libtest inventories required')
 for row in e['inventories']:
  key=row['profile']+'-'+row['variant'];require(key in iq['images'] and key not in inventories,'duplicate/unknown inventory image')
  image=iq['images'][key];stdout,stderr,code=process(row,image,[image['product'],'--list'])
  diagnostics(row,row['profile'],is_inventory=True)
  names=inventory(stdout,stderr,code);inventories[key]=names
  prefixes=('modern_lito::native_login::rental_return_tests::','modern_lito::native_login::teleport_wire_tests::')
  selected={name for name in names if name.startswith(prefixes)}
  require(selected=={case['test_name'] for case in p['cases'][row['variant']]},'actual selected-module inventory differs from source-derived plan')
  inventory_checks.append(dict(profile=row['profile'],variant=row['variant'],actual_inventory=len(names),selected=len(selected)))
 for profile in ['Debug','Release']:
  baseline=set(inventories[profile+'-baseline']);candidate=set(inventories[profile+'-candidate'])
  require(candidate-baseline=={case['test_name'] for case in p['cases']['candidate']} - {case['test_name'] for case in p['cases']['baseline']}
   and not baseline-candidate,'unrelated library inventory changed between variants')
 expected=[]
 for profile in ['Debug','Release']:
  for variant in ['baseline','candidate']:
   cases=p['cases'][variant]
   ordered=[case for case in cases if case['existing_control']]+[case for case in cases if not case['existing_control']]
   expected += [(profile,variant,case) for case in ordered]
 require(len(expected)==len(e['cases'])==102,'missing/extra original component case execution')
 checks=[]
 for row,(profile,variant,case) in zip(e['cases'],expected):
  require(row['profile']==profile and row['variant']==variant and row['case']==case,'case order, exact selection or expectation differs')
  image=iq['images'][profile+'-'+variant]
  stdout,stderr,code=process(row,image,[image['product'],case['test_name'],'--exact','--nocapture','--test-threads=1'])
  site=assertion_site(p,case) if case['expected']=='specific_assertion_failure' else None
  result=selected_result(stdout,stderr,code,case['test_name'],len(inventories[profile+'-'+variant]),case['expected'],site,case['panic_contains'])
  diagnostics(row,profile)
  checks.append(dict(result,profile=profile,variant=variant,panic_site=site))
 require(len(rows)==106,'original inventory/case process inventory differs')
 for profile in ['Debug','Release']:
  before=e['diagnostics_before'][profile];root=owned(before['root'],P/'target/current/verification/native-teleport-wire')
  require(root==P/'target/current/verification/native-teleport-wire'/profile,'diagnostic root escaped profile')
  files=before['files']
  actual={file.name:sha(owned(file,root)) for file in root.iterdir()} if root.exists() else {}
  require(actual==files and e['diagnostic_roots_restored'] is True
   and all((root/name).stat().st_mtime_ns==before['metadata'][name]['mtime_ns'] and (root/name).stat().st_nlink==1 for name in files),
   'prior target diagnostic bytes/timestamps changed after matrix')
 terminal=terminal_observation(rows);check_pins(e['evidence_pins'])
 return dict(status='qualified_selected_baseline_counterexamples_and_candidate_components',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,
  execution=str(execution),execution_sha256=sha(execution),images_qualification=e['images_qualification'],images_qualification_sha256=e['images_qualification_sha256'],
  inventory_checks=inventory_checks,case_checks=checks,actual_case_executions=102,positive_case_passes=90,designated_baseline_assertion_failures=12,
  original_processes_terminal=True,terminal_OS_observation=terminal,prior_diagnostics_restored=True,new_test_executions_by_reader=0,
  formal_server_sessions_executed=0,full_library_qualified=False,full_G06_qualified=False,GUI_verified=False,whole_goal_complete=False,
  validator_sha256=sha(__file__),recorded_utc=datetime.now(timezone.utc).isoformat())

def main():
 ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('execution',type=Path);a=ap.parse_args()
 result=verify(a.execution);out=a.execution.with_name('qualified-selected-components.json');write_new(out,result)
 print(json.dumps(dict(qualification=str(out),actual_cases=102,whole_goal_complete=False),ensure_ascii=False))
if __name__=='__main__':main()
