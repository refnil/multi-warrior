use bevy::prelude::*;

pub struct FxPlugin {
}

impl Plugin for FxPlugin{
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<Events<FxSpawnEvent>>()
            .add_system(spawn_fx_system.system());

    }
}

struct FxSpawnEvent {
    kind: FxKind, 
    transform: Transform,
    duration: f32,
}

enum FxKind {
    Fire,
    Death,
}

fn spawn_fx_system()
{
}
