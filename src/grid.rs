use bevy::prelude::*;
use bevy::render::camera::*;

use crate::camera::*;

#[derive(Default)]
pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<GridRenderDebug>()
            .add_startup_system(init_render_grid.system())
            .add_system(update_grid_debug_visible.system())
            .add_system(update_grid_render_debug.system())
            .add_system(update_grid_transform.system())
            .add_system(update_grid_color.system());
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
    height: f32,
}

impl GridRenderDebug {
    pub fn scale(&self) -> Vec3 {
        Vec3::new(self.width, self.height, 1.0)
    }

    pub fn pos(&self, x: f32, y: f32) -> Vec3 {
        let startx = self.left + self.width * (x + 0.5);
        let starty = self.bottom + self.height * (y + 0.5);
        Vec3::new(startx, starty, -starty / 10000.0)
    }
}

struct GridRenderDebugNode;

pub struct GridTransform {
    pub x: f32,
    pub y: f32,
    pub update_scale: bool,
}

impl GridTransform {
    pub fn on(x: i32, y: i32) -> GridTransform {
        GridTransform {
            x: x as f32,
            y: y as f32,
            update_scale: true,
        }
    }
}

impl FromResources for GridRenderDebug {
    fn from_resources(res: &Resources) -> Self {
        let mut materials = res.get_mut::<Assets<ColorMaterial>>().unwrap();
        Self {
            nothing_color: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
            friend_color: materials.add(Color::rgb(0.0, 1.0, 1.0).into()),
            enemy_color: materials.add(Color::rgb(1.0, 1.0, 0.0).into()),
            visible: false,

            left: 0.0,
            right: 0.0,
            top: 0.0,
            bottom: 0.0,

            width: 0.0,
            height: 0.0,
        }
    }
}

fn init_render_grid(commands: &mut Commands, grid: Res<Grid>) {
    for x in 0..grid.x {
        for y in 0..grid.y {
            commands.spawn(SpriteBundle {
                sprite: Sprite::new(Vec2::new(1.0, 1.0)),
                ..Default::default()
            });
            commands.with(GridRenderDebugNode);
            commands.with(GridTransform::on(x, y));
        }
    }
}

fn update_grid_debug_visible(input: Res<Input<KeyCode>>, mut info: ResMut<GridRenderDebug>) {
    if input.just_pressed(KeyCode::G) {
        info.visible = !info.visible;
        println!("Changing grid debug visibility to {}", info.visible);
    }
}

fn update_grid_render_debug(
    grid: Res<Grid>,
    mut info: ResMut<GridRenderDebug>,
    query: Query<(&MainCamera, &OrthographicProjection)>,
) {
    for (_camera, proj) in query.iter() {
        info.left = proj.left;
        info.right = proj.right;
        info.top = proj.top;
        info.bottom = proj.bottom;

        info.width = (info.right - info.left) / grid.x as f32;
        info.height = (info.top - info.bottom) / grid.y as f32;
    }
}

fn update_grid_transform(
    info: Res<GridRenderDebug>,
    mut query: Query<(&GridTransform, &mut Transform)>,
) {
    for (node, mut transform) in query.iter_mut() {
        transform.translation = info.pos(node.x, node.y);
        if node.update_scale {
            transform.scale = info.scale();
        }
    }
}

fn update_grid_color(
    grid: Res<Grid>,
    grid_debug: Res<GridRenderDebug>,
    mut query_materials: Query<
        (&GridTransform, &mut Handle<ColorMaterial>, &mut Visible),
        With<GridRenderDebugNode>,
    >,
) {
    if grid_debug.visible {
        for (node, mut material, mut draw) in query_materials.iter_mut() {
            let status = grid.get_status(node.x as i32, node.y as i32);
            let target_material = match status {
                Some(GridStatus::Friend) => &grid_debug.friend_color,
                Some(GridStatus::Enemy) => &grid_debug.enemy_color,
                _ => &grid_debug.nothing_color,
            };
            if *material != *target_material {
                *material = target_material.clone();
            }
            draw.is_visible = true;
        }
    } else {
        for (_, _, mut draw) in query_materials.iter_mut() {
            draw.is_visible = false;
        }
    }
}

#[allow(dead_code)]
pub fn change_grid_randomly(mut grid: ResMut<Grid>) {
    use rand::random;

    coz::scope!("change_grid_randomly");
    let max_x = grid.x as u16;
    let max_y = grid.y as u16;

    let random_x = (random::<u16>() % max_x) as i32;
    let random_y = (random::<u16>() % max_y) as i32;

    let random_change = (random::<u16>() % 3) as i32 - 1;

    grid.change_by_count(random_x, random_y, random_change);
}

pub struct Grid {
    people_by_case: Vec<i32>,
    pub x: i32,
    pub y: i32,
}

#[derive(PartialEq)]
pub enum GridStatus {
    Friend,
    Neutral,
    Enemy,
}

impl Grid {
    pub fn new(x: i32, y: i32) -> Grid {
        Grid {
            people_by_case: vec![0; (x * y) as usize],
            x: x,
            y: y,
        }
    }

