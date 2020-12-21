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
    pub fn want_spawn(&self, grid: &Grid, time: &Time) -> bool {
        let status = grid
            .get_status(self.x, self.y)
            .map(|gs| gs == GridStatus::Neutral)
            .unwrap_or(false);
        let count = grid.get_count(self.x, self.y).unwrap_or(0).abs()
            < self.target_unit_count.unwrap_or(u32::MAX) as i32;
        let time = self.last_spawn + self.spawn_delay.unwrap_or(0.0) < time.seconds_since_startup() as f32;
        status && count && time
    }
    pub fn spawn(&self, sur: &mut SpawnUnitRes) {
        sur.spawn_unit(self.x, self.y, self.ally);
        sur.commands.with(MoveOnForceAI::default());
    }
}

fn spawn_info_system(mut sur: SpawnUnitRes, time: Res<Time>, mut query: Query<&mut SpawnInfo>) {
    for mut si in query.iter_mut() {
        if si.want_spawn(&sur.grid, &time) {
            si.spawn(&mut sur);
            si.last_spawn = time.seconds_since_startup() as f32;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anim::*;
    use crate::camera::*;
    use crate::utils::tests::*;

    #[test]
    #[serial]
    fn spawn_something() {
        fn setup_scene(commands: &mut Commands) {
            commands.spawn((SpawnInfo {
                ally: true,
                last_spawn: 0.0,
                spawn_delay: None,
                target_unit_count: Some(3),
                x: 1,
                y: 1,
            },));
        }

        fn check_for_spawn(mut count: ResMut<TestCheck<usize>>, query: Query<&UnitStats>) {
            **count = query.iter().len();
        }

        App::build()
            .add_plugin(Test::Time(1.5))
            .add_plugin(GridPlugin)
            .add_plugin(UnitPlugin)
            .add_plugin(AnimPlugin)
            .add_plugin(SpawnPlugin)
            .add_resource(Grid::new(3, 3))
            .add_startup_system(init_cameras_2d)
            .add_startup_system(setup_scene)
            .add_resource(TestCheck::new(0 as usize).test(|i| i > &1))
            .add_system(check_for_spawn)
            .run();
    }
}
