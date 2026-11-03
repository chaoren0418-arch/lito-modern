"""Derive the 18-command metadata compiler from the pinned four-command ancestor."""
from pathlib import Path
from native_E149_recovery_contract import R,sha,require
T=R/'維護工具';p=T/'compile-native-E149-eruption45-scene-root.py';require(sha(p)=='7150E0FB165F72BF80AC6603C9D6FA38BE20EA365DD8F194A95F16CCEE96A863','compiler ancestor differs');s=p.read_text('utf-8-sig')
def replace(a,b):
 global s
 require(s.count(a)==1,'compiler source anchor differs '+a[:100]);s=s.replace(a,b)
replace("pins.update(contract['original_compile_failure']['source_pins']);pins[str(T/'prepare-native-E149-eruption45-scene-root.py')]=sha(T/'prepare-native-E149-eruption45-scene-root.py')", "pins[str(T/'prepare-native-E149-eruption45-entry-tools.py')]=sha(T/'prepare-native-E149-eruption45-entry-tools.py')")
replace('native-E149-eruption45-scene-root-20261024-093000','native-E149-eruption45-entry-20261024-094100')
s=s.replace('eruption45-scene','eruption45-entry').replace('Four metadata-only commands','Eighteen metadata-only commands').replace('four metadata commands','eighteen metadata commands').replace('four_metadata_compilers_complete_no_tests','eighteen_metadata_compilers_complete_no_tests')
replace("==364,'predecessor module count differs'","==365,'predecessor module count differs'")
replace("old=read(contract['predecessor_metadata_cohort'])", """old=read(contract['predecessor_metadata_cohort'])
 require(sha(contract['integration_predecessor_cohort'])==contract['integration_predecessor_cohort_sha256'],'integration predecessor differs');paired=read(contract['integration_predecessor_cohort'])['paired_tests'];require(set(paired)=={'blind20','slow29','advance79','item_spells','magical73','shadow107','object374448'},'retained integration set differs')""")
replace("for p in [C/'contract.json',Path(contract['inspection']),Path(__file__).resolve()", "for p in [Path(contract['override_chain']),Path(contract['integration_predecessor_cohort']),T/'templates/native_eruption45_entry_tests.rs',*map(Path,paired.values()),C/'contract.json',Path(contract['inspection']),Path(__file__).resolve()")
replace("module_count=365,original_nested_runtime_pairs=old['original_nested_runtime_pairs']", "module_count=365,paired_tests=paired,original_nested_runtime_pairs=old['original_nested_runtime_pairs']")
replace("for kind in ['library','library-test']:", "for kind in ['library','library-test','blind20','slow29','advance79','item_spells','magical73','shadow107','object374448']:")
replace("command[command.index(old['generated_root'])]=str(out/'lib.rs');product=products/(profile+'-'+kind+'.rmeta');command[command.index('-o')+1]=str(product)", "command[command.index(old['generated_root'])]=str(out/'lib.rs') if kind in ['library','library-test'] else paired[kind];product=products/(profile+'-'+kind+'.rmeta');command[command.index('-o')+1]=str(product)")
replace("if kind=='library-test':command.remove('--crate-type=rlib');command+=['--test']", """if kind!='library':command.remove('--crate-type=rlib');command+=['--test']
    if kind in paired:
     command[command.index('--crate-name')+1]='native_'+kind+'_development';command+=['--extern','lito_modern='+str(products/(profile+'-library.rmeta'))]""")
out=T/'compile-native-E149-eruption45-entry.py';require(not out.exists(),'do not overwrite entry compiler');out.write_text(s,encoding='utf-8',newline='\n');print(out)
