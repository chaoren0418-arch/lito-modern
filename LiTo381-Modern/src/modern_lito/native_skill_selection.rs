//! Native skill choices are learned IDs scoped to a live caster, never combo indices.
use super::{native_skills, native_world::World};
use anyhow::{ensure, Context, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Owner {
    connection: u64,
    generation: u64,
    corrections: u64,
    caster: (u32, u64),
}
impl Owner {
    fn current(connection: u64, world: &World) -> Option<Self> {
        if !world.ready() || world.battle_result.is_some() {
            return None;
        }
        let a = world.own_actor()?;
        Some(Self {
            connection,
            generation: world.generation,
            corrections: world.local_corrections,
            caster: (a.id, a.incarnation),
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Target {
    pub id: u32,
    incarnation: u64,
    target_revision: u64,
    generation: u64,
}
impl Target {
    pub fn capture(world: &World, id: u32) -> Option<Self> {
        let a = world.actors.get(&id)?;
        if !world.actor_visible(id) {
            return None;
        }
        Some(Self {
            id,
            incarnation: a.incarnation,
            target_revision: a.target_revision,
            generation: world.generation,
        })
    }
    pub fn resolve(self, world: &World) -> Option<u32> {
        (world.generation == self.generation
            && world.actor_visible(self.id)
                && world.actors.get(&self.id).is_some_and(|a| {
                a.incarnation == self.incarnation
                    && a.target_revision == self.target_revision
                        && !a.dead && !super::native_obstacles::destroyed_guardian(a)
            }))
        .then_some(self.id)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub id: u16,
    pub label: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Choice {
    pub skill: u16,
    pub target: Option<u32>,
    pub ground: Option<[u16; 2]>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ground {
    pub position: [u16; 2],
    owner: Owner,
}
impl Ground {
    pub fn capture(
        connection: u64,
        world: &World,
        cell: super::world_map::WorldCell,
    ) -> Option<Self> {
        Some(Self {
            position: [u16::try_from(cell.x).ok()?, u16::try_from(cell.y).ok()?],
            owner: Owner::current(connection, world)?,
        })
    }
    pub fn resolve(self, connection: u64, world: &World) -> Option<[u16; 2]> {
        (Some(self.owner) == Owner::current(connection, world)).then_some(self.position)
    }
}

/// Shared by the actual native window and isolated ClientSession probes.
/// This returns intent only; the controller still owns the send and pending gates.
pub fn choice(world: &World, skill: u16, target: Option<Target>) -> Result<Choice> {
    let target = if matches!(skill,2|3|13|14|31|32|49|52|54|60|72|78|131|206) {
        None
    } else {
        target
            .map(|t| t.resolve(world).context("選取的目標已失效，請重新點選"))
            .transpose()?
    };
    native_skills::request(world, skill, target)?;
    Ok(Choice {
        skill,
        target,
        ground: None,
    })
}
#[derive(Default)]
pub struct Model {
    owner: Option<Owner>,
    rows: Vec<Row>,
    selected: Option<u16>,
}
impl Model {
    pub fn rows(&self) -> &[Row] {
        &self.rows
    }
    pub fn selected(&self) -> Option<u16> {
        self.selected
    }
    pub fn sync(&mut self, connection: u64, world: &World, active: bool) -> bool {
        let owner = active.then(|| Owner::current(connection, world)).flatten();
        let rows = if owner.is_some() {
            world
                .learned_skills
                .iter()
                .map(|&id| Row {
                    id,
                    label: match id {
                        1 => "#1 治癒術（選取目標／自己）".into(),
                        2 => "#2 日光術（自己）".into(),
                        3 => "#3 護盾（自己）".into(),
                        4 => "#4 光箭（選取目標）".into(),
                        5 => "#5 指定傳送（隨機／書籤）".into(),
                        12 => "#12 擬似魔法武器（選取背包物品）".into(),
                        20 => "#20 闇盲咒術（選取目標，一格內）".into(),
                        21 => "#21 鎧甲護持（選取背包物品）".into(),
                        22 => "#22 寒冰氣息（選取目標，六格內）".into(),
                        23 => "#23 能量感測（選取怪物，畫面內）".into(),
                        25 => "#25 燃燒的火球（選取目標，六格內）".into(),
                        27 => "#27 壞物術（選取目標，四格內；消耗魔法寶石）".into(),
                        6 => "#6 冰箭（選取目標）".into(),
                        7 => "#7 風刃（選取目標）".into(),
                        8 => "#8 神聖武器（選取目標／自己）".into(),
                        9 => "#9 解毒術（選取目標／自己）".into(),
                        10 => "#10 寒冷戰慄（選取目標，三格內）".into(),
                        11 => "#11 毒咒（選取目標，一格內）".into(),
                        14 => "#14 負重強化術（自己）".into(),
                        26 => "#26 敏捷強化（選取目標／自己）".into(),
                        42 => "#42 力量強化（選取目標／自己）".into(),
                        43 => "#43 加速術（選取目標／自己）".into(),
                        31 => "#31 魔法屏障（自己，消耗 1 顆魔法寶石）".into(),
                        32 => "#32 冥想術（自己）".into(),
                        49 => "#49 集體治癒術（周圍角色）".into(),
                        52 => "#52 神聖疾走（自己）".into(),
                        54 => "#54 強力加速術（自己）".into(),
                        72 => "#72 反隱身屏障（周圍範圍）".into(),
                        34 => "#34 極道落雷（選取目標，六格內）".into(),
                        38 => "#38 冰錐術（選取目標，六格內）".into(),
                        77 => "#77 究極光裂術（選取目標，十格內）".into(),
                        78 => "#78 絕對屏障（自己，消耗 2 顆魔法寶石）".into(),
                        206 => "#206 專注（伺服器判定三格內對象）".into(),
                        13 => "#13 偵測術（自己）".into(),
                        15 => "#15 火箭（選取目標）".into(),
                        18 => "#18 起死回生術（選取目標，十格內）".into(),
                        17 => "#17 閃電（選取直線目標，六格內）".into(),
                        16 => "#16 STALAC（選取目標，十格內）".into(),
                        28 => "#28 吸血鬼之吻（選取目標，四格內）".into(),
                        19 => "#19 中級治癒術（選取目標／自己）".into(),
                        35 => "#35 高級治癒術（選取目標／自己）".into(),
                        57 => "#57 全部治癒術（選取目標／自己）".into(),
                        58 => "#58 火牆（點選地面，八格內）".into(),
                        60 => "#60 隱身術（自己，消耗 1 顆魔法寶石）".into(),
                        131 => "#131 世界樹的呼喚（自己）".into(),
                        63 => "#63 生命之泉（點選地面）".into(),
                        _ => format!("#{id} 技能（尚未支援）"),
                    },
                })
                .collect()
        } else {
            Vec::new()
        };
        let changed = self.owner != owner || self.rows != rows;
        if self.owner != owner
            || self
                .selected
                .is_some_and(|id| !world.learned_skills.contains(&id))
        {
            self.selected = None;
        }
        self.owner = owner;
        self.rows = rows;
        changed
    }
    pub fn select(&mut self, id: Option<u16>) -> Result<()> {
        self.selected = None;
        if let Some(id) = id {
            ensure!(
                self.owner.is_some() && self.rows.iter().any(|r| r.id == id),
                "技能選擇已失效"
            );
            self.selected = Some(id);
        }
        Ok(())
    }
    pub fn choice(&self, connection: u64, world: &World, target: Option<Target>) -> Result<Choice> {
        self.choice_at(connection, world, target, None)
    }
    pub fn selected_skill(&self, connection: u64, world: &World) -> Result<u16> {
        ensure!(self.owner.is_some() && self.owner == Owner::current(connection, world), "技能畫面已失效");
        let skill = self.selected.context("請先選擇已學會的技能")?;
        ensure!(world.learned_skills.contains(&skill), "技能清單已更新");
        Ok(skill)
    }
    pub fn choice_at(
        &self,
        connection: u64,
        world: &World,
        target: Option<Target>,
        ground: Option<Ground>,
    ) -> Result<Choice> {
        ensure!(
            self.owner.is_some() && self.owner == Owner::current(connection, world),
            "技能畫面已失效"
        );
        let skill = self.selected.context("請先選擇已學會的技能")?;
        if matches!(skill, 58 | 63) {
            let position = ground
                .and_then(|g| g.resolve(connection, world))
                .context("請點選有效的地面位置")?;
            native_skills::ground_request(world, skill, position)?;
            Ok(Choice {
                skill,
                target: None,
                ground: Some(position),
            })
        } else {
            choice(world, skill, target)
        }
    }
}
