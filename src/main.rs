use bevy::prelude::*;

mod button;
mod fps;
mod game;
mod grid;
mod input;
mod unit;
mod utils;
mod camera;

fn main() {
    App::build().add_plugin(game::Game).run();
}
