use bevy::prelude::*;
use bevy::tasks::prelude::*;

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
        .add_system(add_time_on_unit_info.system())
        .add_system(turning_ai_update.system())
        .add_system(move_on_ai_force_update.system())
        .add_system_to_stage(stage::POST_UPDATE, update_animation_from_state.system())
        .add_system_to_stage(stage::POST_UPDATE, animate_sprite_system.system());
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

fn move_on_ai_force_update(
    mut grid: ResMut<Grid>,
    mut query: Query<(
        &UnitTime,
        &mut UnitState,
        &mut UnitInfo,
        &mut GridTransform,
        &MoveOnForceAI,
    )>,
) {
    for (unit_time, mut state, mut info, mut transform, force) in query.iter_mut() {
        update_pos(&unit_time, &info, &mut transform);
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
                    let mut potential_pos: Option<(Direction, i32, i32)> = None;
                    for d in Direction::iter() {
                        let x = info.last_x + d.x();
                        let y = info.last_y + d.y();
                        if let Some(status) = grid.get_status(x, y) {
                            if status == status_wanted || status == GridStatus::Neutral {
                                potential_pos = Some((d, x, y));
                            }
                        }
                    }

                    if let Some((d, x, y)) = potential_pos {
                        let count_change = if force.ally { 1 } else { -1 };
                        grid.change_by_count(info.last_x, info.last_y, -count_change);
                        grid.change_by_count(x, y, count_change);
                        info.target_x = x;
                        info.target_y = y;

                        UnitState::Moving(d)
                    } else {
                        UnitState::Still(dir.next())
                    }
                }
                UnitState::Moving(dir) => {
                    info.last_x = info.target_x;
                    info.last_y = info.target_y;

                    UnitState::Still(dir.next().next())
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

#[derive(Default)]
pub struct UnitStats {
    pub speed: f32,
}

pub struct TurningAI;
pub struct MoveOnForceAI {
    pub ally: bool,
}
