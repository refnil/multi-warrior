use bevy::prelude::*;

#[derive(Default)]
pub struct ButtonPlugin;

impl Plugin for ButtonPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<ButtonMaterials>()
            .add_startup_system(setup.system())
            .add_system(init_button_material_component.system())
            .add_system(change_material_for_state_system.system());
    }
}

#[derive(Clone)]
struct ButtonMaterials {
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
        (Added<ButtonMaterials>, With<Button>),
    >,
    mut query_default: Query<
        (&Interaction, &mut Handle<ColorMaterial>),
        (With<Button>, Without<ButtonMaterials>),
    >,
) {
    for (interaction, mut material, button_materials) in query.iter_mut() {
        *material = button_materials.choose(interaction);
    }
}

fn setup(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let font = &asset_server.load("fonts/FiraSans-Bold.ttf");
    let mat = ButtonMaterials::from_colors(&mut *materials, Color::RED, Color::GREEN, Color::BLUE);

    let mat2 =
        ButtonMaterials::from_colors(&mut *materials, Color::TEAL, Color::PURPLE, Color::GRAY);
    spawn_button(commands, "Coucou".to_string(), font, Some(mat.clone()));
    spawn_button(commands, "Coucou2".to_string(), font, Some(mat2.clone()));
    spawn_button(commands, "Coucou1".to_string(), font, Some(mat2.clone()));
    spawn_button(commands, "Coucou3".to_string(), font, Some(mat2.clone()));
    spawn_button(commands, "Coucou4".to_string(), font, Some(mat2.clone()));
}

fn spawn_button(
    commands: &mut Commands,
    text: String,
    font: &Handle<Font>,
    custom_material: Option<ButtonMaterials>,
) {
    commands.spawn(ButtonBundle {
        style: Style {
            size: Size::new(Val::Px(150.0), Val::Px(65.0)),
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
    commands.with_children(|parent| {
        parent.spawn(TextBundle {
            text: Text {
                value: text,
                font: font.clone(),
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
