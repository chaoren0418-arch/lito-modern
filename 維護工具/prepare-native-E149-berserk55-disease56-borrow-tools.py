"""Compile corrected QA as a fresh eighteen-command cohort; no partial transfer."""
from native_E149_recovery_contract import R,read,sha,require
T=R/'維護工具';p=T/'compile-native-E149-berserk55-disease56.py';s=p.read_text('utf-8-sig')
def replace(a,b):
 global s
 require(s.count(a)==1,'compiler anchor differs '+a[:90]);s=s.replace(a,b)
replace('native-E149-berserk55-disease56-20261024-111546','native-E149-berserk55-disease56-borrow-20261024-112152')
needle=" inputs=read(old['base_compiled_inputs']);";require(s.count(needle)==1,'input marker differs');s=s.replace(needle," for path in [T/'prepare-native-E149-berserk55-disease56-borrow.py',T/'prepare-native-E149-berserk55-disease56-borrow-tools.py',Path(contract['original_development_candidate']),Path(contract['original_metadata_failure']),*map(lambda v:Path(v['path']),contract['runtime_overrides']['LiTo381-Modern/src/modern_lito/native_skill_effects.rs']['supplements'])]:pins[str(path)]=sha(path)\n"+needle)
s=s.replace('native-E149-berserk55-disease56-metadata','native-E149-berserk55-disease56-borrow-metadata').replace('E149-berserk55-disease56-metadata-v1','E149-berserk55-disease56-borrow-metadata-v1');out=T/'compile-native-E149-berserk55-disease56-borrow.py';require(not out.exists(),'corrected compiler exists');out.write_text(s,encoding='utf-8',newline='\n');print(out)
