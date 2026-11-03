//! Bounded real-peer action scenarios. Barrier files only coordinate input.
use super::{native_combat::HeldAttack, native_item_actions::{Action, Choice},
    native_login::LoginController, native_skills::Feedback, native_world_scene::Frame};
use anyhow::{ensure, Context, Result};
use serde_json::{json, Value};
use std::{path::{Path,PathBuf}, time::{Duration, Instant}};

pub fn valid(profile: &str) -> bool {
    if matches!(profile,"detection-invisibility"|"detection-refused"|"invisibility-refused"|"invisibility-missing") {return true;}
    matches!(profile, "melee"|"heal"|"heal19"|"heal19-full"|"heal19-refused"|"heal35"|"heal35-full"|"heal35-refused"|"heal57"|"heal57-full"|"heal57-refused"|"bolt"|"bolt-refused"|"fire-arrow"|"fire-arrow-refused"|"wind-cutter"|"wind-cutter-refused"|"ice-arrow"|"ice-arrow-refused"|"holy-weapon"|"holy-weapon-missing"|"holy-weapon-refused"|"poison-cure"|"poison-missing"|"poison-refused"|"cure-refused"|"chill-touch"|"chill-touch-full"|"chill-touch-refused"|"vampiric-touch"|"vampiric-touch-full"|"vampiric-touch-refused"|"stalac"|"stalac-refused"|"lightning"|"lightning-refused"|"lightning-multi"|"turn-undead-quest"|"turn-undead-refused"|"turn-undead-player"|"wall"|"wall-refused"|"ranged"|"item-heal"|"doors"|"movement"|"honor-respawn"|"room-admin"|"classic-entry"|"rental-entry"|"rental-shop"|"rental-citizen"|"rental-hotel"|"rental-hotel-entry"|"rental-hotel-middle-entry"|"rental-hotel-expiry"|"rental-hotel-middle-expiry"|"rental-hotel-share"|"rental-merchants"|"rental-weight"|"rental-item-expiry"|"rental-equipped-expiry"|"rental-return")
}

fn higher_heal(profile: &str) -> Option<(u16, u64)> {
    let base = profile.strip_suffix("-full").or_else(|| profile.strip_suffix("-refused")).unwrap_or(profile);
    match base { "heal19" => Some((19, 15)), "heal35" => Some((35, 20)), "heal57" => Some((57, 48)), _ => None }
}

