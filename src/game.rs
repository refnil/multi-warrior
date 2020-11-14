use bevy::prelude::*;
use bevy::render::camera::*;
use bevy::ecs::*;
use rand::random;

use crate::fps::FPSPlugin;
use crate::unit::*;

pub struct Game;

impl Plugin for Game {
    fn build(&self, app: &mut AppBuilder){
        app.add_plugins(DefaultPlugins)
            .add_plugin(FPSPlugin { color: Color::BLACK })
            .add_plugin(UnitPlugin::default())
            .add_resource(Grid::new(3, 5))
            .init_resource::<GridRenderDebug>()
            .add_startup_system(init_cameras.system())
            .add_startup_system(init_render_grid.system())
            .add_startup_system(add_some_friend_and_enemy.system())
            .add_startup_system(spawn_unit.system())

            .add_system(change_grid_randomly.system())

            .add_system(update_grid_debug_visible.system())
            .add_system(update_grid_render_debug.system()) 
            .add_system(update_grid_transform.system()) 
            .add_system(update_grid_color.system())

            /*
            .add_system(_count_query::<(&Handle<TextureAtlas>,)>.system())
            .add_system(_count_query::<(&TextureAtlasSprite,)>.system())
            .add_system(_count_query::<(&Draw,)>.system())
            .add_system(_count_query::<(&UnitInfo,)>.system())
            */
        ;
    }
}

pub struct GridRenderDebug {
    nothing_color: Handle<ColorMaterial>,
    friend_color: Handle<ColorMaterial>,
    enemy_color: Handle<ColorMaterial>,
    visible: bool,

    left: f32,
    right: f32,
    top: f32,
    bottom: f32,

    width: f32,
    height: f32
}

struct GridRenderDebugNode {
    x: i32,
    y: i32
}

impl FromResources for GridRenderDebug {
    fn from_resources(res: &Resources) -> Self {
        let mut materials = res.get_mut::<Assets<ColorMaterial>>().unwrap();
        Self {
            nothing_color: materials.add(Color::rgb(1.0,1.0,1.0).into()),
            friend_color: materials.add(Color::rgb(0.0, 1.0, 0.0).into()),
            enemy_color: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
            visible: false,

            left:0.0,
            right:0.0,
            top:0.0,
            bottom:0.0,

            width: 0.0,
            height: 0.0
        }
    }
}

struct MainCamera;
struct UICamera;

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

fn init_render_grid(mut commands: Commands, 
                    grid: Res<Grid>) {
    for x in 0..grid.x {
        for y in 0..grid.y {
            commands.spawn(SpriteComponents {
                sprite: Sprite::new(Vec2::new(1.0,1.0)),
                ..Default::default()
            });
            commands.with(GridRenderDebugNode { x:x, y:y });
        }
    }
}

fn add_some_friend_and_enemy(mut grid: ResMut<Grid>) {
    grid.add_friend(0,0);
    let x = grid.x - 1;
    let y = grid.y - 1;
    grid.add_enemy(x, y);
}

fn update_grid_debug_visible(input: Res<Input<KeyCode>>, mut info: ResMut<GridRenderDebug>) {
    if input.just_pressed(KeyCode::G) {
        info.visible = !info.visible;
        println!("visible {}", info.visible);
    }
}

fn update_grid_render_debug(grid: Res<Grid>, mut info: ResMut<GridRenderDebug>, _main_camera: &MainCamera, proj: &OrthographicProjection){
    info.left = proj.left;
    info.right = proj.right;
    info.top = proj.top;
    info.bottom = proj.bottom;

    info.width = (info.right - info.left)/ grid.x as f32;
    info.height = (info.top - info.bottom)/ grid.y as f32;
}

fn update_grid_transform(info: Res<GridRenderDebug>, node: &GridRenderDebugNode, mut transform: Mut<Transform>) {
    let nodex = node.x as f32;
    let nodey = node.y as f32;
    let startx = info.left + info.width * (nodex + 0.5);
    let starty = info.bottom + info.height * (nodey + 0.5);

    transform.translation = Vec3::new(startx, starty, 0.0);
    transform.scale = Vec3::new(info.width, info.height, 1.0);
}

fn update_grid_color(grid: Res<Grid>, grid_debug: Res<GridRenderDebug>, node: &GridRenderDebugNode, mut material: Mut<Handle<ColorMaterial>>, mut draw: Mut<Draw>){
    let status = grid.get_status(node.x, node.y);
    let target_material = match status {
        GridStatus::Neutral => &grid_debug.nothing_color,
        GridStatus::Friend => &grid_debug.friend_color,
        GridStatus::Enemy => &grid_debug.enemy_color
    };

    if *material != *target_material {
        *material = target_material.clone();
    }

    draw.is_visible = grid_debug.visible;
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

fn _count_query<Q: HecsQuery>(mut query: Query<Q>){
    println!("{}", query.iter_mut().count());
}

pub struct Grid {
    people_by_case: Vec<i32>,
    pub x: i32,
    pub y: i32,
}

enum GridStatus {
    Friend,
    Neutral,
    Enemy
}

impl Grid {
    pub fn new(x: i32, y: i32) -> Grid {
        Grid {
            people_by_case: vec![0; (x * y) as usize],
            x: x,
            y: y
        }
    }

    fn to_pos(self: &Grid, x: i32, y: i32) -> Option<usize> {
        if 0 <= x && x < self.x && 0 <= y && y < self.y {
            Some((x * self.y + y) as usize)
        }
        else {
            None
        }
    }

    pub fn add_friend(self: &mut Grid, x: i32, y: i32) -> bool {
        if let Some(pos) = self.to_pos(x, y) {
            if self.people_by_case[pos] >= 0 {
                self.people_by_case[pos] += 1;
                return true;
            }
        }        
        return false;
    }

    pub fn add_enemy(self: &mut Grid, x: i32, y: i32) -> bool {
        if let Some(pos) = self.to_pos(x, y) {
            if self.people_by_case[pos] <= 0 {
                self.people_by_case[pos] -= 1;
                return true;
            }
        }        
        return false;
    }

    pub fn change_by_count(self: &mut Grid, x: i32, y: i32, change: i32){
        if let Some(pos) = self.to_pos(x, y) {
            self.people_by_case[pos] += change;
        }        
    }

    pub fn get_status(self: &Grid, x: i32, y: i32) -> GridStatus {
        coz::scope!("grid::get_status");
        let count = self.get_count(x, y);
        if count == 0 {
            GridStatus::Neutral
        }
        else if count < 0 {
            GridStatus::Enemy
        }
        else {
            GridStatus::Friend
        }
    }

    pub fn get_count(self: &Grid, x: i32, y: i32) -> i32 {
        if let Some(pos) = self.to_pos(x, y) {
            return self.people_by_case[pos]
        }        
        return 0;
    }
}
