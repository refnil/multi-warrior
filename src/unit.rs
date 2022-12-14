use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;

use rand::*;
use std::ops::{Deref, DerefMut};

use crate::anim::*;
use crate::fx::*;
use crate::grid::*;
use crate::utils::{Direction, *};

#[derive(Default)]
pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Events<DamageEvent>>()
            .add_system(add_time_on_unit_info)
            .add_system(turning_ai_update)
            .add_system(move_on_ai_force_update)
            .add_system(update_attacking_ai)
            .add_system(damage_event_reader)
            .add_system(remove_dead_unit);
    }
}

#[derive(Debug)]
pub struct DamageEvent {
    pub x: i32,
    pub y: i32,
    pub from: bool,
}

#[derive(Default)]
pub struct UnitBundle {
    pub spritesheet: SpriteSheetBundle,
    pub unit_info: UnitInfo,
    pub unit_state: UnitState,
    pub unit_stats: UnitStats,
}

impl UnitBundle {
    pub fn build(
        self,
        commands: &mut Commands,
        with_unit: impl FnOnce(&mut EntityCommands<'_, '_, '_>),
    ) {
        with_unit(
            commands
                .spawn_bundle(self.spritesheet)
                .insert(self.unit_info)
                .insert(self.unit_state.get_animation())
                .insert(self.unit_state)
                .insert(UnitTime::default())
                .insert(GridTransform {
                    x: -1000.0,
                    y: -1000.0,
                    update_scale: false,
                })
                .insert(self.unit_stats),
        );
    }
}

fn damage_event_reader(
    mut damage_events: ResMut<Events<DamageEvent>>,
    mut query: Query<(&UnitInfo, &mut UnitStats)>,
) {
    damage_events.update();
    let mut reader = damage_events.get_reader();

    for event in reader.iter(&damage_events) {
        info!("Damage done: {:?}", event);
        if let Some((_, mut stats)) = query
            .iter_mut()
            .find(|(info, _)| info.last_x == event.x && info.last_y == event.y)
        {
            stats.life -= 1;
        } else {
            info!("Did not find unit to damage");
        }
    }
}

fn remove_dead_unit(
    mut commands: Commands,
    mut grid: ResMut<Grid>,
    query: Query<(Entity, &UnitStats, &UnitForce, &UnitInfo, &Transform), Changed<UnitStats>>,
    mut fx: ResMut<Events<FxSpawnEvent>>,
) {
    for (entity, stats, force, info, transform) in query.iter() {
        if stats.life <= 0 {
            commands.entity(entity).despawn();
            grid.change_by_count(info.target_x, info.target_y, -force.as_int());
            fx.send(FxSpawnEvent {
                kind: FxKind::Death,
                transform: transform.clone(),
                duration: None,
            });
        }
    }
}

fn add_time_on_unit_info(time: Res<Time>, mut query: Query<&mut UnitTime>) {
    let delta = time.delta_seconds();
    query.par_for_each_mut(64, |mut unit| {
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
        &UnitForce,
    )>,
) {
    for (unit_time, stats, mut state, mut info, mut transform, mut ai, force) in query.iter_mut() {
        update_pos(&unit_time, &info, &mut transform);

        if state.is_still() && info.last_x == ai.target_x && info.last_y == ai.target_y {
            if ai.stick_to_target {
                continue;
            } else {
                ai.target_x = random::<i32>().abs() % grid.x;
                ai.target_y = random::<i32>().abs() % grid.y;
                info!("Target {} {}", ai.target_x, ai.target_y);
            }
        }

        if unit_time.time > info.end_time {
            info.start_time = unit_time.time;
            info.end_time = unit_time.time + info.action_delay;
            let status_wanted = force.as_grid_status();
            *state = match &*state {
                UnitState::Still(dir) => {
                    let potential_pos = find_potential_pos(
                        &grid,
                        info.last_x,
                        info.last_y,
                        ai.target_x,
                        ai.target_y,
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

#[derive(Default, Component)]
pub struct UnitInfo {
    pub last_x: i32,
    pub last_y: i32,

    pub action_delay: f32,

    pub target_x: i32,
    pub target_y: i32,

    pub start_time: f32,
    pub end_time: f32,
}

#[derive(Default, Component)]
pub struct UnitTime {
    pub time: f32,
}

#[derive(Component)]
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

#[derive(Component)]
pub struct UnitForce {
    pub ally: bool,
}

impl UnitForce {
    pub fn as_int(&self) -> i32 {
        if self.ally {
            1
        } else {
            -1
        }
    }

    pub fn as_grid_status(&self) -> GridStatus {
        if self.ally {
            GridStatus::Friend
        } else {
            GridStatus::Enemy
        }
    }
}

#[derive(Component)]
pub struct TurningAI;

#[derive(Default, Component)]
pub struct MoveOnForceAI {
    pub target_x: i32,
    pub target_y: i32,
    pub stick_to_target: bool,
}

#[derive(Component)]
pub struct AttackingAI;

#[derive(Debug, Component)]
pub enum AttackingAIState {
    PrepareAttack,
    AfterAttack,
    MoveToNearestEnemy,
}

fn find_enemy_in_range(grid: &Grid, x: i32, y: i32, ally: bool, range: i32) -> Vec<(i32, i32)> {
    let mut result = Vec::new();
    let mut push = |(n_x, n_y)| {
        if ally && grid.get_status(n_x, n_y) == Some(GridStatus::Enemy) {
            result.push((n_x, n_y));
        } else if !ally && grid.get_status(n_x, n_y) == Some(GridStatus::Friend) {
            result.push((n_x, n_y))
        }
    };

    for cur_range in 1..=range {
        let x_range = (x + -cur_range + 1).max(0)..=(x + cur_range - 1).min(grid.x - 1);

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
fn find_enemy_in_corner() {
    let mut grid = Grid::new(4, 4);
    grid.add_friend(0, 0);
    grid.add_enemy(3, 3);

    assert_eq!(find_enemy_in_range(&grid, 0, 0, true, 10), vec![(3, 3)]);
    assert_eq!(find_enemy_in_range(&grid, 3, 3, false, 10), vec![(0, 0)]);
}

#[test]
fn enemy_in_range() {
    let mut grid = Grid::new(2, 2);

    grid.add_enemy(1, 1);

    assert_eq!(find_enemy_in_range(&grid, 1, 1, true, 1), vec![]);
    assert_eq!(find_enemy_in_range(&grid, 1, 1, false, 1), vec![]);
    assert_eq!(find_enemy_in_range(&grid, 0, 0, false, 1), vec![]);

    assert_eq!(find_enemy_in_range(&grid, 1, 0, true, 1), vec![(1, 1)]);
    assert_eq!(find_enemy_in_range(&grid, 0, 1, true, 1), vec![(1, 1)]);
    grid = Grid::new(5, 5);
    grid.add_friend(3, 3);
    for i in 0..6 {
        assert_eq!(find_enemy_in_range(&grid, 0, 0, false, i), vec![]);
    }

    for i in 6..25 {
        assert_eq!(find_enemy_in_range(&grid, 0, 0, false, i), vec![(3, 3)]);
    }

    grid.add_friend(3, 4);
    grid.add_friend(4, 3);

    let mut final_test = find_enemy_in_range(&grid, 0, 0, false, 8);
    final_test.sort();
    assert_eq!(final_test, vec![(3, 3), (3, 4), (4, 3)]);
}

pub fn update_attacking_ai(
    mut grid: ResMut<Grid>,
    mut damage_events: ResMut<Events<DamageEvent>>,
    mut fx_events: ResMut<Events<FxSpawnEvent>>,
    mut query: Query<
        (
            &mut AttackingAIState,
            &mut UnitInfo,
            &UnitStats,
            &UnitTime,
            &UnitForce,
            &mut UnitState,
            &mut GridTransform,
            &Transform,
        ),
        With<AttackingAI>,
    >,
) {
    for (mut state, mut info, stats, time, force, mut anim_state, mut transform, trans) in
        query.iter_mut()
    {
        update_pos(&time, &info, &mut transform);

        // Do I need to do something else?
        if info.end_time > time.time {
            continue;
        }

        // Find an enemy that is 1 cell away since it is useful in all cases
        let enemy_close = find_enemy_in_range(&grid, info.last_x, info.last_y, force.ally, 1)
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
                        from: force.ally,
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
                let duration = 1.0 / stats.attack_speed;
                let mut t = trans.clone();
                t.scale = t.scale / 2.0;
                fx_events.send(FxSpawnEvent {
                    duration: Some(duration),
                    kind: FxKind::Fire,
                    transform: t,
                });
                (
                    duration,
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
                    find_enemy_in_range(&grid, info.last_x, info.last_y, force.ally, 1000)
                        .iter()
                        .next()
                {
                    if let Some((d, x, y)) = find_potential_pos(
                        &grid,
                        info.last_x,
                        info.last_y,
                        enemy_x.clone(),
                        enemy_y.clone(),
                        GridStatus::Neutral,
                    ) {
                        grid_info_move_to(&mut grid, &mut info, x, y, force.ally);
                        (1.0 / stats.move_speed, UnitState::Moving(d))
                    } else {
                        (1.0, UnitState::Still(Direction::Down))
                    }
                } else {
                    (1.0, UnitState::Still(Direction::Down))
                }
            }
        };

        info!(
            "Change state: {} {} {:?}",
            info.start_time, info.end_time, new_state
        );
        info.start_time = time.time;
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
    with_unit: impl FnOnce(&mut EntityCommands<'_, '_, '_>),
) where
    G: Deref<Target = Grid> + DerefMut,
    TA: Deref<Target = Assets<TextureAtlas>> + DerefMut,
{
    let path = if ally {
        "spritesheet/Female/Female 12-3.png"
    } else {
        "spritesheet/Soldier/Soldier 01-1.png"
    };
    let texture_handle = asset_server.load(path);
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(32.0, 32.0), 3, 4);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    grid.get_count(x, y)
        .expect("Expected valid position for the new unit");
    grid.change_by_count(x, y, if ally { 1 } else { -1 });

    UnitBundle {
        spritesheet: SpriteSheetBundle {
            texture_atlas: texture_atlas_handle.clone(),
            transform: Transform {
                translation: Vec3::splat(-100.0),
                scale: Vec3::splat(3.0),
                ..Default::default()
            },
            ..Default::default()
        },
        unit_info: UnitInfo {
            last_x: x,
            last_y: y,
            target_x: x,
            target_y: y,
            action_delay: 1.0,
            start_time: f32::MAX,
            end_time: f32::MIN,
        },
        unit_state: UnitState::Moving(crate::unit::Direction::Right),
        unit_stats: Default::default(),
    }
    .build(commands, |c| {
        c.insert(UnitForce { ally: ally });
        with_unit(c);
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::camera::init_cameras_2d;
    use crate::utils::tests::*;

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
            mut commands: Commands,
            asset_server: ResMut<AssetServer>,
            mut grid: ResMut<Grid>,
            mut texture_atlases: ResMut<Assets<TextureAtlas>>,
        ) {
            grid.add_enemy(1, 0);
            spawn_unit(
                &mut commands,
                &asset_server,
                &mut grid,
                &mut texture_atlases,
                0,
                0,
                true,
                |c| {
                    c.insert(MoveOnForceAI::default());
                },
            )
        }
        App::new()
            .add_plugin(Test::Frames(10))
            .add_plugin(GridPlugin)
            .add_plugin(FxPlugin)
            .add_plugin(UnitPlugin)
            .add_system(init_cameras_2d)
            .insert_resource(Grid::new(2, 1))
            .add_startup_system(init)
            .add_system(assert_stay_on_0_0)
            .run();
    }

    #[test]
    #[serial]
    fn move_on_force_enemy_wont_go_on_ally() {
        fn init(
            mut commands: Commands,
            asset_server: ResMut<AssetServer>,
            mut grid: ResMut<Grid>,
            mut texture_atlases: ResMut<Assets<TextureAtlas>>,
        ) {
            grid.add_friend(0, 1);
            spawn_unit(
                &mut commands,
                &asset_server,
                &mut grid,
                &mut texture_atlases,
                0,
                0,
                false,
                |c| {
                    c.insert(MoveOnForceAI::default());
                },
            )
        }
        App::new()
            .add_plugin(Test::Frames(10))
            .add_plugin(GridPlugin)
            .add_plugin(FxPlugin)
            .add_plugin(UnitPlugin)
            .add_system(init_cameras_2d)
            .insert_resource(Grid::new(2, 1))
            .add_startup_system(init)
            .add_system(assert_stay_on_0_0)
            .run();
    }

    #[test]
    #[ignore]
    #[serial]
    fn battle_of_two_warriors() {
        fn init(
            mut commands: Commands,
            asset_server: Res<AssetServer>,
            mut grid: ResMut<Grid>,
            mut texture_atlases: ResMut<Assets<TextureAtlas>>,
        ) {
            spawn_unit(
                &mut commands,
                &asset_server,
                &mut grid,
                &mut texture_atlases,
                0,
                0,
                true,
                |c| {
                    c.insert(AttackingAI)
                        .insert(AttackingAIState::MoveToNearestEnemy);
                },
            );

            spawn_unit(
                &mut commands,
                &asset_server,
                &mut grid,
                &mut texture_atlases,
                3,
                3,
                false,
                |c| {
                    c.insert(AttackingAI)
                        .insert(AttackingAIState::MoveToNearestEnemy);
                },
            )
        }

        fn check_unit_count(mut flag: ResMut<TestCheck<bool>>, query: Query<&UnitState>) {
            if query.iter().len() == 1 {
                **flag = true;
            }

            if query.iter().len() == 0 {
                **flag = false;
            }
        }

        App::new()
            .add_plugin(Test::Time(15.0))
            .add_plugin(GridPlugin)
            .add_plugin(FxPlugin)
            .add_plugin(UnitPlugin)
            .add_system(init_cameras_2d)
            .insert_resource(Grid::new(4, 4))
            .insert_resource(TestCheck::new(false).is_true())
            .add_startup_system(init)
            .add_system(check_unit_count)
            .run();
    }

    #[test]
    #[serial]
    fn dead_unit_are_removed_from_grid() {
        fn init(
            mut commands: Commands,
            asset_server: Res<AssetServer>,
            mut grid: ResMut<Grid>,
            mut texture_atlases: ResMut<Assets<TextureAtlas>>,
        ) {
            spawn_unit(
                &mut commands,
                &asset_server,
                &mut grid,
                &mut texture_atlases,
                0,
                0,
                true,
                |c| {
                    c.insert(AttackingAI)
                        .insert(AttackingAIState::MoveToNearestEnemy)
                        .insert(UnitStats {
                            life: 0,
                            ..Default::default()
                        });
                },
            )
        }

        fn check_grid_when_no_unit(grid: Res<Grid>, units: Query<&UnitStats>) {
            if units.iter().len() > 0 {
                return;
            }
            for x in 0..grid.x {
                for y in 0..grid.y {
                    assert_eq!(grid.get_status(x, y).unwrap(), GridStatus::Neutral);
                }
            }
        }

        fn expect_0_unit(mut flag: ResMut<TestCheck<bool>>, query: Query<&UnitStats>) {
            if query.iter().len() == 0 {
                **flag = true;
            }
        }

        App::new()
            .add_plugin(Test::Frames(3))
            .add_plugin(GridPlugin)
            .add_plugin(UnitPlugin)
            .add_plugin(FxPlugin)
            .add_system(init_cameras_2d)
            .insert_resource(Grid::new(4, 4))
            .insert_resource(TestCheck::new(false).is_true())
            .add_startup_system(init)
            .add_system_to_stage(CoreStage::PostUpdate, check_grid_when_no_unit)
            .add_system_to_stage(CoreStage::PostUpdate, expect_0_unit)
            .run();
    }
}
