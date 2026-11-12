//! Bounded, byte-oriented reader for shared Lineage `list.spr` action data.
//!
//! Grammar evidence: `smooth_run/parse.rs`, `spr_action_sql.rs`, and
//! PakViewer's `SprListParser.cs` / `SprListModels.cs`. Unlike the SQL exporter,
//! this retains every frame and ordered event. Labels stay in their original
//! encoding (usually Big5); the parser never guesses UTF-8. Attribute bodies
//! whose execution semantics are unknown remain opaque, not invented actions.
//! This is data parsing, not a renderer or a native-client memory interface.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Clone, Copy, Debug)]
pub struct ActionLimits {
    pub max_bytes: usize,
    pub max_line_bytes: usize,
    pub max_sprites: usize,
    pub max_actions: usize,
    pub max_frames: usize,
    pub max_frames_per_action: usize,
    pub max_events_per_frame: usize,
    pub max_reference_depth: usize,
}

impl Default for ActionLimits {
    fn default() -> Self {
        Self {
            max_bytes: 32 * 1024 * 1024,
            max_line_bytes: 256 * 1024,
            max_sprites: 100_000,
            max_actions: 1_000_000,
            max_frames: 4_000_000,
            max_frames_per_action: 16_384,
            max_events_per_frame: 32,
            max_reference_depth: 64,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionError {
    pub line: usize,
    pub kind: ActionErrorKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActionErrorKind {
    Limit(&'static str),
    Malformed(&'static str),
    Unsupported(&'static str),
    DuplicateSprite(u32),
    DuplicateAction(u32, ActionKey),
    MissingSprite(u32),
    MissingAction(u32, ActionKey),
    CyclicReference(u32, ActionKey),
    QuarantinedAction(u32, ActionKey),
    UninterpretedEvents(u32, ActionKey),
    QuarantinedSprite(u32),
    FrameCount { declared: u32, actual: usize },
    SpriteCount { declared: u32, actual: usize },
}

impl fmt::Display for ActionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "list.spr line {}: {:?}", self.line, self.kind)
    }
}

impl std::error::Error for ActionError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ActionKey {
    pub id: u32,
    /// Explicit `0-1.RunL` etc. Kept separate; never remapped to slots 98/99.
    pub variant: Option<u32>,
}

impl ActionKey {
    pub const fn new(id: u32) -> Self {
        Self { id, variant: None }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Directionality {
    Undirected,
    EightWay,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FrameEvent {
    Hit,
    Sound(u32),
    Overlay(u32),
    Effect(u32),
    /// Observed `^number` in shared list.spr (e.g. sprite 155), omitted by
    /// the reference parser. Syntax is retained; execution is UNVERIFIED.
    UninterpretedNumeric {
        marker: u8,
        value: u32,
    },
    /// The source names `>` SkipFrame. Preserve the event without guessing
    /// which later frame or movement instruction it might skip.
    Skip,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionFrame {
    /// Image group before the dot, NOT the decoded SPR subframe index.
    pub image_id: u32,
    pub subframe: u32,
    pub duration_ticks: u32,
    pub events: Vec<FrameEvent>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Animation {
    pub label_bytes: Vec<u8>,
    pub directionality: Directionality,
    /// Sprite-scoped 110 scalar active at this definition; default 24.
    pub frames_per_second: u32,
    pub frames: Vec<ActionFrame>,
}

impl Animation {
    pub fn has_uninterpreted_events(&self) -> bool {
        self.frames
            .iter()
            .flat_map(|frame| &frame.events)
            .any(|event| {
                matches!(
                    event,
                    FrameEvent::UninterpretedNumeric { .. } | FrameEvent::Skip
                )
            })
    }

    pub fn total_ticks(&self) -> u64 {
        self.frames
            .iter()
            .map(|f| u64::from(f.duration_ticks))
            .sum()
    }

    /// Exact rational duration; callers can accumulate ticks without rounding
    /// every frame to integer milliseconds.
    pub fn duration_ratio(&self) -> (u64, u32) {
        (self.total_ticks(), self.frames_per_second)
    }

    /// Asset naming evidence: PakViewer `ucSprActionViewer.cs:645`.
    /// Graphics identity is explicit because an action reference alone does
    /// not establish whether the caller or the definition owns the image bank.
    pub fn frame_asset(
        &self,
        graphics_id: u32,
        frame: usize,
        heading: u8,
    ) -> Result<FrameAsset, ActionError> {
        if self.has_uninterpreted_events() {
            return Err(error(
                0,
                ActionErrorKind::Unsupported("uninterpreted event playback"),
            ));
        }
        if heading > 7 {
            return Err(error(0, ActionErrorKind::Malformed("heading outside 0..7")));
        }
        let value = self
            .frames
            .get(frame)
            .ok_or_else(|| error(0, ActionErrorKind::Malformed("frame index out of range")))?;
        let direction = if self.directionality == Directionality::EightWay {
            u32::from(heading)
        } else {
            0
        };
        let image_id = value
            .image_id
            .checked_add(direction)
            .ok_or_else(|| error(0, ActionErrorKind::Malformed("directional image overflow")))?;
        Ok(FrameAsset {
            graphics_id,
            image_id,
            subframe: value.subframe,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameAsset {
    pub graphics_id: u32,
    pub image_id: u32,
    pub subframe: u32,
}

impl FrameAsset {
    pub fn file_name(&self) -> String {
        format!("{}-{}.spr", self.graphics_id, self.image_id)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActionDefinition {
    Animation(Animation),
    /// Inspection-only entry whose syntax is not yet verified. Complete raw
    /// command and source line are preserved; resolving it or an alias fails.
    Quarantined {
        raw_bytes: Vec<u8>,
        reason: ActionError,
    },
    /// `action_id=other_sprite_id` inherits the SAME action key. Resolution
    /// retains definition provenance and does not invent cross-action aliases.
    Reference {
        sprite_id: u32,
        label_bytes: Vec<u8>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpaqueAttribute {
    pub key: ActionKey,
    pub label_bytes: Vec<u8>,
    pub body_bytes: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct SpriteActions {
    pub id: u32,
    pub image_count: Option<u32>,
    /// `#id count=gfx` or `#id=gfx`: direct graphics bank identity only.
    /// No implicit action inheritance or recursive graphics dereference.
    pub graphics_id: u32,
    /// Inspection preserves an unverified header but refuses all its actions.
    pub header_issue: Option<ActionError>,
    pub header_bytes: Vec<u8>,
    pub label_bytes: Vec<u8>,
    pub actions: BTreeMap<ActionKey, ActionDefinition>,
    pub attributes: Vec<OpaqueAttribute>,
}

#[derive(Clone, Debug)]
pub struct SpriteActionCatalog {
    /// Header: declared sprite count and two uninterpreted integers. The count
    /// is checked but never used to preallocate or assume contiguous IDs.
    pub header: Option<[u32; 3]>,
    pub sprites: BTreeMap<u32, SpriteActions>,
    reference_depth: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct ResolvedAction<'a> {
    pub requested_sprite_id: u32,
    pub definition_sprite_id: u32,
    pub key: ActionKey,
    pub animation: &'a Animation,
}

impl SpriteActionCatalog {
    pub fn parse(bytes: &[u8], limits: ActionLimits) -> Result<Self, ActionError> {
        Parser::new(limits, false)?.parse(bytes)
    }

    /// Keep bounded unverified action spans for inspection. A missing closing
    /// delimiter quarantines only the current physical line, never a later
    /// command or a fabricated multi-line animation.
    /// Structural errors, bounds, duplicate IDs, missing references and cycles
    /// still fail. This is not a claim that quarantined animations can play.
    pub fn parse_for_inspection(bytes: &[u8], limits: ActionLimits) -> Result<Self, ActionError> {
        Parser::new(limits, true)?.parse(bytes)
    }

    pub fn resolve_action(
        &self,
        sprite_id: u32,
        key: ActionKey,
    ) -> Result<ResolvedAction<'_>, ActionError> {
        let mut current = sprite_id;
        let mut visited = BTreeSet::new();
        for _ in 0..=self.reference_depth {
            if !visited.insert(current) {
                return Err(error(0, ActionErrorKind::CyclicReference(current, key)));
            }
            let sprite = self
                .sprites
                .get(&current)
                .ok_or_else(|| error(0, ActionErrorKind::MissingSprite(current)))?;
            if sprite.header_issue.is_some()
                || sprite.actions.iter().any(|(key, definition)| {
                    key.id >= 100 && matches!(definition, ActionDefinition::Quarantined { .. })
                })
            {
                return Err(error(0, ActionErrorKind::QuarantinedSprite(current)));
            }
            match sprite
                .actions
                .get(&key)
                .ok_or_else(|| error(0, ActionErrorKind::MissingAction(current, key)))?
            {
                ActionDefinition::Animation(animation) => {
                    if animation.has_uninterpreted_events() {
                        return Err(error(0, ActionErrorKind::UninterpretedEvents(current, key)));
                    }
                    return Ok(ResolvedAction {
                        requested_sprite_id: sprite_id,
                        definition_sprite_id: current,
                        key,
                        animation,
                    });
                }
                ActionDefinition::Reference { sprite_id, .. } => current = *sprite_id,
                ActionDefinition::Quarantined { .. } => {
                    return Err(error(0, ActionErrorKind::QuarantinedAction(current, key)));
                }
            }
        }
        Err(error(0, ActionErrorKind::Limit("reference depth")))
    }
}

fn error(line: usize, kind: ActionErrorKind) -> ActionError {
    ActionError { line, kind }
}
fn trim(bytes: &[u8]) -> &[u8] {
    bytes.trim_ascii()
}

struct Parser {
    limits: ActionLimits,
    actions: usize,
    frames: usize,
    quarantine_animations: bool,
}

impl Parser {
    fn new(limits: ActionLimits, quarantine_animations: bool) -> Result<Self, ActionError> {
        if limits.max_bytes == 0
            || limits.max_bytes > 64 * 1024 * 1024
            || limits.max_line_bytes == 0
            || limits.max_line_bytes > 1024 * 1024
            || limits.max_sprites == 0
            || limits.max_sprites > 250_000
            || limits.max_actions == 0
            || limits.max_actions > 2_000_000
            || limits.max_frames == 0
            || limits.max_frames > 8_000_000
            || limits.max_frames_per_action == 0
            || limits.max_frames_per_action > 65_536
            || limits.max_events_per_frame == 0
            || limits.max_events_per_frame > 128
            || limits.max_reference_depth == 0
            || limits.max_reference_depth > 256
        {
            return Err(error(0, ActionErrorKind::Limit("invalid parser limits")));
        }
        Ok(Self {
            limits,
            actions: 0,
            frames: 0,
            quarantine_animations,
        })
    }

    fn parse(mut self, bytes: &[u8]) -> Result<SpriteActionCatalog, ActionError> {
        if bytes.len() > self.limits.max_bytes {
            return Err(error(0, ActionErrorKind::Limit("input bytes")));
        }
        if bytes.contains(&0) {
            return Err(error(0, ActionErrorKind::Malformed("embedded NUL")));
        }
        let mut catalog = SpriteActionCatalog {
            header: None,
            sprites: BTreeMap::new(),
            reference_depth: self.limits.max_reference_depth,
        };
        let mut current: Option<SpriteActions> = None;
        let mut fps = 24;
        for (line_index, raw) in bytes.split(|b| *b == b'\n').enumerate() {
            let line = line_index + 1;
            if raw.len() > self.limits.max_line_bytes {
                return Err(error(line, ActionErrorKind::Limit("line bytes")));
            }
            let data = trim(raw);
            if data.is_empty() || data.starts_with(b";") || data.starts_with(b"//") {
                continue;
            }
            if data.starts_with(b"#") {
                if data.get(1).is_none_or(|b| !b.is_ascii_digit()) {
                    return Err(error(line, ActionErrorKind::Malformed("sprite header")));
                }
                if let Some(sprite) = current.take() {
                    insert_sprite(&mut catalog, sprite, line, self.limits)?;
                }
                let (mut sprite, remaining) = self.header(data, line)?;
                fps = 24;
                self.commands(remaining, line, &mut sprite, &mut fps)?;
                current = Some(sprite);
            } else if let Some(sprite) = current.as_mut() {
                self.commands(data, line, sprite, &mut fps)?;
            } else if catalog.header.is_none() {
                let mut c = Cursor::new(data, line);
                let values = [c.number()?, c.spaced_number()?, c.spaced_number()?];
                if values[0] as usize > self.limits.max_sprites {
                    return Err(error(line, ActionErrorKind::Limit("declared sprites")));
                }
                c.ws();
                if !c.done() {
                    return Err(error(line, ActionErrorKind::Malformed("file header tail")));
                }
                catalog.header = Some(values);
            } else {
                return Err(error(
                    line,
                    ActionErrorKind::Malformed("command before sprite"),
                ));
            }
        }
        if let Some(sprite) = current {
            insert_sprite(&mut catalog, sprite, 0, self.limits)?;
        }
        if catalog.sprites.is_empty() {
            return Err(error(0, ActionErrorKind::Malformed("no sprite entries")));
        }
        if let Some([declared, _, _]) = catalog.header {
            if declared as usize != catalog.sprites.len() {
                return Err(error(
                    1,
                    ActionErrorKind::SpriteCount {
                        declared,
                        actual: catalog.sprites.len(),
                    },
                ));
            }
        }
        // Validate all references, including forward references, without recursive
        // calls or cloning frame sequences. Missing/cyclic aliases fail closed.
        for (&id, sprite) in &catalog.sprites {
            for (&key, definition) in &sprite.actions {
                if matches!(definition, ActionDefinition::Reference { .. }) {
                    if let Err(err) = catalog.resolve_action(id, key) {
                        if !self.quarantine_animations
                            || !matches!(
                                err.kind,
                                ActionErrorKind::QuarantinedAction(..)
                                    | ActionErrorKind::QuarantinedSprite(..)
                                    | ActionErrorKind::UninterpretedEvents(..)
                            )
                        {
                            return Err(err);
                        }
                    }
                }
            }
        }
        Ok(catalog)
    }

    fn header<'a>(
        &self,
        data: &'a [u8],
        line: usize,
    ) -> Result<(SpriteActions, &'a [u8]), ActionError> {
        let mut c = Cursor::new(&data[1..], line);
        let id = c.number()?;
        let (image_count, graphics_id) = if c.take(b'=') {
            (None, c.number()?)
        } else {
            let count = c.spaced_number()?;
            let gfx = if c.take(b'=') { c.number()? } else { id };
            (Some(count), gfx)
        };
        let header_issue = if !c.done() && !c.peek().is_some_and(|b| b.is_ascii_whitespace()) {
            let issue = error(
                line,
                ActionErrorKind::Unsupported("sprite header separator unverified"),
            );
            if !self.quarantine_animations {
                return Err(issue);
            }
            Some(issue)
        } else {
            None
        };
        c.ws();
        let tail = c.rest();
        let start = find_named_command(tail).unwrap_or(tail.len());
        let label = trim(&tail[..start]).to_vec();
        Ok((
            SpriteActions {
                id,
                image_count,
                graphics_id,
                header_issue,
                header_bytes: data.to_vec(),
                label_bytes: label,
                actions: BTreeMap::new(),
                attributes: Vec::new(),
            },
            &tail[start..],
        ))
    }

    fn commands(
        &mut self,
        bytes: &[u8],
        line: usize,
        sprite: &mut SpriteActions,
        fps: &mut u32,
    ) -> Result<(), ActionError> {
        let mut c = Cursor::new(bytes, line);
        loop {
            c.ws();
            if c.done() || c.rest().starts_with(b";") || c.rest().starts_with(b"//") {
                return Ok(());
            }
            let command_start = c.pos;
            let id = c.number()?;
            let variant = if c.take(b'-') {
                Some(c.number()?)
            } else {
                None
            };
            let key = ActionKey { id, variant };
            if c.take(b'=') {
                let target = c.number()?;
                if !c.done() && !c.peek().is_some_and(|b| b.is_ascii_whitespace()) {
                    return Err(error(line, ActionErrorKind::Malformed("reference tail")));
                }
                c.ws();
                let tail = c.rest();
                let next = find_named_command(tail).unwrap_or(tail.len());
                self.add(
                    sprite,
                    key,
                    ActionDefinition::Reference {
                        sprite_id: target,
                        label_bytes: trim(&tail[..next]).to_vec(),
                    },
                    line,
                )?;
                c.pos += next;
            } else if c.take(b'.') {
                let name_start = c.pos;
                while c.peek().is_some_and(|b| b != b'(' && b != b')') {
                    c.pos += 1;
                }
                let label = trim(&c.data[name_start..c.pos]).to_vec();
                if !c.take(b'(') {
                    if self.quarantine_animations && c.take(b')') {
                        self.add(
                            sprite,
                            key,
                            ActionDefinition::Quarantined {
                                raw_bytes: c.data[command_start..c.pos].to_vec(),
                                reason: error(
                                    line,
                                    ActionErrorKind::Unsupported(
                                        "action opener not found before closing delimiter",
                                    ),
                                ),
                            },
                            line,
                        )?;
                        continue;
                    }
                    return Err(error(
                        line,
                        ActionErrorKind::Malformed("missing action opener"),
                    ));
                }
                let body_start = c.pos;
                while c.peek().is_some_and(|b| b != b')' && b != b'(') {
                    c.pos += 1;
                }
                if c.done() && self.quarantine_animations && id != 110 {
                    self.add(
                        sprite,
                        key,
                        ActionDefinition::Quarantined {
                            raw_bytes: c.data[command_start..c.pos].to_vec(),
                            reason: error(
                                line,
                                ActionErrorKind::Unsupported(
                                    "action closing delimiter absent on physical line",
                                ),
                            ),
                        },
                        line,
                    )?;
                    continue;
                }
                if c.peek() == Some(b'(') && self.quarantine_animations && id != 110 {
                    while c.peek().is_some_and(|b| b != b')') {
                        c.pos += 1;
                    }
                    c.expect(b')')?;
                    self.add(
                        sprite,
                        key,
                        ActionDefinition::Quarantined {
                            raw_bytes: c.data[command_start..c.pos].to_vec(),
                            reason: error(
                                line,
                                ActionErrorKind::Unsupported(
                                    "nested action opener; syntax unverified",
                                ),
                            ),
                        },
                        line,
                    )?;
                    continue;
                }
                let body = trim(&c.data[body_start..c.pos]);
                c.expect(b')')?;
                if c.peek() == Some(b')') && self.quarantine_animations && id != 110 {
                    while c.take(b')') {}
                    self.add(
                        sprite,
                        key,
                        ActionDefinition::Quarantined {
                            raw_bytes: c.data[command_start..c.pos].to_vec(),
                            reason: error(
                                line,
                                ActionErrorKind::Unsupported(
                                    "extra closing delimiter; syntax unverified",
                                ),
                            ),
                        },
                        line,
                    )?;
                    continue;
                }
                if id == 110 && variant.is_none() {
                    let mut scalar = Cursor::new(body, line);
                    let value = scalar.number()?;
                    scalar.ws();
                    if !scalar.done() {
                        return Err(error(
                            line,
                            ActionErrorKind::Unsupported("non-scalar 110 framerate"),
                        ));
                    }
                    set_fps(fps, value, line)?;
                    self.attribute(sprite, key, label, body, line)?;
                } else if id != 100 && body.contains(&b',') {
                    let definition = match self.animation(body, label, *fps, line, true) {
                        Ok(animation) => ActionDefinition::Animation(animation),
                        Err(reason)
                            if self.quarantine_animations
                                && matches!(
                                    reason.kind,
                                    ActionErrorKind::Malformed(_)
                                        | ActionErrorKind::Unsupported(_)
                                        | ActionErrorKind::FrameCount { .. }
                                ) =>
                        {
                            ActionDefinition::Quarantined {
                                raw_bytes: c.data[command_start..c.pos].to_vec(),
                                reason,
                            }
                        }
                        Err(err) => return Err(err),
                    };
                    self.add(sprite, key, definition, line)?;
                } else {
                    if id < 100 {
                        let reason = error(
                            line,
                            ActionErrorKind::Unsupported(
                                "animation header comma absent; syntax unverified",
                            ),
                        );
                        if !self.quarantine_animations {
                            return Err(reason);
                        }
                        self.add(
                            sprite,
                            key,
                            ActionDefinition::Quarantined {
                                raw_bytes: c.data[command_start..c.pos].to_vec(),
                                reason,
                            },
                            line,
                        )?;
                    } else {
                        self.attribute(sprite, key, label, body, line)?;
                    }
                }
            } else if self.quarantine_animations
                && id != 110
                && !c.peek().is_some_and(|b| b.is_ascii_whitespace())
            {
                while c.peek().is_some_and(|b| b != b')') {
                    c.pos += 1;
                }
                c.take(b')');
                self.add(
                    sprite,
                    key,
                    ActionDefinition::Quarantined {
                        raw_bytes: c.data[command_start..c.pos].to_vec(),
                        reason: error(
                            line,
                            ActionErrorKind::Unsupported("command separator unverified"),
                        ),
                    },
                    line,
                )?;
            } else {
                if variant.is_some() {
                    return Err(error(line, ActionErrorKind::Unsupported("compact variant")));
                }
                c.require_space()?;
                if id == 110 {
                    let value = c.number()?;
                    set_fps(fps, value, line)?;
                    self.attribute(sprite, key, Vec::new(), value.to_string().as_bytes(), line)?;
                } else if id >= 100 {
                    let start = c.pos;
                    let count = match id {
                        100 => 4,
                        101 | 102 | 104 | 106 | 107 | 108 | 109 | 111 | 128 => 1,
                        105 => {
                            let n = c.number()? as usize;
                            if n > self.limits.max_frames_per_action {
                                return Err(error(
                                    line,
                                    ActionErrorKind::Limit("attach attributes"),
                                ));
                            }
                            for _ in 0..n {
                                c.spaced_number()?;
                            }
                            0
                        }
                        _ => {
                            return Err(error(
                                line,
                                ActionErrorKind::Unsupported("unknown compact attribute boundary"),
                            ));
                        }
                    };
                    for n in 0..count {
                        if n > 0 {
                            c.require_space()?;
                        }
                        c.number()?;
                    }
                    self.attribute(sprite, key, Vec::new(), &c.data[start..c.pos], line)?;
                } else {
                    let directional = c.number()?;
                    let count = c.spaced_number()?;
                    let frames = self.frames(&mut c, count, line, false)?;
                    self.add(
                        sprite,
                        key,
                        ActionDefinition::Animation(Animation {
                            label_bytes: Vec::new(),
                            directionality: directionality(directional, line)?,
                            frames_per_second: *fps,
                            frames,
                        }),
                        line,
                    )?;
                }
                if !c.done() && !c.peek().is_some_and(|b| b.is_ascii_whitespace()) {
                    return Err(error(
                        line,
                        ActionErrorKind::Malformed("compact command separator"),
                    ));
                }
            }
        }
    }

    fn animation(
        &mut self,
        body: &[u8],
        label: Vec<u8>,
        fps: u32,
        line: usize,
        standard: bool,
    ) -> Result<Animation, ActionError> {
        let mut c = Cursor::new(body, line);
        let directional = directionality(c.number()?, line)?;
        let count = c.spaced_number()?;
        c.ws();
        c.expect(b',')?;
        let frames = self.frames(&mut c, count, line, standard)?;
        c.ws();
        if !c.done() {
            return Err(error(
                line,
                ActionErrorKind::Malformed("trailing frame tokens"),
            ));
        }
        Ok(Animation {
            label_bytes: label,
            directionality: directional,
            frames_per_second: fps,
            frames,
        })
    }

    fn frames(
        &mut self,
        c: &mut Cursor<'_>,
        count: u32,
        line: usize,
        standard: bool,
    ) -> Result<Vec<ActionFrame>, ActionError> {
        if count as usize > self.limits.max_frames_per_action
            || count as usize > self.limits.max_frames.saturating_sub(self.frames)
        {
            return Err(error(line, ActionErrorKind::Limit("frames")));
        }
        let mut frames = Vec::new();
        for n in 0..count {
            if !standard || n > 0 {
                c.require_space()?;
            } else {
                c.ws();
            }
            if c.done() {
                return Err(error(
                    line,
                    ActionErrorKind::FrameCount {
                        declared: count,
                        actual: frames.len(),
                    },
                ));
            }
            let image_id = c.number()?;
            let subframe = if standard {
                // Native 43E84D calls numeric reader 43DCC0 three times.
                // That reader skips punctuation before each next integer;
                // original #871 therefore accepts the image/subframe comma
                // in `0,0:1`. Admit this verified separator directly, without
                // rewriting source bytes or relaxing command/event syntax.
                // The same 43DCC0 skip loop (4271D92F, full-disassembly SHA
                // 2A3A6E10) also reads list.spr 523C61AD line 614
                // `0:0:4` as image 0, subframe 0, ticks 4. Only this
                // image/subframe position admits `:`; the ticks separator
                // stays `:` and other punctuation (`-1.`, `0:3.2`) stays
                // rejected or quarantined.
                if matches!(c.peek(), Some(b',' | b':')) { c.pos += 1; } else { c.expect(b'.')?; }
                c.number()?
            } else {
                c.spaced_number()?
            };
            let duration_ticks = if standard {
                c.expect(b':')?;
                c.number()?
            } else {
                c.spaced_number()?
            };
            // Real shared list.spr attack sequences contain `0.0:0` setup
            // frames. Retain their events and zero time; the player must bound
            // zero-time advancement by the already bounded frame count.
            let mut events = Vec::new();
            while let Some(mark) = c.peek() {
                if mark.is_ascii_whitespace() {
                    break;
                }
                if events.len() >= self.limits.max_events_per_frame {
                    return Err(error(line, ActionErrorKind::Limit("frame events")));
                }
                c.pos += 1;
                let event = match mark {
                    b'!' => FrameEvent::Hit,
                    b'>' => FrameEvent::Skip,
                    b'[' => FrameEvent::Sound(c.number()?),
                    b']' => FrameEvent::Overlay(c.number()?),
                    b'<' => FrameEvent::Effect(c.number()?),
                    b'^' => FrameEvent::UninterpretedNumeric {
                        marker: mark,
                        value: c.number()?,
                    },
                    _ => {
                        return Err(error(
                            line,
                            ActionErrorKind::Unsupported("unknown frame modifier"),
                        ));
                    }
                };
                events.push(event);
            }
            frames.push(ActionFrame {
                image_id,
                subframe,
                duration_ticks,
                events,
            });
        }
        self.frames += frames.len();
        Ok(frames)
    }

    fn reserve_command(&mut self, line: usize) -> Result<(), ActionError> {
        if self.actions >= self.limits.max_actions {
            return Err(error(line, ActionErrorKind::Limit("commands")));
        }
        self.actions += 1;
        Ok(())
    }
    fn add(
        &mut self,
        sprite: &mut SpriteActions,
        key: ActionKey,
        action: ActionDefinition,
        line: usize,
    ) -> Result<(), ActionError> {
        self.reserve_command(line)?;
        if sprite.actions.contains_key(&key) {
            return Err(error(
                line,
                ActionErrorKind::DuplicateAction(sprite.id, key),
            ));
        }
        sprite.actions.insert(key, action);
        Ok(())
    }
    fn attribute(
        &mut self,
        sprite: &mut SpriteActions,
        key: ActionKey,
        label_bytes: Vec<u8>,
        body: &[u8],
        line: usize,
    ) -> Result<(), ActionError> {
        self.reserve_command(line)?;
        sprite.attributes.push(OpaqueAttribute {
            key,
            label_bytes,
            body_bytes: body.to_vec(),
        });
        Ok(())
    }
}

fn insert_sprite(
    catalog: &mut SpriteActionCatalog,
    sprite: SpriteActions,
    line: usize,
    limits: ActionLimits,
) -> Result<(), ActionError> {
    if catalog.sprites.len() >= limits.max_sprites {
        return Err(error(line, ActionErrorKind::Limit("sprites")));
    }
    if catalog.sprites.contains_key(&sprite.id) {
        return Err(error(line, ActionErrorKind::DuplicateSprite(sprite.id)));
    }
    catalog.sprites.insert(sprite.id, sprite);
    Ok(())
}
fn set_fps(fps: &mut u32, value: u32, line: usize) -> Result<(), ActionError> {
    if value == 0 || value > 1000 {
        return Err(error(
            line,
            ActionErrorKind::Unsupported("framerate outside 1..1000"),
        ));
    }
    *fps = value;
    Ok(())
}
fn directionality(value: u32, line: usize) -> Result<Directionality, ActionError> {
    match value {
        0 => Ok(Directionality::Undirected),
        1 => Ok(Directionality::EightWay),
        _ => Err(error(
            line,
            ActionErrorKind::Unsupported("directionality outside 0/1"),
        )),
    }
}

/// Find only a token-boundary `id[ -variant ].` or `id=target`. Label digits
/// inside `Skill_2026` are never commands. Big5 trail bytes cannot be ASCII
/// digits; label bytes are otherwise uninterpreted.
fn find_named_command(bytes: &[u8]) -> Option<usize> {
    for start in 0..bytes.len() {
        if !bytes[start].is_ascii_digit() || (start > 0 && !bytes[start - 1].is_ascii_whitespace())
        {
            continue;
        }
        let mut pos = start;
        while bytes.get(pos).is_some_and(u8::is_ascii_digit) {
            pos += 1;
        }
        if bytes.get(pos) == Some(&b'-') {
            pos += 1;
            while bytes.get(pos).is_some_and(u8::is_ascii_digit) {
                pos += 1;
            }
        }
        if matches!(bytes.get(pos), Some(b'.' | b'=')) {
            return Some(start);
        }
    }
    None
}

struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
    line: usize,
}
impl<'a> Cursor<'a> {
    fn new(data: &'a [u8], line: usize) -> Self {
        Self { data, pos: 0, line }
    }
    fn rest(&self) -> &'a [u8] {
        &self.data[self.pos..]
    }
    fn done(&self) -> bool {
        self.pos == self.data.len()
    }
    fn peek(&self) -> Option<u8> {
        self.data.get(self.pos).copied()
    }
    fn ws(&mut self) {
        while self.peek().is_some_and(|b| b.is_ascii_whitespace()) {
            self.pos += 1;
        }
    }
    fn require_space(&mut self) -> Result<(), ActionError> {
        if !self.peek().is_some_and(|b| b.is_ascii_whitespace()) {
            return Err(error(
                self.line,
                ActionErrorKind::Malformed("missing whitespace"),
            ));
        }
        self.ws();
        Ok(())
    }
    fn take(&mut self, byte: u8) -> bool {
        if self.peek() == Some(byte) {
            self.pos += 1;
            true
        } else {
            false
        }
    }
    fn expect(&mut self, byte: u8) -> Result<(), ActionError> {
        if self.take(byte) {
            Ok(())
        } else {
            Err(error(
                self.line,
                ActionErrorKind::Malformed("missing delimiter"),
            ))
        }
    }
    fn number(&mut self) -> Result<u32, ActionError> {
        let start = self.pos;
        let mut value = 0u32;
        while let Some(byte @ b'0'..=b'9') = self.peek() {
            value = value
                .checked_mul(10)
                .and_then(|v| v.checked_add(u32::from(byte - b'0')))
                .ok_or_else(|| error(self.line, ActionErrorKind::Malformed("integer overflow")))?;
            self.pos += 1;
        }
        if self.pos == start {
            return Err(error(
                self.line,
                ActionErrorKind::Malformed("expected unsigned integer"),
            ));
        }
        Ok(value)
    }
    fn spaced_number(&mut self) -> Result<u32, ActionError> {
        self.require_space()?;
        self.number()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parse(bytes: &[u8]) -> Result<SpriteActionCatalog, ActionError> {
        SpriteActionCatalog::parse(bytes, ActionLimits::default())
    }

    #[test]
    fn retains_full_frames_events_and_graphics_identity() {
        let catalog = parse(b"1 0 41210\r\n#0 312=3225 prince\r\n110.framerate(30)\r\n0.walk(1 2,0.0:4<479 8.3:2![7]8<9)\r\n").unwrap();
        let resolved = catalog.resolve_action(0, ActionKey::new(0)).unwrap();
        assert_eq!(catalog.header, Some([1, 0, 41210]));
        assert_eq!(catalog.sprites[&0].graphics_id, 3225);
        assert_eq!(resolved.animation.duration_ratio(), (6, 30));
        assert_eq!(
            resolved.animation.frames[1].events,
            vec![
                FrameEvent::Hit,
                FrameEvent::Sound(7),
                FrameEvent::Overlay(8),
                FrameEvent::Effect(9)
            ]
        );
        let asset = resolved.animation.frame_asset(3225, 1, 4).unwrap();
        assert_eq!(asset.file_name(), "3225-12.spr");
        assert_eq!(asset.subframe, 3);
    }

    #[test]
    fn compact_and_parenthesized_frames_agree() {
        let a = parse(b"#7 80\n110 25 0 1 2 8 0 3! 8 1 4[9\n").unwrap();
        let b = parse(b"#7 80\n110.(25) 0.(1 2,8.0:3! 8.1:4[9)\n").unwrap();
        assert_eq!(
            a.resolve_action(7, ActionKey::new(0)).unwrap().animation,
            b.resolve_action(7, ActionKey::new(0)).unwrap().animation
        );
    }

    #[test]
    fn preserves_big5_labels_without_lossy_utf8() {
        let catalog =
            parse(b"#7 80 \xb4\xfa\xb8\xd5 Skill_2026 0.\xa8\xab\xb8\xf4(0 1,0.0:2)\n").unwrap();
        assert_eq!(
            catalog.sprites[&7].label_bytes,
            b"\xb4\xfa\xb8\xd5 Skill_2026"
        );
        let animation = catalog
            .resolve_action(7, ActionKey::new(0))
            .unwrap()
            .animation;
        assert_eq!(animation.label_bytes, b"\xa8\xab\xb8\xf4");
        assert_eq!(animation.frame_asset(7, 0, 7).unwrap().image_id, 0);
    }

    #[test]
    fn forward_references_preserve_definition_framerate_and_events() {
        let catalog =
            parse(b"#2 80\n4=1 copied\n#1 80\n110.framerate(30)\n4.attack(1 1,9.0:5!)\n").unwrap();
        let action = catalog.resolve_action(2, ActionKey::new(4)).unwrap();
        assert_eq!(action.definition_sprite_id, 1);
        assert_eq!(action.requested_sprite_id, 2);
        assert_eq!(action.animation.frames_per_second, 30);
        assert_eq!(action.animation.frames[0].events, vec![FrameEvent::Hit]);
    }

    #[test]
    fn variants_inline_and_fps_scope_are_preserved() {
        let catalog = parse(b"#1 80 knight 0.walk(1 1,0.0:2) 110.framerate(30) 0-1.RunL(1 1,8.0:2)\n#2=1\n0.idle(0 1,0.0:3)\n").unwrap();
        assert_eq!(
            catalog
                .resolve_action(1, ActionKey::new(0))
                .unwrap()
                .animation
                .frames_per_second,
            24
        );
        assert_eq!(
            catalog
                .resolve_action(
                    1,
                    ActionKey {
                        id: 0,
                        variant: Some(1)
                    }
                )
                .unwrap()
                .animation
                .frames_per_second,
            30
        );
        assert_eq!(
            catalog
                .resolve_action(2, ActionKey::new(0))
                .unwrap()
                .animation
                .frames_per_second,
            24
        );
        assert_eq!(catalog.sprites[&2].graphics_id, 1);
    }

    #[test]
    fn unknown_attributes_stay_opaque_and_do_not_become_actions() {
        let catalog =
            parse(b"#1 80\n999.mystery(raw -1) 100.switch(1 0,) 101.shadow(4)\n").unwrap();
        assert!(catalog.sprites[&1].actions.is_empty());
        assert_eq!(catalog.sprites[&1].attributes[0].body_bytes, b"raw -1");
        assert!(matches!(
            parse(b"#1 80\n999 12\n").unwrap_err().kind,
            ActionErrorKind::Unsupported(_)
        ));
    }

    #[test]
    fn rejects_cyclic_missing_and_excessive_reference_depth() {
        assert!(matches!(
            parse(b"#1 80\n0=2\n#2 80\n0=1\n").unwrap_err().kind,
            ActionErrorKind::CyclicReference(..)
        ));
        assert!(matches!(
            parse(b"#1 80\n0=1\n").unwrap_err().kind,
            ActionErrorKind::CyclicReference(..)
        ));
        assert!(matches!(
            parse(b"#1 80\n0=9\n").unwrap_err().kind,
            ActionErrorKind::MissingSprite(9)
        ));
        assert!(matches!(
            parse(b"#1 80\n0=2\n#2 80\n1.a(0 1,0.0:1)\n")
                .unwrap_err()
                .kind,
            ActionErrorKind::MissingAction(..)
        ));
        let limits = ActionLimits {
            max_reference_depth: 1,
            ..ActionLimits::default()
        };
        assert!(matches!(
            SpriteActionCatalog::parse(b"#1 1\n0=2\n#2 1\n0=3\n#3 1\n0.a(0 1,0.0:1)\n", limits)
                .unwrap_err()
                .kind,
            ActionErrorKind::Limit("reference depth")
        ));
    }

    #[test]
    fn rejects_malformed_frames_counts_duplicates_and_overflow() {
        for input in [
            b"#1 80\n0.a(1 2,0.0:1)".as_slice(),
            b"#1 80\n0.a(1 1,0.0:1 0.1:1)",
            b"#1 80\n0.a(2 1,0.0:1)",
            b"#1 80\n0.a(1 1,-1.0:1)",
            b"#1 80\n0.a(1 1,0.0:1?)",
            b"#1 80\n0.a(1 1,0.0:1[)",
            b"#1 80\n0.a(1 1,0.0:4294967296)",
            b"#1 80\n0.a(1 1,0.0:1",
            b"#1 80\n110.framerate(0)",
            b"#1 80\n0.a(0 1,0.0:1) 0.b(0 1,0.0:1)",
            b"#1 80\n#1 80",
            b"#1 80\n0=abc",
            b"#1 80\0",
        ] {
            assert!(parse(input).is_err(), "unexpected acceptance: {:?}", input);
        }
    }

    #[test]
    fn enforces_input_line_sprite_command_frame_and_event_bounds() {
        let bytes = b"#1 80\n0.a(1 2,0.0:1!! 0.1:1)\n#2 80\n0.a(0 1,0.0:1)\n";
        for limits in [
            ActionLimits {
                max_bytes: 10,
                ..ActionLimits::default()
            },
            ActionLimits {
                max_line_bytes: 10,
                ..ActionLimits::default()
            },
            ActionLimits {
                max_sprites: 1,
                ..ActionLimits::default()
            },
            ActionLimits {
                max_actions: 1,
                ..ActionLimits::default()
            },
            ActionLimits {
                max_frames: 1,
                ..ActionLimits::default()
            },
            ActionLimits {
                max_frames_per_action: 1,
                ..ActionLimits::default()
            },
            ActionLimits {
                max_events_per_frame: 1,
                ..ActionLimits::default()
            },
        ] {
            assert!(matches!(
                SpriteActionCatalog::parse(bytes, limits).unwrap_err().kind,
                ActionErrorKind::Limit(_)
            ));
        }
    }

    #[test]
    fn truncated_and_arbitrary_bytes_do_not_panic() {
        let fixture = b"1 0 41210\n#1 80 test\n0.walk(1 2,0.0:1 0.1:2!)\n";
        for size in 0..fixture.len() {
            let _ = parse(&fixture[..size]);
        }
        for byte in 0..=255 {
            let bytes = [b'#', b'1', b' ', b'1', b'\n', byte];
            let _ = parse(&bytes);
        }
    }

    #[test]
    fn native_43dcc0_colon_image_subframe_separator_matches_dot_and_stays_narrow() {
        // Exact list.spr 523C61AD line 614 command (#266 orim action 2).
        let colon = parse(b"#266 40=183 orim\n2.damage(1 2,0:0:4 24.0:6[79)\n").unwrap();
        let dot = parse(b"#266 40=183 orim\n2.damage(1 2,0.0:4 24.0:6[79)\n").unwrap();
        let a = colon.resolve_action(266, ActionKey::new(2)).unwrap().animation;
        assert_eq!(a, dot.resolve_action(266, ActionKey::new(2)).unwrap().animation);
        assert_eq!(
            a.frames.iter().map(|f| (f.image_id, f.subframe, f.duration_ticks)).collect::<Vec<_>>(),
            vec![(0, 0, 4), (24, 0, 6)]
        );
        assert_eq!(a.frames[1].events, vec![FrameEvent::Sound(79)]);
        // Mid-sequence `80:3:4` (list.spr line 1858 #1245 death) also matches.
        let mid = parse(b"#1 80\n8.death(1 2,80.2:4 80:3:4)\n").unwrap();
        let m = mid.resolve_action(1, ActionKey::new(8)).unwrap().animation;
        assert_eq!((m.frames[1].image_id, m.frames[1].subframe, m.frames[1].duration_ticks), (80, 3, 4));
        // Still strict: ticks separator, swapped order, double separators,
        // signed images and compact syntax are not widened.
        for input in [
            b"#1 80\n0.a(0 1,0.0.4)".as_slice(),
            b"#1 80\n0.a(0 1,0:3.2)",
            b"#1 80\n0.a(0 1,0::0:4)",
            b"#1 80\n0.a(0 1,40.10.:4)",
            b"#1 80\n0.a(0 1,-1.0:4)",
            b"#1 80\n0.a(0 1,0;0:4)",
        ] {
            assert!(parse(input).is_err(), "unexpected acceptance: {:?}", input);
        }
        // Inspection keeps the unverified forms quarantined, never animated.
        let inspected = SpriteActionCatalog::parse_for_inspection(
            b"#1 80\n0.a(0 1,0:3.2) 1.b(0 1,0:0:4)\n",
            ActionLimits::default(),
        )
        .unwrap();
        assert!(matches!(
            inspected.sprites[&1].actions[&ActionKey::new(0)],
            ActionDefinition::Quarantined { .. }
        ));
        assert!(inspected.resolve_action(1, ActionKey::new(1)).is_ok());
    }

    #[test]
    fn real_attack_zero_tick_setup_frame_is_retained() {
        let catalog =
            parse(b"#0 312=3225 prince\n1.attack(1 3,0.0:0 16.0:3[1539 16.1:2!)\n").unwrap();
        let animation = catalog
            .resolve_action(0, ActionKey::new(1))
            .unwrap()
            .animation;
        assert_eq!(animation.frames.len(), 3);
        assert_eq!(animation.frames[0].duration_ticks, 0);
        assert_eq!(animation.duration_ratio(), (5, 24));
    }

    #[test]
    fn declared_sprite_count_is_bounded_and_checked_without_contiguous_ids() {
        assert!(matches!(
            parse(b"1 0 41210\n#5 80\n#9 80\n").unwrap_err().kind,
            ActionErrorKind::SpriteCount {
                declared: 1,
                actual: 2
            }
        ));
        assert!(matches!(
            parse(b"4294967295 0 41210\n#5 80\n").unwrap_err().kind,
            ActionErrorKind::Limit("declared sprites")
        ));
        assert!(parse(b"2 0 41210\n#5 80\n#9 80\n").is_ok());
    }

    #[test]
    #[ignore = "requires explicit LITO_SHARED_RESOURCE_ROOT; parses shared Text/list.spr"]
    fn real_shared_list_actions_are_bounded_and_resolvable() {
        use crate::modern_lito::resources::{SharedArchive, SharedArchiveKind};
        let root = std::env::var_os("LITO_SHARED_RESOURCE_ROOT")
            .expect("set LITO_SHARED_RESOURCE_ROOT to the shared Lineage root");
        let bytes = SharedArchive::open(std::path::Path::new(&root), SharedArchiveKind::Text)
            .unwrap()
            .read_entry("list.spr")
            .unwrap();
        let catalog = SpriteActionCatalog::parse_for_inspection(&bytes, ActionLimits::default())
            .unwrap_or_else(|err| {
                let line = bytes
                    .split(|b| *b == b'\n')
                    .nth(err.line.saturating_sub(1))
                    .unwrap_or_default();
                for next in bytes.split(|b| *b == b'\n').skip(err.line).take(2) {
                    eprintln!("following line bytes: {:?}", &next[..next.len().min(300)]);
                }
                panic!(
                    "{err}; source line bytes: {:?}",
                    &line[..line.len().min(400)]
                );
            });
        let strict_error =
            parse(&bytes).expect_err("known unverified source syntax must fail strict parsing");
        // #165: line 614 `2.damage(1 2,0:0:4 ...)` is admitted per native
        // 43DCC0. The next strict blocker is line 2419 `-1.0:4` (signed
        // image id), which stays rejected until its semantics are verified.
        assert_eq!(strict_error.line, 2419);
        assert_eq!(strict_error.kind, ActionErrorKind::Malformed("expected unsigned integer"));
        let orim = catalog.resolve_action(266, ActionKey::new(2)).unwrap().animation;
        assert_eq!(
            orim.frames.iter().map(|f| (f.image_id, f.subframe, f.duration_ticks)).collect::<Vec<_>>(),
            vec![(0, 0, 4), (24, 0, 6)]
        );
        assert_eq!(orim.frames[1].events, vec![FrameEvent::Sound(79)]);
        eprintln!("Strict source validation remains rejected: {strict_error}");
        let mut frames = 0usize;
        let mut actions = 0usize;
        let mut refs = 0usize;
        let mut zero_ticks = 0usize;
        let mut events = 0usize;
        let mut uninterpreted_events = 0usize;
        let mut uninterpreted_by_marker = BTreeMap::<u8, usize>::new();
        let mut quarantined = Vec::new();
        let mut quarantine_reasons = BTreeMap::<String, usize>::new();
        let mut quarantine_examples =
            BTreeMap::<String, Vec<(u32, ActionKey, usize, String)>>::new();
        let mut unresolved_references = 0usize;
        let mut resolvable_actions = 0usize;
        let mut attributes = BTreeMap::<u32, usize>::new();
        let mut quarantined_headers = Vec::new();
        for sprite in catalog.sprites.values() {
            if let Some(issue) = &sprite.header_issue {
                quarantined_headers.push((sprite.id, issue));
            }
            for (&key, definition) in &sprite.actions {
                actions += 1;
                resolvable_actions += usize::from(catalog.resolve_action(sprite.id, key).is_ok());
                match definition {
                    ActionDefinition::Animation(animation) => {
                        frames += animation.frames.len();
                        for frame in &animation.frames {
                            zero_ticks += usize::from(frame.duration_ticks == 0);
                            events += frame.events.len();
                            uninterpreted_events += frame
                                .events
                                .iter()
                                .filter(|event| {
                                    matches!(
                                        event,
                                        FrameEvent::UninterpretedNumeric { .. } | FrameEvent::Skip
                                    )
                                })
                                .count();
                            for event in &frame.events {
                                match event {
                                    FrameEvent::UninterpretedNumeric { marker, .. } => {
                                        *uninterpreted_by_marker.entry(*marker).or_default() += 1
                                    }
                                    FrameEvent::Skip => {
                                        *uninterpreted_by_marker.entry(b'>').or_default() += 1
                                    }
                                    _ => (),
                                }
                            }
                        }
                    }
                    ActionDefinition::Reference { .. } => {
                        refs += 1;
                        if let Err(err) = catalog.resolve_action(sprite.id, key) {
                            assert!(matches!(
                                err.kind,
                                ActionErrorKind::QuarantinedAction(..)
                                    | ActionErrorKind::QuarantinedSprite(..)
                                    | ActionErrorKind::UninterpretedEvents(..)
                            ));
                            unresolved_references += 1;
                        }
                    }
                    ActionDefinition::Quarantined { reason, raw_bytes } => {
                        quarantined.push((sprite.id, key, reason));
                        let category = format!("{:?}", reason.kind);
                        *quarantine_reasons.entry(category.clone()).or_default() += 1;
                        let examples = quarantine_examples.entry(category).or_default();
                        if examples.len() < 2 {
                            let escaped: String = raw_bytes
                                .iter()
                                .take(220)
                                .flat_map(|b| std::ascii::escape_default(*b))
                                .map(char::from)
                                .collect();
                            examples.push((sprite.id, key, reason.line, escaped));
                        }
                    }
                }
            }
            for attribute in &sprite.attributes {
                *attributes.entry(attribute.key.id).or_default() += 1;
            }
        }
        eprintln!(
            "list.spr: {} bytes; {} sprites; {actions} action definitions ({refs} references); {frames} frames ({zero_ticks} zero-tick); {events} ordered events ({uninterpreted_events} uninterpreted/skip events, NOT playback-verified); opaque attributes by id: {attributes:?}",
            bytes.len(),
            catalog.sprites.len()
        );
        eprintln!(
            "{resolvable_actions} resolvable actions (runtime events still require implementation); {} quarantined/unverified definitions; {unresolved_references} references blocked by unverified data; unverified markers: {uninterpreted_by_marker:?}",
            quarantined.len()
        );
        eprintln!(
            "quarantine reason counts: {quarantine_reasons:?}; examples: {quarantine_examples:?}"
        );
        eprintln!("quarantined sprite headers: {quarantined_headers:?}");
        assert_eq!(catalog.sprites.len(), 11_297);
        assert_eq!(catalog.header, Some([11_297, 0, 41_210]));
        let walk = catalog.resolve_action(0, ActionKey::new(0)).unwrap();
        assert_eq!(walk.animation.frames.len(), 4);
        assert_eq!(
            walk.animation.frames[0].events,
            vec![FrameEvent::Effect(479)]
        );
        assert_eq!(catalog.sprites[&0].graphics_id, 3225);
    }

    #[test]
    fn observed_caret_event_is_preserved_and_flagged_as_uninterpreted() {
        let catalog = parse(b"#155 1 temple\n3.breath(0 1,0.1:4^3164)\n").unwrap();
        assert!(matches!(
            catalog
                .resolve_action(155, ActionKey::new(3))
                .unwrap_err()
                .kind,
            ActionErrorKind::UninterpretedEvents(..)
        ));
        let ActionDefinition::Animation(animation) =
            &catalog.sprites[&155].actions[&ActionKey::new(3)]
        else {
            panic!("expected retained animation data");
        };
        assert!(animation.has_uninterpreted_events());
        assert!(animation.frame_asset(155, 0, 0).is_err());
        assert_eq!(
            animation.frames[0].events,
            vec![FrameEvent::UninterpretedNumeric {
                marker: b'^',
                value: 3164
            }]
        );
    }

    #[test]
    fn inspection_retains_unverified_body_and_ref_cannot_bypass_quarantine() {
        // #165: the former `0:0:4` example is native-verified (43DCC0) and
        // now parses; the signed-image form from list.spr line 2419 stays
        // unverified and keeps exercising quarantine plus reference blocking.
        let bytes = b"#266 40=183 orim\n2.damage(1 2,-1.0:4 24.0:6[79)\n#267 40\n2=266\n";
        assert!(parse(bytes).is_err());
        let catalog =
            SpriteActionCatalog::parse_for_inspection(bytes, ActionLimits::default()).unwrap();
        let ActionDefinition::Quarantined { raw_bytes, reason } =
            &catalog.sprites[&266].actions[&ActionKey::new(2)]
        else {
            panic!("expected explicit quarantine");
        };
        assert_eq!(reason.line, 2);
        assert_eq!(raw_bytes, b"2.damage(1 2,-1.0:4 24.0:6[79)");
        let admitted = b"#266 40=183 orim\n2.damage(1 2,0:0:4 24.0:6[79)\n#267 40\n2=266\n";
        let resolved = parse(admitted).unwrap();
        let through = resolved.resolve_action(267, ActionKey::new(2)).unwrap();
        assert_eq!((through.definition_sprite_id, through.animation.frames.len()), (266, 2));
        for id in [266, 267] {
            assert!(matches!(
                catalog
                    .resolve_action(id, ActionKey::new(2))
                    .unwrap_err()
                    .kind,
                ActionErrorKind::QuarantinedAction(266, _)
            ));
        }
        let cyclic = b"#1 1\n0=2\n#2 1\n0=1\n";
        assert!(matches!(
            SpriteActionCatalog::parse_for_inspection(cyclic, ActionLimits::default())
                .unwrap_err()
                .kind,
            ActionErrorKind::CyclicReference(..)
        ));
    }

    #[test]
    fn skip_and_referenced_uninterpreted_events_cannot_play() {
        let catalog = SpriteActionCatalog::parse_for_inspection(
            b"#1 80\n0.a(0 1,0.0:1>)\n#2 80\n0=1\n",
            ActionLimits::default(),
        )
        .unwrap();
        for id in [1, 2] {
            assert!(matches!(
                catalog
                    .resolve_action(id, ActionKey::new(0))
                    .unwrap_err()
                    .kind,
                ActionErrorKind::UninterpretedEvents(1, _)
            ));
        }
    }

    #[test]
    fn inspection_header_and_metadata_issues_gate_all_affected_sprite_actions() {
        for bytes in [
            b"#1 8.label\n0.a(0 1,0.0:1)\n#2 1\n0=1\n".as_slice(),
            b"#1 8 label\n102.type(5))\n0.a(0 1,0.0:1)\n#2 1\n0=1\n",
        ] {
            assert!(parse(bytes).is_err());
            let catalog =
                SpriteActionCatalog::parse_for_inspection(bytes, ActionLimits::default()).unwrap();
            for id in [1, 2] {
                assert!(matches!(
                    catalog
                        .resolve_action(id, ActionKey::new(0))
                        .unwrap_err()
                        .kind,
                    ActionErrorKind::QuarantinedSprite(1)
                ));
            }
        }
    }

    #[test]
    fn inspection_marks_each_unverified_delimiter_form_without_repairing_it() {
        for command in [
            b"0.bad1 1,0.0:1)".as_slice(),
            b"0.bad((1 1,0.0:1)",
            b"0.bad(1 1,0.0:1",
            b"0.bad(1 1,0.0:1))",
            b"0,bad(1 1,0.0:1)",
            b"0.bad(1 1 0.0:1)",
        ] {
            let mut bytes = b"#1 80\n".to_vec();
            bytes.extend_from_slice(command);
            bytes.extend_from_slice(b"\n1.good(0 1,0.0:1)\n");
            assert!(parse(&bytes).is_err());
            let catalog =
                SpriteActionCatalog::parse_for_inspection(&bytes, ActionLimits::default()).unwrap();
            let ActionDefinition::Quarantined { raw_bytes, .. } =
                &catalog.sprites[&1].actions[&ActionKey::new(0)]
            else {
                panic!("must retain unverified command");
            };
            assert_eq!(raw_bytes, command);
            assert!(catalog.resolve_action(1, ActionKey::new(0)).is_err());
            assert!(catalog.resolve_action(1, ActionKey::new(1)).is_ok());
        }
    }
}
