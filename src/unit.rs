use bevy::prelude::*;
use bevy::tasks::prelude::*;

use rand::*;
use std::ops::{Deref, DerefMut};

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::grid::*;

#[derive(Default)]
pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_resource(AnimTimer {
            timer: Timer::from_seconds(0.1, true),
        })
        .init_resource::<Events<DamageEvent>>()
        .add_system(add_time_on_unit_info.system())
        .add_system(turning_ai_update.system())
        .add_system(move_on_ai_force_update.system())
        .add_system(update_attacking_ai)
        .add_system(simple_damage_event_reader)
        .add_system_to_stage(stage::POST_UPDATE, update_animation_from_state.system())
        .add_system_to_stage(stage::POST_UPDATE, animate_sprite_system.system());
    }
}

#[derive(Debug)]
pub struct DamageEvent {
    pub x: i32,
    pub y: i32,
    pub from: bool,
}

fn simple_damage_event_reader(damage_events: Res<Events<DamageEvent>>) {
    let mut reader = damage_events.get_reader();
    for event in reader.iter(&damage_events) {
        info!("Damage done: {:?}", event);
    }
}

fn animate_sprite_system(
    time: Res<Time>,
    mut timer: ResMut<AnimTimer>,
    mut query: Query<(&mut TextureAtlasSprite, &mut Animation)>,
) {
    timer.timer.tick(time.delta_seconds());
    if timer.timer.just_finished() {
        for (mut sprite, mut animation) in query.iter_mut() {
            animation.current_frame = (animation.current_frame + 1) % animation.frames.len() as u32;
            sprite.index = animation.frames[animation.current_frame as usize];
        }
    }
}

#[derive(Default)]
pub struct UnitBundle {
    pub spritesheet: SpriteSheetBundle,
    pub unit_info: UnitInfo,
    pub unit_state: UnitState,
    pub unit_stats: UnitStats,
}

impl UnitBundle {
    pub fn build(self, commands: &mut Commands) -> &mut Commands {
        commands
            .spawn(self.spritesheet)
            .with(self.unit_info)
            .with(self.unit_state.get_animation())
            .with(self.unit_state)
            .with(UnitTime::default())
            .with(GridTransform {
                x: 0.0,
                y: 0.0,
                update_scale: false,
            })
            .with(self.unit_stats)
    }
}

struct AnimTimer {
    timer: Timer,
}

struct Animation {
    current_frame: u32,
    frames: Vec<u32>,
}

#[derive(Debug, Clone)]
pub enum UnitState {
    Still(Direction),
    Moving(Direction),
}

impl UnitState {
    pub fn is_still(&self) -> bool {
        match self {
            Self::Still(_) => true,
            _ => false,
        }
    }
}

impl UnitState {
    fn get_animation(&self) -> Animation {
        let frames = match self {
            Self::Still(Direction::Down) => vec![1],
            Self::Still(Direction::Right) => vec![7],
            Self::Still(Direction::Up) => vec![10],
            Self::Still(Direction::Left) => vec![4],

            Self::Moving(Direction::Down) => vec![1, 2, 1, 0],
            Self::Moving(Direction::Right) => vec![7, 8, 7, 6],
            Self::Moving(Direction::Up) => vec![10, 11, 10, 9],
            Self::Moving(Direction::Left) => vec![4, 5, 4, 3],
        };
        Animation {
            current_frame: 0,
            frames: frames,
        }
    }
}

fn update_animation_from_state(mut query: Query<(&UnitState, &mut Animation), Changed<UnitState>>) {
    for (state, mut animation) in query.iter_mut() {
        *animation = state.get_animation();
    }
}

impl Default for UnitState {
    fn default() -> Self {
        Self::Still(Direction::default())
    }
}

#[derive(Debug, Copy, Clone, EnumIter)]
pub enum Direction {
    Up,
    Left,
    Right,
    Down,
}

impl Direction {
    fn next(&self) -> Self {
        match self {
            Direction::Up => Direction::Left,
            Direction::Left => Direction::Down,
            Direction::Down => Direction::Right,
            Direction::Right => Direction::Up,
        }
    }

    fn x(&self) -> i32 {
        match self {
            Direction::Up => 0,
            Direction::Left => -1,
            Direction::Down => 0,
            Direction::Right => 1,
        }
    }

