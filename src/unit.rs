use bevy::prelude::*;
use bevy::sprite::entity::*;

#[derive(Default)]
pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut AppBuilder){
        app.add_resource(AnimTimer { timer: Timer::from_seconds(0.1, true) })
           //.add_system(move_unit_system.system())
           .add_system_to_stage(stage::POST_UPDATE, update_animation_from_state.system())
           .add_system_to_stage(stage::POST_UPDATE, animate_sprite_system.system())
        ;
    }
}

/*
fn move_unit_system(grid: ResMut<Grid>, grid_info: Res<GridRenderDebug>, transform: Mut<Transform>, unit_info: Mut<UnitInfo>) {
}
*/

fn animate_sprite_system(
    time: Res<Time>,
    mut timer: ResMut<AnimTimer>,
    mut query: Query<(&mut TextureAtlasSprite, &mut Animation)>,
) {
    timer.timer.tick(time.delta_seconds);
    if timer.timer.finished {
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
    pub unit_state: UnitState
}

impl UnitBundle {
    pub fn build(self, commands: &mut Commands) {
        commands.spawn(self.spritesheet)
            .with(self.unit_info)
            .with(self.unit_state.get_animation())
            .with(self.unit_state);
    }
}

struct AnimTimer {
    timer: Timer
}

struct Animation {
    current_frame: u32,
    frames: Vec<u32>,
}

#[derive(Debug, Clone)]
pub enum UnitState {
    Still(Direction),
    Moving(u32, u32, Direction),
    Attacking,
}

impl UnitState {
    fn get_animation(&self) -> Animation {
       let frames = match self {
            Self::Still(Direction::Down) => vec![1],
            Self::Still(Direction::Right) => vec![7],
            Self::Still(Direction::Up) => vec![10],
            Self::Still(Direction::Left) => vec![4],

            Self::Moving(_,_,Direction::Down) => vec![1, 2, 0],
            Self::Moving(_,_,Direction::Right) => vec![7, 8, 7, 6],
            Self::Moving(_,_,Direction::Up) => vec![10, 11, 9],
            Self::Moving(_,_,Direction::Left) => vec![4, 3, 2],
            _ => panic!("get_animate: {:?}", self)
       };
       Animation {
           current_frame: 0,
           frames: frames
       }
    }
}

fn update_animation_from_state(
    mut query: Query<
        (&UnitState, &mut Animation),
        Changed<UnitState>
    >){
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
    Down
}

impl Default for Direction {
    fn default() -> Self {
        Self::Down
    }
}

#[derive(Default)]
pub struct UnitInfo {
    pub last_x: i32,
    pub last_y: i32,

    pub target_x: i32,
    pub target_y: i32,
}

