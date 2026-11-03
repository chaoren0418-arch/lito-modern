"""Prepare source1015 bank/core runners for both unmodified build profiles."""
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];F=ROOT/'維護工具'
def replace(s,a,b):assert s.count(a)==1,a;return s.replace(a,b)
r=(F/'run-native-background-geometry-tests.py').read_text(encoding='utf-8')
r=replace(r,'Compile source1012 background geometry, pixels and floor regression','Compile source1015 real IMG bank and background/floor regression')
r=replace(r,'import hashlib,json,os,re,subprocess,zipfile','import hashlib,json,os,re,subprocess,sys,zipfile')
r=replace(r,"pins=read(APP/'background-pixel-tests-20261016-115714-009940/compiled-inputs.json');assert len(pins)==1009",
    "profile=sys.argv[1] if len(sys.argv)>1 else 'Debug';assert profile in ['Debug','Release']\npins=read(APP/'background-geometry-tests-20261016-122035-301179/compiled-inputs.json');assert len(pins)==1012")
r=replace(r,"pins['LiTo381-Modern/src/modern_lito/mod.rs']=sha(PROJECT/'src/modern_lito/mod.rs')",
    "pins['LiTo381-Modern/src/modern_lito/mod.rs']=sha(PROJECT/'src/modern_lito/mod.rs')\npins['LiTo381-Modern/src/modern_lito/native_background_pixels.rs']=sha(PROJECT/'src/modern_lito/native_background_pixels.rs')")
r=replace(r,"['src/modern_lito/native_background_geometry.rs','tests/native_background_geometry.rs','tests/fixtures/native-background-geometry-original.json']",
    "['src/modern_lito/native_background.rs','tests/native_background.rs','tests/fixtures/native-background-real-original.json']")
r=replace(r,'assert len(pins)==1012\ndef stable','assert len(pins)==1015\ndef stable')
r=replace(r,"out=APP/('background-geometry-tests-'+", "out=APP/('background-bank-'+profile+'-tests-'+")
r=replace(r,"'profile':'Debug'", "'profile':profile")
r=replace(r,"'compiled_inputs':1012,'suites':['native_background_geometry','native_background_pixels','native_terrain_pixels']",
    "'compiled_inputs':1015,'suites':['native_background','native_background_geometry','native_background_pixels','native_terrain_pixels']")
for prefix in ['compiling','executing']:
    r=replace(r,"'"+prefix+"_Debug_background_geometry_pixel_and_floor_tests'", "'"+prefix+"_'+profile+'_background_bank_and_core_tests'")
r=replace(r,"'Debug_background_geometry_pixel_and_floor_tests_passed'", "profile+'_background_bank_and_core_tests_passed'")
r=replace(r,"save(APP/'background-geometry-tests-current.json'", "save(APP/('background-bank-'+profile+'-tests-current.json')")
r=replace(r,"for n in receipt['suites']:cmd+=['--test',n]", "if profile=='Release':cmd.append('--release')\n    for n in receipt['suites']:cmd+=['--test',n]")
q=(F/'qualify-native-background-geometry-tests.py').read_text(encoding='utf-8')
q=q.replace('source1012','source1015')
q=replace(q,"assert r['status']=='Debug_background_geometry_pixel_and_floor_tests_passed' and r['profile']=='Debug'",
    "profile=r['profile'];assert profile in ['Debug','Release'] and r['status']==profile+'_background_bank_and_core_tests_passed'")
q=q.replace('==1012','==1015').replace("'source_files':1012","'source_files':1015")
q=replace(q,"{'native_background_geometry','native_background_pixels','native_terrain_pixels'}", "{'native_background','native_background_geometry','native_background_pixels','native_terrain_pixels'}")
q=replace(q,"expected_names={'native_background_geometry':", "expected_names={'native_background':{'actual_native_first_IMG_bank_matches_all_twelve_original_whole_frames','invalid_batch_layout_map_phase_or_cancel_never_changes_destination'},'native_background_geometry':")
q=replace(q,"==10 and", "==12 and")
q=replace(q,"'actual_test_executables':3,'passed':10", "'actual_test_executables':4,'passed':12")
q=replace(q,"'independently_verified_source1015_Debug_background_geometry_pixels_and_floor_tests'", "'independently_verified_source1015_'+profile+'_background_bank_and_core_tests'")
q=replace(q,"'observed_utc':", "'profile':profile,'observed_utc':")
q=replace(q,"'Release_tested':False", "'Release_tested':profile=='Release'")
q=replace(q,"'original_geometry_cases':2688", "'real_map303_IMG_frames':12,'native_first_IMG_loader_executed':True,'original_geometry_cases':2688")
q=replace(q,"(APP/'background-geometry-product-Debug-qualified-current.json')", "(APP/('background-bank-'+profile+'-qualified-current.json'))")
q=replace(q,"with zipfile.ZipFile(OUT/'compiled-source.zip') as z:", "assert pins['LiTo381-Modern/tests/fixtures/native-background-real-original.json']=='8CDC0BAC8A9E663964A6C81A0E343C871B1E3CCE71DD6827AA06DAF05037C797'\nwith zipfile.ZipFile(OUT/'compiled-source.zip') as z:")
for name,text in [('run-native-background-bank-tests.py',r),('qualify-native-background-bank-tests.py',q)]:
    compile(text,name,'exec');p=F/name;assert not p.exists();p.write_text(text,encoding='utf-8')
print('Prepared source1015 Debug/Release bank and core runners')