    fn to_pos(self: &Grid, x: i32, y: i32) -> Option<usize> {
        if 0 <= x && x < self.x && 0 <= y && y < self.y {
            Some((x * self.y + y) as usize)
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub fn add_friend(self: &mut Grid, x: i32, y: i32) -> bool {
        if let Some(pos) = self.to_pos(x, y) {
            if self.people_by_case[pos] >= 0 {
                self.people_by_case[pos] += 1;
                return true;
            }
        }
        return false;
    }

    #[allow(dead_code)]
    pub fn add_enemy(self: &mut Grid, x: i32, y: i32) -> bool {
        if let Some(pos) = self.to_pos(x, y) {
            if self.people_by_case[pos] <= 0 {
                self.people_by_case[pos] -= 1;
                return true;
            }
        }
        return false;
    }

    pub fn change_by_count(self: &mut Grid, x: i32, y: i32, change: i32) {
        if let Some(pos) = self.to_pos(x, y) {
            self.people_by_case[pos] += change;
        }
    }

    pub fn get_status(self: &Grid, x: i32, y: i32) -> Option<GridStatus> {
        coz::scope!("grid::get_status");
        self.get_count(x, y).map(|count| {
            if count == 0 {
                GridStatus::Neutral
            } else if count < 0 {
                GridStatus::Enemy
            } else {
                GridStatus::Friend
            }
        })
    }

    pub fn get_count(self: &Grid, x: i32, y: i32) -> Option<i32> {
        if let Some(pos) = self.to_pos(x, y) {
            return Some(self.people_by_case[pos]);
        }
        return None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::tests::*;
    use std::sync::Arc;
    use std::sync::Mutex;

    #[test]
    #[serial]
    fn change_color_on_g() {
        let x = 2;
        let y = 3;
        let count = Arc::new(Mutex::new(0));
        let clone = count.clone();

        App::build()
            .add_plugin(Test::Frames(10))
            .add_plugin(GridPlugin)
            .add_resource(Grid::new(x, y))
            .add_system_to_stage(stage::EVENT, send_g_key)
            .add_system(init_cameras_2d)
            .add_system(change_grid_randomly)
            .add_system_to_stage(stage::POST_UPDATE, move |v, d| {
                assert_node_are_visible((x * y) as usize, v, d)
            })
            .add_system_to_stage(stage::POST_UPDATE, move |q| {
                assert_node_are_changing(clone.clone(), q)
            })
            .run();

        assert!(*(count.lock().unwrap()) > 0);
    }

    fn send_g_key(mut done: Local<bool>, mut keys: ResMut<Input<KeyCode>>) {
        if !*done {
            keys.press(KeyCode::G);
            *done = true;
        }
    }

    fn assert_node_are_visible(
        count: usize,
        visible_entities: Query<&VisibleEntities>,
        debug_nodes: Query<&GridRenderDebugNode>,
    ) {
        let visible_count = visible_entities
            .iter()
            .next()
            .map(|vis| vis.value.len())
            .unwrap();
        let debug_count = debug_nodes.iter().count();

        assert_eq!(visible_count, debug_count);
        assert_eq!(visible_count, count);
    }

    fn assert_node_are_changing(
        changed_count: Arc<Mutex<i32>>,
        query: Query<&Handle<ColorMaterial>>,
    ) {
        if query.iter().len() > 0 {
            let mut unlock = changed_count.lock().unwrap();
            *unlock += 1;
        }
    }

    #[test]
    #[serial]
    fn grid_node_debug_get_updated() {
        fn move_camera(mut query: Query<&mut Transform, With<MainCamera>>) {
            for mut transform in query.iter_mut() {
                transform.translation += Vec3::splat(2.0);
            }
        }
        fn assert_debug_changed(
            query: Query<Entity, (With<GridRenderDebugNode>, Changed<Transform>)>,
        ) {
            assert_ne!(query.iter().count(), 0);
        }
        App::build()
            .add_plugin(Test::Frames(10))
            .add_plugin(GridPlugin)
            .add_resource(Grid::new(2, 3))
            .add_system_to_stage(stage::PRE_UPDATE, move_camera)
            .add_system_to_stage(stage::POST_UPDATE, assert_debug_changed)
            .run()
    }

    #[test]
    fn cannot_add_enemy_after_ally() {
        let x = 3;
        let y = 4;
        let mut grid = Grid::new(x, y);

        for i in 0..x {
            for j in 0..y {
                assert!(grid.add_friend(i, j), "Friend ({}, {})", i, j);
                assert!(!grid.add_enemy(i, j), "Enemy ({}, {})", i, j);
            }
        }
    }

    #[test]
    fn cannot_add_ally_after_enemy() {
        let x = 3;
        let y = 4;
        let mut grid = Grid::new(x, y);

        for i in 0..x {
            for j in 0..y {
                assert!(grid.add_enemy(i, j), "Enemy ({}, {})", i, j);
                assert!(!grid.add_friend(i, j), "Friend ({}, {})", i, j);
            }
        }
    }

    #[test]
    fn grid_get_count_return_get_by_count() {
        let mut grid = Grid::new(1, 1);
        let mut expected = 0;

        for i in 1..3 {
            grid.change_by_count(0, 0, i);
            expected += i;
            assert_eq!(grid.get_count(0, 0).unwrap(), expected);
            assert!(grid.get_status(0, 0).unwrap() == GridStatus::Friend);
        }

        for i in (-10..-7).rev() {
            grid.change_by_count(0, 0, i);
            expected += i;
            assert_eq!(grid.get_count(0, 0).unwrap(), expected);
            assert!(grid.get_status(0, 0).unwrap() == GridStatus::Enemy);
        }
    }

    #[test]
    fn grid_to_pos_is_none_on_empty_grid() {
        let grid = Grid::new(0, 0);
        for x in -2..2 {
            for y in -2..2 {
                assert!(grid.to_pos(x, y).is_none());
            }
        }
    }

    #[test]
    fn grid_to_pos_is_some_only_on_valid_indices() {
        let x = 5;
        let y = 8;
        let grid = Grid::new(x, y);
        for i in -2..(x + 3) {
            for j in -2..(y + 3) {
                let inside_grid = i >= 0 && i < x && j >= 0 && j < y;
                assert_eq!(
                    grid.to_pos(i, j).is_some(),
                    inside_grid,
                    "Error on position x:{} y:{}",
                    i,
                    j
                );
            }
        }
    }

}
