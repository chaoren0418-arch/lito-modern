"""Independently read actual source1012 Debug geometry/background/floor test products."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,re,sys,zipfile

ROOT=Path(__file__).resolve().parents[1]
APP=ROOT/'驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
OUT=Path(sys.argv[1]).resolve(strict=True);assert OUT.is_relative_to(APP)
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
digest=lambda b:hashlib.sha256(b).hexdigest().upper()
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def pe(b):
    e=int.from_bytes(b[60:64],'little');assert b[:2]==b'MZ' and b[e:e+4]==b'PE\0\0' and b[e+4:e+6]==b'\x64\x86' and b[e+24:e+26]==b'\x0b\x02'
r=read(OUT/'receipt.json');assert r['status']=='Debug_background_geometry_pixel_and_floor_tests_passed' and r['profile']=='Debug'
assert r['owned_process'] is None and r['compile_exit_code']==0 and r['source_inputs_unchanged']
assert not r['normal_build_executed'] and not r['world_background_renderer_integrated']
pins=read(OUT/'compiled-inputs.json');assert len(pins)==r['compiled_inputs']==1012
for n,h in pins.items():assert sha(ROOT/n)==h,n
assert pins['LiTo381-Modern/tests/fixtures/native-background-pixels-original.json']=='1E4F702BA605DBEF9C31ECA4B3EAA135208FB680245BAD8021179069D58E995B'
assert pins['LiTo381-Modern/tests/fixtures/native-background-channels-original.bin']=='03D786BB84587F2D06892E90AAFB8428018F2EB3DEA69A877157E7B1E7942FFB'
assert pins['LiTo381-Modern/tests/fixtures/native-background-geometry-original.json']=='F4B9F257B740B197CCE25F9BA2369FF5F13A1F16355BBF27E5F8F4E35BE36091'
with zipfile.ZipFile(OUT/'compiled-source.zip') as z:
    assert set(z.namelist())==set(pins)
    for n,h in pins.items():assert digest(z.read(n))==h,n
products=read(OUT/'products.json');assert set(products)=={'native_background_geometry','native_background_pixels','native_terrain_pixels'}
messages=[json.loads(l) for l in (OUT/'build/stdout.log').read_text(encoding='utf-8-sig').splitlines() if l.startswith('{')]
actual={v['target']['name']:v for v in messages if v.get('reason')=='compiler-artifact' and v.get('executable') and v['profile']['test']}
assert set(actual)==set(products)
with zipfile.ZipFile(OUT/'test-products.zip') as z:
    assert set(z.namelist())=={n+'.exe' for n in products}
    for n,p in products.items():
        b=z.read(n+'.exe');assert digest(b)==sha(p['path'])==p['sha256'];pe(b)
        assert Path(actual[n]['executable']).resolve()==Path(p['path']).resolve() and actual[n]['features']==[]
incidental=read(OUT/'incidental-bin-products.json')
with zipfile.ZipFile(OUT/'incidental-bin-products.zip') as z:
    assert set(z.namelist())=={n+'.exe' for n in incidental}
    for n,p in incidental.items():assert digest(z.read(n+'.exe'))==p['sha256'] and not p['normal_main_qualified']
expected_names={'native_background_geometry':{'all_2688_original_cases_and_17472_ordered_calls_match_rust','signed_negative_odd_phase_and_conditional_522_bottom_are_preserved','malformed_coordinates_and_light_domain_return_errors_without_panics'},'native_background_pixels':{
    'all_original_135168_packed_channel_words_match_rust','all_1584_original_whole_frame_hashes_and_clipped_rectangles_match_rust',
    'invalid_images_layouts_phases_or_strength_fail_before_destination_changes','encoded_black_writes_and_every_nonzero_gate_value_preserves_the_old_word'},
    'native_terrain_pixels':{'original_floor_dispatch_64_whole_surfaces_match_real_tile_decoder_and_gate',
    'clipped_gate_follows_actual_canvas_black_coverage_and_extreme_positions','invalid_floor_or_gate_inputs_fail_before_mutation'}}
metadata={};passed=0
for n,p in products.items():
    log=(OUT/n/'stdout.log').read_text(encoding='utf-8-sig')
    names=re.findall(r'^test (\S+) \.\.\. ok$',log,re.M);assert len(names)==len(expected_names[n]) and set(names)==expected_names[n]
    expected=len(expected_names[n]);assert re.findall(r'test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',log)==[('ok',str(expected),'0','0','0','0')]
    assert r['results'][n]=={'exit_code':0,'passed':expected,'failed':0,'ignored':0,'filtered':0,'product_sha256':p['sha256']}
    invocation=next(v for v in r['process_history'] if v['arguments']==[p['path'],'--test-threads=1']);assert invocation['exit_code']==0
    passed+=expected
    for name in ['stdout.log','stderr.log']:metadata[n+'/'+name]=OUT/n/name
assert passed==r['passed']==10 and r['failed']==r['ignored']==0
for n in ['receipt.json','products.json','compiled-inputs.json','runner.py','incidental-bin-products.json']:metadata[n]=OUT/n
for n in ['stdout.log','stderr.log']:metadata['build/'+n]=OUT/'build'/n
metadata['independent-qualifier.py']=Path(__file__)
hashes={n:sha(p) for n,p in metadata.items()}
with zipfile.ZipFile(OUT/'qualification-raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n,p in metadata.items():z.write(p,n)
with zipfile.ZipFile(OUT/'qualification-raw.zip') as z:
    assert set(z.namelist())==set(metadata)
    for n,h in hashes.items():assert digest(z.read(n))==h
q={'status':'independently_verified_source1012_Debug_background_geometry_pixels_and_floor_tests',
 'observed_utc':datetime.now(timezone.utc).isoformat(),'run':str(OUT),'source_files':1012,'source_manifest_sha256':sha(OUT/'compiled-inputs.json'),
 'actual_test_executables':3,'passed':10,'failed':0,'ignored':0,'background_channel_words':135168,'background_full_frame_cases':1584,
 'original_geometry_cases':2688,'original_geometry_calls':17472,'original_floor_dispatch_cases':64,'background_renderer_integrated':False,'map303_overlay_gate_complete':False,
 'normal_mains_rebuilt':False,'current_normal_GUI_verified':False,'Release_tested':False,
 'full_default_library_TCP_regression_verified':False,'source_archive_sha256':sha(OUT/'compiled-source.zip'),
 'test_products_archive_sha256':sha(OUT/'test-products.zip'),'raw_metadata_archive_sha256':sha(OUT/'qualification-raw.zip'),'raw_metadata_files':hashes,
 'all_archives_readback_verified':True,'whole_goal_complete':False}
dest=OUT/'qualified.json';assert not dest.exists();dest.write_text(json.dumps(q,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');assert read(dest)==q
(APP/'background-geometry-product-Debug-qualified-current.json').write_text(json.dumps(q,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({k:q[k] for k in ['status','source_files','actual_test_executables','passed','failed','ignored']},ensure_ascii=False))
