use bevy::prelude::*;

mod game;
mod fps;
mod unit;

fn main() {
    App::build().add_plugin(game::Game).run();
}

