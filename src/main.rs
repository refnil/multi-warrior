use bevy::prelude::*;

mod game;
mod fps;

fn main() {
    App::build().add_plugin(game::Game).run();
}

