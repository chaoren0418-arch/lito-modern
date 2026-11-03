"""Develop an isolated player-facing target-cycle candidate; execute no tests/builds."""
from pathlib import Path
from datetime import datetime, timezone
import difflib, hashlib, json, zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
digest=lambda b:hashlib.sha256(b).hexdigest().upper()
data=lambda v:(json.dumps(v,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
w=read(R/'工作佇列.json');assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
source=w['current_product_identity']['source_manifest_sha256'];assert source=='3E58268A74B9908597221E71BDEC646C1DFEB82A8658444BDF1C24BFABE94628'
manifest=read(V/'native-installed-resource-profile-normal/application-20261020-123323-122063/applied-compiled-inputs.json')
assert len(manifest)==1049 and all(sha(R/n)==h for n,h in manifest.items())
names=['LiTo381-Modern/src/modern_lito/'+n+'.rs' for n in ['world_raster','native_world_scene','native_login_window']]+['LiTo381-Modern/tests/native_skill_selection.rs']
original={n:(R/n).read_bytes() for n in names};text={n:b.decode('utf-8-sig') for n,b in original.items()}
def replace(name,before,after):
    assert text[name].count(before)==1,(name,before[:100]);text[name]=text[name].replace(before,after)
raster,scene,window,tests=names
replace(scene,'    minimap_visible: bool,','    minimap_visible: bool,\n    environment_key: Option<super::native_render_environment::Key>,')
replace(scene,'            minimap_visible:self.receipt["minimap"]["visible"]==true})',
    '            minimap_visible:self.receipt["minimap"]["visible"]==true, environment_key:self.environment_key})')
replace(raster,'impl DrawVisibility {','''impl DrawVisibility {
    /// Resolve actual composited ownership, scanning each shared plane once.
    /// Pointer identity is used only while all Arc borrows remain alive.
    pub(crate) fn visible_ids(masks: &[(u32, Self)], excluded: Option<[i32; 4]>) -> Vec<u32> {
        use std::collections::{BTreeMap, BTreeSet};
        let mut planes: BTreeMap<(usize, u32, u32), (&Self, BTreeMap<u16, Vec<u32>>)> = BTreeMap::new();
        for (id, mask) in masks {
            if mask.owner == 0 || mask.width == 0 || mask.height == 0 { continue; }
            let key = (mask.pixels.as_ptr() as usize, mask.width, mask.height);
            planes.entry(key).or_insert_with(|| (mask, BTreeMap::new()))
                .1.entry(mask.owner).or_default().push(*id);
        }
        let mut visible = BTreeSet::new();
        for (_, (plane, members)) in planes {
            let mut owners = BTreeSet::new();
            for (at, owner) in plane.pixels.iter().enumerate() {
                if *owner == 0 || owners.contains(owner) || !members.contains_key(owner) { continue; }
                let x = (at % plane.width as usize) as i64;
                let y = (at / plane.width as usize) as i64;
                let outside = excluded.is_none_or(|[left, top, width, height]| {
                    !(i64::from(left)..i64::from(left) + i64::from(width)).contains(&x)
                        || !(i64::from(top)..i64::from(top) + i64::from(height)).contains(&y)
                });
                if y < i64::from(plane.height) && outside {
                    owners.insert(*owner);
                    visible.extend(members[owner].iter().copied());
                    if owners.len() == members.len() { break; }
                }
            }
        }
        visible.into_iter().collect()
    }
''')
text[raster]+='''
#[cfg(test)]
mod keyboard_target_visibility_tests {
    use super::*;
    #[test]
    fn keyboard_target_requires_an_owned_pixel_outside_the_overlay() {
        let plane: Arc<[u16]> = Arc::from(vec![0u16, 1, 2, 2]);
        let masks = vec![(9, DrawVisibility {owner: 2, width: 4, height: 1, pixels: plane.clone()}),
            (3, DrawVisibility {owner: 1, width: 4, height: 1, pixels: plane.clone()}),
            (9, DrawVisibility {owner: 2, width: 4, height: 1, pixels: plane})];
        assert_eq!(DrawVisibility::visible_ids(&masks, None), vec![3, 9]);
        assert_eq!(DrawVisibility::visible_ids(&masks, Some([2, 0, 2, 1])), vec![3]);
        assert_eq!(DrawVisibility::visible_ids(&masks, Some([3, 0, 1, 1])), vec![3, 9]);
        assert_eq!(DrawVisibility::visible_ids(&masks, Some([-20, -20, 2, 2])), vec![3, 9]);
    }
    #[test]
    fn background_and_fully_occluded_owners_cannot_be_keyboard_targets() {
        let plane: Arc<[u16]> = Arc::from(vec![0u16, 1, 1, 0]);
        for owner in [0, 2] {
            let mask = DrawVisibility {owner, width: 4, height: 1, pixels: plane.clone()};
            assert!(DrawVisibility::visible_ids(&[(7, mask)], None).is_empty());
        }
    }
    #[test]
    fn independent_owner_planes_cannot_lend_each_other_visibility() {
        let masks = vec![(9, DrawVisibility {owner: 1, width: 2, height: 1,
            pixels: Arc::from(vec![1u16, 0])}),
            (3, DrawVisibility {owner: 1, width: 2, height: 1,
            pixels: Arc::from(vec![0u16, 1])})];
        assert_eq!(DrawVisibility::visible_ids(&masks, Some([0, 0, 1, 1])), vec![3]);
    }
}
'''
replace(scene,'    pub fn ground_cell(&self,world:&World,x:i32,y:i32)->Option<WorldCell> {','''    /// Cycle only objects with actual pixels in this presented authority frame.
    /// Object ID order is stable across changes in raster/depth iteration order.
    /// This is selection only: the controller still validates every later send.
    pub fn next_target(&self, world: &World,
        current: Option<super::native_skill_selection::Target>,
        environment: super::native_render_environment::Key,
    ) -> Option<super::native_skill_selection::Target> {
        use super::native_skill_selection::Target;
        if self.environment_key != Some(environment) || environment.blind != 0
            || !self.input_ready(world) || !world.ready() || world.battle_result.is_some()
            || !world.hp.is_some_and(|hp| hp[0] > 0) { return None; }
        let own = world.own_actor()?;
        if own.dead { return None; }
        let excluded = self.minimap_visible.then_some(super::native_minimap::RECT);
        let ids: Vec<u32> = super::world_raster::DrawVisibility::visible_ids(&self.visibility, excluded)
            .into_iter().filter(|id| *id != own.id && !self.is_ground_item(*id)
                && Target::capture(world, *id).and_then(|t| t.resolve(world)) == Some(*id)).collect();
        let current = current.and_then(|t| t.resolve(world));
        let id = current.and_then(|id| ids.iter().position(|other| *other == id))
            .and_then(|at| ids.get(at + 1)).or_else(|| ids.first())?;
        Target::capture(world, *id)
    }
    pub fn ground_cell(&self,world:&World,x:i32,y:i32)->Option<WorldCell> {''')
replace(window,'    unsafe fn attack_target(&mut self) -> Result<()> {','''    unsafe fn cycle_visible_target(&mut self) -> Result<()> {
        ensure!(!self.closing && self.world_view && !self.inventory_view && !self.rental.open
            && self.controller.snapshot().phase.active_world()
            && !self.controller.snapshot().observer, "目前不能切換目標");
        ensure!(self.item_cursor.is_none(), "物品選點模式中，請先取消");
        let mut area = RECT::default();
        GetClientRect(self.parent, &mut area)?;
        let dpi = GetDpiForWindow(self.parent).max(96) as i32;
        let view = super::compositor::Viewport::fit(area.right,
            area.bottom - (if self.result_view {260} else {185}) * dpi / 96, 800, 600)
            .context("等待有效世界畫面")?;
        let (epoch, surface) = self.painted_input.as_ref().context("等待世界畫面")?;
        let environment = super::native_render_environment::Key::capture(
            self.controller.transport().snapshot().epoch, &self.controller.snapshot().world,
            &self.controller.snapshot().environment)?;
        ensure!(environment.blind == 0, "視線受限，暫時無法切換目標");
        ensure!(*epoch == self.controller.transport().snapshot().epoch
            && surface.input_ready(&self.controller.snapshot().world)
            && self.painted.is_some_and(|(_, _, painted_view)| painted_view == view),
            "世界畫面已更新，請等待重繪後再切換目標");
        let next = surface.next_target(&self.controller.snapshot().world, self.target, environment);
        self.held.clear(); self.held_attack.clear();
        self.target = next; self.ground = None;
        self.refresh();
        Ok(())
    }
    unsafe fn attack_target(&mut self) -> Result<()> {''')
replace(window,'                if world_focus && message.message==WM_KEYDOWN && message.wParam.0==0x50 {','''                if world_focus && message.message == WM_KEYDOWN && message.wParam.0 == 0x54 {
                    let mut ui = state.0.borrow_mut();
                    if ui.world_view && !ui.inventory_view && !ui.rental.open
                        && ui.controller.snapshot().phase.active_world() {
                        if message.lParam.0 & (1 << 30) == 0 {
                            if let Err(error) = ui.cycle_visible_target() {
                                ui.text(STATUS, &format!("{error:#}"));
                            }
                        }
                        continue;
                    }
                }
                if world_focus && message.message==WM_KEYDOWN && message.wParam.0==0x50 {''')
replace(window,'方向鍵移動；Space攻擊；O門；P入口；Esc清除。','方向鍵移動；T切換目標；Space攻擊；O門；P入口；Esc清除。')
replace(window,'請先點選有效目標','請先點選有效目標，或按 T 切換')
text[tests]+='''
#[test]
fn keyboard_target_cycle_uses_presented_pixels_and_retires_stale_selections() {
    use lito_modern::modern_lito::native_world_scene::Model as Scene;
    use std::{path::Path, sync::atomic::AtomicBool};
    let w = world();
    let before = serde_json::to_value(&w).unwrap();
    let cancel = AtomicBool::new(false);
    let mut scene = Scene::load(Path::new(r"D:\\lineage開發\\0-客戶端"), &w, &cancel).unwrap();
    let mut env = lito_modern::modern_lito::native_environment::State::default();
    env.begin(7).unwrap();env.map_changed(7,w.map_id.unwrap(),Some(0)).unwrap();
    let bound = lito_modern::modern_lito::native_render_environment::Bound::capture(7,&w,&env).unwrap();
    let unbound = scene.draw_at(&w, &cancel, 0).unwrap();
    assert!(unbound.input_surface(&w).unwrap().next_target(&w,None,bound.key()).is_none());
    let frame = scene.draw_at_labels_environment(&w, &cancel, 0, None, Some(&bound)).unwrap();
    let surface = frame.input_surface(&w).unwrap();
    let first = surface.next_target(&w, None, bound.key()).expect("visible fixture target");
    assert_eq!(first.resolve(&w), Some(2));
    assert_eq!(surface.next_target(&w, Some(first), bound.key()), Some(first));
    assert_eq!(serde_json::to_value(&w).unwrap(), before, "selection must not spend MP, predict or send");
    let mut changed = w.clone(); changed.visual_revision += 1;
    assert!(surface.next_target(&changed, Some(first), bound.key()).is_none());
    let mut changed = w.clone(); changed.generation += 1;
    assert!(surface.next_target(&changed, Some(first), bound.key()).is_none());
    let mut changed = w.clone(); changed.actors.get_mut(&1).unwrap().incarnation += 1;
    assert!(surface.next_target(&changed, Some(first), bound.key()).is_none());
    let mut changed = w.clone(); changed.actors.get_mut(&2).unwrap().dead = true;
    assert!(surface.next_target(&changed, Some(first), bound.key()).is_none());
    let mut changed = w.clone(); changed.actors.remove(&2);
    assert!(surface.next_target(&changed, Some(first), bound.key()).is_none());
    let mut changed = w.clone(); changed.hp = Some([0, 50]);
    assert!(surface.next_target(&changed, Some(first), bound.key()).is_none());
    let mut changed = w.clone(); changed.observer_mode = true;
    assert!(surface.next_target(&changed, Some(first), bound.key()).is_none());
    let mut changed = w.clone(); changed.control_interruptions += 1;
    assert!(surface.next_target(&changed, Some(first), bound.key()).is_none());
    let mut changed = w.clone(); changed.actors.get_mut(&2).unwrap().incarnation += 1;
    changed.visual_revision += 1;
    assert!(first.resolve(&changed).is_none());
    assert!(surface.next_target(&changed, Some(first), bound.key()).is_none());
    let fresh_frame = scene.draw_at_labels_environment(&changed, &cancel, 1, None, Some(&bound)).unwrap();
    let fresh_surface = fresh_frame.input_surface(&changed).unwrap();
    let fresh = fresh_surface.next_target(&changed, Some(first), bound.key()).unwrap();
    assert_ne!(fresh, first); assert_eq!(fresh.resolve(&changed), Some(2));
    // The input plane must belong to the effective render environment too.
    // No new target selection through blind masks, including after a cure
    // until a fresh unblinded frame has actually been presented.
    for mode in [1, 2] {
        let mut blind = env.clone();blind.accept(7, &[0x0a,mode]).unwrap();
        let masked = lito_modern::modern_lito::native_render_environment::Bound::capture(7,&w,&blind).unwrap();
        assert!(surface.next_target(&w,Some(first),masked.key()).is_none());
        let masked_frame = scene.draw_at_labels_environment(&w,&cancel,2,None,Some(&masked)).unwrap();
        let masked_surface = masked_frame.input_surface(&w).unwrap();
        assert!(masked_surface.next_target(&w,Some(first),masked.key()).is_none());
        blind.accept(7, &[0x0a,0]).unwrap();
        let cured = lito_modern::modern_lito::native_render_environment::Bound::capture(7,&w,&blind).unwrap();
        assert!(masked_surface.next_target(&w,Some(first),cured.key()).is_none());
        let cured_frame = scene.draw_at_labels_environment(&w,&cancel,3,None,Some(&cured)).unwrap();
        assert_eq!(cured_frame.input_surface(&w).unwrap().next_target(&w,None,cured.key()).unwrap().id,2);
    }
}
'''
out=R/'維護工具/candidates'/('native-keyboard-target-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
after={n:s.encode('utf-8') for n,s in text.items()}
for n,b in after.items():
    target=out/n;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(b)
    (out/(Path(n).name+'.diff')).write_text(''.join(difflib.unified_diff(original[n].decode('utf-8-sig').splitlines(True),text[n].splitlines(True),fromfile=n,tofile=n)),encoding='utf-8',newline='\n')
for filename,files in [('before.zip',original),('candidate.zip',after)]:
    with zipfile.ZipFile(out/filename,'x',zipfile.ZIP_DEFLATED) as z:
        for n,b in files.items():z.writestr(n,b)
    with zipfile.ZipFile(out/filename) as z:assert all(z.read(n)==b for n,b in files.items())
record=dict(status='isolated_development_candidate_not_applied_not_built_not_tested',base_source_manifest_sha256=source,
 candidate_files={n:digest(b) for n,b in after.items()},base_files={n:digest(b) for n,b in original.items()},
 before_archive_sha256=sha(out/'before.zip'),candidate_archive_sha256=sha(out/'candidate.zip'),
 runtime_files=names[:3],test_files=names[3:],new_tests_written=4,tests_executed=0,builds_executed=0,
 production_applied=False,GUI_verified=False,whole_goal_complete=False,
 scope='Player-facing T target cycling, no send or local authority prediction; presented environment key and blind=0 plus epoch/viewport/owner/occlusion fences',
 next_steps=['Static review of candidate; fix issues before execution', 'After user resumes testing: isolated baseline/candidate meaningful tests, type/build checks',
 'Only apply after independent candidate qualification; then final-source library Debug/Release and normal GUI regression',
 'Normal GUI T target cycle, own/peer skill43, lowMP, stale target, cancel/owner retirement; screenshots/mouse separately pending'])
(out/'candidate.json').write_bytes(data(record));(out/'preparer.py').write_bytes(Path(__file__).read_bytes())
(out/'README.md').write_text('# 原生鍵盤目標選取候選\n\nT 僅在世界根視窗切換目前已繪出的有效目標；不施放、不扣MP、不移動。只選取至少一個像素未被遮擋且未在小地圖遮罩內的對象，略過自己與地面物品。每張共用owner平面只掃一次，按物件ID穩定排序及去重。保留已呈現畫面、連線epoch、viewport與目標生命週期檢查，背包／NPC／物品選點／觀戰時不接管輸入。\n\n目前只完成候選程式與四個測試案例的撰寫；依使用者要求，未編譯、未跑測試、未套用正常來源。正常3E58268A之既有資格維持。\n\n續作先靜態審查排序、遮擋與取消／重生邊界；測試恢復後先驗隔離候選，確認後才套用並重跑當版完整library與GUI。\n',encoding='utf-8')
assert all(sha(R/n)==h for n,h in manifest.items()),'Normal source changed during candidate development'
print(json.dumps(dict(candidate=str(out/'candidate.json'),production_applied=False,tests_executed=0,new_tests_written=4),ensure_ascii=False))
