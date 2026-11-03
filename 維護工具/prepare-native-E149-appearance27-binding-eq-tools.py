"""Compile all18 targets anew after the exact Tape comparison correction."""
import ast
from native_E149_recovery_contract import R,read,sha,require
T=R/'維護工具';failed=read(R/'驗證紀錄/current/native-E149-appearance27-binding-metadata/20261024-144741-317912/cohort.json');p=T/'compile-native-E149-appearance27-binding.py';require(sha(p)==failed['input_pins'][str(p)],'original compiler changed');s=p.read_text('utf-8-sig')
def replace(a,b):
 global s
 require(s.count(a)==1,'compiler correction anchor differs '+a[:85]);s=s.replace(a,b)
replace('native-E149-appearance27-binding-admission-20261024-144000','native-E149-appearance27-binding-eq-20261024-144000')
replace("pins[str(T/'prepare-native-E149-appearance27-binding-tools.py')]=sha(T/'prepare-native-E149-appearance27-binding-tools.py')","pins[str(T/'prepare-native-E149-appearance27-binding-eq-tools.py')]=sha(T/'prepare-native-E149-appearance27-binding-eq-tools.py')")
replace("T/'templates/native_appearance27_binding_support.rs'","T/'templates/native_appearance27_binding_eq_support.rs'")
replace(" out=owned(V/'native-E149-appearance27-binding-metadata'/stamp())"," out=owned(V/'native-E149-appearance27-binding-eq-metadata'/stamp())")
replace("P/'target/current/verification/native-E149-appearance27-binding-metadata'","P/'target/current/verification/native-E149-appearance27-binding-eq-metadata'")
replace("schema='E149-appearance27-binding-metadata-v1'","schema='E149-appearance27-binding-eq-metadata-v1'")
anchor=" inputs=read(old['base_compiled_inputs']);"
extra=" original_failed=read(contract['original_failed_metadata_cohort']);require(sha(contract['original_failed_metadata_cohort'])==contract['original_failed_metadata_cohort_sha256'],'original failure changed');pins.update(original_failed['input_pins']);pins[contract['original_failed_metadata_cohort']]=sha(contract['original_failed_metadata_cohort']);pins[contract['original_failed_candidate']]=sha(contract['original_failed_candidate']);pins[str(T/'refine-native-E149-appearance27-binding-eq.py')]=sha(T/'refine-native-E149-appearance27-binding-eq.py');pins[str(T/'native-E149-appearance27-binding-eq.md')]=sha(T/'native-E149-appearance27-binding-eq.md')\n"
replace(anchor,extra+anchor)
ast.parse(s);p=T/'compile-native-E149-appearance27-binding-eq.py';require(not p.exists(),'corrected compiler exists');p.write_text(s,encoding='utf-8',newline='\n');print(p)
