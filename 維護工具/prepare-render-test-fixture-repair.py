"""Correct failing test setup without changing production raster or weakening hits."""
from pathlib import Path
from datetime import datetime,timezone
import ast,hashlib,json,re,difflib
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
w=read(R/'工作佇列.json');SOURCE=w['current_product_identity']['source_manifest_sha256']
assert SOURCE=='9F99C1203C0BB3F2CF87A873C7A81F7B20B08AFD3546A2282A84B62F6C8124C8'
assert not w['active_workloads']and not w['source_and_corresponding_QA_frozen']
failure=w['current_native_catalog_library_failure'];assert failure['observed_exit_code']==1 and failure['recorded_original_owned_processes_all_terminal']
assert sha(failure['raw_archive'])==failure['raw_archive_sha256']
planner=(T/'prepare-current-shared-resource-library-plan.py').read_text('utf-8-sig');tree=ast.parse(planner)
node=next(n for n in tree.body if isinstance(n,ast.FunctionDef)and n.name=='function_body');exec(compile(ast.Module(body=[node],type_ignores=[]),'sealed-extractor','exec'))
path=R/'LiTo381-Modern/src/modern_lito/world_scene.rs';old=path.read_text('utf-8-sig');new=old
def body(text,name):
 pos=text.index('fn '+name+'(');line=text[:pos].count('\n')+1
 return function_body(text,name,max(1,line-1))[0]
name='actual_layer5_pipeline_fades_only_selected_actor_group_and_keeps_hits';original=body(old,name)
assert original.count('info.actors[0]')==3
replacement=original.replace('info.actors[0].visibility','info.actors.iter().find(|p| p.entity_id == 1).expect("inside actor absent").visibility')
replacement=replacement.replace('let p = &info.actors[0];','let p = info.actors.iter().find(|p| p.entity_id == 1).expect("inside actor absent");')
assert 'info.actors[0]'not in replacement;new=new.replace(original,replacement)
name='native_objective_input_order_keeps_depth_ordered_placements_and_hit_labels';original=body(old,name)
assert original.count('Canvas::new(200, 300)')==2
replacement=original.replace('Canvas::new(200, 300)','Canvas::new(512, 256)')
replacement=replacement.replace('''            let mask = placement.visibility.as_ref().unwrap();
            assert!((0..300).any(|y| (0..200).any(|x| mask.contains(x, y))));''','''            assert!(placement.top_left.x >= 0 && placement.top_left.y >= 0);
            assert!(placement.top_left.x + placement.width as i32 <= 512);
            assert!(placement.top_left.y + placement.height as i32 <= 256);
            let mask = placement.visibility.as_ref().unwrap();
            assert!((0..256).any(|y| (0..512).any(|x| mask.contains(x, y))));''')
assert replacement!=original and '(0..512)'in replacement
new=new.replace(original,replacement)
marker='#[cfg(test)]\npub(crate) mod tests {';assert old.count(marker)==new.count(marker)==1
assert old.partition(marker)[0]==new.partition(marker)[0]
out=T/'candidates'/('native-render-test-fixture-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
(out/'world_scene.rs').write_text(new,encoding='utf-8',newline='\n');(out/'before.rs').write_bytes(path.read_bytes())
(out/'change.diff').write_text(''.join(difflib.unified_diff(old.splitlines(True),new.splitlines(True),fromfile=str(path),tofile='world_scene.rs')),encoding='utf-8');(out/'preparer.py').write_bytes(Path(__file__).read_bytes())
name=path.relative_to(R).as_posix()
record=dict(base_source_manifest_sha256=SOURCE,changed_files=[name],created_files=[],before_files={name:sha(path)},candidate_files={name:sha(out/'world_scene.rs')},
    failed_run=failure['run'],failure_archive=failure['raw_archive'],failure_archive_sha256=failure['raw_archive_sha256'],
    exact_regression_tests=failure['failed_tests'],test_only_fixture_repair=True,production_prefix_unchanged=True,
    change_scope='Resolve intended inside actor by entity1 after depth-sorted metadata; place all synthetic fragments in a 512x256 viewport and assert bounds. Preserve all original pixel, fade, hit, equality, output-order and positive-mask assertions.',
    compiled=False,applied=False,whole_goal_complete=False)
(out/'candidate.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(candidate=str(out),test_only=True,compiled=False,applied=False),ensure_ascii=False))
