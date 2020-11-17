use bevy::prelude::*;
use rand::random;

use crate::fps::FPSPlugin;
use crate::grid::*;
use crate::unit::*;
use crate::utils::*;

pub struct Game;

impl Plugin for Game {
    fn build(&self, app: &mut AppBuilder){
        app.add_plugins(DefaultPlugins)
            .add_plugin(FPSPlugin { color: Color::BLACK })
            .add_plugin(UnitPlugin::default())
            .add_plugin(GridPlugin::default())
            .add_resource(Grid::new(3, 5))
            .add_startup_system(init_cameras.system())
            .add_startup_system(add_some_friend_and_enemy.system())
            .add_startup_system(spawn_unit.system())

            .add_system(change_grid_randomly.system())

            .add_system(count_query::<(&TextureAtlasSprite,)>.system())
        ;
    }
}

pub struct MainCamera;
pub struct UICamera;

fn init_cameras(mut commands: Commands){
    // 2d camera
    commands.spawn(Camera2dComponents::default());
    commands.with(MainCamera);
    // UI camera
    commands.spawn(UiCameraComponents::default());
    commands.with(UICamera);
    // Maybe they should have another component each to differenciate them
}

fn spawn_unit(mut commands: Commands, asset_server: Res<AssetServer>, mut texture_atlases: ResMut<Assets<TextureAtlas>>){
    let texture_handle = asset_server.load("spritesheet/Female/Female 12-3.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(32.0, 32.0), 3, 4);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    UnitComponents {
        spritesheet: SpriteSheetComponents {
            texture_atlas: texture_atlas_handle,
            transform: Transform::from_scale(Vec3::splat(6.0)),
            ..Default::default()
        },
        unit_info: UnitInfo {
            last_x: 1,
            last_y: 1,
            ..Default::default()
        },
    }.build(&mut commands);
}


fn add_some_friend_and_enemy(mut grid: ResMut<Grid>) {
    grid.add_friend(0,0);
    let x = grid.x - 1;
    let y = grid.y - 1;
    grid.add_enemy(x, y);
}

fn change_grid_randomly(mut grid: ResMut<Grid>){
    coz::scope!("change_grid_randomly");
    let max_x = grid.x as u16;
    let max_y = grid.y as u16;

    let random_x = (random::<u16>() % max_x) as i32;
    let random_y = (random::<u16>() % max_y) as i32;

    let random_change = (random::<u16>() % 3) as i32 - 1;

    grid.change_by_count(random_x, random_y, random_change);
}

