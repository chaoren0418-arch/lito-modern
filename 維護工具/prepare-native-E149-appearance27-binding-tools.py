"""Retain18 original metadata targets and six nested pairs for the binding candidate."""
import ast
from native_E149_recovery_contract import R,read,sha,require
T=R/'維護工具';q=read(read(R/'工作佇列.json')['current_E149_appearance27_queue_development']['qualification']);p=T/'compile-native-E149-appearance27-queue.py';require(sha(p)==q['source_pins'][str(p)],'compiler ancestor differs');s=p.read_text('utf-8-sig')
def replace(a,b):
 global s
 require(s.count(a)==1,'compiler anchor differs '+a[:90]);s=s.replace(a,b)
replace('native-E149-appearance27-queue-20261024-141000','native-E149-appearance27-binding-admission-20261024-144000')
replace("T/'templates/native_appearance27_queue.rs'","T/'templates/native_appearance27_binding_support.rs',T/'templates/native_appearance27_animator_binding.rs',T/'templates/native_appearance27_combat_binding.rs',T/'templates/native_appearance27_world_tests.rs',T/'templates/native_appearance27_animator_admission_tests.rs'")
replace("==367,'predecessor module count differs'","==368,'predecessor module count differs'")
needle=" pins.update({row['path']:row['sha256'] for row in contract['runtime_overrides'].values()})"
extra="\n cb=read(contract['callback_inspection']);require(sha(contract['callback_inspection'])==contract['callback_inspection_sha256'],'callback proof changed');pins.update(cb['source_pins']);pins[contract['callback_inspection']]=sha(contract['callback_inspection']);pins[contract['source_review_precursor']]=contract['source_review_precursor_sha256'];pins[str(T/'refine-native-E149-appearance27-binding-admission.py')]=sha(T/'refine-native-E149-appearance27-binding-admission.py')"
replace(needle,needle+extra)
s=s.replace('appearance27-queue','appearance27-binding').replace('S27 ordered-body source component only, not yet World/GUI integrated','S27 World/body/layer/History source binding; no GAME or GUI qualification');ast.parse(s)
p=T/'compile-native-E149-appearance27-binding.py';require(not p.exists(),'compiler exists');p.write_text(s,encoding='utf-8',newline='\n');print(p)
