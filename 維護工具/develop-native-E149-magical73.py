"""Prepare and type-check inventory-targeted #73; never execute tests or game code."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,ast,hashlib,json,os,re,subprocess,tomllib,traceback,zipfile
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
from native_E149_owned_process import identity,kernel
T=R/'維護工具';V=R/'驗證紀錄/current'
API=V/'native-E149-item-spell-contract/20261024-014418-307290'
INSPECTION=V/'native-E149-magical73-contract/20261024-022129-469250/inspection.json'
CLIENT=V/'native-formal-reference-refresh/20261023-192435-953238/ClientSession.stdout.txt'

def stamp():return datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
def idle():
 w=read(R/'工作佇列.json');require(not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'workload/source freeze active')
 require(w['current_user_requested_goal']['test_execution_deferred_by_user'],'development-only phase expected')
 require(sha(w['current_formal_reference_identity']['formal'])==FORMAL,'formal target changed');return w
def original_method(path,name):
 text=Path(path).read_text('utf-8-sig');heads=list(re.finditer(r'^  [^ \n].*\([^\n]*\)[^\n]*;\s*$',text,re.M))
 found=[i for i,m in enumerate(heads) if name+'(' in m[0]];require(len(found)==1,'original method missing/ambiguous')
 i=found[0];return text[heads[i].start():heads[i+1].start() if i+1<len(heads) else len(text)]

def prepare():
 w=idle();ref=w['current_E149_integrated_client_development'];require(sha(ref['qualification'])==ref['qualification_sha256'],'integrated qualification changed')
 overlay=read(ref['overlay']);require(sha(ref['overlay'])==ref['overlay_sha256'] and len(overlay['candidate_files'])==19,'integrated overlay changed')
 oldref=w['current_native_item_spells_development'];require(sha(oldref['qualification'])==oldref['qualification_sha256'],'item qualification changed')
 old=read(Path(oldref['compile_run'])/'cohort.json');require(sha(Path(oldref['compile_run'])/'cohort.json')==oldref['compile_cohort_sha256'],'metadata cohort changed')
 check_pins(old['input_pins']);check_pins(old['normal_product_pins'])
 inspection=read(INSPECTION);require(inspection['exit_code']==0 and inspection['jar_sha256']==FORMAL,'RuntimeInventory javap failed');check_pins(inspection['outputs'])
 runtime=original_method(INSPECTION.parent/'RuntimeInventory.stdout.txt','createMagicalWeapon')
 body=original_method(CLIENT,'nativeMaterialMagicalWeaponBody');action=(API/'NativeMagicalWeaponAction.stdout.txt').read_text('utf-8-sig')
 require(all(x in runtime for x in ['items:Ljava/util/Map;','NativeWeaponStatus.safeEnchant:()I','Item.enchantLevel:()I','Item.withEnchantLevel:(I)','Map.put:']), 'original item effect changed')
 require(all(x in body for x in ['Protocol.objectSpellRequest:','ObjectSpellRequest.targetObjectId:()J','NativeMagicalWeaponAction.admit:','Protocol.changeInventory']), 'original item target chain changed')
 require('RuntimeInventory.createMagicalWeapon:(J)Z' in action and 'NativeLightAction.commitSelf:' in action,'original action bridge changed')
 out=owned(T/'candidates'/('native-E149-magical73-'+stamp()));out.mkdir(parents=True)
 contract=dict(formal_target_sha256=FORMAL,skill=73,row=9,column=0,action=19,effect=763,base_MP=200,reuse_millis=0,material=40318,material_count=10,messages=[685,79,161],
  intent_domain='inventory_object_not_self_or_actor',server_effect='existing eligible weapon enchantLevel 0 to 1; no new object ID',runtime_method=runtime,ClientSession_method=body,
  original_pins={str(p):sha(p) for p in [INSPECTION,INSPECTION.parent/'RuntimeInventory.stdout.txt',CLIENT,API/'NativeMagicalWeaponAction.stdout.txt',API/'BattleSpellCatalog$MagicalWeaponSpell.stdout.txt']},game_classes_executed=False,tests_executed=0)
 (out/'contract.json').write_bytes(encoded(contract));changed={};base={}
 changes={
  'LiTo381-Modern/src/modern_lito/native_skills.rs': [('matches!(skill,12|21)','matches!(skill,12|21|73)'),('match skill {77=>5000,60=>0,','match skill {77=>5000,60|73=>0,')],
  'LiTo381-Modern/src/modern_lito/native_skill_selection.rs': [('                        21 => "#21 防具附魔（選取背包物品）".into(),','                        21 => "#21 防具附魔（選取背包物品）".into(),\n                        73 => "#73 創造魔法武器（選取背包武器；消耗 10 顆魔法寶石）".into(),')],
  'LiTo381-Modern/tests/native_item_spells.rs': [('learned_skills.extend([12,21])','learned_skills.extend([12,21,73])'),('[(12,1,3),(21,2,4)]','[(12,1,3),(21,2,4),(73,9,0)]'),('assert_eq!(skills::reuse_millis(id),10)','assert_eq!(skills::reuse_millis(id),if id==73{0}else{10})'),('skills::item_request(&s,7,73,&target).is_err()','skills::item_request(&s,7,74,&target).is_err()')],
 }
 for n,replacements in changes.items():
  source=Path(overlay['candidate_paths'][n]);require(sha(source)==overlay['candidate_files'][n],'base candidate changed');text=source.read_text('utf-8-sig')
  for before,after in replacements:require(text.count(before)==1,'source anchor differs: '+before);text=text.replace(before,after)
  p=out/n;p.parent.mkdir(parents=True,exist_ok=True);p.write_text(text,encoding='utf-8',newline='\n');changed[n]=sha(p);base[n]=sha(source)
 n='LiTo381-Modern/tests/native_magical73.rs';p=out/n;p.write_bytes((T/'templates/native_magical73.rs').read_bytes());changed[n]=sha(p)
 paths=dict(overlay['candidate_paths']);files=dict(overlay['candidate_files'])
 for n,h in changed.items():paths[n]=str(out/n);files[n]=h
 require(len(files)==20,'merged source count differs')
 candidate=dict(schema='E149-magical73-source-v1',base_source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,candidate_files=changed,base_files=base,
  original_integrated_qualification=ref,original_item_qualification=oldref,original_metadata_cohort=str(Path(oldref['compile_run'])/'cohort.json'),
  effective_overlay_files=files,effective_overlay_paths=paths,contract=str(out/'contract.json'),contract_sha256=sha(out/'contract.json'),
  changed_runtime_files=list(changes)[:2],new_tests_written=5,retained_item_tests=8,tests_executed=0,production_applied=False,whole_goal_complete=False)
 (out/'candidate.json').write_bytes(encoded(candidate));print(json.dumps(dict(candidate=str(out/'candidate.json'),changed_files=4,effective_overlay_files=20,tests_executed=0),ensure_ascii=False))

def compile_candidate(cp):
 w=idle();cp=owned(cp.resolve(strict=True));candidate=read(cp);old=read(candidate['original_metadata_cohort'])
 require(candidate['base_source_manifest_sha256']==SOURCE and candidate['formal_target_sha256']==FORMAL,'candidate identity differs')
 pins=dict(old['input_pins']);pins.update({str(R/n):h for n,h in read(old['base_compiled_inputs']).items()})
 pins.update({candidate['effective_overlay_paths'][n]:h for n,h in candidate['effective_overlay_files'].items()})
 contract=read(candidate['contract']);check_pins(contract['original_pins']);pins.update(contract['original_pins'])
 for p in [cp,Path(candidate['contract']),Path(candidate['original_metadata_cohort']),Path(__file__).resolve(),T/'templates/native_magical73.rs',T/'native-E149-magical73.md']:pins[str(p)]=sha(p)
 out=owned(V/'native-E149-magical73-metadata'/stamp());out.mkdir(parents=True);products=owned(P/'target/current/verification/native-E149-magical73-metadata'/out.name);products.mkdir(parents=True);(products/'scratch').mkdir()
 modern=Path(old['generated_mod']).read_text('utf-8-sig')
 for n in candidate['changed_runtime_files']:
  previous=candidate['original_integrated_qualification']['overlay'];base=read(previous)['candidate_paths'][n]
  require(candidate['base_files'][n]==sha(base) and modern.count(Path(base).as_posix())==1,'full module predecessor differs')
  modern=modern.replace(Path(base).as_posix(),(cp.parent/n).as_posix())
 require(len(re.findall(r'#\[path = r#"',modern))==362,'complete module count differs')
 (out/'modern_lito.rs').write_text(modern,encoding='utf-8',newline='\n');lib=Path(old['generated_root']).read_text('utf-8-sig');needle=Path(old['generated_mod']).as_posix();require(lib.count(needle)==1,'library root differs');(out/'lib.rs').write_text(lib.replace(needle,(out/'modern_lito.rs').as_posix()),encoding='utf-8',newline='\n')
 paired=dict(old['paired_tests']);fixture=P/'tests/fixtures/native-world-current.json'
 for kind,name in [('item_spells','native_item_spells.rs'),('magical73','native_magical73.rs')]:
  source=cp.parent/'LiTo381-Modern/tests'/name;text=source.read_text('utf-8-sig');needle='include_bytes!("fixtures/native-world-current.json")';require(text.count(needle)==1,'QA fixture path differs');p=out/(kind+'-paired.rs');p.write_text(text.replace(needle,'include_bytes!(r#"'+fixture.as_posix()+'"#)'),encoding='utf-8',newline='\n');paired[kind]=str(p)
 for p in [out/'lib.rs',out/'modern_lito.rs',*map(Path,paired.values())]:pins[str(p)]=sha(p)
 (out/'developer.py').write_bytes(Path(__file__).read_bytes())
 def guard():
  idle();check_pins(pins);check_pins(old['normal_product_pins']);require(all(sha(p)==h for p,h in old['external_toolchain_dependency_pins'].items()),'compiler dependencies changed')
 guard();state=dict(schema='E149-magical73-metadata-v1',status='compiling_metadata',candidate=str(cp),candidate_sha256=sha(cp),input_pins=pins,
  original_metadata_cohort=candidate['original_metadata_cohort'],original_metadata_cohort_sha256=sha(candidate['original_metadata_cohort']),normal_product_pins=old['normal_product_pins'],
  external_toolchain_dependency_pins=old['external_toolchain_dependency_pins'],generated_root=str(out/'lib.rs'),generated_mod=str(out/'modern_lito.rs'),paired_tests=paired,
  pipeline=identity(kernel().GetCurrentProcess(),os.getpid()),compiler_runs=[],owned_process=None,tests_executed=0,PEs_linked=False,outputs_executed=False,whole_goal_complete=False)
 def save():(out/'cohort.json').write_bytes(encoded(state))
 save();print(json.dumps(dict(run=str(out),scope='fourteen metadata compiler commands; no test execution'),ensure_ascii=False),flush=True)
 env={k:v for k,v in os.environ.items() if not k.startswith(('LITO_','RUST_')) and k not in ['RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','RUSTC_WRAPPER','RUSTC_WORKSPACE_WRAPPER']};env.update(CARGO_MANIFEST_DIR=str(P),CARGO_PKG_VERSION=tomllib.loads((P/'Cargo.toml').read_text('utf-8-sig'))['package']['version'],CARGO_BUILD_JOBS='1',RAYON_NUM_THREADS='1',TEMP=str(products/'scratch'),TMP=str(products/'scratch'))
 try:
  for profile in ['Debug','Release']:
   library=products/('liblito_modern-'+profile+'.rmeta');template=next(r['command'] for r in old['compiler_runs'] if r['profile']==profile and r['kind']=='library')
   for kind in ['library','library-test','blind20','slow29','advance79','item_spells','magical73']:
    guard();command=list(template);require('--emit=metadata' in command and '--test' not in command,'library template differs');command[command.index(old['generated_root'])]=str(out/'lib.rs') if kind in ['library','library-test'] else paired[kind];product=library if kind=='library' else products/(profile+'-'+kind+'.rmeta');command[command.index('-o')+1]=str(product)
    if kind!='library':command.remove('--crate-type=rlib');command+=['--test']
    if kind in paired:command[command.index('--crate-name')+1]='native_'+kind+'_development';command+=['--extern','lito_modern='+str(library)]
    folder=out/(profile+'-'+kind);folder.mkdir()
    with (folder/'stdout.log').open('xb') as stdout,(folder/'stderr.log').open('xb') as stderr:
     child=subprocess.Popen(command,cwd=products,env=env,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW|subprocess.BELOW_NORMAL_PRIORITY_CLASS);row=dict(profile=profile,kind=kind,command=command,original_process=identity(int(child._handle),child.pid),product=str(product),exit_code=None);state['compiler_runs'].append(row);state['owned_process']=row['original_process'];save()
     try:child.wait(timeout=300)
     except BaseException:
      if child.poll() is None:require(identity(int(child._handle),child.pid)==row['original_process'],'compiler handle changed');child.terminate();child.wait(timeout=20);row['original_handle_cleanup']=True
      raise
     finally:row.update(exit_code=child.poll(),finished_utc=datetime.now(timezone.utc).isoformat(),logs={str(folder/n):sha(folder/n) for n in ['stdout.log','stderr.log']});state['owned_process']=None;save()
    require(child.returncode==0,'metadata compiler failed; preserve original source and diagnostics');row.update(product_sha256=sha(product),product_bytes=product.stat().st_size);save();guard()
  state['status']='fourteen_metadata_compilers_complete_no_tests'
 except BaseException:state['status']='original_compile_failure_preserved';(out/'original-failure.txt').write_text(traceback.format_exc(),encoding='utf-8');raise
 finally:state['finished_utc']=datetime.now(timezone.utc).isoformat();save()
 print(json.dumps(dict(run=str(out),exit_codes=[r['exit_code'] for r in state['compiler_runs']],tests_executed=0),ensure_ascii=False))

if __name__=='__main__':
 parser=argparse.ArgumentParser(description=__doc__);sub=parser.add_subparsers(dest='action',required=True);sub.add_parser('prepare');compile_parser=sub.add_parser('compile');compile_parser.add_argument('candidate',type=Path);args=parser.parse_args()
 if args.action=='prepare':prepare()
 else:compile_candidate(args.candidate)
