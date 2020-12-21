use bevy::prelude::*;

use crate::grid::*;
use crate::unit::*;

pub struct SpawnPlugin;

impl Plugin for SpawnPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(spawn_info_system);
    }
}

pub struct SpawnInfo {
    pub target_unit_count: Option<u32>,
    pub spawn_delay: Option<f32>,
    pub last_spawn: f32,
    pub ally: bool,
    pub x: i32,
    pub y: i32,
}

impl SpawnInfo {
    pub fn want_spawn(&self, grid: &Grid) -> bool {
        false
    }
    pub fn spawn(&self, sur: &mut SpawnUnitRes) {
        sur.spawn_unit(self.x, self.y, self.ally);
        //spawn_unit(command, assert_server, grid, texture_atlas, self.x, self.y, self.ally);
    }
}

fn spawn_info_system(mut sur: SpawnUnitRes, time: Res<Time>, mut query: Query<&mut SpawnInfo>) {
    for mut si in query.iter_mut() {
        if si.want_spawn(&sur.grid) {
            si.spawn(&mut sur);
            si.last_spawn = time.seconds_since_startup() as f32;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::tests::*;

    #[test]
    #[serial]
    fn spawn_something() {
        App::build()
            .add_plugin(Test::Frames(3))
            .add_plugin(GridPlugin)
            .add_plugin(SpawnPlugin)
            .add_resource(Grid::new(3, 3))
            .run();
    }
}
