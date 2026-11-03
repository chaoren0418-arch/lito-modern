"""Prepare current dispatcher/terminal observers, preserving every runtime assertion."""
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];F=ROOT/'維護工具'
ps=(F/'run-native-lighting-TW14-followup.ps1').read_text(encoding='utf-8-sig')
ps=ps.replace("param([string]$NormalQualification='',[string]$RustEvidence='')",
    'param([Parameter(Mandatory)][string]$NormalQualification,[Parameter(Mandatory)][string]$RustEvidence)')
ps=ps.replace('native_lighting_GUI_evidence.py','native_object_gate_GUI_evidence.py')
ps=ps.replace('run-native-lighting-current-teleport-GUI.ps1','run-native-object-gate-current-teleport-GUI.ps1')
ps=ps.replace('qualify-native-lighting-teleport-current-cohort.py','qualify-native-object-gate-teleport-current-cohort.py')
ps=ps.replace('qualified_current_lighting_normal_mains14_GUI_teleport_and_recovery_cases','qualified_current_object_gate_normal_mains14_GUI_teleport_and_recovery_cases')
ps=ps.replace('running_current_lighting_TW14_GUI','running_current_object_gate_TW14_GUI')
ps=ps.replace('qualified_current_lighting_TW14_GUI_cases','qualified_current_object_gate_TW14_GUI_cases')
ps=ps.replace('-TW-lighting','-TW-object-gate')
observer=(F/'observe-native-floor-GUI-terminal.py').read_text(encoding='utf-8')
observer=observer.replace('qualified_unique_entry_floor_port_normal_builds','qualified_unique_entry_object_gate_normal_builds')
observer=observer.replace('qualified_current_lighting_TW14_GUI_cases','qualified_current_object_gate_TW14_GUI_cases')
observer=observer.replace('==1005','==1019').replace("'source_files':1005","'source_files':1019")
q=(F/'qualify-native-floor-port-GUI-completion.py').read_text(encoding='utf-8')
q=q.replace('current1005','current1019').replace('==1005','==1019').replace("'source_files':1005","'source_files':1019")
q=q.replace('qualified_current_lighting_TW14_GUI_cases','qualified_current_object_gate_TW14_GUI_cases')
q=q.replace('qualified_unique_entry_floor_port_normal_builds','qualified_unique_entry_object_gate_normal_builds')
q=q.replace('qualified_floor_port_Debug120_Release120_and_normal_mains','qualified_object_gate_Debug135_Release135_and_normal_mains')
q=q.replace('qualified_current_lighting_normal_mains14_GUI_teleport_and_recovery_cases','qualified_current_object_gate_normal_mains14_GUI_teleport_and_recovery_cases')
q=q.replace('dict(passed=120,failed=0,ignored=6,actual_test_executables=19)','dict(passed=135,failed=0,ignored=6,actual_test_executables=23)')
for n,s in [('run-native-object-gate-TW14.ps1',ps),('observe-native-object-gate-GUI-terminal.py',observer),('qualify-native-object-gate-GUI-completion.py',q)]:
    if n.endswith('.py'):compile(s,n,'exec')
    p=F/n;assert not p.exists();p.write_text(s,encoding='utf-8')
print('Prepared source1019 TW14 dispatcher and terminal/independent readback observers; no GUI started')
