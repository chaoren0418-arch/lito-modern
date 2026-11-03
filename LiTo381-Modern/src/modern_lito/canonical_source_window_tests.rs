//! Independent native source-reselection acceptance; not a replacement for the
//! Slow/material/combat HWND audit. Setup authenticates owned fixture sessions.
//! After handoff the observer never submits navigation or fabricates UI input.
use super::*;
use crate::modern_lito::character_data::CharacterData;

#[derive(Clone)]
struct Expected {
    data: Arc<CharacterData>,
    gear: Arc<equipment::Snapshot>,
    skills: Vec<crate::modern_lito::spell_protocol::Skill>,
}
impl Expected {
    fn capture(c: &Client, context: SelectionContext) -> Self {
        saved(c, context); // Independent fixed SQL fixture values, including >2^53 EXP.
        let view = c.view();
        let CharacterDataState::Ready(data) = view.character_data else {
            panic!("source fixture data missing")
        };
        let SkillsState::Ready(skills) = view.skills else {
            panic!("source fixture skills missing")
        };
        Self {
            data,
            gear: gear_snapshot(c),
            skills: skills.skills.clone(),
        }
    }

    fn matches(&self, view: &LoginSnapshot, context: SelectionContext) -> bool {
        let (
            CharacterDataState::Ready(data),
            equipment::State::Ready(gear),
            SkillsState::Ready(skills),
        ) = (&view.character_data, &view.equipment, &view.skills)
        else {
            return false;
        };
        view.phase == LoginPhase::ChoosingChannel
            && view.selected_source == Some(context.key.source_id)
            && view.navigation.as_ref().is_some_and(|nav| {
                nav.context == context
                    && nav.status == crate::modern_lito::application_protocol::NavigationStatus::Ok
                    && nav.action == crate::modern_lito::application_protocol::NavigationAction::SelectCharacter
                    && nav.snapshot.phase == crate::modern_lito::application_protocol::NavigationPhase::Channels
                    && nav.snapshot.selected.as_ref() == Some(&self.data.character)
                    && nav.snapshot.channel_id.is_none()
            })
            && !view.equipment_pending
            && !view.spell_pending
            && !view.record_pending
            && !view.world_pending
            && !view.canonical.attached()
            && data.context == context
            && data.character == self.data.character
            && data.stats == self.data.stats
            && data.items == self.data.items
            && gear.source == *data
            && gear.validate().is_ok()
            && gear.flags == self.gear.flags
            && gear.stale == self.gear.stale
            && gear.modifiers == self.gear.modifiers
            && gear.magic_reduction_base == self.gear.magic_reduction_base
            && gear.lifetimes == self.gear.lifetimes
            && gear.weight == self.gear.weight
            && skills.context == context
            && skills.status == crate::modern_lito::spell_protocol::Status::Ok
            && skills.skills == self.skills
            && view.record.context() == Some(context)
            && view.record.totals() == Some(record::Totals {
                stored: false, wins: 0, losses: 0, revision: 0,
            })
    }
}

