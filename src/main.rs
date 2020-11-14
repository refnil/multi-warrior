use bevy::prelude::*;

mod game;
mod fps;
mod unit;
mod grid;

fn main() {
    App::build().add_plugin(game::Game).run();
}

