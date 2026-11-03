"""Generate a separate main candidate; reversible insertion proof preserves default routes."""
from pathlib import Path
import json,hashlib
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003';O=P/'target/r479b-player-entry-candidate';O.mkdir(exist_ok=False)
def pin(p):
 b=p.read_bytes();return dict(path=str(p),bytes=len(b),sha256=hashlib.sha256(b).hexdigest().upper())
original=P/'src/main.rs';before=original.read_bytes();t=before.decode('utf-8');nl='\r\n' if '\r\n' in t else '\n';patches=[]
def insert(marker,addition):
 global t
 assert t.count(marker)==1;replacement=marker+addition;patches.append((marker,replacement));t=t.replace(marker,replacement)
insert('    Help,'+nl,'    CandidateEntry,'+nl+'    OfflineMap,'+nl)
insert('        None => Ok(Route::Pick),'+nl,'        Some(x) if x == OsStr::new("--candidate-menu") && args.len() == 2 => Ok(Route::CandidateEntry),'+nl+'        Some(x) if x == OsStr::new("--offline-map-preview") && args.len() == 2 => Ok(Route::OfflineMap),'+nl)
insert('    let selected = match route(&args)? {'+nl,'        Route::CandidateEntry => candidate_offline_entry(false)?,'+nl+'        Route::OfflineMap => candidate_offline_entry(true)?,'+nl)
helper='''fn candidate_offline_entry(start_at_maps: bool) -> Result<Option<PathBuf>> {
    use modern_lito::{native_offline_preview_resources::Inputs, native_resources::BytePin, native_map_batch_window::{self, Outcome}};
    let hash = env!("R479_BINDING_SHA256");
    let mut sha256 = [0; 32];
    for (i, byte) in sha256.iter_mut().enumerate() { *byte = u8::from_str_radix(&hash[i*2..i*2+2], 16)?; }
    let inputs = Inputs { project: PathBuf::from(env!("R479_MODERN_ROOT")), manifest: PathBuf::from(env!("R479_BINDING_MANIFEST")), manifest_pin: BytePin { bytes: env!("R479_BINDING_BYTES").parse()?, sha256 } };
    let hash = env!("R479_SKIN_SHA256");
    let mut skin_sha256 = [0; 32];
    for (i, byte) in skin_sha256.iter_mut().enumerate() { *byte = u8::from_str_radix(&hash[i*2..i*2+2], 16)?; }
    let skin_inputs = Inputs { project: PathBuf::from(env!("R479_MODERN_ROOT")), manifest: PathBuf::from(env!("R479_SKIN_MANIFEST")), manifest_pin: BytePin { bytes: env!("R479_SKIN_BYTES").parse()?, sha256: skin_sha256 } };
    match native_map_batch_window::run(inputs, skin_inputs, start_at_maps)? {
        Outcome::Network => modern_lito::choose_profile_path(),
        Outcome::Exit => Ok(None),
    }
}
'''.replace('\n',nl)
marker='fn version_document() -> serde_json::Value {';assert t.count(marker)==1;patches.append((marker,helper+marker));t=t.replace(marker,helper+marker)
insert('        "application": "LiToModern",'+nl,'        "candidate_revision": "r479b",'+nl+'        "offline_entry_flags": ["--candidate-menu", "--offline-map-preview"],'+nl+'        "offline_window_gui_verified": false,'+nl)
candidate=O/'LiToModern-R479b.rs';candidate.write_bytes(t.encode('utf-8'))
reverse=t
for old,new in reversed(patches):assert reverse.count(new)==1;reverse=reverse.replace(new,old)
assert reverse.encode('utf-8')==before;assert original.read_bytes()==before
binding=L/'r479b-map-binding.json';assert len(json.loads(binding.read_text(encoding='utf-8'))['bindings'])==6
record=dict(state='SEPARATE_CANDIDATE_MAIN_INSERTIONS_ONLY_DEFAULT_MAIN_BYTE_UNCHANGED',original_main=pin(original),candidate_main=pin(candidate),binding=pin(binding),reverse_all_insertions_yields_exact_original_bytes=True,default_no_args_route='Pick -> existing choose_profile_path -> existing SessionController::new -> login_window::run',profile_network_routes_unchanged=True,added_flags=['--candidate-menu','--offline-map-preview'],network_choice_from_new_menu='Retire preview worker, then existing choose_profile_path and same existing normal profile/login path',UI_assets='R479 qualified thumb6231 display at native x143/y15+floor41top/(count-1); mode-specific rows including verified Hot Random and Random, cleared initial and mode-rebuild selection/top, explicit ordinary flag0 and diagnostic all40; Modern mode selector and font/keys; Qualified original LTsurf6113 crop157x81 and6232/6234 normal arrows; pressed variants6233/6235 wired to captured press/release; native GUI pending; 40 exact mode/map rows; six qualified Native564/514/509/508/505/516 static previews, remaining34 explicitly unavailable. Native top0..count-1 scroll/press/release; Modern scaled clipped Segoe UI and keyboard selection. Existing Modern standard Windows BUTTON/COMBOBOX/STATIC, stock GUI font, existing Viewport::fit and Canvas::copy_bgra top-down DIB + GDI StretchDIBits; exact original LTtext labels/Native564 terrain pixels',original_LTsurf_bitmap_bindings_verified=True,native_GUI_executed=False,R472_timers_advanced=False)

(L/'r479b-entry-source-contract.json').write_text(json.dumps(record,ensure_ascii=False,indent=2),encoding='utf-8')

record['skin_binding']=pin(L/'r478-native-menu-skin-binding.json')
(L/'r479b-entry-source-contract.json').write_text(json.dumps(record,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps(dict(reverse_exact=True,candidate_main=str(candidate),skin_binding=record['skin_binding'])))
