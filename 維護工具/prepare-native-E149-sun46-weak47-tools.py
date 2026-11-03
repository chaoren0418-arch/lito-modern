"""Derive unchanged 18-target compiler mechanics for the seven-source candidate."""
from pathlib import Path
import re
from native_E149_recovery_contract import R,sha,require
T=R/'維護工具';p=T/'compile-native-E149-eruption45-entry-libname.py';require(sha(p)=='BF98CE591013605530BBA4467F728904D15162C092752C994EF9EB238B2D0FF6','compiler ancestor differs');s=p.read_text('utf-8-sig')
def replace(a,b):
 global s
 require(s.count(a)==1,'compiler source anchor differs '+a[:100]);s=s.replace(a,b)
line=next(l for l in s.splitlines() if l.startswith(" pins=dict(old['input_pins']);external="));replace(line," pins=dict(old['input_pins']);external=dict(old['external_toolchain_dependency_pins'])")
replace('original_tooling_failure=original_tooling_failure,','historical_compiler_failure=old.get(\'original_tooling_failure\'),')
replace('native-E149-eruption45-entry-20261024-094100','native-E149-sun46-weak47-20261024-100201')
replace("Path(contract['override_chain']),Path(contract['integration_predecessor_cohort']),T/'templates/native_eruption45_entry_tests.rs'", "Path(contract['integration_predecessor_cohort']),T/'templates/native_sun46_class1_support.rs',T/'templates/native_sun46_class1_tests.rs',T/'templates/native_sun46_weak47_tests.rs',T/'templates/native_weak47_mix_tests.rs'")
s=s.replace('eruption45-entry','sun46-weak47').replace('complete predecessor graph plus received S23/129 presentation','46/47 requests, class1 history and world-owned finite audio')
out=T/'compile-native-E149-sun46-weak47.py';require(not out.exists(),'do not overwrite compiler');out.write_text(s,encoding='utf-8',newline='\n');print(out)
