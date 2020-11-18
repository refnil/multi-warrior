use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*
};

#[derive(Default, Clone)]
pub struct FPSPlugin {
    pub color: Color
}

impl Plugin for FPSPlugin{
    fn build(&self, app: &mut AppBuilder){
        app.add_plugin(FrameTimeDiagnosticsPlugin::default())
           .add_resource(self.clone())
           .add_startup_system(Self::setup.system())
           .add_system(Self::text_update_system.system());
    }
}

// A unit struct to help identify the FPS UI component, since there may be many Text components
struct FpsText;

impl FPSPlugin {
    fn text_update_system(diagnostics: Res<Diagnostics>, mut query: Query<(&mut Text, &FpsText)>) { 
        for (mut text, _tag) in query.iter_mut() {
            if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
                if let Some(average) = fps.average() {
                    text.value = format!("FPS: {:.2}", average);
                }
            }
        }
    }

    fn setup(commands: &mut Commands, fps: Res<FPSPlugin>, asset_server: Res<AssetServer>) {
        commands
            // texture
            .spawn(TextBundle {
                style: Style {
                    //max_size: Size::new(Val::Percent(25.0), Val::Undefined),
                    //align_self: AlignSelf::FlexEnd,
                    flex_grow: 2.0,
                    ..Default::default()
                },
                text: Text {
                    value: "FPS:".to_string(),
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    style: TextStyle {
                        font_size: 60.0,
                        color: fps.color,
                        ..Default::default()
                    },
                },
                ..Default::default()
            })
            .with(FpsText);
    }
}

