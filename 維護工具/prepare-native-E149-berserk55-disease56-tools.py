"""Derive the original eighteen metadata commands for six new runtime overrides."""
from pathlib import Path
from native_E149_recovery_contract import R,read,sha,require
T=R/'維護工具';w=read(R/'工作佇列.json');q=read(w['current_E149_summon51_development']['qualification']);p=T/'compile-native-E149-freeze50.py';require(sha(p)==q['source_pins'][str(p)],'compiler ancestor differs');s=p.read_text('utf-8-sig')
def replace(a,b):
 global s
 require(s.count(a)==1,'compiler anchor differs '+a[:90]);s=s.replace(a,b)
replace('native-E149-freeze50-20261024-103539','native-E149-berserk55-disease56-20261024-111546')
replace("T/'templates/native_freeze50_request_tests.rs',T/'templates/native_freeze50_clock_tests.rs',T/'templates/native_freeze50_scene_tests.rs'","T/'templates/native_berserk55_disease56_request_tests.rs',T/'templates/native_berserk55_disease56_definition_tests.rs',T/'templates/native_berserk55_disease56_render_tests.rs',Path(contract['original_inspector_failure']),Path(contract['original_inspector_failure']).with_name('failed-inspector.py')")
s=s.replace('freeze50','berserk55-disease56').replace('50 request and silent class4 plus flight history/failed preparation','55/56 requests, S37 frozen heading/eight banks and Multiply')
out=T/'compile-native-E149-berserk55-disease56.py';require(not out.exists(),'compiler exists');out.write_text(s,encoding='utf-8',newline='\n');print(out)
