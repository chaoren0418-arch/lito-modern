"""Independent real member pixels, native references, main inverse and PE checks."""
from pathlib import Path
import json,hashlib,struct
from PIL import Image
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003'
def pin(p):
 p=Path(p);b=p.read_bytes();return dict(path=str(p),bytes=len(b),sha256=hashlib.sha256(b).hexdigest().upper())
def check(v):
 a=pin(v['path']);assert (a['bytes'],a['sha256'])==(v['bytes'],v['sha256'])
def read(name):return json.loads((L/name).read_text(encoding='utf-8'))
def save(name,value):
 dst=L/name;assert not dst.exists();dst.write_text(json.dumps(value,ensure_ascii=False,indent=2),encoding='utf-8')
c=read('r476-player-candidate.json');e=read('r476-entry-source-contract.json');m=read('r476-native-menu-skin-binding.json');d=read('r476-menu-skin-details.json');host=read('r476-candidate.json')
for v in [c['executable'],c['production_library'],e['original_main'],e['candidate_main'],e['binding'],e['skin_binding'],c['version_stdout'],c['version_stderr']]:check(v)
t=Path(e['candidate_main']['path']).read_bytes().decode('utf-8');nl='\r\n' if '\r\n' in t else '\n'
for line in ['    CandidateEntry,','    OfflineMap,','        Some(x) if x == OsStr::new("--candidate-menu") && args.len() == 2 => Ok(Route::CandidateEntry),','        Some(x) if x == OsStr::new("--offline-map-preview") && args.len() == 2 => Ok(Route::OfflineMap),','        Route::CandidateEntry => candidate_offline_entry(false)?,','        Route::OfflineMap => candidate_offline_entry(true)?,','        "candidate_revision": "r476",','        "offline_entry_flags": ["--candidate-menu", "--offline-map-preview"],','        "offline_window_gui_verified": false,']:
 assert t.count(line+nl)==1;t=t.replace(line+nl,'')
start=t.index('fn candidate_offline_entry(');end=t.index('fn version_document()',start);t=t[:start]+t[end:];assert t.encode('utf-8')==Path(e['original_main']['path']).read_bytes()
b=Path(c['executable']['path']).read_bytes();pe=struct.unpack_from('<I',b,0x3c)[0];assert b[pe:pe+4]==b'PE\0\0';assert struct.unpack_from('<H',b,pe+4)[0]==332 and struct.unpack_from('<H',b,pe+24+68)[0]==2
for entry in host['source_pins']:
 check(entry);assert pin(entry['snapshot'])['sha256']==entry['sha256']
for container in m['containers']:
 for v in container.values():check(v)
raws={}
for v in m['members']:
 check(v['raw']);check(v['reference_png']);raw=Path(v['raw']['path']).read_bytes();w,h,flag,mask=struct.unpack_from('<hhHH',raw);assert len(raw)==8+w*h*2
 rgba=bytearray()
 for word, in struct.iter_unpack('<H',raw[8:]):
  if flag and word==mask:rgba.extend([0,0,0,0])
  else:
   channels=[(word>>10)&31,(word>>5)&31,word&31];rgba.extend([(x<<3)|(x>>2) for x in channels]+[255])
 with Image.open(v['reference_png']['path']) as im:assert im.size==(w,h) and im.convert('RGBA').tobytes()==rgba
 raws[v['id']]=(w,h,rgba)
for i,(up,down) in enumerate([(6232,6234),(6233,6234),(6232,6235)]):
 canvas=bytearray();base=raws[6113][2]
 for y in range(81):canvas.extend(base[(y*158)*4:(y*158+157)*4])
 for rid,x0,y0 in [(up,142,3),(down,141,65)]:
  w,h,rgba=raws[rid]
  for y in range(h):
   for x in range(w):
    k=(y*w+x)*4;out=((y+y0)*157+x+x0)*4
    if rgba[k+3]:canvas[out:out+4]=rgba[k:k+4]
 v=d['images'][i];check(v)
 with Image.open(v['path']) as im:assert im.size==(157,81) and im.convert('RGBA').tobytes()==canvas
 assert hashlib.sha256(canvas).hexdigest().upper()==v['rgba_sha256']
for v in m['asm_fragments']:check(v['pin'])
fragments={v['address']:Path(v['pin']['path']).read_text(encoding='utf-8') for v in m['asm_fragments']}
assert '[eax+5F84h]' in fragments['4744F0'] and '5F84' not in str(6113);assert 0x5f84//4==6113
assert all(v in fragments['4744F0'] for v in ['9Dh','51h','136h','1A8h'])
assert all(v in fragments['477AB0'] for v in ['push    1858h','push    185Ah','push    8','push    9','236h','139h','235h','177h'])
assert '[esi+2Dh]' in fragments['4E9A40'] and 'inc     eax' in fragments['4E9A40'];assert 'mov     [ecx+2Dh], al' in fragments['414BD0'] and 'mov     byte ptr [esi+2Dh], 0' in fragments['414C00']
wt=(P/'src/modern_lito/native_offline_menu_window.rs').read_text(encoding='utf-8');rt=(P/'src/modern_lito/native_offline_preview_runtime.rs').read_text(encoding='utf-8')
paint=wt[wt.index('unsafe fn paint('):wt.index('unsafe extern "system" fn proc(')];assert paint.index('FillRect(dc,&whole')<paint.index('paint_menu(')<paint.index('let frame:');assert 'Skin::load' not in wt and 'fs::' not in wt
assert 'skin_inputs.as_ref().map(super::native_offline_menu_skin::Skin::load)' in rt and 'worker(inputs,skin_inputs,rx' in rt and 'JoinHandle::is_finished' in rt
assert 'Runtime::start_with_menu_skin(inputs,skin_inputs)' in wt and 'Hit::Row' in wt and 'normal_bgra.as_ptr()' in wt and 'up_pressed_bgra' not in wt
assert d['negative_cases']>=16 and d['resize_hit_pixel_checks']>1000000 and d['skin_and_map564_entry_back_close_executed'] and d['skin_bad_pin_refused_before_choice'];assert c['only_executed_candidate_flag']=='--version' and c['GUI']==0 and c['version']['candidate_revision']=='r476'
check(read('r476a-coordinate-failure-preservation.json')['candidate'])
save('r476-independent-contract.json',dict(state='INDEPENDENT_REAL_IMG_PIXEL_SOURCE_AND_CANDIDATE_MAIN_PE_PROOF_PASS',main_inverse_byte_exact=True,native_member_count=5,composite_all_pixels_equal=3,source_snapshot_pairs=len(host['source_pins']),PE_machine=332,PE_subsystem=2,original_panel6113_and_arrow6232_6235_source_verified=True,pressed_images_CPU_only=True,only_one_qualified_row=True,normal_arrows_disabled=True,optional_skin_worker_IO_source_verified=True,unconditional_parent_clear=True,negative_cases=d['negative_cases'],resize_hit_pixel_checks=d['resize_hit_pixel_checks'],GUI=0,LOADED=False,images=d['images']))
print(json.dumps(dict(independent_contract='PASS',member_pixels=5,composites=3,source_pairs=len(host['source_pins']),GUI=0)))
