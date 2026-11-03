"""Qualify four actually linked library-test images; never execute an image."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,struct,subprocess,zipfile
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,owned,check_pins,require,write_new

def qualify(run,terminal_chunk):
 run=owned(run.resolve(strict=True),R/'驗證紀錄/current/native-E149-component-matrix-compile')
 c=read(run/'cohort.json');p=read(c['plan'])
 require(c['schema']=='E149-recovery-component-matrix-compile-v1' and c['status']=='linked_test_images_complete_no_execution'
  and c['metadata_only'] is False and c['owned_process'] is None,'four completed original link commands required')
 require(c['source_manifest_sha256']==SOURCE and c['formal_target_sha256']==FORMAL,'linked source/target differs')
 require(c['tests_executed']==0 and c['PEs_executed'] is False,'not a compiler-only build receipt')
 require(sha(c['plan'])==c['plan_sha256'],'original component plan changed')
 check_pins(c['input_pins']);check_pins(c['normal_product_pins'])
 require(all(sha(n)==h for n,h in c['external_toolchain_dependency_pins'].items()),'read-only toolchain changed')
 inputs=read(p['base_compiled_inputs']);require(len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items()),'normal input identity differs')
 require([(x['profile'],x['variant']) for x in c['compiler_runs']]==[(profile,variant) for profile in ['Debug','Release'] for variant in ['baseline','candidate']],
  'original profile/variant link inventory differs')
 images={};identities=[c['pipeline']]
 for row in c['compiler_runs']:
  profile,variant=row['profile'],row['variant'];command=list(p['compiler_templates'][profile])
  command[command.index('--emit=metadata')]='--emit=link';command[command.index(p['roots']['candidate'])]=p['roots'][variant]
  command[command.index('-o')+1]=row['product']
  for flag in p['native_link_search_paths'][profile]:command+=['-L',flag]
  require(row['command']==command and row['exit_code']==0 and row['finished_utc'] and not row.get('exact_original_compiler_cleanup'),
   'original link command did not finish normally');check_pins(row['logs'])
  product=owned(row['product'],P/'target/current/verification/native-E149-component-matrix')
  require(sha(product)==row['product_sha256'] and product.stat().st_size==row['product_bytes'],'actual PE changed')
  with product.open('rb') as file:
   dos=file.read(64);require(dos[:2]==b'MZ','not a PE image')
   offset=struct.unpack_from('<I',dos,0x3c)[0];require(64<=offset<=product.stat().st_size-26,'PE header offset invalid')
   file.seek(offset);header=file.read(26)
  require(header[:4]==b'PE\0\0' and struct.unpack_from('<H',header,4)[0]==0x8664 and struct.unpack_from('<H',header,24)[0]==0x20b,
   'not x64 PE32+');identities.append(row['original_process'])
  images[profile+'-'+variant]=dict(profile=profile,variant=variant,product=str(product),sha256=row['product_sha256'],bytes=row['product_bytes'])
 ids=','.join(str(x['pid']) for x in identities)
 code="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);$ErrorActionPreference='Stop';@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
 observed=subprocess.check_output(['pwsh','-NoProfile','-NonInteractive','-Command',code],creationflags=subprocess.CREATE_NO_WINDOW)
 live=json.loads(observed.decode('utf-8-sig')) if observed.strip() else [];live=[live] if isinstance(live,dict) else live;epoch=datetime(1970,1,1,tzinfo=timezone.utc)
 for row in identities:
  for actual in live:
   if actual['ProcessId']==row['pid']:
    age=datetime.fromisoformat(actual['created'].replace('Z','+00:00'))-epoch
    millis=(age.days*86400+age.seconds)*1000+age.microseconds//1000
    require(millis!=row['start_epoch_millis'],'same original compiler/pipeline remains live')
 out=run/('qualified-images-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir();(out/'control-inputs').mkdir()
 # Preserve the compiler controls actually checked above. Later tool edits
 # must not relabel these original images or invalidate their source bytes.
 controls={str(R/'維護工具'/name) for name in ['prepare-native-E149-component-matrix.py','compile-native-E149-component-matrix.py']}
 pins=dict(c['input_pins']);snapshots={}
 for name in controls:
  require(name in pins,'original compiler control pin absent')
  target=out/'control-inputs'/Path(name).name;target.write_bytes(Path(name).read_bytes());require(sha(target)==pins[name],'compiler control snapshot differs')
  snapshots[name]=dict(snapshot=str(target),sha256=pins.pop(name));pins[str(target)]=sha(target)
 (out/'owned-CIM.json').write_bytes(observed)
 sealed=dict(c,input_pins=pins,original_observed_cohort=str(run/'cohort.json'),original_observed_cohort_sha256=sha(run/'cohort.json'),control_input_snapshots=snapshots)
 write_new(out/'qualified-build.json',sealed)
 result=dict(schema='E149-component-images-qualified-v1',status='qualified_linked_selected_component_matrix_images',
  source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,plan=c['plan'],plan_sha256=c['plan_sha256'],
  compile_cohort=str(out/'qualified-build.json'),compile_cohort_sha256=sha(out/'qualified-build.json'),
  original_observed_cohort=str(run/'cohort.json'),original_observed_cohort_sha256=sha(run/'cohort.json'),
  images=images,original_compiler_exit_codes=[0,0,0,0],original_compilers_terminal=True,
  compiler_controls_archived=True,compiler_control_snapshots=snapshots,observed_original_tool_chunk=terminal_chunk,observed_actual_exit_code=0,
  test_harness_inventory_executed=False,tests_executed=0,PEs_executed=False,full_library_qualified=False,full_G06_qualified=False,whole_goal_complete=False,
  qualifier_sha256=sha(__file__),recorded_utc=datetime.now(timezone.utc).isoformat())
 write_new(out/'qualified.json',result);(out/'qualifier.py').write_bytes(Path(__file__).read_bytes())
 return out/'qualified.json'

def main():
 ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('run',type=Path);ap.add_argument('--terminal-chunk',required=True);a=ap.parse_args()
 result=qualify(a.run,a.terminal_chunk);print(json.dumps(dict(qualification=str(result),linked_images=4,tests_executed=0),ensure_ascii=False))
if __name__=='__main__':main()
