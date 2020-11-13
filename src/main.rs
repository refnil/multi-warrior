use bevy::prelude::*;

mod game;
mod fps;

fn main() {
    let mut builder = &mut App::build();

    builder.add_plugin(game::Game);

    builder.run();

    /*app.startup_schedule.initialize(&mut app.resources);
    app.startup_executor.run(
        &mut app.startup_schedule,
        &mut app.world,
        &mut app.resources,
    );
    */
    //app.world.
}