fn cleared(view: &LoginSnapshot, source: u32) -> bool {
    view.phase == LoginPhase::CharactersReady
        && view.selected_source == Some(source)
        && view.roster_epoch > 0
        && view.characters.len() == 1
        && view.characters[0].source_id == source
        && view.characters[0].object_id == 3333
        && view.navigation.is_none()
        && matches!(view.character_data, CharacterDataState::NotRequested)
        && matches!(view.equipment, equipment::State::NotRequested)
        && matches!(view.skills, SkillsState::NotRequested)
        && matches!(view.record, record::State::NotRequested)
        && matches!(view.shop, shop::State::NotRequested)
        && !view.equipment_pending
        && !view.spell_pending
        && !view.record_pending
        && !view.room_pending
        && !view.shop_pending
        && !view.world_pending
        && view.rooms.joined().is_none()
        && !view.canonical.attached()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Step {
    Select(u32),
    Return(u32),
    Source(u32),
    Close,
}
const ROUTE: [Step; 9] = [
    Step::Select(7),
    Step::Return(7),
    Step::Source(42),
    Step::Select(42),
    Step::Return(42),
    Step::Source(7),
    Step::Select(7),
    Step::Return(7),
    Step::Close,
];

#[derive(Clone, Debug)]
struct Audit {
    next: usize,
    pending: Option<(Step, Option<SelectionContext>)>,
    last_epoch: u64,
    selected: Option<SelectionContext>,
    failed: bool,
}
impl Audit {
    fn new(epoch: u64) -> Self {
        Self {
            next: 0,
            pending: None,
            last_epoch: epoch,
            selected: None,
            failed: false,
        }
    }
    fn begin(
        &mut self,
        step: Step,
        context: Option<SelectionContext>,
        admitted: bool,
        timely: bool,
    ) {
        let context_ok = match step {
            Step::Select(source) => context.is_some_and(|c| {
                c.key
                    == CharacterKey {
                        source_id: source,
                        object_id: 3333,
                    }
                    && c.epoch == self.last_epoch
                    && self.selected.is_none()
            }),
            Step::Return(source) => {
                context.is_some_and(|c| c.key.source_id == source && self.selected == Some(c))
            }
            Step::Source(_) | Step::Close => context.is_none() && self.selected.is_none(),
        };
        if self.pending.is_some()
            || ROUTE.get(self.next) != Some(&step)
            || !context_ok
            || !admitted
            || !timely
        {
            self.failed = true;
            return;
        }
        self.pending = Some((step, context));
    }
    fn confirm(&mut self, matched: bool, epoch: u64) {
        let Some((step, context)) = self.pending else {
            return;
        };
        if !matched {
            return;
        }
        if matches!(step, Step::Return(_) | Step::Source(_)) && epoch <= self.last_epoch {
            self.failed = true;
            return;
        }
        match step {
            Step::Select(_) => self.selected = context,
            Step::Return(_) | Step::Source(_) => {
                self.selected = None;
                self.last_epoch = epoch;
            }
            Step::Close => {}
        }
        self.next += 1;
        self.pending = None;
        eprintln!(
            "SOURCE_HWND_CONFIRMED step={step:?} next={} epoch={}",
            self.next, self.last_epoch
        );
    }
    fn complete(&self) -> bool {
        !self.failed && self.next == ROUTE.len() && self.pending.is_none()
    }
}

struct Observer {
    owner: SessionController,
    expected: [Expected; 2],
    audit: Arc<std::sync::Mutex<Audit>>,
    deadline: Instant,
}
impl LoginController for Observer {
    fn poll(&mut self) {
        self.owner.poll();
    }
    fn is_quiescent(&self) -> bool {
        self.owner.is_quiescent()
    }
    fn command(&mut self, command: LoginCommand) -> Result<(), String> {
        // Do not format LoginCommand: it can contain credentials/chat text.
        let step = match &command {
            LoginCommand::SelectCharacter(c) => Some((Step::Select(c.key.source_id), Some(*c))),
            LoginCommand::ReturnCharacters(c) => Some((Step::Return(c.key.source_id), Some(*c))),
            LoginCommand::SelectSource(s) => Some((Step::Source(*s), None)),
            LoginCommand::Shutdown => Some((Step::Close, None)),
            _ => None,
        };
        let read_only = matches!(
            &command,
            LoginCommand::ReadSkills | LoginCommand::ReadRecord
        ) || matches!(&command, LoginCommand::Equipment(q) if q.action == equipment::Action::Read);
        let timely = Instant::now() < self.deadline;
        let result = self.owner.command(command);
        let mut audit = self.audit.lock().unwrap();
        if let Some((step, context)) = step {
            audit.begin(step, context, result.is_ok(), timely);
            eprintln!(
                "SOURCE_HWND_COMMAND step={step:?} admitted={} failed={}",
                result.is_ok(),
                audit.failed
            );
        } else if !read_only || result.is_err() {
            audit.failed = true;
            eprintln!("SOURCE_HWND_UNEXPECTED_COMMAND admitted={}", result.is_ok());
        }
        result
    }
    fn snapshot(&self) -> LoginSnapshot {
        let view = self.owner.snapshot();
        let mut audit = self.audit.lock().unwrap();
        if let Some((step, context)) = audit.pending {
            let matched = match step {
                Step::Select(source) => self
                    .expected
                    .iter()
                    .find(|e| e.data.context.key.source_id == source)
                    .is_some_and(|e| context.is_some_and(|c| e.matches(&view, c))),
                Step::Return(source) | Step::Source(source) => cleared(&view, source),
                Step::Close => view.phase == LoginPhase::Disconnected && self.owner.is_quiescent(),
            };
            audit.confirm(matched, view.roster_epoch);
        }
        view
    }
}

#[test]
#[ignore = "INTERACTIVE independent v47 source HWND: reselect Sun, return, Moon, return, Sun, return, close; actual Main/isolated SQL, no injected input or combat acceptance"]
fn java_v47_main_sql_window_reselect_cross_source_and_close() {
    let d = Deadline(Instant::now() + Duration::from_secs(590));
    let fixtures = explicit_path("LITO_MODERN_TLS_TEST_FIXTURES");
    let shared = explicit_path("LITO_SHARED_RESOURCE_ROOT");
    let mut peer = JavaPeer::start_mode_lifetime(
        &fixtures,
        &d,
        "local.lito381.server.ModernSlowSqlTestPeer",
        "slow-window",
        Some("600000"),
    );
    let profile = profile(peer.endpoint, &shared, &private_root(&fixtures));
    let mut cs = std::array::from_fn(|_| Client {
        controller: SessionController::new_slow_development(profile.clone()),
        runtime: BattleRuntime::new(shared.clone()).unwrap(),
        surface: None,
        load: false,
        audio: SoundAudit::default(),
    });
    let mut expected = Vec::new();
    let mut contexts = Vec::new();
    for i in 0..2 {
        send(&mut cs, i, &d, || LoginCommand::Connect);
        wait(&mut cs, &d, "source window TLS fixture", |cs| {
            cs[i].view().phase == LoginPhase::LoginReady
        });
        send(&mut cs, i, &d, || LoginCommand::Login {
            account: "fixture-user".into(),
            password: "fixture-password".into(),
        });
        wait(&mut cs, &d, "source window fixture authentication", |cs| {
            cs[i].view().phase == LoginPhase::ChoosingSource
        });
        let context = select(&mut cs, i, &d, if i == 0 { 7 } else { 42 });
        expected.push(Expected::capture(&cs[i], context));
        contexts.push(context);
    }
    // The fixture's second connection retires before handoff, leaving Moon
    // available to the operator. No background actor drives owner navigation.
    send(&mut cs, 1, &d, || LoginCommand::Shutdown);
    wait(&mut cs, &d, "source window guest retires", |cs| {
        cs[1].controller.is_quiescent()
    });
    send(&mut cs, 0, &d, || {
        LoginCommand::ReturnCharacters(contexts[0])
    });
    wait(
        &mut cs,
        &d,
        "source window initial acknowledged return",
        |cs| cleared(&cs[0].view(), 7),
    );
    let audit = Arc::new(std::sync::Mutex::new(Audit::new(cs[0].view().roster_epoch)));
    for c in &mut cs {
        c.runtime.shutdown();
    }
    wait(&mut cs, &d, "source window setup renderers retired", |cs| {
        cs.iter().all(|c| c.runtime.is_quiescent())
    });
    let [owner, _guest] = cs;
    let deadline = (Instant::now() + Duration::from_secs(540)).min(d.0 - Duration::from_secs(10));
    eprintln!("SOURCE_HWND_HANDOFF source=7 cleared=true route=Sun-Moon-Sun");
    let result = crate::modern_lito::login_window::run_source_selection_test(
        &profile,
        Box::new(Observer {
            owner: owner.controller,
            expected: expected.try_into().ok().unwrap(),
            audit: audit.clone(),
            deadline,
        }),
        deadline,
    );
    peer.expect_successful_exit(&d);
    eprintln!("SOURCE_HWND_PEER_CLEANUP verified=true");
    result.unwrap();
    let audit = audit.lock().unwrap();
    eprintln!("SOURCE_HWND_AUDIT {audit:?}");
    assert!(
        audit.complete(),
        "native source reselection incomplete: {audit:?}"
    );
}

#[test]
fn source_window_audit_needs_ordered_commands_confirmed_fresh_epochs_and_operator_close() {
    let mut audit = Audit::new(2);
    let mut epoch = 2;
    for step in ROUTE {
        let context = match step {
            Step::Select(source) | Step::Return(source) => Some(SelectionContext {
                key: CharacterKey {
                    source_id: source,
                    object_id: 3333,
                },
                epoch,
            }),
            _ => None,
        };
        audit.begin(step, context, true, true);
        assert!(!audit.failed);
        let previous = audit.next;
        audit.confirm(false, epoch);
        assert_eq!(
            audit.next, previous,
            "input without authority cannot advance"
        );
        if matches!(step, Step::Return(_) | Step::Source(_)) {
            epoch += 1;
        }
        audit.confirm(true, epoch);
        if step != Step::Close {
            assert!(!audit.complete());
        }
    }
    assert!(audit.complete());
}

#[test]
fn source_window_audit_rejects_wrong_source_context_replay_and_deadline_shutdown() {
    let sun = SelectionContext { key: SUN, epoch: 2 };
    for (step, context, admitted, timely) in [
        (
            Step::Select(42),
            Some(SelectionContext {
                key: MOON,
                epoch: 2,
            }),
            true,
            true,
        ),
        (
            Step::Select(7),
            Some(SelectionContext {
                key: MOON,
                epoch: 2,
            }),
            true,
            true,
        ),
        (
            Step::Select(7),
            Some(SelectionContext { key: SUN, epoch: 1 }),
            true,
            true,
        ),
        (Step::Select(7), Some(sun), false, true),
        (Step::Close, None, true, false),
    ] {
        let mut audit = Audit::new(2);
        audit.begin(step, context, admitted, timely);
        assert!(audit.failed);
        assert!(!audit.complete());
    }
    let mut audit = Audit::new(2);
    audit.begin(Step::Select(7), Some(sun), true, true);
    audit.confirm(true, 2);
    audit.begin(Step::Return(7), Some(sun), true, true);
    audit.confirm(true, 2);
    assert!(audit.failed, "same roster epoch must not prove a return");
    assert!(!audit.complete());
}
