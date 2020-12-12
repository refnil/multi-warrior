use bevy::prelude::*;
use rand::random;

use crate::button::*;
use crate::fps::FPSPlugin;
use crate::grid::*;
use crate::input::InputPlugin;
use crate::unit::*;

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
            .add_startup_system(spawn_unit.system())

            .add_system(change_grid_randomly.system())
            .add_system(on_button_click.system())

            //.add_system(count_query::<(&TextureAtlasSprite,)>.system())
        ;
    }
}

pub struct MainCamera;
pub struct UICamera;

fn init_cameras(commands: &mut Commands) {
    // 2d camera
    commands.spawn(Camera2dBundle::default());
    commands.with(MainCamera);
    // UI camera
    commands.spawn(CameraUiBundle::default());
    commands.with(UICamera);
    // Maybe they should have another component each to differenciate them
}

fn spawn_unit(
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
                last_x: 2*i,
                last_y: 2*j,
                target_x: 2*i,
                target_y: 2*j,
                action_delay: 2.0 + ((i+1) as f32*(j+1) as f32).sin(),
                ..Default::default()
            },
            unit_state: UnitState::Moving(crate::unit::Direction::Right),
        }
        .build(commands);
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

fn change_grid_randomly(mut grid: ResMut<Grid>) {
    coz::scope!("change_grid_randomly");
    let max_x = grid.x as u16;
    let max_y = grid.y as u16;

    let random_x = (random::<u16>() % max_x) as i32;
    let random_y = (random::<u16>() % max_y) as i32;

    let random_change = (random::<u16>() % 3) as i32 - 1;

    grid.change_by_count(random_x, random_y, random_change);
}
