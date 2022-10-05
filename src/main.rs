//! # Multi warrior documentation
//! You will find more documentation in the [multi_warrior_lib]

use bevy::prelude::*;
use multi_warrior_lib::Game;

fn main() {
    App::new().add_plugin(Game).run();
}