    fn y(&self) -> i32 {
        match self {
            Direction::Up => 1,
            Direction::Left => 0,
            Direction::Down => -1,
            Direction::Right => 0,
        }
    }

    fn from_points(x1: i32, y1: i32, x2: i32, y2: i32) -> Self {
        let x_diff = (x1 - x2).abs();
        let x_dir = if x1 < x2 { Self::Right } else { Self::Left };

        let y_diff = (y1 - y2).abs();
        let y_dir = if y1 < y2 { Self::Up } else { Self::Down };

        if x_diff < y_diff {
            x_dir
        } else {
            y_dir
        }
    }
}

impl Default for Direction {
    fn default() -> Self {
        Self::Down
    }
}

fn add_time_on_unit_info(
    time: Res<Time>,
    pool: Res<ComputeTaskPool>,
    mut query: Query<&mut UnitTime>,
) {
    let delta = time.delta_seconds();
    query.par_iter_mut(64).for_each(&pool, |mut unit| {
        unit.time += delta;
    });
}

fn turning_ai_update(
    mut query: Query<
        (&UnitTime, &mut UnitState, &mut UnitInfo, &mut GridTransform),
        With<TurningAI>,
    >,
) {
    for (unit_time, mut state, mut info, mut transform) in query.iter_mut() {
        update_pos(&unit_time, &info, &mut transform);
        if unit_time.time > info.end_time {
            info.start_time = unit_time.time;
            info.end_time = unit_time.time + info.action_delay;
            *state = match *state {
                UnitState::Still(dir) => {
                    let new_dir = dir.next();
                    info.target_x = info.last_x + new_dir.x();
                    info.target_y = info.last_y + new_dir.y();
                    UnitState::Moving(new_dir)
                }
                UnitState::Moving(dir) => {
                    info.last_x = info.target_x;
                    info.last_y = info.target_y;

                    UnitState::Still(dir)
                }
            };
        }
    }
}

fn find_potential_pos(
    grid: &Grid,
    cur_x: i32,
    cur_y: i32,
    target_x: i32,
    target_y: i32,
    status_wanted: GridStatus,
) -> Option<(Direction, i32, i32)> {
    let mut potential_pos: Option<(Direction, i32, i32)> = None;
    let mut pos_distance = i32::MAX;
    for d in Direction::iter() {
        let x = cur_x + d.x();
        let y = cur_y + d.y();
        if let Some(status) = grid.get_status(x, y) {
            if status == status_wanted || status == GridStatus::Neutral {
                let distance = (target_x - x).abs() + (target_y - y).abs();
                if distance < pos_distance {
                    potential_pos = Some((d, x, y));
                    pos_distance = distance;
                } else if distance == pos_distance && random::<bool>() {
                    potential_pos = Some((d, x, y));
                }
            }
        }
    }
    potential_pos
}

fn grid_info_move_to(grid: &mut Grid, mut info: &mut UnitInfo, x: i32, y: i32, ally: bool) {
    let count_change = if ally { 1 } else { -1 };
    grid.change_by_count(info.last_x, info.last_y, -count_change);
    grid.change_by_count(x, y, count_change);
    info.target_x = x;
    info.target_y = y;
}

fn move_on_ai_force_update(
    mut grid: ResMut<Grid>,
    mut query: Query<(
        &UnitTime,
        &UnitStats,
        &mut UnitState,
        &mut UnitInfo,
        &mut GridTransform,
        &mut MoveOnForceAI,
    )>,
) {
    for (unit_time, stats, mut state, mut info, mut transform, mut force) in query.iter_mut() {
        update_pos(&unit_time, &info, &mut transform);

        if state.is_still() && info.last_x == force.target_x && info.last_y == force.target_y {
            if force.stick_to_target {
                continue;
            } else {
                force.target_x = random::<i32>().abs() % grid.x;
                force.target_y = random::<i32>().abs() % grid.y;
                info!("Target {} {}", force.target_x, force.target_y);
            }
        }

        if unit_time.time > info.end_time {
            info.start_time = unit_time.time;
            info.end_time = unit_time.time + info.action_delay;
            let status_wanted = if force.ally {
                GridStatus::Friend
            } else {
                GridStatus::Enemy
            };
            *state = match &*state {
                UnitState::Still(dir) => {
                    let potential_pos = find_potential_pos(
                        &grid,
                        info.last_x,
                        info.last_y,
                        force.target_x,
                        force.target_y,
                        status_wanted,
                    );

                    if let Some((d, x, y)) = potential_pos {
                        grid_info_move_to(&mut grid, &mut info, x, y, force.ally);
                        info.end_time = unit_time.time + (info.action_delay / stats.move_speed);
                        UnitState::Moving(d)
                    } else {
                        UnitState::Still(dir.next())
                    }
                }
                UnitState::Moving(dir) => {
                    info.last_x = info.target_x;
                    info.last_y = info.target_y;

                    UnitState::Still(dir.clone())
                }
            };
        }
    }
}