#[derive(Default)]
pub struct Probe {
    held: HeldAttack,
    pressed: bool,
    sent: Option<Instant>,
    before: Value,
    line_goal:Option<[u16;2]>,
    choice: Option<Choice>,
    actions: Vec<Value>,
    frames: Vec<Value>,
    images: Vec<Value>,
    pending_images: Vec<(PathBuf,Vec<u8>)>,
    seen_drawn: bool,
    mid_captured: bool,
    health: Vec<Value>,
    lowest_target_ratio: Option<u8>,
    damage_paused: bool,
    done: Option<Instant>,
    endpoint: Value,
    pub attacks: usize,
    pub moves: usize,
}
impl Probe {
    fn capture(&mut self,root:&Path,role:&str,label:&str,frame:&Frame)->Result<()> {
        ensure!(self.images.len()<32,"action image memory bound");let path=root.join(format!("{role}-round1-{label}.png"));
        self.images.push(json!({"png":path,"frame":frame.receipt}));self.pending_images.push((path,frame.canvas.rgba().to_vec()));Ok(())
    }
    pub fn save_images(&mut self)->Result<()> {
        for (path,pixels) in self.pending_images.drain(..){let mut e=png::Encoder::new(std::fs::File::create(path)?,800,600);
            e.set_color(png::ColorType::Rgba);e.set_depth(png::BitDepth::Eight);e.write_header()?.write_image_data(&pixels)?;}
        Ok(())
    }
    fn state(c: &LoginController) -> Value {
        let s=c.snapshot();
        json!({"world":s.world,"inventory":s.inventory,"skill_feedback":s.skill_feedback,
            "item_feedback":s.item_feedback,"attack_feedback":s.attack_feedback})
    }
    pub fn receipt(&self) -> Value {
        json!({"actions":self.actions,"frames":self.frames,"images":self.images,"attacks_sent":self.attacks,
            "moves_sent":self.moves,"endpoint":self.endpoint,"observed_health":self.health,
            "lowest_target_ratio":self.lowest_target_ratio})
    }
    pub fn tick(&mut self, c: &mut LoginController, frame: &Frame, root: &Path, role: &str, profile: &str) -> Result<bool> {
        let other=if role=="a" {"b"} else {"a"};
        let target=if role=="a" {3} else {1};
        let w=&c.snapshot().world;
        let own=w.own_actor().context("action observer")?;
        let health=json!({"hp":w.hp,"mp":w.mp,"id":own.id,"position":[own.x,own.y]});
        if self.health.last()!=Some(&health){ensure!(self.health.len()<256,"health observation bound");self.health.push(health);}
        let ratio=if profile=="lightning-multi"&&role=="c" {w.actors.get(&target).map_or(100,|a|a.hp_ratio)}else{w.actors[&target].hp_ratio};
        self.lowest_target_ratio=Some(self.lowest_target_ratio.map_or(ratio,|v|v.min(ratio)));
        let higher=higher_heal(profile);
        let restoring=(higher.is_some() && !profile.ends_with("-full") && !profile.ends_with("-refused")) || matches!(profile,"heal"|"item-heal"|"chill-touch"|"vampiric-touch");
        let active=if higher.is_some() || restoring || profile.starts_with("holy-weapon") || (profile.starts_with("chill-touch") || profile.starts_with("vampiric-touch")) {"b"} else {"a"};
        let fx_name=match profile {"ranged"=>"projectiles","lightning"|"lightning-refused"|"lightning-multi"=>"many_effects","bolt"|"bolt-refused"|"fire-arrow"|"fire-arrow-refused"|"wind-cutter"|"wind-cutter-refused"|"ice-arrow"|"ice-arrow-refused"=>"spell_projectiles",
            "chill-touch"|"chill-touch-full"|"chill-touch-refused"|"vampiric-touch"|"vampiric-touch-full"|"vampiric-touch-refused"|"stalac"|"stalac-refused"=>"class1_effects","wall"|"wall-refused"=>"ground_effects","item-heal"=>"item_effects",_=>"skill_effects"};
        let fx=&frame.receipt[fx_name];
        if higher.is_some() && !profile.ends_with("-refused") {
            ensure!(fx["effects"].as_array().is_none_or(|rows|rows.iter().all(|row|
                row["elapsed_ns"].as_u64().is_none_or(|ns|ns<12_000_000_000))),"peer finite effect exceeded original12s bound");
        }
        if (higher.is_some() && (fx["active"].as_u64().is_some_and(|n|n>0) || fx["effects"].as_array().is_some_and(|v|!v.is_empty()) || fx["entered_events"].as_array().is_some_and(|v|!v.is_empty()))) || profile=="turn-undead-player" || self.sent.is_some() || self.damage_paused || root.join("action-finished").exists() || ((matches!(profile,"wind-cutter"|"ice-arrow") || profile.starts_with("holy-weapon") || (profile.starts_with("chill-touch") || profile.starts_with("vampiric-touch") || profile.starts_with("stalac") || profile.starts_with("lightning") || profile=="turn-undead-player")) && (fx["active"].as_u64().is_some_and(|n|n>0) || fx["entered_events"].as_array().is_some_and(|v|!v.is_empty()))) {
            ensure!(self.frames.len()<1024,"peer action frame budget");
            if fx["effects"].as_array().is_some_and(|rows|rows.iter().any(|r|r["drawn"]==true)) {
                if !self.seen_drawn {
                    self.capture(root,role,"action-effect",frame)?;
                }
                self.seen_drawn=true;
                if !self.mid_captured && fx["effects"].as_array().is_some_and(|rows|rows.iter().any(|r|
                    r["drawn"]==true && r["elapsed_ns"].as_u64().is_some_and(|ns|ns>=200_000_000))) {
                    self.capture(root,role,"action-effect-mid",frame)?;self.mid_captured=true;
                }
            }
            if matches!(profile,"wind-cutter"|"ice-arrow") {
                let flights=fx["flights"].as_array().context("wind flight rows")?;
                let (flight_gfx,impact_gfx)=if profile=="ice-arrow"{(1797,1798)}else{(1799,1800)};
                if !self.seen_drawn && flights.iter().any(|v|v["graphics"]==flight_gfx&&v["drawn"]==true) {
                    super::native_peer_probe::capture(root,role,1,if profile=="ice-arrow"{"ice-flight"}else{"wind-flight"},frame)?;self.seen_drawn=true;
                }
                if !self.mid_captured && flights.iter().any(|v|v["graphics"]==impact_gfx&&v["drawn"]==true) {
                    super::native_peer_probe::capture(root,role,1,if profile=="ice-arrow"{"ice-impact"}else{"wind-impact"},frame)?;self.mid_captured=true;
                }
            }
            self.frames.push(json!({"elapsed_ns":frame.receipt["elapsed_ns"],"effect":fx,"audio":frame.receipt["audio"],
                "hp":c.snapshot().world.hp,"mp":c.snapshot().world.mp,"casts":c.snapshot().world.own_cast_packets}));
        }
        if root.join("action-finished").exists() {
            let since=self.done.get_or_insert_with(Instant::now);
            if higher.is_some() && !profile.ends_with("-refused") {
                ensure!(since.elapsed()<Duration::from_secs(12),"peer finite effect did not retire before closure");
                ensure!(fx["omitted"]==0 && fx["load_error"].is_null(),"peer finite effect failed before closure");
                // Each peer must finish its own original callbacks before it
                // publishes its endpoint; the caster cannot close it early.
                if fx["started"]!=1 || fx["active"]!=0 {return Ok(false);}
            }
            // Let the passive peer receive the final authority before it records
            // its endpoint; neither connection closes before both snapshots exist.
            if since.elapsed()>Duration::from_millis(500) && self.endpoint.is_null() {
                self.save_images()?;
                self.endpoint=Self::state(c);
                std::fs::write(root.join(format!("{role}-action-endpoint")),b"received")?;
            }
            return Ok(!self.endpoint.is_null() && (root.join(format!("{other}-action-endpoint")).exists() && (profile!="lightning-multi" || ["a","b","c"].iter().all(|r|root.join(format!("{r}-action-endpoint")).exists()))));
        }
        let w=&c.snapshot().world;
        if !frame.movement_ready(w) { return Ok(false); }
        if role=="a" && self.sent.is_none() && self.actions.is_empty() && !self.damage_paused && !(higher.is_some() && !restoring) {
            let own=w.own_actor().context("peer own actor")?;
            let remote=w.actors.get(&target).context("peer target")?;
            // Fire Wall projects an eight-direction ray; an arbitrary distant
            // clicked actor need not lie on that ray. Adjacent cells do align.
            let range=if restoring || profile.starts_with("wall") {1} else if profile.starts_with("vampiric-touch"){4}else if profile.starts_with("chill-touch"){3}else if profile.starts_with("lightning"){6}else {8};
            if own.x.abs_diff(remote.x).max(own.y.abs_diff(remote.y))>range {
                let route=frame.navigation.route_to_adjacent_avoiding([own.x,own.y],[remote.x,remote.y],|x,y|super::native_obstacles::blocks(w,x,y))?;
                ensure!(self.moves<128,"action route budget");
                c.move_checked(*route.first().context("action route empty")?,&frame.navigation)?;
                self.moves+=1;return Ok(false);
            }
            if restoring {
                if remote.hp_ratio<=if matches!(profile,"chill-touch"|"vampiric-touch") {65}else{85} && w.own_attack_packets==self.attacks as u64 {
                    ensure!(self.attacks>0 && !remote.dead,"real peer damage missing");
                    self.held.clear();self.damage_paused=true;
                    std::fs::write(root.join("damage-paused"),serde_json::to_vec(&Self::state(c))?)?;
                    return Ok(false);
                }
                if !self.pressed {self.held.press(target,c.transport().snapshot().epoch,w);self.pressed=true;}
                if c.continue_attack(&mut self.held,frame)? {self.attacks+=1;}
                ensure!(self.attacks<64,"pre-heal attack budget");
                return Ok(false);
            }
        }
        if profile=="lightning-multi" {
            if role=="a" && self.sent.is_none() {
                let a=w.actors.get(&1).context("line caster")?;let b=w.actors.get(&3).context("line primary")?;
                if a.x.abs_diff(b.x).max(a.y.abs_diff(b.y))<=6 && !root.join("a-line-origin-ready").exists() {std::fs::write(root.join("a-line-origin-ready"),serde_json::to_vec(&[a.x,a.y,b.x,b.y])?)?;}
                let Some(third)=w.actors.get(&6)else{return Ok(false);};
                if !root.join("c-line-ready").exists()||!line_cells([a.x,a.y],[b.x,b.y]).contains(&[third.x,third.y]){return Ok(false);}
            }
            if role=="c" {
                if !root.join("a-line-origin-ready").exists(){return Ok(false);}
                let own=w.own_actor().context("third line own")?;
                let bytes=std::fs::read(root.join("a-line-origin-ready"))?;let Ok(ends)=serde_json::from_slice::<[u16;4]>(&bytes)else{return Ok(false);};
                let from=[ends[0],ends[1]];let to=[ends[2],ends[3]];
                if self.line_goal.is_none() {
                    let mut points=line_cells(from,to);points.retain(|p|*p!=from&&*p!=to&&frame.navigation.standing(i32::from(p[0]),i32::from(p[1]))&&!super::native_obstacles::blocks(w,p[0],p[1]));
                    points.sort_by_key(|p|own.x.abs_diff(p[0]).max(own.y.abs_diff(p[1])));self.line_goal=points.first().copied();
                }
                let goal=self.line_goal.context("no free interior line cell")?;
                if [own.x,own.y]!=goal {
                    let heading=if own.x.abs_diff(goal[0]).max(own.y.abs_diff(goal[1]))==1 {
                        (0..8).find(|h|super::native_world::destination(own.x,own.y,*h).ok()==Some((goal[0],goal[1]))).context("line goal heading")?
                    }else{let path=frame.navigation.route_to_adjacent_avoiding([own.x,own.y],goal,|x,y|super::native_obstacles::blocks(w,x,y))?;*path.first().context("line approach route")?};
                    ensure!(self.moves<128,"line third movement budget");c.move_checked(heading,&frame.navigation)?;self.moves+=1;return Ok(false);
                }
                std::fs::write(root.join("c-line-ready"),b"normal authoritative movement reached interior line cell")?;
                return Ok(false);
            }
        }
        if role!=active { return Ok(false); }
        if profile.starts_with("holy-weapon") || (profile.starts_with("chill-touch") || profile.starts_with("vampiric-touch")) {
            let own=w.own_actor().unwrap();let remote=&w.actors[&target];
            // The already-learned caster is player 3. Player 1 approaches
            // through normal movement; never override a received skill book.
            if own.x.abs_diff(remote.x).max(own.y.abs_diff(remote.y))>if profile.starts_with("vampiric-touch"){4}else if profile.starts_with("chill-touch"){3}else{8} {return Ok(false);}
        }
        if restoring && !root.join("damage-paused").exists() { return Ok(false); }
        if let Some(sent)=self.sent {
            ensure!(sent.elapsed()<Duration::from_secs(12),"action did not settle: {profile}: {}",Self::state(c));
            if sent.elapsed()<Duration::from_millis(3500) {return Ok(false);}
            // Keep observing original playback until it settles, within the
            // existing12s deadline above even when callbacks arrive late.
            if higher.is_some() && !profile.ends_with("-refused") && fx["active"].as_u64().is_some_and(|n|n>0) {return Ok(false);}
            let w=&c.snapshot().world;
            let before=&self.before["world"];
            let refused=profile.ends_with("refused");
            ensure!(fx["omitted"]==0 && fx.get("load_error").is_none_or(Value::is_null),"action resource missing: {fx}");
            if profile=="ranged" {
                ensure!(w.own_attack_packets==self.attacks as u64,"arrow acknowledgement missing");
                ensure!(fx["launched"]==self.attacks && fx["arrivals"]==self.attacks && fx["active"]==0,"arrow flight incomplete: {fx}");
                ensure!(w.own_actor().unwrap().walk_action==20,"bow stance absent");
            } else if profile=="item-heal" {
                let choice=self.choice.as_ref().unwrap();
                ensure!(c.snapshot().item_feedback==super::native_item_actions::Feedback::InventoryChanged,
                    "potion inventory acknowledgement absent");
                ensure!(c.snapshot().inventory.items.get(&choice.item.id).map_or(0,|i|i.count)==choice.item.count-1,
                    "potion count mismatch");
                ensure!(choice.validate(c.snapshot()).is_err(),"stale potion choice admitted");
                ensure!(w.hp.unwrap()[0] as u64>=before["hp"][0].as_u64().unwrap()
                    && (self.actions.len()>0 || w.hp.unwrap()[0] as u64>before["hp"][0].as_u64().unwrap()),"potion did not heal");
                ensure!(w.own_cast_packets==0 && w.own_attack_packets==0,"potion invented attack/cast");
                ensure!(fx["started"]==self.actions.len()+1 && fx["active"]==0 && self.seen_drawn,"potion effect incomplete");
            } else if refused {
                ensure!(w.mp==Some([2,2]) && w.own_cast_packets==0 && w.effects.is_empty(),"refusal changed authority");
                let feedback=if higher.is_some() || matches!(profile,"wall-refused"|"holy-weapon-refused"|"turn-undead-refused") {Feedback::NoResponse} else {Feedback::Rejected(278)};
                ensure!(c.snapshot().skill_feedback==feedback,"refusal feedback differs");
                ensure!(fx[if higher.is_some() || matches!(profile,"wall-refused"|"holy-weapon-refused"|"turn-undead-refused"|"chill-touch-refused"|"vampiric-touch-refused"|"stalac-refused"|"lightning-refused") {"started"} else {"launched"}]==0,"refused action rendered");
            } else if profile=="turn-undead-player" {
                ensure!(w.own_cast_packets==before["own_cast_packets"].as_u64().unwrap()+1,"TU player action missing");
                ensure!(w.mp.unwrap()[0] as u64+20==before["mp"][0].as_u64().unwrap(),"TU player MP debit differs");
                ensure!(w.actors[&target].hp_ratio==100&&!w.actors[&target].dead,"TU predicted or received player damage/death");
                let successful=c.snapshot().skill_feedback==Feedback::CastAction;
                ensure!(successful||c.snapshot().skill_feedback==Feedback::Rejected(280),"TU player probability response missing");
                ensure!(w.effects.len()==before["effects"].as_array().unwrap().len()+usize::from(successful),"TU player effect count differs");
                if successful {ensure!(fx["active"]==0&&self.seen_drawn,"TU player effect did not finish");}
            } else {
                ensure!(c.snapshot().skill_feedback==Feedback::CastAction
                    && w.own_cast_packets==before["own_cast_packets"].as_u64().unwrap()+1,"cast authority missing");
                let cost=if let Some((_,cost))=higher{cost}else if profile.starts_with("lightning"){20}else if profile.starts_with("stalac"){6}else if profile.starts_with("vampiric-touch"){12}else if profile.starts_with("chill-touch"){8}else if profile.starts_with("holy-weapon") {10} else if profile=="heal" {4} else if profile=="bolt" {3} else if profile=="fire-arrow" {6} else if matches!(profile,"wind-cutter"|"ice-arrow") {4} else {60};
                ensure!(w.mp.unwrap()[0] as u64+cost==before["mp"][0].as_u64().unwrap(),"cast MP delta mismatch");
                if (profile.starts_with("chill-touch") || profile.starts_with("vampiric-touch") || profile.starts_with("stalac") || profile.starts_with("lightning")) {
                    ensure!(fx["started"]==self.actions.len()+1 && fx["hits"]==self.actions.len()+1 && fx["active"]==0 && self.seen_drawn,"class1 effect incomplete");
                    ensure!(self.lowest_target_ratio.is_some_and(|v|v<100),"drain target damage absent");
                    if matches!(profile,"chill-touch"|"vampiric-touch") {ensure!(w.hp.unwrap()[0] as u64>before["hp"][0].as_u64().unwrap(),"wounded caster did not drain life");}
                    else if !(profile.starts_with("stalac")||profile.starts_with("lightning")) {ensure!(w.hp.unwrap()[0]==w.hp.unwrap()[1],"full HP cap changed");}
                } else if profile.starts_with("holy-weapon") {
                    ensure!(fx["started"]==1 && fx["active"]==0 && self.seen_drawn,"holy weapon effect incomplete");
                    ensure!(w.effects.len()==1 && w.effects[0].graphics==2165 && w.effects[0].target.map(|v|v.0)==Some(target),"holy weapon target/gfx mismatch");
                } else if higher.is_some() || profile=="heal" {
                    if profile.ends_with("-full") {ensure!(w.hp.unwrap()[0]==w.hp.unwrap()[1],"full heal exceeded or lowered HP cap");}
                    else {ensure!(w.hp.unwrap()[0] as u64>before["hp"][0].as_u64().unwrap(),"heal did not restore real damage");}
                    ensure!(fx["started"]==1 && fx["active"]==0 && self.seen_drawn,"heal effect incomplete");
                } else if matches!(profile,"bolt"|"fire-arrow"|"wind-cutter"|"ice-arrow") {
                    ensure!(fx["launched"]==self.actions.len()+1 && fx["arrivals"]==self.actions.len()+1
                        && fx["active"]==0 && self.lowest_target_ratio.is_some_and(|v|v<100),"bolt flight/damage incomplete");
                } else {
                    ensure!(fx["started"].as_u64().is_some_and(|n|(1..=8).contains(&n))
                        && fx["active"]==0 && self.seen_drawn && self.lowest_target_ratio.is_some_and(|v|v<100),"wall effect/damage incomplete");
                }
            }
            self.actions.push(json!({"profile":profile,"before":self.before,"after":Self::state(c),"effect":fx,
                "no_prediction":true,"duplicate_pending_rejected":true,"turn_undead_success":profile=="turn-undead-player"&&c.snapshot().skill_feedback==Feedback::CastAction}));
            self.sent=None;
            let finished=if profile=="turn-undead-player" {
                ensure!(self.actions.len()<=12,"TU player natural probability attempt budget");
                let success=self.actions.iter().any(|a|a["turn_undead_success"]==true);
                let failed=self.actions.iter().any(|a|a["after"]["skill_feedback"]==serde_json::json!({"Rejected":280}));
                ensure!(self.actions.len()<12||(success&&failed),"TU player both natural outcomes not observed");success&&failed
            }else if profile=="ranged" {self.lowest_target_ratio.is_some_and(|v|v<100)}
                else {self.actions.len()==if matches!(profile,"bolt"|"fire-arrow"|"wind-cutter"|"ice-arrow"|"item-heal"|"chill-touch"|"stalac"|"lightning"|"lightning-multi") {2} else {1}};
            if finished {
                std::fs::write(root.join("action-finished"),b"verified")?;return Ok(false);
            }
            ensure!(profile!="ranged" || self.attacks<16,"16 acknowledged arrows without target damage");
        }
        self.before=Self::state(c);
        let w=&c.snapshot().world;
        let target_position=w.actors.get(&target).map(|a|[a.x,a.y]).context("action target")?;
        if profile=="ranged" {
            self.held.press(target,c.transport().snapshot().epoch,w);
            ensure!(c.continue_attack(&mut self.held,frame)?,"arrow not sent");self.attacks+=1;self.held.clear();
            ensure!(c.attack(target).is_err(),"duplicate pending arrow admitted");
        } else if profile=="item-heal" {
            let id=if let Some(choice)=&self.choice {choice.item.id} else {
                let ids:Vec<_>=c.snapshot().inventory.items.values().filter(|i|i.use_type==0 && i.count==2).map(|i|i.id).collect();
                ensure!(ids.len()==1,"unique fixture potion");ids[0]
            };
            let choice=Choice::capture(c.snapshot(),id)?;
            c.item_action(choice.clone(),Action::Use)?;
            ensure!(c.item_action(choice.clone(),Action::Use).is_err(),"duplicate pending potion admitted");
            self.choice=Some(choice);
        } else if profile.starts_with("wall") {
            c.cast_ground(58,target_position,frame)?;
            ensure!(c.cast_ground(58,target_position,frame).is_err(),"duplicate pending wall admitted");
        } else {
            let skill=if let Some((skill,_))=higher {skill}else if profile.starts_with("turn-undead"){18}else if profile.starts_with("lightning"){17}else if profile.starts_with("stalac"){16}else if profile.starts_with("vampiric-touch"){28}else if profile.starts_with("chill-touch"){10}else if profile.starts_with("holy-weapon") {8} else if profile=="heal" {1} else if profile.starts_with("fire-arrow") {15} else if profile.starts_with("wind-cutter") {7} else if profile.starts_with("ice-arrow"){6} else {4};
            let target=if higher.is_some() || profile=="heal" {None} else {Some(target)};
            c.cast(skill,target,frame)?;
            ensure!(c.cast(skill,target,frame).is_err(),"duplicate pending cast admitted");
        }
        ensure!(self.before["world"]==serde_json::to_value(&c.snapshot().world)?
            && self.before["inventory"]==serde_json::to_value(&c.snapshot().inventory)?,"action predicted authority");
        self.sent=Some(Instant::now());
        Ok(false)
    }
}

/// Probe-only coordinate planning matching the formal retained World.createLineMap.
/// This never predicts damage or writes authority; every movement goes through C_MOVE.
fn line_cells(from:[u16;2],to:[u16;2])->Vec<[u16;2]> {
    let (mut x,mut y)=(i32::from(from[0]),i32::from(from[1]));let (x1,y1)=(i32::from(to[0]),i32::from(to[1]));
    let (dx,dy)=((x1-x).abs(),(y1-y).abs());let (sx,sy)=(if x1>x{1}else{-1},if y1>y{1}else{-1});let mut out=Vec::new();
    let mut error=-dx.max(dy);
    for _ in 0..=dx.max(dy) {out.push([x as u16,y as u16]);if dx>=dy{x+=sx;error+=2*dy;if error>=0{y+=sy;error-=2*dx;}}else{y+=sy;error+=2*dx;if error>=0{x+=sx;error-=2*dy;}}}
    out
}
