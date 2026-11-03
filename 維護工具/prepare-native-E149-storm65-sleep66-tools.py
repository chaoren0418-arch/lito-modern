"""Derive the unchanged eighteen metadata command graph; do not run test outputs."""
import ast
from native_E149_recovery_contract import R,read,sha,require
T=R/'維護工具';q=read(read(R/'工作佇列.json')['current_E149_earthquake62_silence64_development']['qualification']);p=T/'compile-native-E149-earthquake62-silence64.py';require(sha(p)==q['source_pins'][str(p)],'compiler predecessor differs');s=p.read_text('utf-8-sig')
def replace(a,b):
 global s
 require(s.count(a)==1,'compiler anchor differs '+a[:90]);s=s.replace(a,b)
replace('native-E149-earthquake62-silence64-20261024-132800','native-E149-storm65-sleep66-20261024-135000')
replace("T/'templates/native_earthquake62_silence64_request_tests.rs',T/'templates/native_earthquake62_support.rs',T/'templates/native_earthquake62_tests.rs',T/'templates/native_silence64_render_tests.rs'", "T/'templates/native_storm65_sleep66_request_tests.rs',T/'templates/native_storm65_support.rs',T/'templates/native_storm65_tests.rs',T/'templates/native_sleep66_definition_tests.rs',T/'templates/native_sleep66_render_tests.rs'")
s=s.replace('earthquake62-silence64','storm65-sleep66').replace('62 centered triple-layer finite and64 actor XY/material request','65 class1 overlap audio and66 actor XY/material finite request');ast.parse(s)
p=T/'compile-native-E149-storm65-sleep66.py';require(not p.exists(),'compiler exists');p.write_text(s,encoding='utf-8',newline='\n');print(p)
