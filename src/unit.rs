use bevy::prelude::*;
use bevy::sprite::entity::*;

use crate::grid::*;

#[derive(Default)]
pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_resource(AnimTimer {
            timer: Timer::from_seconds(0.1, true),
        })
        //.add_system(move_unit_system.system())
        .add_system(unit_update.system())
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
    pub fn build(self, commands: &mut Commands) -> &mut Commands{
        commands
            .spawn(self.spritesheet)
            .with(self.unit_info)
            .with(self.unit_state.get_animation())
            .with(self.unit_state)
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
    Attacking,
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
            _ => panic!("get_animate: {:?}", self),
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

#[derive(Debug, Clone)]
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

fn unit_update(
    time: Res<Time>,
    grid_proj: Res<GridRenderDebug>,
    mut query: Query<(&mut UnitState, &mut UnitInfo, &mut Transform), With<TurningAI>>,
) {
    for (mut state, mut info, mut transform) in query.iter_mut() {
        info.time += time.delta_seconds();
        update_pos(&grid_proj, &info, &mut transform);
        if info.time > info.end_time {
            info.start_time = info.time;
            info.end_time = info.time + info.action_delay;
            *state = match &*state {
                UnitState::Still(dir) => {
                    let new_dir = dir.next();
                    info.target_x = info.last_x + new_dir.x();
                    info.target_y = info.last_y + new_dir.y();
                    UnitState::Moving(new_dir)
                }
                UnitState::Moving(dir) => {
                    info.last_x = info.target_x;
                    info.last_y = info.target_y;

                    UnitState::Still(dir.clone())
                }
                UnitState::Attacking => {
                    panic!("TurningAI doesn't attack.");
                }
            };
        }
    }
}

fn update_pos(grid: &GridRenderDebug, info: &UnitInfo, mut transform: &mut Transform) {
    let ratio = (info.time - info.start_time) / (info.end_time - info.start_time);
    let x = info.last_x as f32 + ratio * (info.target_x - info.last_x) as f32;
    let y = info.last_y as f32 + ratio * (info.target_y - info.last_y) as f32;
    transform.translation = grid.pos(x, y);
}

#[derive(Default)]
pub struct UnitInfo {
    pub last_x: i32,
    pub last_y: i32,

    pub action_delay: f32,

    pub target_x: i32,
    pub target_y: i32,

    pub time: f32,
    pub start_time: f32,
    pub end_time: f32,
}

struct UnitMovingState {}

#[derive(Default)]
pub struct UnitStats {
    pub speed: f32,
}

pub struct TurningAI;
