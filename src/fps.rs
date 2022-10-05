use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

#[derive(Default, Clone)]
pub struct FPSPlugin {
    pub color: Color,
}

impl Plugin for FPSPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(FrameTimeDiagnosticsPlugin::default())
            .insert_resource(self.clone())
            .add_startup_system(Self::setup)
            .add_system(Self::text_update_system);
    }
}

// A unit struct to help identify the FPS UI component, since there may be many Text components
#[derive(Component)]
struct FpsText;

impl FPSPlugin {
    fn text_update_system(diagnostics: Res<Diagnostics>, mut query: Query<(&mut Text, &FpsText)>) {
        for (mut text, _tag) in query.iter_mut() {
            if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
                if let Some(average) = fps.average() {
                    text.sections[0].value = format!("FPS: {:.2}", average);
                }
            }
        }
    }

    fn setup(mut commands: Commands, fps: Res<FPSPlugin>, asset_server: Res<AssetServer>) {
        commands
            // texture
            .spawn_bundle(TextBundle {
                style: Style {
                    //max_size: Size::new(Val::Percent(25.0), Val::Undefined),
                    //align_self: AlignSelf::FlexEnd,
                    flex_grow: 2.0,
                    ..Default::default()
                },
                text: Text::from_section(
                    "FPS:".to_string(),
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 60.0,
                        color: fps.color,
                        ..Default::default()
                    },
                ),
                ..Default::default()
            })
            .insert(FpsText);
    }
}
