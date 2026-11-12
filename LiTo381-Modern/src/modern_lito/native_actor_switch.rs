//! list.spr attribute 100 (`switch`): an object sprite is drawn with the stable
//! half writer while the local actor stands on one of its listed cells.
//!
//! Static evidence, unpacked image SHA256 4271D92F…5C2AB6E9 (never executed):
//! - 43E377 reads attribute 100 as `count` then `count` × (x0 y0 x1 y1) into
//!   the per-sprite table at 647E80.
//! - 4E6C50 runs for every S_PUT_OBJECT actor. For each rectangle it visits
//!   `y = actor+30 + y0 ..= actor+30 + y1` and `x = actor+2C + x0`, stepping
//!   two native half-cells while `x <= actor+2C + x1`, and registers the actor
//!   in that cell through 483890 with kind 2 (table 62F3AC → flag 5).
//! - 48AFC0 (own step 4E2F2F, S_PUT_OBJECT 4E7F5D/4E7F9A) compares the local
//!   actor's exact half-cell. Entries with bit 4 call 4E6B90(actor, 1) for the
//!   new cell and 4E6BF0(actor) for the old one. Both rewrite the fragment
//!   flags of the body and its linked sprites as `(flags & !3) | value` via
//!   4CDAF0; actor+1A keeps the value for later frame insertions (4CD9D0).
//! - 488DD0 routes flag 0x101 to the half writer after hidden (2) and flag 8.
//!
//! There is no 50 ms ramp on this path: 433210/433460 are only reached for
//! Layer5 kinds 0/1. The modern model is a pure function of the two logical
//! cells, which equals the native result for objects that do not move after
//! S_PUT_OBJECT (every door). Registration follow-up for moving sprites and the
//! native 256x128 half-cell window scroll (48B1D0) are not reconstructed.
use super::{
    native_actor_states::{Presentation, Style},
    sprite_actions::{ActionKey, SpriteActionCatalog},
    world_map::WorldCell,
};
use anyhow::{ensure, Context, Result};
use std::collections::BTreeMap;

const ATTRIBUTE: u32 = 100;
/// The pinned list.spr uses at most 7 rectangles and offsets in -18..=58.
const MAX_RECTS: usize = 16;
const MAX_OFFSET: i32 = 4096;

/// Inclusive native half-cell X and cell Y offsets from the object anchor.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Rect {
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct Switch {
    rects: Vec<Rect>,
}

impl Switch {
    /// `count x0,y0,x1,y1 …`. Commas and ASCII blanks both separate numbers;
    /// any other byte, a wrong arity or an oversized table is rejected rather
    /// than partially applied.
    pub fn parse(body: &[u8]) -> Result<Self> {
        let text = std::str::from_utf8(body).context("non-ASCII switch attribute")?;
        let numbers = text
            .split(|c: char| c == ',' || c.is_ascii_whitespace())
            .filter(|token| !token.is_empty())
            .map(|token| token.parse::<i32>().context("switch attribute number"))
            .collect::<Result<Vec<_>>>()?;
        let (&count, values) = numbers.split_first().context("switch attribute empty")?;
        let count = usize::try_from(count).context("negative switch rectangle count")?;
        ensure!(
            count <= MAX_RECTS && values.len() == count * 4,
            "switch rectangle count/arity"
        );
        ensure!(
            values.iter().all(|v| (-MAX_OFFSET..=MAX_OFFSET).contains(v)),
            "switch rectangle offset budget"
        );
        Ok(Self {
            rects: values
                .chunks_exact(4)
                .map(|v| Rect {
                    x0: v[0],
                    y0: v[1],
                    x1: v[2],
                    y1: v[3],
                })
                .collect(),
        })
    }

    /// `None` when the sprite has no attribute 100. Variants and duplicates
    /// are not qualified sources.
    pub fn from_catalog(catalog: &SpriteActionCatalog, logical: u32) -> Result<Option<Self>> {
        let Some(row) = catalog.sprites.get(&logical) else {
            return Ok(None);
        };
        let mut found = row.attributes.iter().filter(|a| a.key.id == ATTRIBUTE);
        let Some(attribute) = found.next() else {
            return Ok(None);
        };
        ensure!(
            attribute.key == ActionKey::new(ATTRIBUTE) && found.next().is_none(),
            "ambiguous switch attribute"
        );
        Self::parse(&attribute.body_bytes).map(Some)
    }

    pub fn rects(&self) -> &[Rect] {
        &self.rects
    }

    /// Whether 4E6C50 registered `local`'s exact native half-cell for an
    /// object anchored at `object`. Whole cells map to even half-cells, so an
    /// odd `x0` lists cells no actor can occupy.
    pub fn covers(&self, object: WorldCell, local: WorldCell) -> bool {
        let dx = 2 * (i64::from(local.x) - i64::from(object.x));
        let dy = i64::from(local.y) - i64::from(object.y);
        self.rects.iter().any(|r| {
            let (x0, x1) = (i64::from(r.x0), i64::from(r.x1));
            (i64::from(r.y0)..=i64::from(r.y1)).contains(&dy)
                && (x0..=x1).contains(&dx)
                && (dx - x0) % 2 == 0
        })
    }
}

