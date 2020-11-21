use bevy::prelude::*;

use bevy::input::{keyboard::*, *};

use crate::utils::*;

#[derive(Default)]
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<KeyboardCombinationInput>()
            .add_startup_system(test::add_some_input.system())
            //.add_system(test::change_input_at_random.system())
            .add_system_to_stage(stage::PRE_UPDATE, combination_input_update.system())
            .add_system_to_stage(stage::POST_UPDATE, combination_reset_end_frame.system());
    }
}

fn combination_input_update(
    mut reserver: ResMut<KeyboardCombinationInput>,
    mut query: Query<&mut CombinationInput>,
) {
    for mut comb in query.iter_mut() {
        let have_combination = comb.combination.is_some();
        if comb.want_combination && !have_combination {
            let reservation = reserver.reserve();
            if reservation.is_some() {
                comb.combination = reservation;
                comb.change_in_frame = true;
            }
        } else if !comb.want_combination && have_combination {
            comb.swap_combination(None).map(|x| reserver.liberate(x));
            comb.change_in_frame = true;
        }
    }
}

fn combination_reset_end_frame(mut query: Query<&mut CombinationInput, Changed<CombinationInput>>) {
    for mut comb in query.iter_mut() {
        comb.change_in_frame = false;
    }
}

#[derive(Default, Debug)]
pub struct CombinationInput {
    pub want_combination: bool,
    combination: Option<KeyboardCombination>,
    change_in_frame: bool,
}

impl CombinationInput {
    pub fn new(want_combination: bool) -> Self {
        CombinationInput {
            want_combination: want_combination,
            combination: None,
            change_in_frame: false,
        }
    }

    fn swap_combination(
        &mut self,
        new: Option<KeyboardCombination>,
    ) -> Option<KeyboardCombination> {
        let old = self.combination.take();
        self.combination = new;
        return old;
    }
}

impl ToString for CombinationInput {
    fn to_string(&self) -> String {
        self.combination
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or("".to_string())
    }
}

pub struct KeyboardCombinationInput {
    available: Vec<KeyCode>,
    given: Vec<KeyCode>,
}

impl KeyboardCombinationInput {
    pub fn reserve(&mut self) -> Option<KeyboardCombination> {
        let available_code = self
            .available
            .pop()
            .map(|x| KeyboardCombination { keycode: x });
        if let Some(comb) = available_code {
            self.given.push(comb.keycode);
            return Some(comb);
        } else {
            return None;
        }
    }

    pub fn liberate(&mut self, comb: KeyboardCombination) {
        let keycode = comb.keycode;
        self.available.push(keycode);
        let index_given = self
            .given
            .iter()
            .enumerate()
            .find(|(_i, x)| x == &&keycode)
            .unwrap()
            .0;
        self.given.swap_remove(index_given);
    }
}

impl Default for KeyboardCombinationInput {
    fn default() -> Self {
        KeyboardCombinationInput {
            available: vec![KeyCode::A, KeyCode::B, KeyCode::C, KeyCode::D, KeyCode::E],
            given: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct KeyboardCombination {
    keycode: KeyCode,
}

impl ToString for KeyboardCombination {
    fn to_string(&self) -> String {
        format!("{:?}", self.keycode)
    }
}

pub trait InputTrait<T> {
    fn pressed_t(&self, elem: &T) -> bool;
    fn just_pressed_t(&self, elem: &T) -> bool;
    fn just_released_t(&self, elem: &T) -> bool;
}

impl InputTrait<KeyboardCombination> for Input<KeyCode> {
    fn pressed_t(&self, comb: &KeyboardCombination) -> bool {
        self.pressed(comb.keycode)
    }
    fn just_pressed_t(&self, comb: &KeyboardCombination) -> bool {
        self.just_pressed(comb.keycode)
    }
    fn just_released_t(&self, comb: &KeyboardCombination) -> bool {
        self.just_released(comb.keycode)
    }
}

impl InputTrait<CombinationInput> for Input<KeyCode> {
    fn pressed_t(&self, comb: &CombinationInput) -> bool {
        comb.combination
            .as_ref()
            .map(|x| InputTrait::pressed_t(self, x))
            .unwrap_or(false)
    }
    fn just_pressed_t(&self, comb: &CombinationInput) -> bool {
        comb.combination
            .as_ref()
            .map(|x| InputTrait::just_pressed_t(self, x))
            .unwrap_or(false)
    }
    fn just_released_t(&self, comb: &CombinationInput) -> bool {
        comb.combination
            .as_ref()
            .map(|x| InputTrait::just_released_t(self, x))
            .unwrap_or(comb.change_in_frame)
    }
}

mod test {
    use bevy::prelude::*;
    use rand::random;

    pub fn add_some_input(commands: &mut Commands) {
        commands.spawn((super::CombinationInput::default(),));
        commands.spawn((super::CombinationInput::default(),));
    }

    pub fn change_input_at_random(mut query: Query<&mut super::CombinationInput>) {
        for mut combination in query.iter_mut() {
            let target = random::<bool>();
            if combination.want_combination != target {
                combination.want_combination = target;
            }
        }
    }
}
