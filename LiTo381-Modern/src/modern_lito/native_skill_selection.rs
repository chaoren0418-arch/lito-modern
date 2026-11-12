//! Native skill choices are learned IDs scoped to a live caster, never combo indices.
use super::{native_skills, native_world::World};
#[path = "native_skill_catalog.rs"]
pub mod catalog;
#[cfg(test)]
#[path = "native_skillbook_tests.rs"]
mod skill_ui_tests;
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
        if a.dead || !world.hp.is_some_and(|hp| hp[0] > 0) {
            return None;
        }
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
    corpse_action: Option<u64>,
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
            corpse_action: a.dead.then_some(a.action_sequence),
        })
    }
    /// A living selection cannot become a corpse authorization after death,
    /// and a captured corpse cannot become a living authorization on revival.
    pub fn resolve_for_skill(self, world: &World, skill: u16) -> Option<u32> {
        if !matches!(skill, 41 | 61) { return self.resolve(world); }
        (world.generation == self.generation && world.actor_visible(self.id)
            && world.actors.get(&self.id).is_some_and(|a|
                a.incarnation == self.incarnation && a.target_revision == self.target_revision
                && a.dead && self.corpse_action == Some(a.action_sequence)
                && !super::native_obstacles::destroyed_guardian(a))).then_some(self.id)
    }
    pub fn resolve(self, world: &World) -> Option<u32> {
        (self.corpse_action.is_none() && world.generation == self.generation
            && world.actor_visible(self.id)
                && world.actors.get(&self.id).is_some_and(|a| {
                a.incarnation == self.incarnation
                    && a.target_revision == self.target_revision
                        && !a.dead && !super::native_obstacles::destroyed_guardian(a)
            }))
        .then_some(self.id)
    }
}
/// T key: advance to the next object in the presented draw order. Only ids the
/// caller actually drew in a current frame are candidates; a retired world
/// (teleport handoff, cancel, failure) has no own actor and yields nothing.
/// `corpse` selects corpse targets for #41, otherwise living targets only.
/// The result is a fresh capture, never the stale `current` selection.
pub fn cycle_target(world: &World, presented: impl IntoIterator<Item = u32>, own: u32,
    current: Option<Target>, corpse: bool) -> Option<Target> {
    if !world.ready() || world.own_actor().is_none_or(|a| a.id != own) { return None; }
    let mut ids: Vec<u32> = Vec::new();
    for id in presented {
        if id != own && !ids.contains(&id) { ids.push(id); }
    }
    let start = current.and_then(|t| ids.iter().position(|&id| id == t.id)).map_or(0, |i| i + 1);
    (0..ids.len()).map(|n| ids[(start + n) % ids.len()]).find_map(|id| {
        Target::capture(world, id).filter(|t| if corpse {
            t.resolve_for_skill(world, 41).is_some()
        } else { t.resolve(world).is_some() })
    })
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
    let named = matches!(skill, 116 | 118);
    let target = if native_skills::selector_only(skill) || native_skills::self_object(skill) || named || matches!(skill,2|3|13|60|131) {
        None
    } else {
        target
            .map(|t| t.resolve_for_skill(world, skill).context("選取的目標已失效，請重新點選"))
            .transpose()?
    };
    if named { native_skills::admit_named(world, skill)?; } else { native_skills::request(world, skill, target)?; }
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
    bindings: [Option<u16>; 8],
}
impl Model {
    pub fn rows(&self) -> &[Row] {
        &self.rows
    }
    pub fn selected(&self) -> Option<u16> {
        self.selected
    }
    pub fn bindings(&self) -> &[Option<u16>; 8] {
        &self.bindings
    }
    /// Bind identities, never the current row index. None explicitly clears.
    pub fn bind(&mut self, slot: usize, id: Option<u16>) -> Result<()> {
        ensure!(slot < self.bindings.len(), "快捷格超出範圍");
        ensure!(self.owner.is_some(), "技能書已失效");
        ensure!(id.is_none_or(|id| self.rows.iter().any(|r| r.id == id)), "尚未學會此技能");
        self.bindings[slot] = id;
        Ok(())
    }
    pub fn bound(&self, connection: u64, world: &World, slot: usize) -> Result<u16> {
        ensure!(self.owner.is_some() && self.owner == Owner::current(connection, world), "快捷列已失效");
        let id = self.bindings.get(slot).copied().flatten().context("快捷格尚未綁定技能")?;
        ensure!(world.learned_skills.contains(&id), "尚未學會此技能");
        Ok(id)
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
                        97 => "#97 暗隱術（自己）".into(),
                        98 => "#98 附加劇毒（自己）".into(),
                        99 => "#99 影之防護（自己）".into(),
                        100 => "#100 提煉魔石（選取背包物品）".into(),
                        2 => "#2 日光術（自己）".into(),
                        3 => "#3 護盾（自己）".into(),
                        4 => "#4 光箭（選取目標）".into(),
                        5 => "#5 指定傳送（隨機／書籤）".into(),
                        12 => "#12 擬似魔法武器（對裝備中的武器）".into(),
                        20 => "#20 闇盲咒術（選取目標，一格內）".into(),
                        21 => "#21 鎧甲護持（選取背包物品）".into(),
                        22 => "#22 寒冰氣息（選取目標，六格內）".into(),
                        23 => "#23 能量感測（選取怪物，畫面內）".into(),
                        25 => "#25 燃燒的火球（選取目標，六格內）".into(),
                        27 => "#27 壞物術（選取目標，四格內；消耗魔法寶石）".into(),
                        29 => "#29 緩速術（選取目標，八格內）".into(),
                        30 => "#30 巖牢（選取目標，八格內）".into(),
                        33 => "#33 木乃伊的詛咒（選取目標，兩格內）".into(),
                        36 => "#36 迷魅術（選取怪物，畫面內）".into(),
                        37 => "#37 聖潔之光（選取目標／自己）".into(),
                        39 => "#39 魔力奪取（選取目標／自己，四格內）".into(),
                        40 => "#40 黑闇之影（選取目標，三格內）".into(),
                        41 => "#41 造屍術（選取屍體，畫面內）".into(),
                        6 => "#6 冰箭（選取目標）".into(),
                        7 => "#7 風刃（選取目標）".into(),
                        8 => "#8 神聖武器（選取目標／自己）".into(),
                        9 => "#9 解毒術（選取目標／自己）".into(),
                        10 => "#10 寒冷戰慄（選取目標，三格內）".into(),
                        11 => "#11 毒咒（選取目標，一格內）".into(),
                        14 => "#14 負重強化術（自己）".into(),
                        26 => "#26 通暢氣脈術（自己／同血盟角色）".into(),
                        42 => "#42 力量強化（選取目標／自己）".into(),
                        68 => "#68 聖結界（自己／同血盟角色）".into(),
                        43 => "#43 加速術（選取目標／自己）".into(),
                        31 => "#31 魔法屏障（自己，消耗 1 顆魔法寶石）".into(),
                        32 => "#32 冥想術（自己）".into(),
                        44 => "#44 魔法相消術（選取目標／自己）".into(),
                        45 => "#45 地裂術（選取目標，八格內）".into(),
                        46 => "#46 烈炎術（選取目標，三格內）".into(),
                        47 => "#47 弱化術（選取目標，三格內）".into(),
                        48 => "#48 祝福魔法武器（選取目標／自己）".into(),
                        50 => "#50 冰矛圍籬（選取目標，六格內）".into(),
                        51 => "#51 召喚術（選擇召喚種類）".into(),
                        53 => "#53 龍捲風（自己周圍四格）".into(),
                        55 => "#55 狂暴術（選取角色／自己，畫面內；同盟資格由伺服器判定）".into(),
                        56 => "#56 疾病術（選取目標，三格內；消耗 1 顆魔法寶石）".into(),
                        59 => "#59 冰雪暴（自己周圍十格）".into(),
                        61 => "#61 返生術（選取屍體，畫面內；對方須同意）".into(),
                        62 => "#62 震裂術（自己周圍三格）".into(),
                        64 => "#64 魔法封印（選取目標，三格內；消耗 1 顆魔法寶石）".into(),
                        65 => "#65 雷霆風暴（選取目標，十格內；周圍三格）".into(),
                        66 => "#66 沉睡之霧（選取目標，十格內；消耗 1 顆魔法寶石）".into(),
                        49 => "#49 集體治癒術（周圍角色）".into(),
                        52 => "#52 神聖疾走（自己）".into(),
                        54 => "#54 強力加速術（自己）".into(),
                        72 => "#72 反隱身屏障（周圍範圍）".into(),
                        34 => "#34 極道落雷（選取目標，六格內）".into(),
                        38 => "#38 冰錐術（選取目標，六格內）".into(),
                        77 => "#77 究極光裂術（選取目標，十格內）".into(),
                        78 => "#78 絕對屏障（自己，消耗 2 顆魔法寶石）".into(),
                        206 => "#206 專注（伺服器判定三格內對象）".into(),
                        101 => "#101 行走加速（自己）".into(),
                        102 => "#102 燃燒鬥志（自己）".into(),
                        103 => "#103 暗黑盲咒（選取目標，三格內）".into(),
                        104 => "#104 毒性抵抗（自己）".into(),
                        105 => "#105 雙重破壞（自己）".into(),
                        106 => "#106 暗影閃避（自己）".into(),
                        107 => "#107 暗影之牙（對裝備中的武器）".into(),
                        108 => "#108 會心一擊（選取目標，一格內）".into(),
                        109 => "#109 力量提升（自己）".into(),
                        110 => "#110 敏捷提升（自己）".into(),
                        111 => "#111 閃避提升（自己）".into(),
                        112 => "#112 破壞盔甲（伺服器判定一格內目標；消耗 HP）".into(),
                        113 => "#113 精準目標（選取目標／自己，畫面內；可輸入標記文字）".into(),
                        114 => "#114 激勵士氣（自己與周圍隊員）".into(),
                        115 => "#115 鋼鐵士氣（自己與周圍隊員／盟友）".into(),
                        116 => "#116 呼喚盟友（輸入盟友名稱）".into(),
                        117 => "#117 衝擊士氣（自己與周圍隊員）".into(),
                        118 => "#118 援護盟友（輸入盟友名稱）".into(),
                        145 => "#145 釋放元素（選取召喚物，畫面內；消耗精靈玉）".into(),
                        146 => "#146 魂體轉換（自己，消耗 HP）".into(),
                        147 => "#147 單屬性防禦（自己，消耗精靈玉）".into(),
                        148 => "#148 火焰武器（自己）".into(),
                        149 => "#149 風之神射（自己）".into(),
                        150 => "#150 風之疾走（自己）".into(),
                        151 => "#151 大地防護（自己）".into(),
                        152 => "#152 地面障礙（選取目標，十格內；消耗精靈玉）".into(),
                        153 => "#153 魔法消除（選取目標，十格內；消耗精靈玉）".into(),
                        154 => "#154 召喚屬性精靈（自己，消耗精靈玉）".into(),
                        155 => "#155 烈炎氣息（自己與周圍隊員）".into(),
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
                        73 => "#73 創造魔法武器（開啟背包選取物品）".into(),
                        _ => format!("#{id} {}（尚未支援）", catalog::name(id).unwrap_or("未知技能")),
                    },
                })
                .collect()
        } else {
            Vec::new()
        };
        let changed = self.owner != owner || self.rows != rows;
        if self.owner != owner {
            self.bindings = [None; 8];
        } else {
            for binding in &mut self.bindings {
                if binding.is_some_and(|id| !world.learned_skills.contains(&id)) {
                    *binding = None;
                }
            }
        }
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
        let skill = self.selected.context("請先選擇已學會的技能")?;
        self.choice_id_at(connection, world, skill, target, ground)
    }
    /// Hotbar and book share exactly the same selected-ground/target guards.
    pub fn choice_id_at(&self, connection: u64, world: &World, skill: u16,
        target: Option<Target>, ground: Option<Ground>) -> Result<Choice> {
        ensure!(
            self.owner.is_some() && self.owner == Owner::current(connection, world),
            "技能畫面已失效"
        );
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
