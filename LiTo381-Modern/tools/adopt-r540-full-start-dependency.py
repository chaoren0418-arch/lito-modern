import json,pathlib,hashlib,shutil
p=pathlib.Path(__file__).resolve().parent.parent;l=p/'logs/continuation-20261003';dest=p/'target/r540-native-pet-race-start-inputs'
def read(f):return json.loads(f.read_text(encoding='utf-8-sig'))
def pin(f):return {'path':str(f),'bytes':f.stat().st_size,'sha256':hashlib.sha256(f.read_bytes()).hexdigest().upper()}
def check(d):assert pin(pathlib.Path(d['path']))==d
root=p.parent.parent/'lito伺服器/Native-PetRace-full-start依賴與攔截交接.json';assert pin(root)['sha256']=='CB7F77285D1814AFCA7B043903645102FDC5E139710177BCF6B21D40C35D5FF4';v=read(root)
refs=[d for d in v.values() if isinstance(d,dict) and {'path','bytes','sha256'}<=d.keys()];assert len(refs)==12
for d in refs:check(d)
m=read(pathlib.Path(v['manifest']['path']));assert len(m['files'])==18
for d in m['files']:assert pathlib.Path(d['path']).resolve().is_relative_to(pathlib.Path(v['root']).resolve());check(d)
shutil.copyfile(root,dest/'full-start-dependency-root.json')
for key in ['constructor_audit','dependency_matrix','fixture_preflight','verification','manifest']:shutil.copyfile(v[key]['path'],dest/('full-start-'+key+'.json'))
audit={'root':pin(root),'refs_checked':12,'manifest_files_checked':18,'full_WORLD_blockers':['Real GameStateStore constructor reaches stopped auction initializer; SQL does not remove this prerequisite.','Bookmarks/profile migration, selected identity and bootstrap conditional castle dependencies remain unqualified.'],'full_start_executable':False,'new_store_SQL_transport_or_full_start_runs':0,'component_scope_unchanged':True,'no_constructor_or_BATTLE_bypass':True}
with (l/'r540-full-start-blocker-adoption.json').open('x',encoding='utf-8') as f:json.dump(audit,f,ensure_ascii=False,indent=2);f.write('\n')
print(json.dumps(audit,ensure_ascii=False))
