use bevy::prelude::*;
use bevy::sprite::entity::*;

#[derive(Default)]
pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut AppBuilder){
        app.add_resource(AnimTimer { timer: Timer::from_seconds(0.1, true) })
           //.add_system(move_unit_system.system())
           .add_system(animate_sprite_system.system())
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
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(&mut TextureAtlasSprite, &Handle<TextureAtlas>)>,
) {
    timer.timer.tick(time.delta_seconds);
    if timer.timer.finished {
        for (mut sprite, texture_atlas_handle) in query.iter_mut() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            sprite.index = ((sprite.index as usize + 1) % texture_atlas.textures.len()) as u32;
        }
    }
}

#[derive(Default)]
pub struct UnitBundle {
    pub spritesheet: SpriteSheetBundle,
    pub unit_info: UnitInfo,
}

impl UnitBundle {
    pub fn build(self, commands: &mut Commands) {
        commands.spawn(self.spritesheet)
            .with(self.unit_info);
    }
}

struct AnimTimer {
    timer: Timer
}

pub enum UnitState {
    Still,
    Moving,
    Attacking,
}

impl Default for UnitState {
    fn default() -> Self {
        Self::Still
    }
}

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

    pub state: UnitState,
    pub watch_direction: Direction,
}
