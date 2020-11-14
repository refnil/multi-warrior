use bevy::prelude::*;

mod game;
mod fps;
mod unit;
mod grid;
mod utils;

fn main() {
    App::build().add_plugin(game::Game).run();
}