/// Flag 1 on the body and every linked sprite. Hidden actors submit nothing.
/// Half already precedes the green/dim branches for a body (488DD0); ordinary
/// attachments become half and flag-8 attachments keep their screen writer,
/// which is exactly the existing `Style::Half` composition.
pub fn present(base: Presentation, switched: bool) -> Presentation {
    if switched && base.state != Style::Hidden {
        Presentation {
            state: Style::Half,
            ..base
        }
    } else {
        base
    }
}

/// Per-model parse cache keyed by logical graphics. A malformed attribute is
/// reported once per query and never treated as "covered".
#[derive(Default)]
pub struct Cache {
    plans: BTreeMap<u16, std::result::Result<Option<Switch>, String>>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct Decision {
    /// The sprite carries attribute 100.
    pub listed: bool,
    pub switched: bool,
    pub error: Option<String>,
}

impl Cache {
    /// `local` is the logical own cell (including the current predicted step);
    /// `None` for an observer, which has no native local actor to compare.
    pub fn decide(
        &mut self,
        catalog: &SpriteActionCatalog,
        graphics: u16,
        object: [u16; 2],
        local: Option<[u16; 2]>,
    ) -> Decision {
        let plan = self.plans.entry(graphics).or_insert_with(|| {
            Switch::from_catalog(catalog, u32::from(graphics)).map_err(|e| format!("{e:#}"))
        });
        let cell = |p: [u16; 2]| WorldCell {
            x: i32::from(p[0]),
            y: i32::from(p[1]),
        };
        match plan {
            Ok(None) => Decision::default(),
            Ok(Some(switch)) => Decision {
                listed: true,
                switched: local.is_some_and(|l| switch.covers(cell(object), cell(l))),
                error: None,
            },
            Err(error) => Decision {
                listed: true,
                switched: false,
                error: Some(error.clone()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modern_lito::sprite_actions::ActionLimits;
    use std::collections::BTreeSet;

    /// Rows copied from the pinned client list.spr (SHA256 523C61AD…E8B9F127).
    const ROWS: &[u8] = b"#38   1       door two            28.open(0 1,0.1:4[205) 29.close(0 1,0.0:4[204) 100.switch(2 -2,-1,2,0 -8,-5,4,-2) 102.type(8)\n\
#104  1       battle ship         0.walk(0 1,0.0:30) 100.switch(3 16,-6,30,2 14,-5,15,2 42,-18,58,-10) 102.type(1)\n\
#2083 1   riderhouse a r door     28.open(0 1,0.1:4[8) 29.close(0 1,0.0:4[9) 100.switch(1 -2,0,2,0) 102.type(8)\n\
#2084 16  scarecrow               0.walk(1 1,0.0:4) 8.death(1 2,8.0:4 8.1:4) 101.shadow(2085) 102.type(10)\n\
#2085 16  scarecrow s             102.type(0)\n\
#2089 1   riderhouse a l door     28.open(0 1,0.1:4[8) 29.close(0 1,0.0:4[9) 100.switch(1 0,-1,2,1) 102.type(8)\n";

    fn catalog() -> SpriteActionCatalog {
        SpriteActionCatalog::parse_for_inspection(ROWS, ActionLimits::default()).unwrap()
    }
    fn cell(x: i32, y: i32) -> WorldCell {
        WorldCell { x, y }
    }
    /// Literal 4E6C50 loop over native half-cells, independent of `covers`.
    fn registered(switch: &Switch, object: WorldCell) -> BTreeSet<(i64, i64)> {
        let (ax, ay) = (2 * i64::from(object.x) - 0x8000, i64::from(object.y));
        let mut cells = BTreeSet::new();
        for r in switch.rects() {
            let mut y = ay + i64::from(r.y0);
            while y <= ay + i64::from(r.y1) {
                let mut x = ax + i64::from(r.x0);
                while x <= ax + i64::from(r.x1) {
                    cells.insert((x, y));
                    x += 2;
                }
                y += 1;
            }
        }
        cells
    }

    #[test]
    fn pinned_rows_parse_to_native_rectangles_and_absent_rows_stay_unlisted() {
        let c = catalog();
        let door = Switch::from_catalog(&c, 2083).unwrap().unwrap();
        assert_eq!(
            door.rects(),
            [Rect {
                x0: -2,
                y0: 0,
                x1: 2,
                y1: 0
            }]
        );
        assert_eq!(Switch::from_catalog(&c, 38).unwrap().unwrap().rects().len(), 2);
        assert_eq!(
            Switch::from_catalog(&c, 104).unwrap().unwrap().rects()[2],
            Rect {
                x0: 42,
                y0: -18,
                x1: 58,
                y1: -10
            }
        );
        assert_eq!(Switch::from_catalog(&c, 2084).unwrap(), None);
        assert_eq!(Switch::from_catalog(&c, 9999).unwrap(), None);
    }

    #[test]
    fn malformed_tables_are_rejected_instead_of_partially_applied() {
        for body in [
            b"".as_slice(),
            b"1",
            b"1 -2,0,2",
            b"2 -2,0,2,0",
            b"1 -2,0,2,0 1",
            b"-1",
            b"1 a,0,2,0",
            b"1 -2,0,2,5000",
            b"17 0,0,0,0 0,0,0,0 0,0,0,0 0,0,0,0 0,0,0,0 0,0,0,0 0,0,0,0 0,0,0,0 0,0,0,0 0,0,0,0 0,0,0,0 0,0,0,0 0,0,0,0 0,0,0,0 0,0,0,0 0,0,0,0 0,0,0,0",
        ] {
            assert!(Switch::parse(body).is_err(), "{:?}", std::str::from_utf8(body));
        }
        assert!(Switch::parse(b"0").unwrap().rects().is_empty());
        assert!(!Switch::parse(b"0").unwrap().covers(cell(5, 5), cell(5, 5)));
    }

    #[test]
    fn coverage_equals_the_native_registration_loop_for_every_pinned_row() {
        let c = catalog();
        for id in [38, 104, 2083, 2089] {
            let switch = Switch::from_catalog(&c, id).unwrap().unwrap();
            for object in [cell(32736, 32800), cell(0, 0), cell(65535, 65535)] {
                let cells = registered(&switch, object);
                let mut reachable = 0;
                for dy in -24..=24 {
                    for dx in -40..=40 {
                        let local = cell(object.x + dx, object.y + dy);
                        let native =
                            cells.contains(&(2 * i64::from(local.x) - 0x8000, i64::from(local.y)));
                        assert_eq!(switch.covers(object, local), native, "{id} {dx} {dy}");
                        reachable += usize::from(native);
                    }
                }
                assert!(reachable > 0, "{id}");
            }
        }
        // Arena door 2083: the doorway cell and both horizontal neighbours only.
        let door = Switch::from_catalog(&c, 2083).unwrap().unwrap();
        let at = cell(32736, 32800);
        for (dx, dy, expected) in [
            (-1, 0, true),
            (0, 0, true),
            (1, 0, true),
            (-2, 0, false),
            (2, 0, false),
            (0, -1, false),
            (0, 1, false),
        ] {
            assert_eq!(door.covers(at, cell(at.x + dx, at.y + dy)), expected);
        }
        // An odd half-cell start lists cells between two whole cells.
        let odd = Switch::parse(b"1 -1,0,1,0").unwrap();
        assert!((-3..=3).all(|dx| !odd.covers(at, cell(at.x + dx, at.y))));
        assert_eq!(registered(&odd, at).len(), 2);
    }

    #[test]
    fn decision_cache_follows_the_local_cell_and_never_switches_for_observers() {
        let c = catalog();
        let mut cache = Cache::default();
        let door = [32736, 32800];
        let inside = cache.decide(&c, 2083, door, Some([32737, 32800]));
        assert_eq!(
            inside,
            Decision {
                listed: true,
                switched: true,
                error: None
            }
        );
        assert!(!cache.decide(&c, 2083, door, Some([32738, 32800])).switched);
        assert!(!cache.decide(&c, 2083, door, None).switched);
        assert_eq!(cache.decide(&c, 2084, door, Some(door)), Decision::default());
        assert_eq!(cache.plans.len(), 2);
        let broken = SpriteActionCatalog::parse_for_inspection(
            b"#7 1 broken door 100.switch(2 -2,0,2,0) 102.type(8)\n",
            ActionLimits::default(),
        )
        .unwrap();
        let decision = Cache::default().decide(&broken, 7, door, Some(door));
        assert!(decision.listed && !decision.switched && decision.error.is_some());
    }

    #[test]
    fn switched_presentation_is_half_for_body_and_linked_layers_but_never_reveals_hidden() {
        for state in [Style::Normal, Style::Green(10), Style::Dim, Style::Half] {
            for body_half in [false, true] {
                let base = Presentation { state, body_half };
                assert_eq!(present(base, false), base);
                let switched = present(base, true);
                assert_eq!(switched.state, Style::Half);
                assert_eq!(switched.body_style(), Style::Half);
                assert_eq!(switched.body_half, body_half);
            }
        }
        let hidden = Presentation {
            state: Style::Hidden,
            body_half: true,
        };
        assert_eq!(present(hidden, true), hidden);
        assert_eq!(present(hidden, true).body_style(), Style::Hidden);
    }
}
