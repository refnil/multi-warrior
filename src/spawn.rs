use bevy::ecs::*;
use bevy::prelude::*;

use crate::grid::*;
use crate::unit::*;

pub struct SpawnPlugin;

impl Plugin for SpawnPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(spawn_info_system.system());
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
    pub fn want_spawn(&self, grid: &Grid, time: &Time, count_of_force: u32) -> bool {
        let status = grid
            .get_status(self.x, self.y)
            .map(|gs| gs == GridStatus::Neutral)
            .unwrap_or(false);
        let count = count_of_force < self.target_unit_count.unwrap_or(u32::MAX);
        let time =
            self.last_spawn + self.spawn_delay.unwrap_or(0.0) < time.seconds_since_startup() as f32;
        status && count && time
    }
    pub fn spawn(&self, sur: &mut SpawnUnitRes) {
        sur.spawn_unit(self.x, self.y, self.ally);
    }
}

fn spawn_info_system(
    mut sur: SpawnUnitRes,
    time: Res<Time>,
    mut query: Query<(Entity, &mut SpawnInfo)>,
    query_of_ai: Query<(
        Entity,
        Option<&TurningAI>,
        Option<&MoveOnForceAI>,
        Option<&AttackingAI>,
    )>,
    count_force: Query<&UnitForce, With<UnitTime>>,
) {
    let mut good = 0;
    let mut bad = 0;

    for force in count_force.iter() {
        if force.ally {
            good += 1;
        } else {
            bad += 1;
        }
    }

    for (entity, mut si) in query.iter_mut() {
        let count = if si.ally { good } else { bad };
        if si.want_spawn(&sur.grid, &time, count) {
            si.spawn(&mut sur);
            si.last_spawn = time.seconds_since_startup() as f32;

            if query_of_ai.get_component::<TurningAI>(entity).is_ok() {
                sur.commands.with(TurningAI);
            } else if query_of_ai.get_component::<MoveOnForceAI>(entity).is_ok() {
                sur.commands.with(MoveOnForceAI::default());
            } else if query_of_ai.get_component::<AttackingAI>(entity).is_ok() {
                sur.commands.with(AttackingAI);
                sur.commands.with(AttackingAIState::MoveToNearestEnemy);
            } else {
                warn!("No ai found while spawning a new unit");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anim::*;
    use crate::camera::*;
    use crate::utils::tests::*;
    use crate::fx::*;

    mod spawn_something {
        use super::*;
        fn setup_scene(commands: &mut Commands) {
            commands.spawn((
                SpawnInfo {
                    ally: true,
                    last_spawn: 0.0,
                    spawn_delay: None,
                    target_unit_count: Some(3),
                    x: 1,
                    y: 1,
                },
                MoveOnForceAI::default(),
            ));
        }

        fn check_for_spawn(mut count: ResMut<TestCheck<usize>>, query: Query<&UnitStats>) {
            **count = query.iter().len();
        }
        #[test]
        #[serial]
        fn spawn_something() {
            App::build()
                .add_plugin(Test::Time(1.5))
                .add_plugin(GridPlugin)
                .add_plugin(UnitPlugin)
                .add_plugin(AnimPlugin)
            .add_plugin(FxPlugin)
                .add_plugin(SpawnPlugin)
                .add_resource(Grid::new(3, 3))
                .add_startup_system(init_cameras_2d.system())
                .add_startup_system(setup_scene.system())
                .add_resource(TestCheck::new(0 as usize).test(|i| i > &1))
                .add_system(check_for_spawn.system())
                .run();
        }
    }

    #[test]
    #[serial]
    fn small_battle() {
        battle_of_two_spawners(Test::Time(2.0), 3, 1.2, 2);
    }

    #[test]
    #[serial]
    #[ignore]
    fn big_battle() {
        battle_of_two_spawners(Test::NoStop, 10, 3.1416, 10);
    }

    fn battle_of_two_spawners(test: Test, size: i32, delay: f32, units: u32) {
        let setup_scene = move |commands: &mut Commands| {
            commands
                .spawn((
                    SpawnInfo {
                        ally: true,
                        last_spawn: f32::MIN,
                        spawn_delay: Some(delay),
                        target_unit_count: Some(units),
                        x: 0,
                        y: 0,
                    },
                    AttackingAI,
                ))
                .with(AttackingAIState::MoveToNearestEnemy);
            commands
                .spawn((
                    SpawnInfo {
                        ally: false,
                        last_spawn: f32::MIN,
                        spawn_delay: Some(delay),
                        target_unit_count: Some(units),
                        x: size - 1,
                        y: size - 1,
                    },
                    AttackingAI,
                ))
                .with(AttackingAIState::MoveToNearestEnemy);
        };

        App::build()
            .add_plugin(test)
            .add_plugin(GridPlugin)
            .add_plugin(UnitPlugin)
            .add_plugin(AnimPlugin)
            .add_plugin(SpawnPlugin)
            .add_plugin(FxPlugin)
            .add_resource(Grid::new(size, size))
            .add_resource(TestCheck::<usize>::new(0).test(move |v| *v >= units as usize * 2))
            .add_startup_system(init_cameras_2d.system())
            .add_startup_system(Box::new(setup_scene).system())
            .add_system(total_unit.system())
            .run();
    }

    fn total_unit(mut val: ResMut<TestCheck<usize>>, query: Query<&UnitInfo>) {
        **val = query.iter().len().max(**val);
    }
}
