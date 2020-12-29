use bevy::ecs::*;

pub use strum::IntoEnumIterator;
use strum_macros::EnumIter;

// Add std to the doc generated by cargo doc
#[doc(inline)]
pub use std;

#[allow(dead_code)]
pub fn count_query_filter<Q: WorldQuery, F: QueryFilter>(mut query: Query<Q, F>) {
    let name = std::any::type_name::<Q>();
    println!("{}: {}", name, query.iter_mut().count());
}

#[allow(dead_code)]
pub fn count_query<Q: WorldQuery>(query: Query<Q>) {
    count_query_filter::<Q, ()>(query);
}


#[derive(Debug, Copy, Clone, EnumIter)]
pub enum Direction {
    Up,
    Left,
    Right,
    Down,
}

impl Direction {
    pub fn next(&self) -> Self {
        match self {
            Direction::Up => Direction::Left,
            Direction::Left => Direction::Down,
            Direction::Down => Direction::Right,
            Direction::Right => Direction::Up,
        }
    }

    pub fn x(&self) -> i32 {
        match self {
            Direction::Up => 0,
            Direction::Left => -1,
            Direction::Down => 0,
            Direction::Right => 1,
        }
    }

    pub fn y(&self) -> i32 {
        match self {
            Direction::Up => 1,
            Direction::Left => 0,
            Direction::Down => -1,
            Direction::Right => 0,
        }
    }

    pub fn from_points(x1: i32, y1: i32, x2: i32, y2: i32) -> Self {
        let x_diff = (x1 - x2).abs();
        let x_dir = if x1 < x2 { Self::Right } else { Self::Left };

        let y_diff = (y1 - y2).abs();
        let y_dir = if y1 < y2 { Self::Up } else { Self::Down };

        if x_diff < y_diff {
            x_dir
        } else {
            y_dir
        }
    }
}

impl Default for Direction {
    fn default() -> Self {
        Self::Down
    }
}

#[cfg(test)]
pub mod tests {
    use bevy::app::*;
    use bevy::prelude::*;
    use bevy::winit::*;
    pub use serial_test::serial;
    use std::ops::{Deref, DerefMut};
    use std::thread;

    #[test]
    #[serial]
    fn empty_test_app_with_frames() {
        App::build().add_plugin(Test::Frames(5)).run();
    }

    #[test]
    #[serial]
    fn empty_test_app_with_times() {
        App::build().add_plugin(Test::Time(0.5)).run();
    }

    #[derive(Clone)]
    pub enum Test {
        Frames(i32),
        Time(f32),
        NoStop,
    }

    impl Test {
        #[allow(dead_code)]
        pub fn debug(self) -> Self { Self::NoStop }

        fn system(&self) -> Option<impl System<In = (), Out = ()>> {
            match self.clone() {
                Self::Frames(count) => Some(IntoSystem::system(Box::new(move |c: Local<i32>, e: ResMut<Events<AppExit>>| Self::frames(count, c, e)))),
                Self::Time(time) => Some(Box::new(move |c: Local<f32>, t: Res<Time>, e: ResMut<Events<AppExit>>| Self::times(time, c, t, e)).system()),
                Self::NoStop => None,
            }
        }

        fn frames(max: i32, mut current: Local<i32>, mut exit: ResMut<Events<AppExit>>) {
            *current += 1;
            if max <= *current {
                exit.send(AppExit);
            }
        }

        fn times(
            end_time: f32,
            mut current_time: Local<f32>,
            time: Res<Time>,
            mut exit: ResMut<Events<AppExit>>,
        ) {
            *current_time += time.delta_seconds();
            if end_time <= *current_time {
                exit.send(AppExit);
            }
        }
    }

    impl Plugin for Test {
        fn build(&self, app: &mut AppBuilder) {
            app.add_plugins(NoLogPlugins);
            app.set_runner(winit_runner_any_thread);
            app.add_resource(WinitConfig {
                return_from_run: true,
            });
            if let Some(system) = self.system() {
                app.add_system_to_stage(stage::POST_UPDATE, system);
            }
        }
    }

    // Test system
    // Test event
    pub struct TestCheck<T> {
        val: T,
        test: Vec<Box<dyn FnOnce(&T) -> bool + Send + 'static + Sync>>,
    }

    impl<T> TestCheck<T> {
        pub fn new(val: T) -> Self {
            TestCheck {
                val: val,
                test: Vec::new(),
            }
        }

        pub fn test<Func>(mut self, f: Func) -> Self
        where Func: FnOnce(&T) -> bool + Send + 'static + Sync
        {
            self.test.push(Box::new(f));
            self
        }
    }

    impl TestCheck<bool> {
        fn is_true_inner(val: &bool) -> bool {
            val.clone()
        }

        pub fn is_true(self) -> Self {
            self.test(Self::is_true_inner)
        }
    }

    impl<T> Deref for TestCheck<T> {
        type Target = T;
        fn deref(&self) -> &T {
            &self.val
        }
    }

    impl<T> DerefMut for TestCheck<T> {
        fn deref_mut(&mut self) -> &mut T {
            &mut self.val
        }
    }

    impl<T> Drop for TestCheck<T> {
        fn drop(&mut self) {
            if !thread::panicking() {
                let tests = std::mem::replace(&mut self.test, Vec::new());
                for f in tests.into_iter() {
                    assert!(f(&self.val));
                }
            }
        }
    }

    struct NoLogPlugins;
    impl PluginGroup for NoLogPlugins {
        fn build(&mut self, group: &mut PluginGroupBuilder) {
            group.add(bevy::reflect::ReflectPlugin::default());
            group.add(bevy::core::CorePlugin::default());
            group.add(bevy::transform::TransformPlugin::default());
            group.add(bevy::diagnostic::DiagnosticsPlugin::default());
            group.add(bevy::input::InputPlugin::default());
            group.add(bevy::window::WindowPlugin::default());
            group.add(bevy::asset::AssetPlugin::default());
            group.add(bevy::scene::ScenePlugin::default());
            group.add(bevy::render::RenderPlugin::default());
            group.add(bevy::sprite::SpritePlugin::default());
            group.add(bevy::pbr::PbrPlugin::default());
            group.add(bevy::ui::UiPlugin::default());
            group.add(bevy::text::TextPlugin::default());
            group.add(bevy::audio::AudioPlugin::default());
            group.add(bevy::gltf::GltfPlugin::default());
            group.add(bevy::winit::WinitPlugin::default());
            group.add(bevy::wgpu::WgpuPlugin::default());
        }
    }
}
