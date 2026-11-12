//! Loopback two-player exercise using the production controller and renderer.
//! Files coordinate fixture phases only; all gameplay authority comes over TCP.
use super::{native_combat::HeldAttack, native_connection::Limits,
    native_login::{LoginController, LoginPhase}, native_world_scene::{Frame, Model}};
use anyhow::{bail, ensure, Context, Result};
use serde_json::{json, Value};
use std::{net::SocketAddr, path::{Path, PathBuf}, sync::atomic::AtomicBool, time::{Duration, Instant}};

pub(super) fn capture(root: &Path, role: &str, round: usize, kind: &str, frame: &Frame) -> Result<Value> {
    let path = root.join(format!("{role}-round{round}-{kind}.png"));
    let mut enc = png::Encoder::new(std::fs::File::create(&path)?, 800, 600);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header()?.write_image_data(frame.canvas.rgba())?;
    Ok(json!({"png":path,"frame":frame.receipt}))
}

pub fn run(peer: SocketAddr, account: &str, password: &str, source: usize, role: &str) -> Result<Value> {
    ensure!(peer.ip().is_loopback() && peer.port()!=0, "peer probe requires loopback");

    let profile=std::env::var("LITO_NATIVE_PEER_PROFILE").unwrap_or_else(|_|"melee".into());
    let visibility=matches!(profile.as_str(),"detection-invisibility"|"detection-refused"|"invisibility-refused"|"invisibility-missing");
    ensure!(matches!(role,"a"|"b") || (role=="c"&&profile=="lightning-multi"),"unknown peer role");
    ensure!(super::native_peer_actions::valid(&profile),"unknown peer profile");
    let withdrawal=std::env::var("LITO_NATIVE_BATTLE_WITHDRAWAL").ok().as_deref()==Some("1");
    ensure!(!withdrawal||profile=="melee","withdrawal only in ordinary battle probe");
    let root = PathBuf::from(std::env::var("LITO_NATIVE_PEER_ROOT").context("peer fixture directory required")?).canonicalize()?;
    let allowed = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/native-real-peers").canonicalize()?;
    ensure!(root.starts_with(allowed), "peer fixture output outside owned target");
    if profile=="room-admin" {return super::native_peer_room_probe::run(peer,account,password,source,role,&root);}
    if profile=="classic-entry" {return super::native_peer_classic_probe::run(peer,account,password,source,role,&root);}
    if profile.starts_with("rental-") {return super::native_peer_rental_probe::run(peer,account,password,source,role,&root);}
    let other = if role=="a" {"b"} else {"a"};
    // IDs are the actual two MemoryAccountRepository records, asserted against
    // both ClientSession trace and received S_STATUS/S_PUT_OBJECT by the runner.
    let (own_id, target) = if role=="a" {(1,3)} else if role=="b" {(3,1)} else {(6,1)};
    let mut c=LoginController::default();
    c.start(peer,Limits{read_chunk:2,write_chunk:3,..Limits::default()})?;
    let start=Instant::now();
    let mut trace=Vec::new();
    let mut previous=Value::Null;
    let outcome=(|| -> Result<Value> {
        let cancel=AtomicBool::new(false);
        let mut submitted=false;
        let mut rounds=Vec::new();
        let mut withdrawal_requested=false;
        let mut model:Option<Model>=None;
        let mut clock=Instant::now();
        let mut generation=0;
        let mut first=Value::Null;
        let mut attack=HeldAttack::default();
        let mut pressed=false;
        let mut attacks=0usize;
        let mut steps=0usize;
        let mut result_since=None;
        let mut returning=false;
        let mut force_requested=false;
        let mut mode_attempts=0;
        let mut last_refresh=Instant::now()-Duration::from_secs(2);
        let mut action_probe=super::native_peer_actions::Probe::default();
        let mut visibility_probe=VisibilityProbe::default();
        let mut turn_undead_probe=super::native_turn_undead_probe::Probe::default();
        let mut poison_probe=super::native_poison_probe::Probe::default();
        let mut world_probe=super::native_peer_world_probe::Probe::default();
        let mut lifecycle_probe=super::native_peer_lifecycle_probe::Probe::default();
        let mut last_frame=Instant::now()-Duration::from_secs(1);
        loop {
            c.poll();
            let s=c.snapshot();
            let observation=json!({"phase":s.phase,"world":s.world.generation,"hp":s.world.hp,"mp":s.world.mp,
                "own":s.world.own_actor().map(|a|(a.id,a.x,a.y,a.dead)),"attack_packets":s.world.attack_packets,
                "feedback":s.attack_feedback,"result":s.world.battle_result,"room":s.joined_room,
                "error":s.error});
            if observation!=previous {
                ensure!(trace.len()<4096,"peer trace bound");
                trace.push(json!({"ms":start.elapsed().as_millis(),"state":observation}));previous=observation;
            }
            ensure!(start.elapsed()<Duration::from_secs(if profile=="turn-undead-quest"{600}else if profile=="honor-respawn" {360}else{210}),"peer timeout: {previous}");
            if s.notice.as_ref().is_some_and(|n|n.acknowledgement().is_some()) { c.acknowledge_notice()?; }
            match c.snapshot().phase {
                LoginPhase::ReadyForLogin if !submitted => {c.login(account,password)?;submitted=true;}
                LoginPhase::ReadyForLogin => bail!("peer LOGIN rejected"),
                LoginPhase::SourcesReady => c.select_source(source)?,
                LoginPhase::RedirectReady => {
                    ensure!(c.snapshot().redirect.context("GAME redirect")?.ip().is_loopback(),"non-loopback redirect");
                    c.begin_game()?;
                }
                LoginPhase::CharactersReady => c.select_character(0)?,
                LoginPhase::CharacterSelected => c.enter_channel(0)?,
                LoginPhase::LobbyReady => {
                    if returning {
                        std::fs::write(root.join(format!("{role}-returned-{}",rounds.len())),b"lobby")?;
                        returning=false;
                    }
                    if rounds.len()==2 {
                        // Keep both players alive until both have returned, so a
                        // disconnect cannot manufacture the other player's win.
                        if root.join(format!("{other}-returned-2")).exists() {
                            return Ok(json!({"status":"two_real_peer_rounds_completed","role":role,"rounds":rounds,"battle_withdrawal":withdrawal,
                                "transport":c.transport().snapshot(),"final_lobby":c.snapshot()}));
                        }
                    } else if rounds.is_empty() || root.join(format!("{other}-returned-{}",rounds.len())).exists() {
                        if c.snapshot().rooms.is_empty() {
                            if last_refresh.elapsed()>Duration::from_secs(1) {c.refresh_rooms()?;last_refresh=Instant::now();}
                        } else {
                            c.join_room(0,"")?; force_requested=false;
                        }
                    }
                }
                LoginPhase::RoomReady => {
                    let s=c.snapshot();
                    let room=s.joined_room.as_ref().context("peer room")?;
                    let own=room.members.iter().find(|m|Some(m.name.as_str())==s.selected_character.as_deref()).context("peer membership")?;
                    if profile=="turn-undead-quest"&&room.mode!=2 {
                        if s.selected_character.as_deref()==Some(room.owner.as_str()) {ensure!(mode_attempts<6,"Quest mode cycle budget");c.step_room_mode(1)?;mode_attempts+=1;}
                        std::thread::sleep(Duration::from_millis(8));continue;
                    }
                    if profile=="honor-respawn"&&room.mode!=5 {
                        if s.selected_character.as_deref()==Some(room.owner.as_str()) {
                            ensure!(mode_attempts==0,"formal honor mode change was refused");
                            c.step_room_mode(1)?;mode_attempts+=1;
                        }
                        std::thread::sleep(Duration::from_millis(8));continue;
                    }
                    let desired = if profile=="turn-undead-quest" {if role=="a"{1}else{0}} else if role=="a" {0} else {1};
                    if own.force!=desired && !force_requested {
                        c.set_force(desired)?;force_requested=true;
                    } else if own.force==desired && room.members.len()==if profile=="lightning-multi"{3}else{2}
                        && room.members.iter().any(|m|m.force!=desired) && own.state==0 {
                        c.set_ready(true)?;
                    }
                }
                LoginPhase::BattlePending | LoginPhase::BattleResult => {
                    let w=&c.snapshot().world;
                    if !w.ready() || w.integrity_challenge.is_none() || (!(profile=="turn-undead-quest"||(profile=="lightning-multi"&&role=="c")||(visibility&&model.is_some())) && !w.actors.contains_key(&target)) {
                        std::thread::sleep(Duration::from_millis(3));continue;
                    }
                    if model.is_none() {
                        ensure!(w.own_actor().context("own actor")?.id==own_id,"unexpected fixture identity");
                        ensure!(w.generation>generation && w.battle_result.is_none() && w.scores.is_empty()
                            && w.own_attack_packets==0 && w.death_confirmations_sent==0,"stale state in next round");
                        ensure!(w.hp.is_some_and(|hp|hp[0]>0 && hp[0]==hp[1]),"peer initial HP");
                        generation=w.generation;
                        model=Some(Model::load(Path::new(r"D:\lineage開發\0-客戶端"),w,&cancel)?);
                        clock=Instant::now();
                        attack.clear();pressed=false;attacks=0;steps=0;result_since=None;withdrawal_requested=false;
                    }
                    if profile!="melee" && last_frame.elapsed()<Duration::from_millis(50) {
                        std::thread::sleep(Duration::from_millis(3));continue;
                    }
                    last_frame=Instant::now();
                    let frame=match model.as_mut().unwrap().draw_at(&c.snapshot().world,&cancel,clock.elapsed().as_nanos() as u64) {
                        Ok(frame)=>frame,
                        Err(e)=>{if profile=="turn-undead-quest" {turn_undead_probe.save_error(&root,role)?;}return Err(e);}
                    };
                    if first.is_null() {
                        first=json!({"world":c.snapshot().world,"render":capture(&root,role,rounds.len()+1,"start",&frame)?});
                        std::fs::write(root.join(format!("{role}-rendered-{}",rounds.len()+1)),b"rendered")?;
                    }
                    if profile!="melee" {
                        // A hidden remote actor must reach this branch even
                        // when the authority no longer includes it in a roster.
                        if visibility {
                            let progress=if root.join(format!("{other}-rendered-1")).exists() {
                                visibility_probe.tick(&mut c,&frame,&root,role,&profile)
                            } else {Ok(false)};
                            if progress.is_err() {
                                std::fs::write(root.join(format!("{role}-visibility-journal.json")),serde_json::to_vec(&visibility_probe.receipt())?)?;
                                std::fs::write(root.join(format!("{role}-visibility-error-snapshot.json")),serde_json::to_vec(c.snapshot())?)?;
                            }
                            if progress? {
                                return Ok(json!({"status":"real_peer_action_completed","role":role,"action_profile":profile,
                                    "start":first,"visibility_probe":visibility_probe.receipt(),"end":c.snapshot(),
                                    "render":capture(&root,role,1,"visibility-end",&frame)?,"transport":c.transport().snapshot()}));
                            }
                            std::thread::sleep(Duration::from_millis(8));continue;
                        }
                        if profile.starts_with("poison-") || profile=="cure-refused" {
                            let progress=if (root.join(format!("{other}-rendered-1")).exists() && (profile!="lightning-multi" || ["a","b","c"].iter().all(|r|root.join(format!("{r}-rendered-1")).exists()))) {
                                poison_probe.tick(&mut c,&frame,&root,role,&profile)
                            } else {Ok(false)};
                            if progress.is_err() {
                                poison_probe.save_images()?;
                                std::fs::write(root.join(format!("{role}-poison-journal.json")),serde_json::to_vec(&poison_probe.receipt())?)?;
                                std::fs::write(root.join(format!("{role}-poison-error-snapshot.json")),serde_json::to_vec(c.snapshot())?)?;
                            }
                            if progress? {
                                return Ok(json!({"status":"real_peer_action_completed","role":role,"action_profile":profile,
                                    "start":first,"poison_probe":poison_probe.receipt(),"end":c.snapshot(),
                                    "render":capture(&root,role,1,"poison-end",&frame)?,"transport":c.transport().snapshot()}));
                            }
                            std::thread::sleep(Duration::from_millis(8));continue;
                        }
                        if profile=="honor-respawn" {
                            if !(root.join(format!("{other}-rendered-1")).exists() && (profile!="lightning-multi" || ["a","b","c"].iter().all(|r|root.join(format!("{r}-rendered-1")).exists()))){std::thread::sleep(Duration::from_millis(8));continue;}
                            if let Some(render)=lifecycle_probe.tick(&mut c,frame,&root,role)? {
                                return Ok(json!({"status":"real_peer_lifecycle_completed","role":role,"action_profile":profile,
                                    "start":first,"lifecycle_probe":lifecycle_probe.receipt(),"end":c.snapshot(),
                                    "render":render,"transport":c.transport().snapshot()}));
                            }
                            std::thread::sleep(Duration::from_millis(8));continue;
                        }
                        if profile=="turn-undead-quest" {
                            if root.join(format!("{other}-rendered-1")).exists()&&match turn_undead_probe.tick(&mut c,&frame,&root,role){Ok(done)=>done,Err(e)=>{turn_undead_probe.save_error(&root,role)?;return Err(e);}} {
                                return Ok(json!({"status":"real_peer_turn_undead_completed","role":role,"action_profile":profile,"start":first,"turn_undead_probe":turn_undead_probe.receipt(),"end":c.snapshot(),"render":capture(&root,role,1,"tu-end",&frame)?,"transport":c.transport().snapshot()}));
                            }
                            std::thread::sleep(Duration::from_millis(8));continue;
                        }
                        if frame.death_ready(&c.snapshot().world) {c.confirm_death(&frame)?;}
                        if matches!(profile.as_str(),"doors"|"movement") {
                            if (root.join(format!("{other}-rendered-1")).exists() && (profile!="lightning-multi" || ["a","b","c"].iter().all(|r|root.join(format!("{r}-rendered-1")).exists())))
                                &&world_probe.tick(&mut c,&frame,&root,role,&profile)? {
                                return Ok(json!({"status":"real_peer_world_completed","role":role,"action_profile":profile,
                                    "start":first,"world_probe":world_probe.receipt(),"end":c.snapshot(),
                                    "render":capture(&root,role,1,"world-end",&frame)?,"transport":c.transport().snapshot()}));
                            }
                            std::thread::sleep(Duration::from_millis(8));continue;
                        }
                        if (root.join(format!("{other}-rendered-1")).exists() && (profile!="lightning-multi" || ["a","b","c"].iter().all(|r|root.join(format!("{r}-rendered-1")).exists())))
                            && action_probe.tick(&mut c,&frame,&root,role,&profile)? {
                            return Ok(json!({"status":"real_peer_action_completed","role":role,"action_profile":profile,
                                "start":first,"action_probe":action_probe.receipt(),"end":c.snapshot(),
                                "render":capture(&root,role,1,"action-end",&frame)?,"transport":c.transport().snapshot()}));
                        }
                        std::thread::sleep(Duration::from_millis(8));continue;
                    }
                    if c.snapshot().phase==LoginPhase::BattleResult {
                        let since=result_since.get_or_insert_with(Instant::now);
                        if since.elapsed()>Duration::from_secs(2) && !c.snapshot().world.scores.is_empty() {
                            rounds.push(json!({"start":first,"end":c.snapshot(),"attacks_sent":attacks,"moves_sent":steps,"withdrawal_requested":withdrawal_requested,
                                "render":capture(&root,role,rounds.len()+1,"result",&frame)?}));
                            c.return_from_battle()?;returning=true;model=None;first=Value::Null;
                        }
                    } else if withdrawal {
                        if !withdrawal_requested && (role=="a")==rounds.is_empty()
                            &&root.join(format!("{other}-rendered-{}",rounds.len()+1)).exists(){
                            c.request_battle_withdrawal()?;withdrawal_requested=true;
                            ensure!(c.request_battle_withdrawal().is_err(),"duplicate withdrawal sent");
                        }
                    } else if frame.death_ready(&c.snapshot().world) {
                        c.confirm_death(&frame)?;
                    } else if (role=="a")==rounds.is_empty()
                        && root.join(format!("{other}-rendered-{}",rounds.len()+1)).exists()
                        && frame.movement_ready(&c.snapshot().world) {
                        let w=&c.snapshot().world;
                        let own=w.own_actor().unwrap();let remote=&w.actors[&target];
                        if !remote.dead && own.x.abs_diff(remote.x).max(own.y.abs_diff(remote.y))>1 {
                            let route=frame.navigation.route_to_adjacent_avoiding([own.x,own.y],[remote.x,remote.y],|x,y|super::native_obstacles::blocks(w,x,y))?;
                            ensure!(steps<128,"peer movement budget");
                            c.move_checked(*route.first().context("empty peer route")?,&frame.navigation)?;steps+=1;
                        } else if !remote.dead {
                            if !pressed {attack.press(target,c.transport().snapshot().epoch,w);pressed=true;}
                            if c.continue_attack(&mut attack,&frame)? {attacks+=1;ensure!(attacks<=128,"peer attack budget");}
                            ensure!(attack.active() || c.snapshot().world.actors[&target].dead,"held attack interrupted: {:?}",c.snapshot().attack_feedback);
                        }
                    }
                }
                LoginPhase::Failed | LoginPhase::Closed => bail!("peer connection ended: {:?}",c.snapshot().error),
                _ => {}
            }
            std::thread::sleep(Duration::from_millis(8));
        }
    })();
    c.cancel();
    let end=Instant::now()+Duration::from_secs(3);
    while !c.is_quiescent() && Instant::now()<end {c.poll();std::thread::sleep(Duration::from_millis(3));}
    c.poll();
    std::fs::write(root.join(format!("{role}-client-trace.json")),serde_json::to_vec_pretty(&trace)?)?;
    ensure!(c.is_quiescent(),"peer worker did not stop");
    let mut result=outcome?;
    result["worker_quiescent"]=json!(true);result["shutdown"]=json!(c.snapshot().phase);
    Ok(result)
}

