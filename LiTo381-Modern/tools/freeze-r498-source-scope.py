from r491_common import *
import shutil,difflib
old=read(L/'r497-production-source-snapshot.json');delta=[];diffs=[]
for q in old['source_pins']:
 assert pin(q['snapshot'])['sha256']==q['sha256']
 if pin(q['path'])['sha256']!=q['sha256']:
  delta.append(Path(q['path']).name);diffs.append(''.join(difflib.unified_diff(Path(q['snapshot']).read_text(encoding='utf-8').splitlines(True),Path(q['path']).read_text(encoding='utf-8').splitlines(True),fromfile=q['snapshot'],tofile=q['path'])))
assert set(delta)=={'native_city_runtime.rs','native_city_window.rs','native_mode_menu_skin.rs','native_city_presentation.rs'},delta
new=P/'src/modern_lito/native_character_selection.rs';diffs.append(''.join(difflib.unified_diff([],new.read_text(encoding='utf-8').splitlines(True),fromfile='/dev/null',tofile=str(new))))
diff=L/'r498-source-difference.patch';diff.write_text('\n'.join(diffs),encoding='utf-8');files=[p for p in sorted((P/'src').rglob('*'))if p.is_file()];assert len(files)==612
out=P/'target/r498-production-source';out.mkdir(exist_ok=False);pins=[]
for p in files:
 dest=out/p.relative_to(P);dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,dest);q=pin(p);assert pin(dest)['sha256']==q['sha256'];q['snapshot']=str(dest);pins.append(q)
save(L/'r498-production-source-snapshot.json',dict(state='R498_ALL612_SOURCE_FROZEN_BEFORE_NEW_CONSUMER',source_pins=pins,source_difference=pin(diff),GUI=0))
scope=read(L/'r497-source-scope.json');scope.pop('changed_from_fixed_R496',None);scope.pop('added_from_fixed_R496',None);scope.update(state='R498_CHARACTER_SELECTION_SCENE_AND_NATIVE_BUTTON_ART_SCOPE',source_files=612,changed_from_fixed_R497=delta,added_from_fixed_R497=['native_character_selection.rs'],authorization='Continued R498 originalcharacterUI mapping andclosedsubset integration;Modern owned outputs andpreviousauthorizedModernASCII scratch',scope='4changed+1new production;607previous unchanged;originalresourcesread-only;noNativeSocket/auth/provider/JavaSQL/server/sharedBIN/GUI execution.')
scope['new_source_files'].append(dict(path='src/modern_lito/native_character_selection.rs',sha256=pin(new)['sha256']))
for q in scope['new_source_files']:q['sha256']=pin(P/q['path'])['sha256']
for q in scope['frozen_binary_pins']:assert pin(q['path'])['sha256']==q['sha256']
save(L/'r498-source-scope.json',scope);save(L/'source-change-scope.json',scope)
print('R498 frozen612production/4changed+1new;diff',diff.stat().st_size,'bytes;607previous unchanged')
