"""Develop #20 against sealed formal75 contracts; no build, server or test run."""
from pathlib import Path
from datetime import datetime, timezone
import argparse,difflib,hashlib,json,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
digest=lambda b:hashlib.sha256(b).hexdigest().upper()
data=lambda v:(json.dumps(v,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
ap=argparse.ArgumentParser();ap.add_argument('--contract',required=True,type=Path);a=ap.parse_args()
w=read(R/'工作佇列.json');assert not w['active_workloads'] and w['current_user_requested_goal']['test_execution_deferred_by_user']
source=w['current_product_identity']['source_manifest_sha256'];assert source=='3E58268A74B9908597221E71BDEC646C1DFEB82A8658444BDF1C24BFABE94628'
manifest=read(V/'native-installed-resource-profile-normal/application-20261020-123323-122063/applied-compiled-inputs.json')
assert all(sha(R/n)==h for n,h in manifest.items())
contract=a.contract.resolve(strict=True);proof=read(contract);methods=read(proof['selected_methods'])
assert proof['formal_jar_sha256']=='75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
assert sha(proof['formal_jar'])==proof['formal_jar_sha256'] and sha(proof['selected_methods'])==proof['selected_methods_sha256']
selector=next(v for k,v in methods['Protocol'].items() if ' spellSelector(' in k)
attack=next(v for k,v in methods['Protocol'].items() if ' attackSpellRequest(' in k)
assert 'Protocol$Reader.d:' in attack and attack.count('Protocol$Reader.h:')==2 and 'invalid native attack C_USE_SPELL layout' in attack
assert 'Protocol$SpellSelector' in selector
classic=next(v for k,v in methods['ClientSession'].items() if k.startswith('private void useClassicWorldBlindSpell('))
assert 'Protocol.attackSpellRequest' in classic and 'BlindSpell.maximumRange' in classic and 'ClassicMap.attackPosition' in classic
assert 'Protocol.blind' in classic and 'BlindSpell.durationSeconds' in classic and 'Protocol.manaPoint' in classic
assert 'BlindSpell.reuseDelayMillis' in classic and 'Protocol.effectObject' in classic
block=methods['curse-blind-constructor']
for token in ['bipush        20','iconst_2','iconst_3','sipush        746','sipush        500','iconst_1','bipush        16','String curse-blind']:
    assert token in block,token
additional=R/'LiTo381-Modern/src/modern_lito/native_additional_finite_effects.rs'
assert 'Spec{sprite:746,bank:746,' in additional.read_text('utf-8-sig')
names=['LiTo381-Modern/src/modern_lito/native_skills.rs','LiTo381-Modern/src/modern_lito/native_skill_selection.rs','LiTo381-Modern/tests/native_blind.rs']
assert not(R/names[2]).exists()
original={n:(R/n).read_bytes() for n in names[:2]};text={n:b.decode('utf-8-sig') for n,b in original.items()}
def replace(n,before,after):
    assert text[n].count(before)==1,(n,before);text[n]=text[n].replace(before,after)
request,selection,tests=names
replace(request,'4 | 6 | 7 | 10 | 11 | 15 | 16 | 17 | 18 | 28 | 34 | 38 | 77 =>',
    '4 | 6 | 7 | 10 | 11 | 15 | 16 | 17 | 18 | 20 | 28 | 34 | 38 | 77 =>')
replace(request,'match skill {10=>3,11=>1,28=>4,17|34|38=>6,_=>10}',
    'match skill {10=>3,11|20=>1,28=>4,17|34|38=>6,_=>10}')
replace(request,'11|13|17|18|49|57|72=>500','11|13|17|18|20|49|57|72=>500')
replace(selection,'                        19 => "#19 中級治癒術（選取目標／自己）".into(),',
    '                        19 => "#19 中級治癒術（選取目標／自己）".into(),\n                        20 => "#20 致盲（選取目標，一格內）".into(),')
text[tests]=r'''use lito_modern::modern_lito::{native_skills as skills, native_skill_selection::{Model,Target},native_world::World};
fn world() -> World {
    let f:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();
    let mut w=World::default();
    for key in ["world","start","status","actor","bot"] {
        let hex=f["packets"][key].as_str().unwrap();
        let bytes:Vec<u8>=(0..hex.len()).step_by(2).map(|i|u8::from_str_radix(&hex[i..i+2],16).unwrap()).collect();
        w.accept(&bytes).unwrap();
    }
    let mut own=w.actors.remove(&0xfedcba98).unwrap();own.id=1;own.x=32770;own.y=32818;
    w.actors.insert(1,own);w.status.as_mut().unwrap().object_id=1;
    let target=w.actors.get_mut(&2).unwrap();target.x=32771;target.y=32818;
    w.hp=Some([50,50]);w.mp=Some([100,100]);w.learned_skills.insert(20);w
}
#[test]
fn blind_20_uses_exact_formal_target_and_coordinates_without_prediction() {
    let w=world();let before=serde_json::to_value(&w).unwrap();
    assert_eq!(skills::request(&w,20,Some(2)).unwrap(),[0x14,2,3,2,0,0,0,0x03,0x80,0x32,0x80]);
    assert_eq!(skills::reuse_millis(20),500);
    assert_eq!(serde_json::to_value(&w).unwrap(),before);
    for (dx,dy) in [(-1,-1),(-1,0),(-1,1),(0,-1),(0,1),(1,-1),(1,0),(1,1)] {
        let mut close=w.clone();let (x,y)=((32770+dx) as u16,(32818+dy) as u16);
        {let t=close.actors.get_mut(&2).unwrap();t.x=x;t.y=y;}
        let body=skills::request(&close,20,Some(2)).unwrap();
        assert_eq!(u16::from_le_bytes(body[7..9].try_into().unwrap()),x);
        assert_eq!(u16::from_le_bytes(body[9..11].try_into().unwrap()),y);
    }
}
#[test]
fn blind_20_rejects_bad_targets_and_caster_states_without_mutation() {
    let w=world();
    for target in [None,Some(0),Some(1),Some(999)] {assert!(skills::request(&w,20,target).is_err());}
    for kind in 0..10 {
        let mut bad=w.clone();match kind {
            0=>bad.actors.get_mut(&2).unwrap().x+=1,
            1=>bad.actors.get_mut(&2).unwrap().dead=true,
            2=>bad.actors.get_mut(&2).unwrap().draw_state.invisible=true,
            3=>bad.actors.get_mut(&2).unwrap().draw_state.suppressed=true,
            4=>{bad.learned_skills.remove(&20);},5=>bad.mp=Some([0,100]),6=>bad.hp=Some([0,50]),
            7=>bad.actors.get_mut(&1).unwrap().dead=true,8=>bad.started=false,
            _=>{bad.movement_blocks.insert(2);},
        }
        let before=serde_json::to_value(&bad).unwrap();assert!(skills::request(&bad,20,Some(2)).is_err(),"case {kind}");
        assert_eq!(serde_json::to_value(&bad).unwrap(),before);
    }
}
#[test]
fn blind_20_menu_keeps_learned_id_and_rejects_retired_target_lives() {
    let mut w=world();let mut menu=Model::default();menu.sync(7,&w,true);menu.select(Some(20)).unwrap();
    assert_eq!(menu.rows().iter().find(|r|r.id==20).unwrap().label,"#20 致盲（選取目標，一格內）");
    let target=Target::capture(&w,2).unwrap();assert_eq!(menu.choice(7,&w,Some(target)).unwrap().target,Some(2));
    w.actors.get_mut(&2).unwrap().incarnation+=1;assert!(menu.choice(7,&w,Some(target)).is_err());
    let target=Target::capture(&w,2).unwrap();assert!(menu.choice(8,&w,Some(target)).is_err());
    w.learned_skills.remove(&20);menu.sync(7,&w,true);assert_eq!(menu.selected(),None);
}
#[test]
fn blind_20_low_mp_and_feedback_keep_server_authority() {
    use std::time::{Duration,Instant};
    let mut w=world();w.mp=Some([1,100]);let now=Instant::now();
    let pending=skills::Pending{since:now,casts:w.own_cast_packets,messages:w.message_count,
        generation:w.generation,incarnation:w.own_actor().unwrap().incarnation};
    let before=serde_json::to_value(&w).unwrap();assert!(skills::request(&w,20,Some(2)).is_ok());
    assert_eq!(pending.observe(&w,now+Duration::from_secs(2)),skills::Feedback::NoResponse);
    assert_eq!(serde_json::to_value(&w).unwrap(),before);
    w.accept(&[0x20,1,0,0,0,19]).unwrap();assert_eq!(pending.observe(&w,now),skills::Feedback::CastAction);
    assert_eq!(w.mp,Some([1,100]));
    let mut rejected=world();rejected.last_skill_rejection=Some((pending.messages+1,280));
    assert_eq!(pending.observe(&rejected,now),skills::Feedback::Rejected(280));
    let mut retired=world();retired.generation+=1;assert_eq!(pending.observe(&retired,now),skills::Feedback::Interrupted);
}
#[test]
fn blind_received_modes_and_cancel_stay_owned_by_game_environment() {
    use lito_modern::modern_lito::{native_environment::State,native_render_environment::Key};
    let w=world();let before=serde_json::to_value(&w).unwrap();let mut env=State::default();
    env.begin(7).unwrap();env.map_changed(7,w.map_id.unwrap(),Some(0)).unwrap();
    for mode in [1,2,0] {assert!(env.accept(7,&[0x0a,mode]).unwrap());assert_eq!(Key::capture(7,&w,&env).unwrap().blind,mode);}
    let before_env=serde_json::to_value(&env).unwrap();assert!(env.accept(6,&[0x0a,2]).is_err());
    assert_eq!(serde_json::to_value(&env).unwrap(),before_env);
    env.accept(7,&[0x0a,2]).unwrap();env.retire();assert_eq!(env.blind,0);
    assert!(Key::capture(7,&w,&env).is_err());assert!(env.accept(7,&[0x0a,1]).is_err());
    assert_eq!(serde_json::to_value(&w).unwrap(),before);
}
#[test]
fn blind_746_original_finite_frames_sound_once_and_world_retirement() {
    use lito_modern::modern_lito::{native_skill_effects::Effects,native_world_effects::Target as DrawTarget,
        resources::{SharedArchive,SharedArchiveKind},sprite_actions::{ActionKey,ActionLimits,FrameEvent,SpriteActionCatalog},
        world_map::{WorldCell,PixelPoint,ViewProjection}};
    use std::{path::Path,sync::atomic::AtomicBool};
    let root=Path::new(r"D:\lineage開發\0-客戶端");let cancel=AtomicBool::new(false);
    let bytes=SharedArchive::open(root,SharedArchiveKind::Text).unwrap().read_entry("list.spr").unwrap();
    let catalog=SpriteActionCatalog::parse_for_inspection(&bytes,ActionLimits::default()).unwrap();
    let a=catalog.resolve_action(746,ActionKey::new(0)).unwrap().animation;
    assert_eq!(a.frames.len(),8);assert_eq!(a.frames_per_second,24);
    assert_eq!(a.frames.iter().map(|f|(f.subframe,f.duration_ticks)).collect::<Vec<_>>(),
        vec![(7,3),(0,3),(1,4),(2,4),(3,4),(4,4),(5,4),(6,4)]);
    assert_eq!(a.frames[1].events,vec![FrameEvent::Sound(116)]);
    let mut w=world();w.accept(&[0x37,2,0,0,0,0xea,2]).unwrap();
    let view=ViewProjection{anchor_cell:WorldCell{x:32771,y:32818},anchor_pixel:PixelPoint{x:400,y:300}};
    let mut effects=Effects::default();effects.prepare(root,&w,&cancel);let mut due=0u64;let mut sounds=Vec::new();
    for step in 0..=8 {
        let mut fragments=Vec::new();let (receipt,events)=effects.draw_to(&w,&mut DrawTarget::Fragments(&mut fragments),view,due*1_000_000);
        assert!(receipt["load_error"].is_null(),"{receipt}");sounds.extend(events.into_iter().filter_map(|e|e.sound));
        if step==8 {assert_eq!(receipt["active"],0);assert!(fragments.is_empty());}
        else {assert_eq!(receipt["active"],1);assert!(!fragments.is_empty());let source=if step<7{step+1}else{0};due+=u64::from(a.frames[source].duration_ticks)*1000/24;}
    }
    assert_eq!(sounds,vec![116]);
    let mut live=Effects::default();live.prepare(root,&w,&cancel);
    let mut fragments=Vec::new();let(receipt,_)=live.draw_to(&w,&mut DrawTarget::Fragments(&mut fragments),view,0);
    assert_eq!(receipt["active"],1);assert!(!fragments.is_empty());
    let mut retired=w.clone();retired.clear_preserving_generation();
    let mut fragments=Vec::new();let(receipt,events)=live.draw_to(&retired,&mut DrawTarget::Fragments(&mut fragments),view,1);
    assert_eq!(receipt["active"],0);assert!(fragments.is_empty()&&events.is_empty());
}
'''
out=R/'維護工具/candidates'/('native-blind20-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
after={n:s.encode('utf-8') for n,s in text.items()}
for n,b in after.items():
    p=out/n;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(b)
    (out/(Path(n).name+'.diff')).write_text(''.join(difflib.unified_diff(original.get(n,b'').decode('utf-8-sig').splitlines(True),text[n].splitlines(True),fromfile=n,tofile=n)),encoding='utf-8',newline='\n')
for filename,files in [('before.zip',original),('candidate.zip',after)]:
    with zipfile.ZipFile(out/filename,'x',zipfile.ZIP_DEFLATED) as z:
        for n,b in files.items():z.writestr(n,b)
    with zipfile.ZipFile(out/filename) as z:assert set(z.namelist())==set(files) and all(z.read(n)==b for n,b in files.items())
record=dict(status='isolated_development_blind20_not_applied_not_built_not_tested',base_source_manifest_sha256=source,
 contract=str(contract),contract_sha256=sha(contract),selected_methods_sha256=proof['selected_methods_sha256'],
 candidate_files={n:digest(b) for n,b in after.items()},base_files={n:digest(b) for n,b in original.items()},new_files=[tests],
 before_archive_sha256=sha(out/'before.zip'),candidate_archive_sha256=sha(out/'candidate.zip'),runtime_files=names[:2],test_files=[tests],
 new_tests_written=6,tests_executed=0,builds_executed=0,production_applied=False,GUI_verified=False,whole_goal_complete=False,
 request=[20,2,3,'target object u32 LE','current x/y u16 LE'],range=1,reuse_millis=500,
 server_only_cost_and_probability=dict(mp=20,duration_seconds=16,rejection_message=280,action=19,graphics=746),
 reused_production_paths={n:sha(R/n) for n in ['LiTo381-Modern/src/modern_lito/native_environment.rs','LiTo381-Modern/src/modern_lito/native_render_environment.rs','LiTo381-Modern/src/modern_lito/native_additional_finite_effects.rs']},
 client_cast_clock_original_proven=False,formal_runtime_sessions=0,
 pending=['Static source review then isolated Debug/Release candidate components when user resumes tests',
 'Original formal75 learned book, successful/failed blind roll and lowMP silent refusal; no injected success',
 'Native S0A masks, finite746/sound116, stale target/cancel/handoff/epoch GUI cases',
 'Apply only after candidate evidence; final-source full library Debug/Release and normal GUI regression'])
(out/'candidate.json').write_bytes(data(record));(out/'preparer.py').write_bytes(Path(__file__).read_bytes())
(out/'README.md').write_text('# 原生 #20 致盲候選\n\n正式75DA69B0之C14 row2／column3、目標ID與目前座標；一格內，既有待回覆及500ms重用限制。MP20、機率、16秒狀態及失敗280均由伺服器決定；不在本機預扣或預測成功。正常核心已接S0A狀態、繪圖環境及精確746有限效果，此候選只補請求與選單。\n\n新增6個待執行案例：封包／八方向、拒絕邊界、目標生命週期、低MP與回覆、GAME視覺狀態退役、746有限影格與sound116。未編譯、未測試、未套用，不能宣告技能完成；正常來源仍3E58268A／38個已識別ID。\n\n測試依使用者要求暫緩。後續須以原正式服務觀測實際技能書與成功／失敗／低MP結果，驗候選後才套用，再跑新來源完整library及GUI。\n',encoding='utf-8')
assert all(sha(R/n)==h for n,h in manifest.items()) and not(R/tests).exists()
print(json.dumps(dict(candidate=str(out/'candidate.json'),production_applied=False,new_tests_written=6,tests_executed=0),ensure_ascii=False))
