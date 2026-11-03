"""Independent final pins, reversible main, native source boundaries and new-branch proof."""
from pathlib import Path
import json,hashlib,struct
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003'
def pin(p):
 p=Path(p);b=p.read_bytes();return dict(path=str(p),bytes=len(b),sha256=hashlib.sha256(b).hexdigest().upper())
def check(v):
 a=pin(v['path']);assert(a['bytes'],a['sha256'])==(v['bytes'],v['sha256'].upper()),v['path']
def read(n):return json.loads((L/n).read_text(encoding='utf-8-sig'))
entry=read('r478-entry-source-contract.json');player=read('r478-player-candidate.json');host=read('r478-candidate.json');details=read('r478-mode-menu-details.json')
for v in [entry['original_main'],entry['candidate_main'],entry['binding'],entry['skin_binding'],player['executable'],player['production_library'],player['version_stdout'],player['version_stderr']]:check(v)
t=Path(entry['candidate_main']['path']).read_bytes().decode('utf-8');nl='\r\n' if '\r\n' in t else '\n'
for line in ['    CandidateEntry,','    OfflineMap,','        Some(x) if x == OsStr::new("--candidate-menu") && args.len() == 2 => Ok(Route::CandidateEntry),','        Some(x) if x == OsStr::new("--offline-map-preview") && args.len() == 2 => Ok(Route::OfflineMap),','        Route::CandidateEntry => candidate_offline_entry(false)?,','        Route::OfflineMap => candidate_offline_entry(true)?,','        "candidate_revision": "r478",','        "offline_entry_flags": ["--candidate-menu", "--offline-map-preview"],','        "offline_window_gui_verified": false,']:
 assert t.count(line+nl)==1;t=t.replace(line+nl,'')
start=t.index('fn candidate_offline_entry(');end=t.index('fn version_document()',start);t=t[:start]+t[end:];assert t.encode('utf-8')==Path(entry['original_main']['path']).read_bytes()
b=Path(player['executable']['path']).read_bytes();pe=struct.unpack_from('<I',b,0x3c)[0];assert b[pe:pe+4]==b'PE\0\0';assert struct.unpack_from('<H',b,pe+4)[0]==332 and struct.unpack_from('<H',b,pe+24+68)[0]==2
assert len(host['source_pins'])==132
for v in host['source_pins']:check(v);assert pin(v['snapshot'])['sha256']==v['sha256']
skin=read('r478-native-menu-skin-binding.json');assert [m['id'] for m in skin['members']]==[6113,6232,6233,6234,6235,6231]
for row in skin['members']:
 check(row['raw']);raw=Path(row['raw']['path']).read_bytes();assert hashlib.sha256(raw).hexdigest().upper()==row['sha256'];check(row['reference_png'])
assert skin['members'][-1]['mask_flag']==0 and skin['members'][-1]['width']==7 and skin['members'][-1]['height']==10
check(skin['header_table']);lines=Path(skin['header_table']['path']).read_bytes().splitlines();assert int(lines[0])==296 and lines[109]==b'Random' and lines[270]==b'Hot Random'
fr=read('r478-source-fragments.json');check(fr['original_disassembly']);original=Path(fr['original_disassembly']['path']).read_text(encoding='utf-8')
for v in fr['fragments']:check(v);assert Path(v['path']).read_text(encoding='utf-8') in original
source=(P/'src/modern_lito/native_mode_menu.rs').read_text(encoding='utf-8');runtime=(P/'src/modern_lito/native_mode_menu_runtime.rs').read_text(encoding='utf-8');window=(P/'src/modern_lito/native_mode_menu_window.rs').read_text(encoding='utf-8');skin_source=(P/'src/modern_lito/native_mode_menu_skin.rs').read_text(encoding='utf-8')
for marker in ['map_mode','547|548','549|550','map_id%2!=0','41*top/(count-1)','selected:None','key:-100,index:269','key:-1,index:108']:assert marker in source
for marker in ['Action::Filter(filter)','let menu=Menu::for_filter(&self.rows,filter)?;self.next_generation()?;self.invalidate();self.selected=None','場地不在目前模式','generation!=self.generation','c.mode!=pixels.mode||c.map_id!=pixels.map_id']:assert marker in runtime
assert 'let first=' not in runtime and 'self.selected=rows.iter().position' not in runtime
for marker in ['CBN_SELCHANGE','Filter::DiagnosticAll','alternate:false','skin.compose(menu.top,menu.count(),menu.pressed)','msg.hwnd==hwnd','msg.hwnd==controls[MAP]&&msg.wParam.0==0x1b','DrawTextW','Menu::row_rect','Menu::font_height','DeleteObject']:assert marker in window
for marker in ['CopyMode::ColorKey','id==6231{(7,10)}','thumb before child arrow images','LTstring-k.tbl','lines[109]==b"Random"']:assert marker in skin_source
assert details['mode_map_counts']==[4,15,9,3,1,8] and details['native_mode_header_count']==2 and details['thumb_reference_composites']==8
old=read('r477-multi-map-details.json');images={tuple(i['identity']):i for i in old['images']}
assert details['actual_initial564_rgba_sha256']==images[(1,564)]['rgba_sha256'] and details['actual_initial514_rgba_sha256']==images[(1,514)]['rgba_sha256']
assert details['actual_mode_filter_queued_pan_rejections']>=1 and details['actual_filter_during_load_cancelled'] and details['actual_worker_retired']
assert details['GUI']==0 and details['LOADED'] is False
assert pin(L/'r477-native-multi-map-menu-handoff.json')['sha256']=='00327C39FE4B3DA884C2BDFD1B286AB52E7BF8C7382ABEBB84ABB097F7200E73'
assert pin(P/'target/r408-source-selector/i686-pc-windows-msvc/debug/LiToModern.exe')['sha256']=='58F4B73E2E975BDB63153643A88ED2F364778D90FDF3B190C03A730D0157CCB3'
result=dict(state='PASS_R478_FINAL_PIN_MAIN_PE_SOURCE_NATIVE_FILTER_HEADER_THUMB_AND_REAL_SCENE_IDENTITY_CONTRACT',source_pairs=132,source_fragment_count=len(fr['fragments']),thumb_composites=8,old_scene_hash_pairs=2,GUI=0,LOADED=False,player=player['executable'],details=pin(L/'r478-mode-menu-details.json'),verifier=pin(Path(__file__)))
(L/'r478-independent-contract.json').write_text(json.dumps(result,ensure_ascii=False,indent=2),encoding='utf-8');print(json.dumps(result,ensure_ascii=False))
