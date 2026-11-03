"""Preserve the failed compiler/source; explicitly type only the QA resource PathBuf."""
from pathlib import Path
from native_E149_recovery_contract import R,read,sha,encoded,require
T=R/'維護工具';V=R/'驗證紀錄/current';C=T/'candidates/native-E149-eruption45-scene-20261024-091703';D=T/'candidates/native-E149-eruption45-scene-root-20261024-093000'
failed=V/'native-E149-eruption45-scene-metadata/20261024-092452-006183/cohort.json'
c=read(failed);require(c['status']=='original_compile_failure_preserved' and [(r['profile'],r['kind'],r['exit_code']) for r in c['compiler_runs']]==[('Debug','library',0),('Debug','library-test',1)] and c['owned_process'] is None,'original compiler failure differs')
k=read(C/'contract.json');require(sha(C/'contract.json')==c['contract_sha256'],'failed contract changed')
source=C/'native_eruption45_scene.rs';old=source.read_text('utf-8-sig');a='let root=std::env::var_os("LITO_SHARED_RESOURCE_ROOT")';b='let root:std::path::PathBuf=std::env::var_os("LITO_SHARED_RESOURCE_ROOT")';require(old.count(a)==1,'QA path anchor differs')
D.mkdir();adapter=D/source.name;adapter.write_text(old.replace(a,b),encoding='utf-8',newline='\n');name='LiTo381-Modern/src/modern_lito/native_spell_projectiles.rs';wrapper=D/name;wrapper.parent.mkdir(parents=True);wrapper.write_bytes((C/name).read_bytes())
for n,p in [(name,wrapper),('LiTo381-Modern/src/modern_lito/native_eruption45_scene.rs',adapter)]:k['effective_overlay_paths'][n]=str(p);k['effective_overlay_files'][n]=sha(p)
k['modules']['native_eruption45_scene']=dict(path=str(adapter),sha256=sha(adapter));k['runtime_overrides'][name].update(path=str(wrapper),sha256=sha(wrapper))
failure_pins={str(failed):sha(failed),str(C/'contract.json'):sha(C/'contract.json'),str(source):sha(source),str(C/name):sha(C/name)}
for row in c['compiler_runs']:failure_pins.update(row['logs'])
k['original_compile_failure']=dict(cohort=str(failed),cohort_sha256=sha(failed),observed_original_tool_chunk='c12ec5',observed_actual_exit_code=1,source_pins=failure_pins,
 corrected_only_QA_path_type=dict(original=str(source),original_sha256=sha(source),replacement=str(adapter),replacement_sha256=sha(adapter),before=a,after=b),prior_partial_library_success_transferred=False,tests_executed=0)
(D/'contract.json').write_bytes(encoded(k));compiler=T/'compile-native-E149-eruption45-scene.py';s=compiler.read_text('utf-8-sig');needle="C=T/'candidates/native-E149-eruption45-scene-20261024-091703'";require(s.count(needle)==1,'compiler candidate anchor differs');s=s.replace(needle,"C=T/'candidates/native-E149-eruption45-scene-root-20261024-093000'")
needle="pins.update({row['path']:row['sha256'] for row in contract['runtime_overrides'].values()})";require(s.count(needle)==1,'failure pin insertion differs');s=s.replace(needle,needle+"\n pins.update(contract['original_compile_failure']['source_pins']);pins[str(T/'prepare-native-E149-eruption45-scene-root.py')]=sha(T/'prepare-native-E149-eruption45-scene-root.py')")
out=T/'compile-native-E149-eruption45-scene-root.py';require(not out.exists(),'do not overwrite fixed compiler');out.write_text(s,encoding='utf-8',newline='\n');print(D/'contract.json')
