"""Independent final candidate/source and exact scene pixel provenance checks."""
from pathlib import Path
from PIL import Image
import json,hashlib,struct
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003'
def pin(p):
 p=Path(p);b=p.read_bytes();return dict(path=str(p),bytes=len(b),sha256=hashlib.sha256(b).hexdigest().upper())
def check(v):
 a=pin(v['path']);assert (a['bytes'],a['sha256'])==(v['bytes'],v['sha256'])
def read(n):return json.loads((L/n).read_text(encoding='utf-8'))
c=read('r477-player-candidate.json');e=read('r477-entry-source-contract.json');d=read('r477-multi-map-details.json');m=read('r477-multi-map-binding.json');h=read('r477-candidate.json')
for v in [c['executable'],c['production_library'],e['original_main'],e['candidate_main'],e['binding'],e['skin_binding'],c['version_stdout'],c['version_stderr']]:check(v)
t=Path(e['candidate_main']['path']).read_bytes().decode('utf-8');nl='\r\n' if '\r\n' in t else '\n'
for line in ['    CandidateEntry,','    OfflineMap,','        Some(x) if x == OsStr::new("--candidate-menu") && args.len() == 2 => Ok(Route::CandidateEntry),','        Some(x) if x == OsStr::new("--offline-map-preview") && args.len() == 2 => Ok(Route::OfflineMap),','        Route::CandidateEntry => candidate_offline_entry(false)?,','        Route::OfflineMap => candidate_offline_entry(true)?,','        "candidate_revision": "r477",','        "offline_entry_flags": ["--candidate-menu", "--offline-map-preview"],','        "offline_window_gui_verified": false,']:
 assert t.count(line+nl)==1;t=t.replace(line+nl,'')
start=t.index('fn candidate_offline_entry(');end=t.index('fn version_document()',start);t=t[:start]+t[end:];assert t.encode('utf-8')==Path(e['original_main']['path']).read_bytes()
exe=Path(c['executable']['path']).read_bytes();pe=struct.unpack_from('<I',exe,0x3c)[0];assert exe[pe:pe+4]==b'PE\0\0';assert struct.unpack_from('<H',exe,pe+4)[0]==332 and struct.unpack_from('<H',exe,pe+24+68)[0]==2
assert len(h['source_pins'])==126
for v in h['source_pins']:check(v);assert pin(v['snapshot'])['sha256']==v['sha256']
original=read('r474-mode-map-binding.json');assert m['catalog40']==[v['original_table'] for v in original['rows']];assert [(v['mode'],v['map_id']) for v in m['bindings']]==[(1,564),(1,514)]
for b in m['bindings']:
 assert b['namespace']=='NATIVE_LITO_LTMAP_V1' and len(b['sectors'])==1 and len(b['archives'])==4
 for s in b['sectors']:check(s['pin']);assert s['map_id']==b['map_id'] and Path(s['pin']['path'])==Path(b['map_root'])/str(b['map_id'])/f"{s['x']:04x}{s['y']:04x}.s32"
 for a in b['archives']:check(a)
check(m['additional_sector_origin']);check(m['qualification_probe']);check(m['probe_reference'])
refs=[Path(m['probe_reference']['path']),P/'target/r474-offline-map-flow/01-entry-battle-ground.png']
for v,ref,id in zip(d['images'],refs,[(1,514),(1,564)]):
 check(v);assert tuple(v['identity'])==id
 with Image.open(v['path']) as a,Image.open(ref) as b:
  rgba=a.convert('RGBA').tobytes();assert a.size==b.size==(800,600) and rgba==b.convert('RGBA').tobytes();assert hashlib.sha256(rgba).hexdigest().upper()==v['rgba_sha256']
assert d['images'][0]['rgba_sha256']!=d['images'][1]['rgba_sha256']
for v in read('r477-menu-source-contract.json')['fragments']:check(v)
wt=(P/'src/modern_lito/native_multi_map_window.rs').read_text(encoding='utf-8');rt=(P/'src/modern_lito/native_multi_map_runtime.rs').read_text(encoding='utf-8');mt=(P/'src/modern_lito/native_multi_map_menu.rs').read_text(encoding='utf-8')
assert 'f.select(1,564)' not in rt and 'pixels.mode' in rt and 'c.map_id!=pixels.map_id' in rt
assert 'Command::Enter{mode:choice.mode,map_id:choice.map_id' in rt and 'flow.select(mode,map_id)' in rt
assert 'Action::Select(mode,map_id)' in rt and 'self.invalidate();self.selected=Some(i)' in rt
assert 'row.available' in rt and '資源未合格，不能預覽' in rt
assert 'Menu::font_height(&vp)' in wt and 'Menu::row_rect(&vp,visible)' in wt and 'DrawTextW(dc,&mut text,&mut rect' in wt and 'DT_END_ELLIPSIS|DT_NOPREFIX' in wt
assert 'msg.wParam.0==0x0d&&msg.hwnd==hwnd' in wt and 'SelectObject(dc,old)' in wt and 'DeleteObject(HGDIOBJ(font.0))' in wt
assert 'WM_LBUTTONDOWN' in wt and 'WM_LBUTTONUP' in wt and 'WM_MOUSEMOVE' in wt and 'WM_CAPTURECHANGED|WM_KILLFOCUS' in wt
assert 'self.top+1<self.count' in mt and '+156)/157' in mt and '+80)/81' in mt
assert d['new_focused_tests']==1 and d['historical_PASS_reruns']==0 and d['negative_cases']>=16 and d['actual_old_map_queued_frame_and_old_pan_discarded']>=2 and d['actual_missing514_no_564_fallback']
assert c['only_executed_candidate_flag']=='--version' and c['GUI']==0 and c['version']['candidate_revision']=='r477'
for n in ['r477a-compile-failure-preservation.json','r477b-unexecuted-layout-refinement.json','r477c-unexecuted-focus-refinement.json']:
 v=read(n)
 for s in v.get('source_snapshots',v.get('sources',[])):check(s)
out=dict(state='INDEPENDENT_R477_EXACT_MODE_MAP_SCENE_PIXELS_AND_MAIN_SOURCE_PE_PROOF_PASS',main_inverse_byte_exact=True,PE_machine=332,PE_subsystem=2,catalog_records=40,qualified_identities=[[1,564],[1,514]],source_snapshot_pairs=126,new_scene_pixel_pairs=2,negative_cases=d['negative_cases'],row_text_clip_equals_hit_interval_ceil_projection=True,font_scaled_to_panel=True,focused_standard_button_Enter_not_intercepted=True,menu_native_top_bound_count_minus_one=True,held_old_map_and_pan_refused=True,missing514_no564_fallback=True,UI_assets='Native panel/arrow normal/press pixels; Modern scaled/clipped font, selection highlight and keyboard behavior. Original thumb6231 and repeat timer excluded.',GUI=0,LOADED=False,images=d['images'])
dst=L/'r477-independent-contract.json';assert not dst.exists();dst.write_text(json.dumps(out,ensure_ascii=False,indent=2),encoding='utf-8');print(json.dumps(dict(independent='PASS',scene_pixels=2,source_pairs=126,GUI=0)))
