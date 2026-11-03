"""Stage stable paint-proof labels and depth-ordered render metadata.

The native raster enqueue order and all pixel blending remain unchanged.
"""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,difflib
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
name='LiTo381-Modern/src/modern_lito/world_scene.rs';p=R/name
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
w=read(R/'工作佇列.json');source=w['current_product_identity']['source_manifest_sha256']
assert source=='BC2CA7E90123F5CA6194737AF8ACA41D5B1997A86BC9BFC8350E3ECDC24A9D3B'
assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
failure=w['current_native_library_shared_resource_optin'];assert failure['original_failure_preserved']and failure['profiles']['Debug']['failed']==7
old=p.read_text('utf-8-sig');new=old
anchor='''        for (index, plan) in plans.iter().enumerate() {
            cancelled(cancel)?;
            let world = projection'''
assert new.count(anchor)==1
new=new.replace(anchor,'''        // Owner labels and returned placements describe scene geometry, not
        // caller vector positions. Keep raster insertion in its original order:
        // equal-depth native fragments still use the established roster policy.
        let mut placement_order: Vec<usize> = (0..plans.len()).collect();
        placement_order.sort_by_key(|index| plans[*index].key);
        let mut owners = vec![0u16; plans.len()];
        for (rank, index) in placement_order.iter().enumerate() {
            owners[*index] = u16::try_from(rank + 1)?;
        }
        for (index, plan) in plans.iter().enumerate() {
            cancelled(cancel)?;
            let world = projection''')
assert new.count('let owner = u16::try_from(index + 1)?;')==1
new=new.replace('let owner = u16::try_from(index + 1)?;', 'let owner = owners[index];')
assert new.count('for (plan, mask) in plans.iter_mut().zip(masks) {')==1
new=new.replace('for (plan, mask) in plans.iter_mut().zip(masks) {',
'''for (index, mask) in placement_order.into_iter().zip(masks) {
            let plan = &mut plans[index];''')
marker='''    #[test]
    fn actual_layer5_pipeline_fades_only_selected_actor_group_and_keeps_hits()'''
assert new.count(marker)==1
test='''    #[test]
    fn native_objective_input_order_keeps_depth_ordered_placements_and_hit_labels() {
        use super::super::sprite::{CompactSpriteBlock, SpriteBlockReference, SpriteFrameFragments};
        let mut assets = synthetic_assets();
        for sprite in assets.sprites.values_mut() {
            for frame in &mut sprite.decoded.frames {
                frame.width = 24;
                frame.height = 24;
                frame.x_offset = 0;
                frame.y_offset = 0;
                frame.rgba = [240, 40, 30, 255].repeat(576);
                frame.fragments = Some(SpriteFrameFragments {
                    references: vec![SpriteBlockReference {
                        half_x: 0, y: 0, depth: 10, block_id: 0,
                    }],
                    atlas: Arc::new(vec![CompactSpriteBlock::test_rgba(&frame.rgba)]),
                });
            }
        }
        let (ox, oy) = key().world_origin();
        let scene = snapshot(&assets.map, 1, 1, vec![
            actor(1, 7, ox + 20, oy + 20),
            actor(2, 8, ox + 19, oy + 19),
        ]);
        let frame = &assets.sprites.values().next().unwrap().decoded.frames[0];
        let objective = |entity_id, x, y| ObjectiveDraw {
            blocking: true, entity_id, cell: WorldCell { x, y }, frame,
            shadow: None, graphics_id: 10, source_frame: 0,
            sprite_name: "synthetic-fragment.spr", subframe: 0,
        };
        let mut draws = vec![objective(3, ox + 21, oy + 21), objective(4, ox + 24, oy + 24)];
        let cancel = AtomicBool::new(false);
        let mut first = Canvas::new(200, 300).unwrap();
        let before = assets.render_into_objectives(&scene, &mut first, &cancel, &[], &draws).unwrap();
        draws.reverse();
        let mut second = Canvas::new(200, 300).unwrap();
        let after = assets.render_into_objectives(&scene, &mut second, &cancel, &[], &draws).unwrap();
        assert_eq!(first.rgba(), second.rgba());
        assert_eq!(before.actors, after.actors);
        assert_eq!(before.objectives, after.objectives);
        assert_eq!(before.actors.iter().map(|p| p.entity_id).collect::<Vec<_>>(), [2, 1]);
        assert_eq!(before.objectives.iter().map(|p| p.entity_id).collect::<Vec<_>>(), [3, 4]);
        for placement in before.actors.iter().chain(&before.objectives) {
            let mask = placement.visibility.as_ref().unwrap();
            assert!((0..300).any(|y| (0..200).any(|x| mask.contains(x, y))));
        }
    }

'''
new=new.replace(marker,test+marker)
assert old!=new
out=R/'維護工具/candidates'/('native-render-placement-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
(out/'before.rs').write_bytes(p.read_bytes());(out/'world_scene.rs').write_text(new,encoding='utf-8',newline='\n')
(out/'change.diff').write_text(''.join(difflib.unified_diff(old.splitlines(True),new.splitlines(True),fromfile=name,tofile=name)),encoding='utf-8')
record=dict(base_source_manifest_sha256=source,changed_files=[name],before_files={name:sha(p)},candidate_files={name:sha(out/'world_scene.rs')},
    failed_evidence=failure['analysis'],failure_archive_sha256=failure['raw_archive_sha256'],
    change_scope='Stable owner labels and scene-depth ordered RenderInfo; unchanged raster enqueue, blending, authority, resources and clocks; one new synthetic native-fragment regression.',
    compiled=False,applied=False,whole_goal_complete=False)
(out/'candidate.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
(out/'preparer.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(candidate=str(out),changed_files=[name],compiled=False,applied=False),ensure_ascii=False))
