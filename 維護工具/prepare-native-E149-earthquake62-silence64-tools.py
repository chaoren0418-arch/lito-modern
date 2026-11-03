"""Retain all eighteen Debug/Release metadata commands and original 367-module graph."""
from native_E149_recovery_contract import R,read,sha,require
T=R/'維護工具';q=read(read(R/'工作佇列.json')['current_E149_finite_history_development']['qualification']);p=T/'compile-native-E149-finite-history.py';require(sha(p)==q['source_pins'][str(p)],'compiler ancestor differs');s=p.read_text('utf-8-sig')
def replace(a,b):
 global s
 require(s.count(a)==1,'compiler anchor differs '+a[:95]);s=s.replace(a,b)
replace('native-E149-finite-history-20261024-130100','native-E149-earthquake62-silence64-20261024-132800');replace("T/'templates/native_finite_history_tests.rs',T/'templates/native_finite_scene_history_tests.rs'","T/'templates/native_earthquake62_silence64_request_tests.rs',T/'templates/native_earthquake62_support.rs',T/'templates/native_earthquake62_tests.rs',T/'templates/native_silence64_render_tests.rs'");s=s.replace('finite-history','earthquake62-silence64').replace('finite profile source retirement, late admission and active audio retention','62 centered triple-layer finite and64 actor XY/material request');p=T/'compile-native-E149-earthquake62-silence64.py';require(not p.exists(),'compiler exists');p.write_text(s,encoding='utf-8',newline='\n');print(p)
