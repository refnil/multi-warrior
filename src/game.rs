//! # Multi warrior library
//!
//! The plugin Game is the main one and include everything else needed to run the game.
use bevy::prelude::*;

mod button;
mod fps;
mod grid;
mod input;
mod unit;
mod utils;
mod camera;

use button::*;
use fps::FPSPlugin;
use grid::*;
use input::InputPlugin;
use unit::*;
use camera::*;

pub struct Game;

impl Plugin for Game {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugins(DefaultPlugins)
            .add_plugin(FPSPlugin { color: Color::BLACK })
            .add_plugin(UnitPlugin::default())
            .add_plugin(GridPlugin::default())
            .add_plugin(InputPlugin::default())
            .add_plugin(ButtonPlugin::default())
            .add_resource(Grid::new(10, 10))
            .add_startup_system(init_cameras.system())
            .add_startup_system(add_some_friend_and_enemy.system())
            .add_startup_system(test::spawn_unit.system())

            .add_system(change_grid_randomly.system())
            .add_system(on_button_click.system())

            //.add_system(crate::utils::count_query::<(&TextureAtlasSprite,)>.system())
        ;
    }
}

#[derive(Clone)]
struct StateSetter {
    state: UnitState,
    entity: Entity,
}

fn on_button_click(
    query: Query<(&StateSetter, &Interaction), (Mutated<Interaction>, With<Button>)>,
    mut update_query: Query<&mut UnitState>,
) {
    for (state, interaction) in query.iter() {
        if interaction == &Interaction::Clicked {
            if let Ok(mut unit_state) = update_query.get_mut(state.entity) {
                *unit_state = state.state.clone();
            }
        }
    }
}

fn add_some_friend_and_enemy(mut grid: ResMut<Grid>) {
    grid.add_friend(0, 0);
    let x = grid.x - 1;
    let y = grid.y - 1;
    grid.add_enemy(x, y);
}

mod test {
    use super::*;

    pub fn spawn_unit(
        commands: &mut Commands,
        asset_server: Res<AssetServer>,
        mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    ) {
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
                last_x: 2,
                last_y: 2,
                target_x: 2,
                target_y: 2,
                action_delay: 1.0,
                ..Default::default()
            },
            unit_state: UnitState::Moving(crate::unit::Direction::Right),
        }
        .build(commands)
        .with(MoveOnForceAI { ally: false });
    }

    #[allow(dead_code)]
    pub fn spawn_dancing_unit(
        commands: &mut Commands,
        asset_server: Res<AssetServer>,
        mut texture_atlases: ResMut<Assets<TextureAtlas>>,
        spawner: Res<ButtonSpawner>,
    ) {
        let texture_handle = asset_server.load("spritesheet/Female/Female 12-3.png");
        let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(32.0, 32.0), 3, 4);
        let texture_atlas_handle = texture_atlases.add(texture_atlas);
        for i in 0..6 {
            for j in 0..6 {
                UnitBundle {
                    spritesheet: SpriteSheetBundle {
                        texture_atlas: texture_atlas_handle.clone(),
                        transform: Transform::from_scale(Vec3::splat(3.0)),
                        ..Default::default()
                    },
                    unit_info: UnitInfo {
                        last_x: 2 * i,
                        last_y: 2 * j,
                        target_x: 2 * i,
                        target_y: 2 * j,
                        action_delay: 2.0 + ((i + 1) as f32 * (j + 1) as f32).sin(),
                        ..Default::default()
                    },
                    unit_state: UnitState::Moving(crate::unit::Direction::Right),
                }
                .build(commands)
                .with(TurningAI);
            }
        }
        let unit = commands.current_entity().unwrap();

        spawner.spawn_button(commands, "Left".to_string(), None);
        commands.with(StateSetter {
            state: UnitState::Still(crate::unit::Direction::Left),
            entity: unit,
        });
        spawner.spawn_button(commands, "Right".to_string(), None);
        commands.with(StateSetter {
            state: UnitState::Still(crate::unit::Direction::Right),
            entity: unit,
        });
        spawner.spawn_button(commands, "Down".to_string(), None);
        commands.with(StateSetter {
            state: UnitState::Still(crate::unit::Direction::Down),
            entity: unit,
        });
        spawner.spawn_button(commands, "Up".to_string(), None);
        commands.with(StateSetter {
            state: UnitState::Still(crate::unit::Direction::Up),
            entity: unit,
        });
    }
}
