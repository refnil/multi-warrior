use bevy::prelude::*;
use bevy::render::camera::*;
use bevy::ecs::*;
use rand::random;

use crate::fps::FPSPlugin;

pub struct Game;

impl Plugin for Game {
    fn build(&self, app: &mut AppBuilder){
        app.add_plugins(DefaultPlugins)
            .add_plugin(FPSPlugin { color: Color::BLACK })
            .add_resource(Grid::new(10, 20))
            .init_resource::<GridRenderDebug>()
            .add_startup_system(init_cameras.system())
            .add_startup_system(init_render_grid.system())
            .add_startup_system(add_some_friend_and_enemy.system())
            .add_system(draw_grid.system())
            .add_system(draw_grid_debug.system())
            .add_system(change_grid_randomly.system());
    }
}

struct GridRenderDebug {
    nothing_color: Handle<ColorMaterial>,
    friend_color: Handle<ColorMaterial>,
    enemy_color: Handle<ColorMaterial>,
    visible: bool
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
            visible: true
        }
    }
}

struct Skill {
}

fn init_cameras(mut commands: Commands){
    // 2d camera
    commands.spawn(Camera2dComponents::default());
    // UI camera
    commands.spawn(UiCameraComponents::default());
    // Maybe they should have another component each to differenciate them
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


fn draw_grid(mut commands: Commands, grid: Res<Grid>){
}

fn draw_grid_debug(grid_debug: Res<GridRenderDebug>, grid: Res<Grid>, mut debug_query: Query<(&GridRenderDebugNode, &mut Transform, &mut Handle<ColorMaterial>)>, query_proj: Query<&OrthographicProjection>) {
    coz::scope!("draw_grid_debug");
    if let Some(proj) = query_proj.iter().next() {
        // Put the debug node at the right place in the camera
        let width = (proj.right - proj.left)/ grid.x as f32;
        let height = (proj.top - proj.bottom)/ grid.y as f32;

        for (node, mut transform, mut material) in debug_query.iter_mut() {

            let nodex = node.x as f32;
            let nodey = node.y as f32;
            let startx = proj.left + width * nodex + width * 0.5;
            let starty = proj.bottom + height * nodey + height * 0.5;

            transform.translation = Vec3::new(startx, starty, 0.0);
            transform.scale = Vec3::new(width, height, 1.0);

            let status = grid.get_status(node.x, node.y);
            *material = match status {
                GridStatus::Neutral => grid_debug.nothing_color.clone(),
                GridStatus::Friend => grid_debug.friend_color.clone(),
                GridStatus::Enemy => grid_debug.enemy_color.clone()
            };
        }
    }
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

struct Grid {
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
