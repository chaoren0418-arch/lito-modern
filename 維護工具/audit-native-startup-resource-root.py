"""Record direct resource-root forwarding; this is not a GUI relocation test."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,re,zipfile
ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
baseline=ROOT/'驗證紀錄/current/native-startup/validation-20261016-193539-919973'
pins=read(baseline/'source-inputs.json');proof=read(baseline/'qualified/qualified.json')
names=['src/main.rs','src/modern_lito/native_startup.rs','src/modern_lito/native_startup_window.rs','src/modern_lito/native_login_window.rs','src/modern_lito/native_world_view.rs','src/modern_lito/native_world_scene.rs','src/modern_lito/native_rental_window.rs','src/modern_lito/native_inventory_names.rs','src/modern_lito/native_inventory_icons.rs','src/modern_lito/native_npc_document.rs','src/modern_lito/native_login.rs']
texts={};source_pins={}
for name in names:
    relative='LiTo381-Modern/'+name;p=ROOT/relative
    assert sha(p)==pins[relative];source_pins[relative]=pins[relative];texts[name]=p.read_text(encoding='utf-8-sig')
checks=[
    ('src/main.rs','Route::Pick => return run_native_default(false)','default route'),
    ('src/main.rs','run_with_resources(settings.endpoint(), settings.resource_root().to_owned())','explicit settings to LOGIN'),
    ('src/modern_lito/native_login_window.rs','rental:rental::View::new(resource_root.clone())?','NPC view root'),
    ('src/modern_lito/native_login_window.rs','native_inventory_icons::Service::new(resource_root.clone())','inventory icons root'),
    ('src/modern_lito/native_login_window.rs','&self.resource_root,','scene tick root'),
    ('src/modern_lito/native_world_view.rs','root.to_path_buf(),','render job root'),
    ('src/modern_lito/native_world_scene.rs','None => Model::load(&root, &world, &token)?','new model root'),
    ('src/modern_lito/native_world_scene.rs','Self::load_terrain(root,world,cancel)?','terrain root'),
    ('src/modern_lito/native_world_scene.rs','Bank::open(root)?','actor bank root'),
    ('src/modern_lito/native_rental_window.rs','documents:Service::start(root)?','NPC document service root'),
    ('src/modern_lito/native_inventory_names.rs','Catalog::load(&root, &token)','inventory names worker root'),
    ('src/modern_lito/native_npc_document.rs','Catalog::load(&root,&token)','NPC worker root'),
]
observations=[];snippets={}
for name,needle,label in checks:
    text=texts[name];assert needle in text,(label,needle)
    index=text.index(needle);line=text[:index].count('\n')+1;lines=text.split('\n');part='\n'.join(f'{n+1}: {lines[n]}' for n in range(max(0,line-4),min(len(lines),line+4)))+'\n'
    key=f'{len(observations):02d}-{Path(name).stem}.txt';snippets[key]=part
    observations.append(dict(direct_check=label,path='LiTo381-Modern/'+name,line=line,excerpt=key))
main=texts['src/main.rs'];assert 'native_login_window::run(peer)' not in main
assert 'native_login_window::run_with_resources' in main
login=texts['src/modern_lito/native_login.rs'];diagnostic_start=login.index('fn probe_flow(')
hardcodes=[]
for name in ['src/modern_lito/native_login.rs','src/modern_lito/native_login_window.rs']:
    text=texts[name]
    for match in re.finditer(re.escape(r'D:\lineage開發\0-客戶端'),text):
        if name.endswith('native_login.rs'):assert match.start()>diagnostic_start
        hardcodes.append(dict(path='LiTo381-Modern/'+name,line=text[:match.start()].count('\n')+1,scope='diagnostic probe section' if name.endswith('native_login.rs') else 'legacy public run(peer), not called by current main'))
assert len(hardcodes)==5
out=ROOT/'驗證紀錄/current/native-startup-root-audit'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
for name,text in snippets.items():(out/name).write_text(text,encoding='utf-8',newline='\n')
(out/'auditor.py').write_bytes(Path(__file__).read_bytes());(out/'source-pins.json').write_text(json.dumps(source_pins,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
files={p.name:sha(p) for p in out.iterdir() if p.is_file()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
    for name in files:z.write(out/name,name)
with zipfile.ZipFile(out/'raw.zip') as z:
    for name,h in files.items():
        with z.open(name) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
result=dict(status='recorded_direct_native_startup_resource_root_forwarding',source_manifest_sha256=proof['source_manifest_sha256'],compiled_normal_scope=str(baseline/'qualified/qualified.json'),checked_files=len(names),direct_checks=observations,remaining_hardcoded_development_paths=hardcodes,all_transitive_fallbacks_exhaustively_proven_absent=False,probe_paths_portable=False,GUI_or_relocated_resource_gameplay_verified=False,additional_actual_formal_sessions=0,whole_goal_complete=False,raw_archive_sha256=sha(out/'raw.zip'),raw_files=files,observed_utc=datetime.now(timezone.utc).isoformat())
(out/'audit.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
print(json.dumps(dict(audit=str(out/'audit.json'),direct_checks=len(observations),remaining_development_hardcodes=len(hardcodes),GUI_verified=False),ensure_ascii=False))
