"""Derive the original18 metadata graph plus one source module, without execution."""
import ast
from native_E149_recovery_contract import R,read,sha,require
T=R/'維護工具';q=read(read(R/'工作佇列.json')['current_E149_storm65_sleep66_development']['qualification']);p=T/'compile-native-E149-storm65-sleep66.py';require(sha(p)==q['source_pins'][str(p)],'compiler differs');s=p.read_text('utf-8-sig')
def replace(a,b):
 global s
 require(s.count(a)==1,'compiler anchor differs '+a[:90]);s=s.replace(a,b)
replace('native-E149-storm65-sleep66-20261024-135000','native-E149-appearance27-queue-20261024-141000')
replace("T/'templates/native_storm65_sleep66_request_tests.rs',T/'templates/native_storm65_support.rs',T/'templates/native_storm65_tests.rs',T/'templates/native_sleep66_definition_tests.rs',T/'templates/native_sleep66_render_tests.rs'","T/'templates/native_appearance27_queue.rs'")
replace("==367,'supplementary module count differs'","==368,'supplementary module count differs'")
replace('module_count=367','module_count=368')
s=s.replace('storm65-sleep66','appearance27-queue').replace('65 class1 overlap audio and66 actor XY/material finite request','S27 ordered-body source component only, not yet World/GUI integrated');ast.parse(s);p=T/'compile-native-E149-appearance27-queue.py';require(not p.exists(),'compiler exists');p.write_text(s,encoding='utf-8',newline='\n');print(p)
