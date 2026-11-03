"""Prepare current gate scopes and normal-main qualifier, retaining historical tools."""
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];F=ROOT/'維護工具'
def replace(s,a,b):assert s.count(a)==1,a;return s.replace(a,b)
r=(F/'run-native-floor-port-tests.py').read_text(encoding='utf-8')
start=r.index("BASE=APP/");end=r.index('\ndef check():',start)
r=r[:start]+'''BASE=APP/'background-bank-Debug-tests-20261016-123538-210580'
pins=read(BASE/'compiled-inputs.json');assert len(pins)==1015
changed=['mod.rs','world_raster.rs','native_world_layers.rs','native_world_scene.rs','native_terrain_pixels.rs','native_resources.rs','native_multisector_scene.rs']
for name in changed:pins['LiTo381-Modern/src/modern_lito/'+name]=sha(PROJECT/'src/modern_lito'/name)
for name in ['src/modern_lito/native_object_dispatch.rs','tests/native_object_dispatch.rs','tests/fixtures/native-object-dispatch-original.json','tests/fixtures/native-object-dispatch-outcomes.bin']:pins['LiTo381-Modern/'+name]=sha(PROJECT/name)
assert len(pins)==1019
''' + r[end:]
r=r.replace('Source1005 floor/native gate','Source1019 typed object gate')
r=r.replace("'floor-tests-'","'object-gate-tests-'").replace("'floor-tests-current.json'","'object-gate-tests-current.json'")
r=replace(r,"if '--include-relation-labels' in sys.argv[1:]", "suites+=['native_object_dispatch','native_background','native_background_geometry','native_background_pixels']\nif '--include-relation-labels' in sys.argv[1:]")
r=replace(r,"'results':{}, 'owned_process':None", "'results':{}, 'owned_process':None,'process_history':[]")
r=replace(r,"receipt['owned_process']={'pid':child.pid,'arguments':command,'observed_started_utc':now()}; persist()",
    "identity={'pid':child.pid,'arguments':command,'observed_started_utc':now()};receipt['owned_process']=identity;receipt['process_history'].append(identity);persist()")
r=replace(r,"code = child.wait()\n        receipt['owned_process']=None; persist()", "code = child.wait();identity.update(exit_code=code,finished_utc=now())\n        receipt['owned_process']=None; persist()")
r=replace(r,"check()\noutput =", '''check()
command="$r='"+str(ROOT).replace("'","''")+"';$p=@(Get-CimInstance Win32_Process|Where-Object {$_.Name -in @('cargo.exe','rustc.exe') -or ($_.Name -eq 'java.exe' -and $_.CommandLine -and $_.CommandLine.Contains($r)) -or ($_.ExecutablePath -and $_.ExecutablePath.StartsWith($r,[StringComparison]::OrdinalIgnoreCase) -and ($_.Name -match '^(LiToModern|native_|lito_modern-|oracle)' -or ($_.ExecutablePath.Split([IO.Path]::DirectorySeparatorChar) -contains 'deps')))});if($p.Count){exit 1}"
idle=subprocess.run(['powershell.exe','-NoProfile','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert idle.returncode==0,'Owned workload remains active'
output =''')
q=(F/'qualify-native-floor-port-tests.py').read_text(encoding='utf-8').replace('source1005','source1019')
q=q.replace('==1005','==1019').replace("'source_files':1005","'source_files':1019")
q=replace(q,"required={'native_light_color'", "required={'native_object_dispatch','native_background','native_background_geometry','native_background_pixels','native_light_color'")
q=replace(q,"['lib-world-raster']['passed']==7", "['lib-world-raster']['passed']==12")
q=replace(q,"for n in ['compiled-inputs.json'", "assert r['results']['native_object_dispatch']['passed']==1 and r['results']['native_background']['passed']==2 and r['results']['native_background_geometry']['passed']==3 and r['results']['native_background_pixels']['passed']==4\nassert counts=={'passed':135,'failed':0,'ignored':6}\nfor n in ['compiled-inputs.json'")
q=q.replace('independently_qualified_floor_port_profile_and_selected_lib_groups','independently_qualified_object_gate_profile_and_selected_lib_groups')
q=q.replace("'integration_targets':18","'integration_targets':22").replace("'actual_test_executables':19","'actual_test_executables':23").replace("'runtime_groups':20","'runtime_groups':24")
q=q.replace("'floor_gate_only':True", "'floor_gate_only':False,'supported_first_pass_object_gate_integrated':True,'full_object_dispatch_original_cases':263423")
n=(F/'qualify-native-floor-port-normal.py').read_text(encoding='utf-8').replace('source1005','source1019')
n=replace(n,"DEBUG=APP/'floor-tests-20261016-101417-462693'", "DEBUG=Path(sys.argv[2]).resolve(strict=True);assert DEBUG.is_relative_to(APP)")
n=n.replace('==1005','==1019').replace("'source_files':1005","'source_files':1019")
n=n.replace('independently_qualified_floor_port_profile_and_selected_lib_groups','independently_qualified_object_gate_profile_and_selected_lib_groups')
n=n.replace('(120,0,6,19,20)','(135,0,6,23,24)').replace("['integration_targets']==18","['integration_targets']==22").replace('len(products)==19','len(products)==23')
n=n.replace("'passed':120","'passed':135").replace("'actual_test_executables':19","'actual_test_executables':23")
n=n.replace('qualified_unique_entry_floor_port_normal_builds','qualified_unique_entry_object_gate_normal_builds')
n=n.replace('qualified_floor_port_Debug120_Release120_and_normal_mains','qualified_object_gate_Debug135_Release135_and_normal_mains')
n=n.replace("'integration_targets':18","'integration_targets':22").replace("'runtime_groups_per_profile':20","'runtime_groups_per_profile':24")
n=n.replace("'floor_gate_only':True","'floor_gate_only':False,'supported_first_pass_object_gate_integrated':True,'full_object_dispatch_original_cases':263423")
n=n.replace("'water_extra_effect_complete':True","'water_extra_effect_complete':False")
for name,s in [('run-native-object-gate-tests.py',r),('qualify-native-object-gate-tests.py',q),('qualify-native-object-gate-normal.py',n)]:
    compile(s,name,'exec');p=F/name;assert not p.exists();p.write_text(s,encoding='utf-8')
print('Prepared source1019: 22 integrations and two library groups; both normal mains qualify separately')