/// Uses the same public choice, controller, transport and draw path as the
/// window. Files only delay the next input until both peers observed authority.
#[derive(Default)]
struct VisibilityProbe {
    round:usize,
    moves:usize,
    old_target:Option<super::native_skill_selection::Target>,
    old_surface:Option<super::native_world_scene::InputSurface>,
    hide_sent:bool,
    detect_sent:bool,
    hidden:bool,
    revealed:bool,
    effect_captured:bool,
    stable:Option<Instant>,
    negative_sent:bool,
    actions:Vec<Value>,
    observations:Vec<Value>,
    images:Vec<Value>,
}
impl VisibilityProbe {
    fn receipt(&self)->Value {
        json!({"completed_rounds":self.round,"moves_sent":self.moves,"actions":self.actions,
            "observations":self.observations,"images":self.images})
    }
    fn state(c:&LoginController)->Value {
        json!({"world":c.snapshot().world,"inventory":c.snapshot().inventory,
            "skill_feedback":c.snapshot().skill_feedback})
    }
    fn picture(&mut self,root:&Path,role:&str,label:&str,frame:&Frame)->Result<()> {
        ensure!(self.images.len()<16,"visibility image bound");
        self.images.push(capture(root,role,self.round+1,label,frame)?);Ok(())
    }
    fn cast(&mut self,c:&mut LoginController,frame:&Frame,skill:u16,root:&Path,role:&str)->Result<()> {
        let before=Self::state(c);
        let choice=super::native_skill_selection::choice(&c.snapshot().world,skill,self.old_target)?;
        ensure!(choice.target.is_none()&&choice.ground.is_none(),"self skill acquired a target");
        c.cast(choice.skill,choice.target,frame)?;
        let after=Self::state(c);
        ensure!(before["world"]["mp"]==after["world"]["mp"]&&before["inventory"]==after["inventory"],
            "client spent authority while queuing visibility spell");
        self.actions.push(json!({"round":self.round+1,"skill":skill,"before":before,"immediately_after":after,
            "self_intent":true,"native_4D69F0_clock_provider":skill==60}));
        std::fs::write(root.join(format!("{role}-visibility-{}-cast-{skill}",self.round)),b"queued")?;
        Ok(())
    }
    fn tick(&mut self,c:&mut LoginController,frame:&Frame,root:&Path,role:&str,profile:&str)->Result<bool> {
        let other=if role=="a"{"b"}else{"a"};let target=if role=="a"{3}else{1};
        let positive=profile=="detection-invisibility";
        let total=if positive{2}else{1};
        if self.round==total {return Ok(true);}
        let w=&c.snapshot().world;
        ensure!(w.battle_result.is_none(),"visibility battle ended unexpectedly");
        let fx=&frame.receipt["skill_effects"];
        ensure!(fx["omitted"]==0&&fx["load_error"].is_null(),"visibility effect loader failed");
        ensure!(self.observations.len()<2048,"visibility frame bound");
        self.observations.push(json!({"round":self.round+1,"elapsed_ns":frame.receipt["elapsed_ns"],
            "generation":w.generation,"mp":w.mp,"casts":w.own_cast_packets,"skill_feedback":c.snapshot().skill_feedback,
            "actor_states":w.actors.iter().filter(|(id,_)|**id==1||**id==3).map(|(id,a)|json!({"id":id,
                "draw_state":a.draw_state,"incarnation":a.incarnation,"target_revision":a.target_revision})).collect::<Vec<_>>(),
            "actors":frame.receipt["actors"],"effects":fx,"audio":frame.receipt["audio"]}));
        if self.old_target.is_none() {
            let own=w.own_actor().context("visibility own actor")?;
            let remote=w.actors.get(&target).context("visibility initial peer")?;
            if own.x.abs_diff(remote.x).max(own.y.abs_diff(remote.y))>4 {
                if role=="a"&&frame.movement_ready(w) {
                    ensure!(self.moves<128,"visibility route bound");
                    let route=frame.navigation.route_to_adjacent_avoiding([own.x,own.y],[remote.x,remote.y],
                        |x,y|super::native_obstacles::blocks(w,x,y))?;
                    c.move_checked(*route.first().context("visibility empty route")?,&frame.navigation)?;self.moves+=1;
                }
                return Ok(false);
            }
            if !frame.movement_ready(w) {return Ok(false);}
            self.old_target=super::native_skill_selection::Target::capture(w,target);
            ensure!(self.old_target.is_some(),"visibility initial target absent");
            self.old_surface=frame.input_surface(w);
            ensure!(self.old_surface.is_some(),"visibility initial surface absent");
            self.picture(root,role,"visibility-before",frame)?;
            std::fs::write(root.join(format!("{role}-visibility-{}-ready",self.round)),b"ready")?;
        }
        if !root.join(format!("{other}-visibility-{}-ready",self.round)).exists() {return Ok(false);}
        if !positive {
            if role=="b"&&!self.negative_sent&&frame.movement_ready(&c.snapshot().world) {
                self.cast(c,frame,if profile=="detection-refused"{13}else{60},root,role)?;
                self.negative_sent=true;return Ok(false);
            }
            // Reviewed C02 helpers log and return without a rejection packet
            // on insufficient MP or reagent. Keep the real controller timeout.
            if role=="b"&&self.negative_sent&&c.snapshot().skill_feedback==super::native_skills::Feedback::NoResponse {
                std::fs::write(root.join("visibility-refusal-observed"),b"ordinary no-response gate")?;
            }
            if !root.join("visibility-refusal-observed").exists() {return Ok(false);}
            let w=&c.snapshot().world;
            ensure!(w.effects.iter().all(|e|e.graphics!=749)&&w.actors.values().all(|a|!a.draw_state.invisible),
                "refused visibility skill changed presentation");
            if self.stable.get_or_insert_with(Instant::now).elapsed()<Duration::from_millis(500){return Ok(false);}
            let marker=root.join(format!("{role}-visibility-0-complete"));
            if !marker.exists(){
                self.picture(root,role,"visibility-refused",frame)?;
                std::fs::write(marker,serde_json::to_vec(&Self::state(c))?)?;
            }
            if root.join(format!("{other}-visibility-0-complete")).exists(){self.round=1;return Ok(true);}
            // Capture the endpoint once while keeping the connection alive.
            return Ok(false);
        }
        let hider=if self.round==0{"b"}else{"a"};let hider_id=if hider=="a"{1}else{3};
        if role==hider&&!self.hide_sent&&frame.movement_ready(&c.snapshot().world) {
            self.cast(c,frame,60,root,role)?;self.hide_sent=true;return Ok(false);
        }
        let authority_hidden=if role==hider {
            c.snapshot().world.actors.get(&hider_id).is_some_and(|a|a.draw_state.invisible)
        } else {
            // C02 sends S15 to remote peers and S34 only to the caster.
            !c.snapshot().world.actors.contains_key(&hider_id)
        };
        if !self.hidden&&authority_hidden {
            let w=&c.snapshot().world;
            if role==hider {
                ensure!(w.actor_visible(hider_id)&&frame.receipt["actors"].as_array().context("actor receipts")?
                    .iter().any(|a|a["id"]==hider_id&&a["draw_style"]=="Half"&&a["rendered"]==true),
                    "own invisible actor was not drawn half");
            } else {
                ensure!(!w.actor_visible(hider_id)&&self.old_target.unwrap().resolve(w).is_none(),"hidden target still selectable");
                ensure!(!self.old_surface.as_ref().unwrap().input_ready(w),"old input surface survived hide");
                ensure!(!frame.visibility.iter().any(|(id,_)|*id==hider_id),"hidden actor retained pixel hit ownership");
                let queued=c.transport().queued_tx_counter();
                ensure!(c.attack(hider_id).is_err()&&c.transport().queued_tx_counter()==queued,"hidden attack was queued");
            }
            self.picture(root,role,"visibility-hidden",frame)?;self.hidden=true;
            self.actions.push(json!({"round":self.round+1,"observed_hidden":true,"local_half":role==hider,
                "remote_target_and_input_retired":role!=hider,"state":Self::state(c)}));
            std::fs::write(root.join(format!("{role}-visibility-{}-hidden",self.round)),b"observed")?;
        }
        if self.hidden&&role!=hider&&!self.detect_sent&&root.join(format!("{other}-visibility-{}-hidden",self.round)).exists()
            &&frame.movement_ready(&c.snapshot().world) {
            self.cast(c,frame,13,root,role)?;self.detect_sent=true;return Ok(false);
        }
        let detector=if hider=="a"{"b"}else{"a"};
        if self.hidden&&root.join(format!("{detector}-visibility-{}-cast-13",self.round)).exists()
            &&c.snapshot().world.actors.get(&hider_id).is_some_and(|a|!a.draw_state.invisible)
            &&c.snapshot().world.actor_visible(hider_id) {
            if role!=hider {
                ensure!(self.old_target.unwrap().resolve(&c.snapshot().world).is_none(),"reveal revived old target token");
                ensure!(super::native_skill_selection::Target::capture(&c.snapshot().world,hider_id).is_some(),"revealed actor cannot be selected");
            }
            if !self.revealed {self.picture(root,role,"visibility-revealed",frame)?;self.revealed=true;}
            let drawn=fx["effects"].as_array().is_some_and(|v|v.iter().any(|e|e["drawn"]==true&&e["graphics"]==749));
            if drawn&&!self.effect_captured {self.picture(root,role,"detection-effect",frame)?;self.effect_captured=true;}
            let received=c.snapshot().world.effects.iter().filter(|e|e.graphics==749).count();
            if (role==detector&&(!self.effect_captured||received==0))||fx["active"]!=0||fx["started"]!=received {return Ok(false);}
            if self.stable.get_or_insert_with(Instant::now).elapsed()<Duration::from_millis(500){return Ok(false);}
            let marker=root.join(format!("{role}-visibility-{}-complete",self.round));
            if !marker.exists(){
                self.actions.push(json!({"round":self.round+1,"revealed_and_effects_retired":true,"received749":received,"state":Self::state(c)}));
                std::fs::write(marker,serde_json::to_vec(&Self::state(c))?)?;
            }
            if root.join(format!("{other}-visibility-{}-complete",self.round)).exists() {
                self.round+=1;self.old_target=None;self.old_surface=None;self.hide_sent=false;self.detect_sent=false;
                self.hidden=false;self.revealed=false;self.effect_captured=false;self.stable=None;
            }
        }
        Ok(self.round==total)
    }
}
