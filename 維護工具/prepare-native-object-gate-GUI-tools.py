"""Adapt only version binding; retain all existing fourteen-scenario assertions."""
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];F=ROOT/'維護工具'
def replace(s,a,b):assert s.count(a)==1,a;return s.replace(a,b)
case=(F/'qualify-native-lighting-teleport-current-case.py').read_text(encoding='utf-8')
case=case.replace('native_lighting_GUI_evidence','native_object_gate_GUI_evidence')
cohort=(F/'qualify-native-lighting-teleport-current-cohort.py').read_text(encoding='utf-8')
cohort=cohort.replace('native_lighting_GUI_evidence','native_object_gate_GUI_evidence')
cohort=replace(cohort,'len(pins) in [994,999,1002,1005]','len(pins)==1019')
cohort=cohort.replace('run-native-lighting-current-teleport-GUI.ps1','run-native-object-gate-current-teleport-GUI.ps1')
cohort=cohort.replace('qualify-native-lighting-teleport-current-case.py','qualify-native-object-gate-teleport-current-case.py')
cohort=cohort.replace('qualified_current_lighting_normal_mains14_GUI_teleport_and_recovery_cases','qualified_current_object_gate_normal_mains14_GUI_teleport_and_recovery_cases')
ps=(F/'run-native-lighting-current-teleport-GUI.ps1').read_text(encoding='utf-8-sig')
start=ps.index('$floorPort=');end=ps.index('if($main.features.Count',start)
ps=ps[:start]+'''$expectedSourceCount=1019;$expectedPass=135;$expectedTests=23;$expectedIgnored=6
if($manifest.status -ne 'qualified_unique_entry_object_gate_normal_builds' -or $rust.status -ne 'qualified_object_gate_Debug135_Release135_and_normal_mains' -or $rust.counts.$BuildProfile.actual_test_executables -ne 23 -or $rust.integration_targets -ne 22 -or $rust.runtime_groups_per_profile -ne 24 -or ($rust.selected_library_groups -join ',') -ne 'world_raster::tests::,world_scene::tests::' -or $rust.ignored_test_names.$BuildProfile.Count -ne 6 -or $rust.floor_gate_only -or !$rust.supported_first_pass_object_gate_integrated -or $rust.map303_overlay_and_background_complete){throw 'Exact source1019 object-gate135 and selected-lib scope required'}
''' + ps[end:]
ps=ps.replace('qualify-native-lighting-teleport-current-case.py','qualify-native-object-gate-teleport-current-case.py')
for name,s in [('qualify-native-object-gate-teleport-current-case.py',case),('qualify-native-object-gate-teleport-current-cohort.py',cohort),('run-native-object-gate-current-teleport-GUI.ps1',ps)]:
    if name.endswith('.py'):compile(s,name,'exec')
    p=F/name;assert not p.exists();p.write_text(s,encoding='utf-8')
print('Prepared source1019 GUI version bindings; no GUI or server started')
