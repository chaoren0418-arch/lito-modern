"""Compile corrected pending policy; do not transfer initial metadata success."""
from native_E149_recovery_contract import R,sha,require
T=R/'維護工具';p=T/'compile-native-E149-summon51.py';s=p.read_text('utf-8-sig')
def replace(a,b):
 global s
 require(s.count(a)==1,'compiler anchor differs '+a[:90]);s=s.replace(a,b)
replace('native-E149-summon51-20261024-105000','native-E149-summon51-hold-20261024-105525')
needle=" inputs=read(old['base_compiled_inputs']);";require(s.count(needle)==1,'input anchor differs');s=s.replace(needle," for path in [T/'prepare-native-E149-summon51-hold.py',T/'prepare-native-E149-summon51-hold-tools.py',T/'native-E149-summon51-hold.md',T/'templates/native_summon51_controller_tests.rs',Path(contract['original_development_candidate']),Path(contract['original_metadata_cohort'])]:pins[str(path)]=sha(path)\n"+needle)
s=s.replace('native-E149-summon51-metadata','native-E149-summon51-hold-metadata').replace('E149-summon51-metadata-v1','E149-summon51-hold-metadata-v1')
out=T/'compile-native-E149-summon51-hold.py';require(not out.exists(),'corrected compiler exists');out.write_text(s,encoding='utf-8',newline='\n');print(out)