fn update_pos(time: &UnitTime, info: &UnitInfo, mut transform: &mut GridTransform) {
    let ratio = (time.time - info.start_time) / (info.end_time - info.start_time);
    transform.x = info.last_x as f32 + ratio * (info.target_x - info.last_x) as f32;
    transform.y = info.last_y as f32 + ratio * (info.target_y - info.last_y) as f32;
}

#[derive(Default)]
pub struct UnitInfo {
    pub last_x: i32,
    pub last_y: i32,

    pub action_delay: f32,

    pub target_x: i32,
    pub target_y: i32,

    pub start_time: f32,
    pub end_time: f32,
}

#[derive(Default)]
pub struct UnitTime {
    pub time: f32,
}

pub struct UnitStats {
    pub life: i32,
    pub move_speed: f32,
    pub damage: i32,
    pub attack_speed: f32,
}

impl Default for UnitStats {
    fn default() -> Self {
        Self {
            move_speed: 1.0,
            attack_speed: 1.0,
            life: 1,
            damage: 1,
        }
    }
}

pub struct TurningAI;

#[derive(Default)]
pub struct MoveOnForceAI {
    pub ally: bool,
    pub target_x: i32,
    pub target_y: i32,
    pub stick_to_target: bool,
}

pub struct AttackingAI {
    pub ally: bool,
}

pub enum AttackingAIState {
    PrepareAttack,
    AfterAttack,
    MoveToNearestEnemy,
}

fn find_enemy_in_range(grid: &Grid, x: i32, y: i32, ally: bool, range: i32) -> Vec<(i32, i32)> {
    let mut result = Vec::new();
    let mut push = |(n_x, n_y)| {
        if ally && grid.get_status(n_x, n_y) == Some(GridStatus::Friend) {
            result.push((n_x, n_y));
        } else if !ally && grid.get_status(n_x, n_y) == Some(GridStatus::Enemy) {
            result.push((n_x, n_y))
        }
    };

    for cur_range in 1..=range {
        let x_range = (x + -cur_range + 1).max(0)..=(x + cur_range - 1).min(grid.x);

        for r_x in x_range {
            let r_y = cur_range - (x - r_x).abs();

            push((r_x, y + r_y));
            push((r_x, y - r_y));
        }
        push((x - cur_range, y));
        push((x + cur_range, y));
    }

    result
}

#[test]
fn enemy_in_range() {
    let mut grid = Grid::new(2, 2);

    grid.add_enemy(1, 1);

    assert_eq!(find_enemy_in_range(&grid, 1, 1, true, 1), vec![]);
    assert_eq!(find_enemy_in_range(&grid, 1, 1, false, 1), vec![]);
    assert_eq!(find_enemy_in_range(&grid, 0, 0, false, 1), vec![]);

    assert_eq!(find_enemy_in_range(&grid, 1, 0, false, 1), vec![(1, 1)]);
    assert_eq!(find_enemy_in_range(&grid, 0, 1, false, 1), vec![(1, 1)]);
    grid = Grid::new(5, 5);
    grid.add_friend(3, 3);
    for i in 0..6 {
        assert_eq!(find_enemy_in_range(&grid, 0, 0, true, i), vec![]);
    }

    for i in 6..25 {
        assert_eq!(find_enemy_in_range(&grid, 0, 0, true, i), vec![(3, 3)]);
    }

    grid.add_friend(3, 4);
    grid.add_friend(4, 3);

    let mut final_test = find_enemy_in_range(&grid, 0, 0, true, 8);
    final_test.sort();
    assert_eq!(final_test, vec![(3, 3), (3, 4), (4, 3)]);
}

