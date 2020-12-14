use bevy::prelude::*;

pub struct MainCamera;
pub struct UICamera;

pub fn init_cameras(commands: &mut Commands) {
    init_cameras_2d(commands);
    init_cameras_ui(commands);
}

pub fn init_cameras_2d(commands: &mut Commands) {
    // 2d camera
    commands.spawn(Camera2dBundle::default());
    commands.with(MainCamera);
}

pub fn init_cameras_ui(commands: &mut Commands) {
    // UI camera
    commands.spawn(CameraUiBundle::default());
    commands.with(UICamera);
}
