use bevy::prelude::*;

#[derive(Component)]
pub struct MainCamera;
#[derive(Component)]
pub struct UICamera;

pub fn init_cameras(mut commands: Commands) {
    init_cameras_2d(commands);
}

pub fn init_cameras_2d(mut commands: Commands) {
    // 2d camera
    commands.spawn_bundle(Camera2dBundle::default()).insert(MainCamera).insert(UICamera);
}