pub fn update_attacking_ai(
    mut grid: ResMut<Grid>,
    mut damage_events: ResMut<Events<DamageEvent>>,
    mut query: Query<(
        &AttackingAI,
        &mut AttackingAIState,
        &mut UnitInfo,
        &UnitStats,
        &UnitTime,
        &mut UnitState,
        &mut GridTransform,
    )>,
) {
    for (ai, mut state, mut info, stats, time, mut anim_state, mut transform) in query.iter_mut() {
        update_pos(&time, &info, &mut transform);

        // Do I need to do something else?
        if info.end_time < time.time {
            continue;
        }

        // Find an enemy that is 1 cell away since it is useful in all cases
        let enemy_close = find_enemy_in_range(&grid, info.last_x, info.last_y, ai.ally, 1)
            .iter()
            .next()
            .cloned();

        // Find the next state
        let new_state = match *state {
            AttackingAIState::PrepareAttack => {
                if let Some((enemy_x, enemy_y)) = enemy_close {
                    damage_events.send(DamageEvent {
                        x: enemy_x,
                        y: enemy_y,
                        from: ai.ally,
                    });
                    AttackingAIState::AfterAttack
                } else {
                    AttackingAIState::MoveToNearestEnemy
                }
            }

            AttackingAIState::AfterAttack => {
                if enemy_close.is_some() {
                    AttackingAIState::PrepareAttack
                } else {
                    AttackingAIState::MoveToNearestEnemy
                }
            }

            AttackingAIState::MoveToNearestEnemy => {
                info.last_x = info.target_x;
                info.last_y = info.target_y;
                if enemy_close.is_some() {
                    AttackingAIState::PrepareAttack
                } else {
                    AttackingAIState::MoveToNearestEnemy
                }
            }
        };

        // From the new state, we find the new value that we need to set
        let (delay, new_anim_state) = match new_state {
            AttackingAIState::PrepareAttack => {
                let (enemy_x, enemy_y) = enemy_close.unwrap();
                (
                    1.0 / stats.attack_speed,
                    UnitState::Still(Direction::from_points(
                        info.last_x,
                        info.last_y,
                        enemy_x.clone(),
                        enemy_y.clone(),
                    )),
                )
            }

            AttackingAIState::AfterAttack => (1.0, UnitState::Still(Direction::Down)),

            AttackingAIState::MoveToNearestEnemy => {
                if let Some((enemy_x, enemy_y)) =
                    find_enemy_in_range(&grid, info.last_x, info.last_y, ai.ally, 1000)
                        .iter()
                        .next()
                {
                    let (d, x, y) = find_potential_pos(
                        &grid,
                        info.last_x,
                        info.last_y,
                        enemy_x.clone(),
                        enemy_y.clone(),
                        GridStatus::from_force(ai.ally),
                    )
                    .unwrap();
                    grid_info_move_to(&mut grid, &mut info, x, y, ai.ally);
                    (1.0 / stats.move_speed, UnitState::Moving(d))
                } else {
                    (1.0, UnitState::Still(Direction::Down))
                }
            }
        };

        info.end_time = time.time + delay;
        *anim_state = new_anim_state;
        *state = new_state;
    }
}

