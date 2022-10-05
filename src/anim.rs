use bevy::prelude::*;
 use bevy::utils::Duration;

use crate::utils::Direction;

#[derive(Default)]
pub struct AnimPlugin;

impl Plugin for AnimPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AnimTimer::new(0.1))
            .add_system_to_stage(CoreStage::PostUpdate, update_animation_from_state)
            .add_system_to_stage(CoreStage::PostUpdate, animate_sprite_system);
    }
}
fn animate_sprite_system(
    time: Res<Time>,
    mut timer: ResMut<AnimTimer>,
    mut query_sync: Query<(&mut TextureAtlasSprite, &mut Animation), Without<AnimTimer>>,
    mut query_with_timer: Query<(&mut TextureAtlasSprite, &mut Animation, &mut AnimTimer)>,
) {
    let delta = Duration::from_secs_f32(time.delta_seconds());
    timer.timer.tick(delta);
    if timer.timer.just_finished() {
        for (mut sprite, mut animation) in query_sync.iter_mut() {
            animation.change_frame();
            sprite.index = animation.current_frame();
        }
    }

    for (mut sprite, mut anim, mut timer) in query_with_timer.iter_mut() {
        timer.timer.tick(delta);
        if timer.timer.just_finished() {
            anim.change_frame();
            sprite.index = anim.current_frame();
        }
    }
}

#[derive(Component)]
pub struct AnimTimer {
    timer: Timer,
}

impl AnimTimer {
    pub fn new(duration: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration, true),
        }
    }
}

#[derive(Eq, PartialEq)]
pub enum AnimationMode {
    Loop,
    #[allow(dead_code)]
    Zip(bool),
    Stop,
}

#[derive(Component)]
pub struct Animation {
    current_frame: usize,
    mode: AnimationMode,
    frames: Vec<usize>,
}


impl Animation {
    pub fn new(mode: AnimationMode, frames: Vec<usize>) -> Self {
        assert!(frames.len() > 0);
        Self {
            current_frame: 0,
            mode,
            frames,
        }
    }
}

impl Animation {
    fn change_frame(&mut self) {
        self.current_frame = match self.mode {
            AnimationMode::Loop => (self.current_frame + 1) % self.frame_count(),
            AnimationMode::Stop => (self.current_frame + 1).min(self.frame_count() - 1),
            AnimationMode::Zip(forward) => {
                let len = self.frame_count();
                let (res, change_value) = if forward {
                    ((self.current_frame + 1) % len, len - 1)
                } else {
                    ((self.current_frame - 1) % len, 0)
                };
                if res == change_value {
                    self.mode = AnimationMode::Zip(!forward);
                }
                res
            }
        };
    }

    pub fn current_frame(&self) -> usize {
        self.frames[self.current_frame]
    }

    pub fn is_stopped(&self) -> bool {
        self.mode == AnimationMode::Stop && self.current_frame == self.frame_count() - 1
    }

    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }
}

impl UnitState {
    pub fn is_still(&self) -> bool {
        match self {
            Self::Still(_) => true,
            _ => false,
        }
    }

    pub fn get_animation(&self) -> Animation {
        let frames = match self {
            Self::Still(Direction::Down) => vec![1],
            Self::Still(Direction::Right) => vec![7],
            Self::Still(Direction::Up) => vec![10],
            Self::Still(Direction::Left) => vec![4],

            Self::Moving(Direction::Down) => vec![1, 2, 1, 0],
            Self::Moving(Direction::Right) => vec![7, 8, 7, 6],
            Self::Moving(Direction::Up) => vec![10, 11, 10, 9],
            Self::Moving(Direction::Left) => vec![4, 5, 4, 3],
        };
        Animation {
            current_frame: 0,
            mode: AnimationMode::Loop,
            frames: frames,
        }
    }
}

fn update_animation_from_state(mut query: Query<(&UnitState, &mut Animation), Changed<UnitState>>) {
    for (state, mut animation) in query.iter_mut() {
        *animation = state.get_animation();
    }
}

#[derive(Debug, Clone, Component)]
pub enum UnitState {
    Still(Direction),
    Moving(Direction),
}

impl Default for UnitState {
    fn default() -> Self {
        Self::Still(Direction::default())
    }
}
