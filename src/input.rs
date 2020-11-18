use bevy::prelude::*;

use bevy::input::{keyboard::*, *};

use crate::utils::*;

#[derive(Default)]
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<KeyboardCombinationInput>()
            .add_system(combination_input_update.system())
            .add_startup_system(test::add_some_input.system())
            .add_system(count_query::<&mut CombinationInput>.system())
            ;
    }
}

//type ciu_query<'a> = (Mut<'a, CombinationInput>, Changed<'a, CombinationInput>);

fn combination_input_update(
    mut reserver: ResMut<KeyboardCombinationInput>,
    mut query: Query<(&mut CombinationInput,), Changed<CombinationInput>>,
) {
    for (mut comb,) in query.iter_mut() {
        if comb.want_combination && comb.combination.is_none() {
            comb.combination = reserver.reserve()
        } else if !comb.want_combination {
            comb.swap_combination(None).map(|x| reserver.liberate(x));
        }
    }
}

#[derive(Default)]
pub struct CombinationInput {
    pub want_combination: bool,
    combination: Option<KeyboardCombination>,
}

impl CombinationInput {
    fn swap_combination(
        &mut self,
        new: Option<KeyboardCombination>,
    ) -> Option<KeyboardCombination> {
        let old = self.combination.take();
        self.combination = new;
        return old;
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
            available: vec![KeyCode::A, KeyCode::B, KeyCode::C],
            given: Vec::new(),
        }
    }
}

pub struct KeyboardCombination {
    keycode: KeyCode,
}

trait InputTrait<T> {
    fn pressed(&self, elem: &T) -> bool;
    fn just_pressed(&self, elem: &T) -> bool;
    fn just_released(&self, elem: &T) -> bool;
}

impl InputTrait<KeyboardCombination> for Input<KeyCode> {
    fn pressed(&self, comb: &KeyboardCombination) -> bool {
        self.pressed(comb.keycode)
    }
    fn just_pressed(&self, comb: &KeyboardCombination) -> bool {
        self.just_pressed(comb.keycode)
    }
    fn just_released(&self, comb: &KeyboardCombination) -> bool {
        self.just_released(comb.keycode)
    }
}

mod test {
    use bevy::prelude::*;

    pub fn add_some_input(commands: &mut Commands) {
        commands.spawn((super::CombinationInput::default(),));
        commands.spawn((super::CombinationInput::default(),));
    }
}
