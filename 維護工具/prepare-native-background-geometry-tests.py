"""Prepare version-bound runners without altering historical qualification tools."""
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
folder=ROOT/'維護工具'
def replace(text,old,new):
    assert text.count(old)==1,old
    return text.replace(old,new)
r=(folder/'run-native-background-pixel-tests.py').read_text(encoding='utf-8')
r=replace(r,'Compile source1009 background pixels and floor regression','Compile source1012 background geometry, pixels and floor regression')
r=replace(r,"pins=read(NORMAL/'source-inputs.json');assert len(pins)==1005", "pins=read(APP/'background-pixel-tests-20261016-115714-009940/compiled-inputs.json');assert len(pins)==1009")
r=replace(r,"['src/modern_lito/native_background_pixels.rs','tests/native_background_pixels.rs','tests/fixtures/native-background-pixels-original.json','tests/fixtures/native-background-channels-original.bin']", "['src/modern_lito/native_background_geometry.rs','tests/native_background_geometry.rs','tests/fixtures/native-background-geometry-original.json']")
r=replace(r,"assert len(pins)==1009\ndef stable", "assert len(pins)==1012\ndef stable")
r=r.replace('background-pixel-tests-', 'background-geometry-tests-')
r=r.replace('background-geometry-tests-20261016-115714-009940','background-pixel-tests-20261016-115714-009940')
r=r.replace('background_pixel_and_floor','background_geometry_pixel_and_floor')
r=replace(r,"'compiled_inputs':1009,'suites':['native_background_pixels','native_terrain_pixels']", "'compiled_inputs':1012,'suites':['native_background_geometry','native_background_pixels','native_terrain_pixels']")
q=(folder/'qualify-native-background-pixel-tests.py').read_text(encoding='utf-8')
q=q.replace('source1009','source1012').replace('background/floor','geometry/background/floor').replace('background_pixel_and_floor','background_geometry_pixel_and_floor')
q=q.replace("==1009","==1012").replace("'source_files':1009","'source_files':1012")
q=replace(q,"{'native_background_pixels','native_terrain_pixels'}", "{'native_background_geometry','native_background_pixels','native_terrain_pixels'}")
q=replace(q,"expected_names={'native_background_pixels':{", "expected_names={'native_background_geometry':{'all_2688_original_cases_and_17472_ordered_calls_match_rust','signed_negative_odd_phase_and_conditional_522_bottom_are_preserved','malformed_coordinates_and_light_domain_return_errors_without_panics'},'native_background_pixels':{")
q=replace(q,"==7 and", "==10 and")
q=replace(q,"'actual_test_executables':2,'passed':7", "'actual_test_executables':3,'passed':10")
q=q.replace('Debug_background_pixels_and_floor_tests','Debug_background_geometry_pixels_and_floor_tests')
q=replace(q,"'original_floor_dispatch_cases':64", "'original_geometry_cases':2688,'original_geometry_calls':17472,'original_floor_dispatch_cases':64")
q=q.replace('background-pixel-Debug-qualified-current.json','background-geometry-product-Debug-qualified-current.json')
q=replace(q,"with zipfile.ZipFile(OUT/'compiled-source.zip') as z:", "assert pins['LiTo381-Modern/tests/fixtures/native-background-geometry-original.json']=='F4B9F257B740B197CCE25F9BA2369FF5F13A1F16355BBF27E5F8F4E35BE36091'\nwith zipfile.ZipFile(OUT/'compiled-source.zip') as z:")
for name,text in [('run-native-background-geometry-tests.py',r),('qualify-native-background-geometry-tests.py',q)]:
    compile(text,name,'exec')
    p=folder/name;assert not p.exists();p.write_text(text,encoding='utf-8')
print('Prepared source1012 geometry/pixel/floor Debug runner and independent qualifier')
