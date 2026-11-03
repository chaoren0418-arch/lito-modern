"""Reuse the original eighteen compiler-only commands; append one new module."""
from native_E149_recovery_contract import R,read,sha,require
T=R/'維護工具';w=read(R/'工作佇列.json');q=read(w['current_E149_berserk55_disease56_development']['qualification']);p=T/'compile-native-E149-freeze50.py';require(sha(p)==q['source_pins'][str(p)],'compiler ancestor differs');s=p.read_text('utf-8-sig')
def replace(a,b):
 global s
 require(s.count(a)==1,'compiler anchor differs '+a[:100]);s=s.replace(a,b)
replace('native-E149-freeze50-20261024-103539','native-E149-blizzard59-20261024-120000')
replace("T/'templates/native_freeze50_request_tests.rs',T/'templates/native_freeze50_clock_tests.rs',T/'templates/native_freeze50_scene_tests.rs'","T/'templates/native_blizzard59_request_tests.rs',T/'templates/native_blizzard59.rs',T/'templates/native_blizzard59_tests.rs'")
replace("==365,'supplementary module count differs'","==366,'supplementary module count differs'")
replace('module_count=365','module_count=366')
s=s.replace('freeze50','blizzard59').replace('50 request and silent class4 plus flight history/failed preparation','59 selector, twenty-five independent nodes and scene history')
out=T/'compile-native-E149-blizzard59.py';require(not out.exists(),'compiler exists');out.write_text(s,encoding='utf-8',newline='\n');print(out)
