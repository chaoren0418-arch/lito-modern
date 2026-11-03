"""Derive immutable eighteen-command metadata compiler, without running outputs."""
from pathlib import Path
from native_E149_recovery_contract import R,read,sha,require
T=R/'維護工具';w=read(R/'工作佇列.json');q=read(w['current_E149_sun46_weak47_development']['qualification']);p=T/'compile-native-E149-sun46-weak47.py';require(sha(p)==q['source_pins'][str(p)],'compiler ancestor differs');s=p.read_text('utf-8-sig')
def replace(a,b):
 global s
 require(s.count(a)==1,'compiler source anchor differs '+a[:80]);s=s.replace(a,b)
replace('native-E149-sun46-weak47-20261024-100201','native-E149-freeze50-20261024-103539')
replace("T/'templates/native_sun46_class1_support.rs',T/'templates/native_sun46_class1_tests.rs',T/'templates/native_sun46_weak47_tests.rs',T/'templates/native_weak47_mix_tests.rs'","T/'templates/native_freeze50_request_tests.rs',T/'templates/native_freeze50_clock_tests.rs',T/'templates/native_freeze50_scene_tests.rs'")
s=s.replace('sun46-weak47','freeze50').replace('46/47 requests, class1 history and world-owned finite audio','50 request and silent class4 plus flight history/failed preparation')
out=T/'compile-native-E149-freeze50.py';require(not out.exists(),'compiler already exists');out.write_text(s,encoding='utf-8',newline='\n');print(out)
