use bevy::prelude::*;

use crate::input::*;

#[derive(Default)]
pub struct ButtonPlugin;

impl Plugin for ButtonPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<ButtonMaterials>()
            .init_resource::<ButtonSpawner>()
            .add_system_to_stage(stage::PRE_UPDATE, press_button_from_input.system())
            .add_system(init_button_material_component.system())
            .add_system(action_button_on_press.system())
            .add_system(change_material_for_state_system.system())
            .add_system(change_text_for_button.system());
    }
}

#[derive(Clone)]
pub struct ButtonMaterials {
    normal: Handle<ColorMaterial>,
    hovered: Handle<ColorMaterial>,
    pressed: Handle<ColorMaterial>,
}

impl ButtonMaterials {
    fn from_colors(
        assets: &mut Assets<ColorMaterial>,
        normal: Color,
        hovered: Color,
        pressed: Color,
    ) -> Self {
        ButtonMaterials {
            normal: assets.add(normal.into()),
            hovered: assets.add(hovered.into()),
            pressed: assets.add(pressed.into()),
        }
    }

    fn choose(&self, interaction: &Interaction) -> Handle<ColorMaterial> {
        match interaction {
            Interaction::Clicked => self.pressed.clone(),
            Interaction::Hovered => self.hovered.clone(),
            Interaction::None => self.normal.clone(),
        }
    }
}

impl FromResources for ButtonMaterials {
    fn from_resources(resources: &Resources) -> Self {
        let mut materials = resources.get_mut::<Assets<ColorMaterial>>().unwrap();
        ButtonMaterials::from_colors(
            &mut *materials,
            Color::rgb(0.15, 0.15, 0.15),
            Color::rgb(0.25, 0.25, 0.25),
            Color::rgb(0.35, 0.75, 0.35),
        )
    }
}

pub struct ButtonSpawner {
    font: Handle<Font>,
}

impl FromResources for ButtonSpawner {
    fn from_resources(resources: &Resources) -> Self {
        let asset_server = resources.get::<AssetServer>().unwrap();
        let font = asset_server.load("fonts/FiraSans-Bold.ttf");
        ButtonSpawner { font: font }
    }
}

fn change_material_for_state_system(
    button_materials: Res<ButtonMaterials>,
    mut interaction_query: Query<
        (&Interaction, &mut Handle<ColorMaterial>),
        (Mutated<Interaction>, With<Button>, Without<ButtonMaterials>),
    >,
    mut interaction_query_materials: Query<
        (&Interaction, &mut Handle<ColorMaterial>, &ButtonMaterials),
        (Mutated<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut material) in interaction_query.iter_mut() {
        *material = button_materials.choose(interaction);
    }

    for (interaction, mut material, custom_materials) in interaction_query_materials.iter_mut() {
        *material = custom_materials.choose(interaction);
    }
}

fn init_button_material_component(
    button_materials: Res<ButtonMaterials>,
    mut query: Query<
        (&Interaction, &mut Handle<ColorMaterial>, &ButtonMaterials),
        (
            Added<Handle<ColorMaterial>>,
            Added<ButtonMaterials>,
            With<Button>,
        ),
    >,
    mut query_default: Query<
        (&Interaction, &mut Handle<ColorMaterial>),
        (
            Added<Handle<ColorMaterial>>,
            With<Button>,
            Without<ButtonMaterials>,
        ),
    >,
) {
    for (interaction, mut material) in query_default.iter_mut() {
        *material = button_materials.choose(interaction);
    }

    for (interaction, mut material, custom_materials) in query.iter_mut() {
        *material = custom_materials.choose(interaction);
    }
}

fn change_text_for_button(
    query: Query<(&CombinationInput, &Children), (Changed<CombinationInput>, With<Button>)>,
    mut query_text: Query<&mut Text>,
) {
    for (comb, children) in query.iter() {
        for child in children.iter() {
            if let Ok(mut text) = query_text.get_mut(*child) {
                let pos = text
                    .value
                    .char_indices()
                    .find(|(_i, c)| c == &':')
                    .map(|(i, _c)| i);
                if let Some(pos) = pos {
                    text.value.truncate(pos);
                }
                let new_string = match comb.want_combination {
                    true => comb.to_string(),
                    false => "N/A".to_string(),
                };
                text.value.push_str(": ");
                text.value.push_str(&new_string);
            }
        }
    }
}

fn press_button_from_input(
    input: Res<Input<KeyCode>>,
    mut query: Query<(&CombinationInput, &mut Interaction)>,
) {
    for (comb, mut interaction) in query.iter_mut() {
        if input.pressed_t(&*comb) {
            *interaction = Interaction::Clicked;
        } else if input.just_released_t(&*comb) {
            *interaction = Interaction::None;
        }
    }
}

fn action_button_on_press(
    mut query: Query<(&mut CombinationInput, &Interaction), Changed<Interaction>>,
) {
    for (mut comb, interaction) in query.iter_mut() {
        if interaction == &Interaction::Clicked {
            comb.want_combination = !comb.want_combination;
        }
    }
}

impl ButtonSpawner {
    pub fn spawn_button(
        &self,
        commands: &mut Commands,
        text: String,
        custom_material: Option<ButtonMaterials>,
    ) {
        commands.spawn(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(250.0), Val::Px(65.0)),
                // center button
                margin: Rect {
                    left: Val::Px(10.0),
                    right: Val::Px(10.0),
                    ..Default::default()
                },
                // horizontally center child text
                //justify_content: JustifyContent::Center,
                // vertically center child text
                //align_items: AlignItems::Center,
                //align_self: AlignSelf::FlexEnd,
                flex_wrap: FlexWrap::Wrap,
                ..Default::default()
            },
            ..Default::default()
        });
        if let Some(mat) = custom_material {
            commands.with(mat);
        }
        commands.with(CombinationInput::new(true));
        commands.with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text {
                    value: text,
                    font: self.font.clone(),
                    style: TextStyle {
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                        ..Default::default()
                    },
                },
                ..Default::default()
            });
        });
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::camera::*;
    use crate::utils::tests::*;

    pub fn create_n_buttons(
        button_count: i32,
        commands: &mut Commands,
        spawner: Res<ButtonSpawner>,
        mut materials: ResMut<Assets<ColorMaterial>>,
    ) {
        let mut mat =
            ButtonMaterials::from_colors(&mut *materials, Color::RED, Color::GREEN, Color::BLUE);
        let mut mat2 =
            ButtonMaterials::from_colors(&mut *materials, Color::TEAL, Color::PURPLE, Color::GRAY);

        for i in 0..button_count {
            spawner.spawn_button(commands, format!("Coucou {}", i), Some(mat.clone()));
            std::mem::swap(&mut mat,&mut mat2);
        }
    }

    #[test]
    #[serial]
    fn spawn_some_button(){
        let button_number = 4;
        let assert_6_buttons = move |query: Query<&Button>|{
            let button_found = query.iter().len();
            assert_eq!(button_found, button_number, "{} button found, {} expected", button_found, button_number);
        };
        App::build()
            .add_plugin(Test::Frames(2))
            .add_plugin(ButtonPlugin)
            .add_startup_system(init_cameras_ui)
            .add_startup_system((move |c, s, m| { create_n_buttons(button_number as i32, c, s, m) }).system())
            .add_system(assert_6_buttons)
            .run()
    }
}