pub fn spawn_unit<'a, G, TA>(
    commands: &'a mut Commands,
    asset_server: &impl Deref<Target = AssetServer>,
    grid: &mut G,
    texture_atlases: &mut TA,
    x: i32,
    y: i32,
    ally: bool,
) -> &'a mut Commands
where
    G: Deref<Target = Grid> + DerefMut,
    TA: Deref<Target = Assets<TextureAtlas>> + DerefMut,
{
    let texture_handle = asset_server.load("spritesheet/Female/Female 12-3.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(32.0, 32.0), 3, 4);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    UnitBundle {
        spritesheet: SpriteSheetBundle {
            texture_atlas: texture_atlas_handle.clone(),
            transform: Transform::from_scale(Vec3::splat(3.0)),
            ..Default::default()
        },
        unit_info: UnitInfo {
            last_x: x,
            last_y: y,
            target_x: x,
            target_y: y,
            action_delay: 0.40,
            ..Default::default()
        },
        unit_state: UnitState::Moving(crate::unit::Direction::Right),
        unit_stats: Default::default(),
    }
    .build(commands);

    grid.get_count(x, y)
        .expect("Expected valid position for the new unit");
    grid.change_by_count(x, y, if ally { 1 } else { -1 });

    commands
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::camera::init_cameras_2d;
    use crate::utils::tests::*;
    use crate::utils::*;
    use std::sync::Arc;
    use std::sync::Mutex;

    fn assert_stay_on_0_0(query: Query<&UnitInfo>) {
        let iter = query.iter();
        assert_eq!(iter.len(), 1, "Expected 1 unit, got {}", iter.len());
        for info in iter {
            assert_eq!(info.last_x, 0, "Expected units to have last_x = 0");
            assert_eq!(info.last_y, 0, "Expected units to have last_y = 0");
            assert_eq!(info.target_x, 0, "Expected units to have target_x = 0");
            assert_eq!(info.target_y, 0, "Expected units to have target_y = 0");
        }
    }

    #[test]
    #[serial]
    fn move_on_force_ally_wont_go_on_enemy() {
        fn init(
            commands: &mut Commands,
            asset_server: ResMut<AssetServer>,
            mut grid: ResMut<Grid>,
            mut texture_atlases: ResMut<Assets<TextureAtlas>>,
        ) {
            grid.add_enemy(1, 0);
            spawn_unit(
                commands,
                &asset_server,
                &mut grid,
                &mut texture_atlases,
                0,
                0,
                true,
            )
            .with(MoveOnForceAI {
                ally: true,
                ..Default::default()
            });
        }
        App::build()
            .add_plugin(Test::Frames(10))
            .add_plugin(GridPlugin)
            .add_plugin(UnitPlugin)
            .add_system(init_cameras_2d)
            .add_resource(Grid::new(2, 1))
            .add_startup_system(init)
            .add_system(assert_stay_on_0_0)
            .run();
    }

    #[test]
    #[serial]
    fn move_on_force_enemy_wont_go_on_ally() {
        fn init(
            commands: &mut Commands,
            asset_server: ResMut<AssetServer>,
            mut grid: ResMut<Grid>,
            mut texture_atlases: ResMut<Assets<TextureAtlas>>,
        ) {
            grid.add_friend(0, 1);
            spawn_unit(
                commands,
                &asset_server,
                &mut grid,
                &mut texture_atlases,
                0,
                0,
                false,
            )
            .with(MoveOnForceAI {
                ally: false,
                ..Default::default()
            });
        }
        App::build()
            .add_plugin(Test::Frames(10))
            .add_plugin(GridPlugin)
            .add_plugin(UnitPlugin)
            .add_system(init_cameras_2d)
            .add_resource(Grid::new(2, 1))
            .add_startup_system(init)
            .add_system(assert_stay_on_0_0)
            .run();
    }

    #[test]
    #[serial]
    fn battle_of_two_warriors() {
        fn init(
            commands: &mut Commands,
            asset_server: Res<AssetServer>,
            mut grid: ResMut<Grid>,
            mut texture_atlases: ResMut<Assets<TextureAtlas>>,
        ) {
            spawn_unit(
                commands,
                &asset_server,
                &mut grid,
                &mut texture_atlases,
                0,
                0,
                true,
            )
            .with(AttackingAI { ally: true })
            .with(AttackingAIState::MoveToNearestEnemy);

            spawn_unit(
                commands,
                &asset_server,
                &mut grid,
                &mut texture_atlases,
                3,
                3,
                false,
            )
            .with(AttackingAI { ally: false })
            .with(AttackingAIState::MoveToNearestEnemy);
        }

        fn check_unit_count(flag: Arc<Mutex<bool>>, query: Query<&UnitState>) {
            if query.iter().len() == 1 {
                let mut unlock = flag.lock().unwrap();
                *unlock = true;
            }
        }

        let own = Arc::new(Mutex::new(false));
        let copy = own.clone();

        App::build()
            .add_plugin(Test::NoStop)
            .add_plugin(bevy::log::LogPlugin)
            .add_plugin(GridPlugin)
            .add_plugin(UnitPlugin)
            .add_system(init_cameras_2d)
            .add_resource(Grid::new(4, 4))
            .add_startup_system(init)
            .add_system(move |q| check_unit_count(copy.clone(), q))
            .run();

        assert!(*(own.lock().unwrap()));
    }
}
