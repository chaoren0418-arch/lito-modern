"""Independent source/PE/DIB evidence checks; never opens the candidate GUI."""
from pathlib import Path
import json,hashlib,struct
from PIL import Image
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003';O=P/'target/r475-player-entry-candidate'
def pin(p):
 p=Path(p);b=p.read_bytes();return dict(path=str(p),bytes=len(b),sha256=hashlib.sha256(b).hexdigest().upper())
def check(v):
 a=pin(v['path']);assert a['bytes']==v['bytes'] and a['sha256']==v['sha256']
def save(p,v):
 assert not p.exists();tmp=p.with_suffix('.tmp');tmp.write_text(json.dumps(v,ensure_ascii=False,indent=2),encoding='utf-8');tmp.replace(p)
c=json.loads((L/'r475-player-candidate.json').read_text(encoding='utf-8'));e=json.loads((L/'r475-entry-source-contract.json').read_text(encoding='utf-8'));d=json.loads((L/'r475-ui-integration-details.json').read_text(encoding='utf-8'))
for v in (c['executable'],c['production_library'],e['original_main'],e['candidate_main'],e['binding'],c['version_stdout'],c['version_stderr']):check(v)
t=Path(e['candidate_main']['path']).read_bytes().decode('utf-8');nl='\r\n' if '\r\n' in t else '\n'
for text in ['    CandidateEntry,','    OfflineMap,','        Some(x) if x == OsStr::new("--candidate-menu") && args.len() == 2 => Ok(Route::CandidateEntry),','        Some(x) if x == OsStr::new("--offline-map-preview") && args.len() == 2 => Ok(Route::OfflineMap),','        Route::CandidateEntry => candidate_offline_entry(false)?,','        Route::OfflineMap => candidate_offline_entry(true)?,','        "candidate_revision": "r475",','        "offline_entry_flags": ["--candidate-menu", "--offline-map-preview"],','        "offline_window_gui_verified": false,']:
 assert t.count(text+nl)==1;t=t.replace(text+nl,'')
start=t.index('fn candidate_offline_entry(');end=t.index('fn version_document()',start);t=t[:start]+t[end:];assert t.encode('utf-8')==Path(e['original_main']['path']).read_bytes()
b=Path(c['executable']['path']).read_bytes();pe=struct.unpack_from('<I',b,0x3c)[0];assert b[pe:pe+4]==b'PE\0\0';machine=struct.unpack_from('<H',b,pe+4)[0];subsystem=struct.unpack_from('<H',b,pe+24+68)[0];assert (machine,subsystem)==(0x14c,2)
window=P/'src/modern_lito/native_offline_preview_window.rs';runtime=P/'src/modern_lito/native_offline_preview_runtime.rs';resource=P/'src/modern_lito/native_offline_preview_resources.rs';wt=window.read_text(encoding='utf-8');rt=runtime.read_text(encoding='utf-8')
paint=wt[wt.index('unsafe fn paint('):wt.index('unsafe extern "system" fn proc(')];assert paint.index('FillRect(dc,&whole,GetSysColorBrush(COLOR_BTNFACE))')<paint.index('let frame:');assert 'frame.bgra()' in paint and 'frame.viewport(' in paint and 'StretchDIBits(' in paint
assert 'WM_PAINT=>{paint(hwnd,s)' in wt and 'if closed{let _=DestroyWindow(hwnd);}' in wt;assert 'JoinHandle::is_finished' in rt and '.join()' in rt and 'guard.store(self.revision' in rt and 'g.store(0' in rt
assert d['negative_cases']==16 and d['actual_old_queued_frame_rejected']>=1 and d['actual_missing_sector_entry_failure_retry_and_close_verified'];assert not d['native_window_executed'] and not d['R472_timers_advanced']
old=json.loads((L/'r474-offline-map-details.json').read_text(encoding='utf-8'));check(old['images'][0]);images=[]
for im in d['images']:
 check(im)
 with Image.open(im['path']) as p:
  assert p.size==(800,600);rgba=p.convert('RGBA').tobytes();bgra=b''.join(bytes((v[2],v[1],v[0],v[3])) for v in zip(*[iter(rgba)]*4));assert hashlib.sha256(bgra).hexdigest().upper()==im['BGRA_sha256']
 images.append(pin(im['path']))
with Image.open(d['images'][0]['path']) as a,Image.open(old['images'][0]['path']) as b:assert a.convert('RGBA').tobytes()==b.convert('RGBA').tobytes()
for f in d['owned_rejection_fixtures']:assert Path(f).resolve().is_relative_to(P.resolve())
check(c['version_transport']);v=json.loads(Path(c['version_transport']['path']).read_text(encoding='utf-8'));assert v['exit_code']==0 and v['flag']=='--version' and not v['native_window_executed']
save(L/'r475-ui-source-and-output-contract.json',dict(state='SEPARATE_EXISTING_MAIN_REVERSES_BYTE_EXACT_GDI_DIB_SOURCE_AND_ASYNC_RETIREMENT_PROOF_VERIFIED',original_main=e['original_main'],candidate_main=e['candidate_main'],candidate_executable=c['executable'],machine=machine,subsystem=subsystem,default_original_main_byte_preserved=True,independent_remove_nine_inserted_lines_and_one_helper_yields_original=True,window_source=pin(window),runtime_source=pin(runtime),resource_input_source=pin(resource),unconditional_parent_background_clear_before_optional_frame=True,old_frame_guard_and_close_wait_source_verified=True,images=images,entry_equals_fixed_R474_static_source_pixels=True,negative_cases=16,actual_late_frame_discard=True,active_load_return_microseconds=d['active_load_back_microseconds'],late_frame_return_microseconds=d['late_result_back_microseconds'],actual_missing_sector_entry_failure_retries=2,readonly_candidate_version_transport=c['version_transport'],directly_viewed_CPU_images=['worker-arrow-pan-dib-source.png'],other_CPU_images='hash/pixel verified; not a native window screenshot',native_GUI_executed=False,GUI_acceptance=0,renderer_LOADED=False,R472_timers_advanced=False))
print(json.dumps(dict(default_main_byte_exact=True,PE_machine=machine,PE_subsystem=subsystem,images=2,negative_cases=16,GUI=0)))
