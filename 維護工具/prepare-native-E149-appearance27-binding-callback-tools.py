"""Recompile18 targets for the callback-floor change after the original Eq run exits."""
import ast
from native_E149_recovery_contract import R,read,sha,require
T=R/'維護工具';cohort=R/'驗證紀錄/current/native-E149-appearance27-binding-eq-metadata/20261024-145109-485605/cohort.json';intermediate=read(cohort);require(intermediate['status']=='eighteen_metadata_compilers_complete_no_tests' and intermediate['owned_process'] is None and [r['exit_code']for r in intermediate['compiler_runs']]==[0]*18,'intermediate is not complete');p=T/'compile-native-E149-appearance27-binding-eq.py';require(sha(p)==intermediate['input_pins'][str(p)],'intermediate compiler changed');s=p.read_text('utf-8-sig')
def replace(a,b):
 global s
 require(s.count(a)==1,'callback compiler anchor differs '+a[:90]);s=s.replace(a,b)
replace('native-E149-appearance27-binding-eq-20261024-144000','native-E149-appearance27-binding-callback-20261024-144000')
replace("pins[str(T/'prepare-native-E149-appearance27-binding-eq-tools.py')]=sha(T/'prepare-native-E149-appearance27-binding-eq-tools.py')","pins[str(T/'prepare-native-E149-appearance27-binding-callback-tools.py')]=sha(T/'prepare-native-E149-appearance27-binding-callback-tools.py')")
replace("T/'templates/native_appearance27_animator_admission_tests.rs'","T/'templates/native_appearance27_animator_callback_tests.rs'")
replace(" out=owned(V/'native-E149-appearance27-binding-eq-metadata'/stamp())"," out=owned(V/'native-E149-appearance27-binding-callback-metadata'/stamp())")
replace("P/'target/current/verification/native-E149-appearance27-binding-eq-metadata'","P/'target/current/verification/native-E149-appearance27-binding-callback-metadata'")
replace("schema='E149-appearance27-binding-eq-metadata-v1'","schema='E149-appearance27-binding-callback-metadata-v1'")
anchor=" inputs=read(old['base_compiled_inputs']);"
extra=" intermediate=read(contract['intermediate_eq_metadata_cohort']);require(sha(contract['intermediate_eq_metadata_cohort'])==contract['intermediate_eq_metadata_cohort_sha256'],'intermediate Eq evidence changed');pins.update(intermediate['input_pins']);pins[contract['intermediate_eq_metadata_cohort']]=sha(contract['intermediate_eq_metadata_cohort']);pins[contract['intermediate_eq_candidate']]=sha(contract['intermediate_eq_candidate']);pins[str(T/'refine-native-E149-appearance27-binding-callback.py')]=sha(T/'refine-native-E149-appearance27-binding-callback.py');pins[str(T/'native-E149-appearance27-binding-callback.md')]=sha(T/'native-E149-appearance27-binding-callback.md')\n"
replace(anchor,extra+anchor)
ast.parse(s);p=T/'compile-native-E149-appearance27-binding-callback.py';require(not p.exists(),'callback compiler exists');p.write_text(s,encoding='utf-8',newline='\n');print(p)
