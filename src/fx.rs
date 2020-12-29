use bevy::prelude::*;

use crate::anim::*;

pub struct FxPlugin;

impl Plugin for FxPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<Events<FxSpawnEvent>>()
            .init_resource::<SpawnFXData>()
            .add_system_to_stage(
                stage::POST_UPDATE,
                Events::<FxSpawnEvent>::update_system.system(),
            )
            .add_system(spawn_fx_system.system())
            .add_system(remove_finished_fx.system());
    }
}

pub struct FxSpawnEvent {
    pub kind: FxKind,
    pub transform: Transform,
    pub duration: Option<f32>,
}

pub enum FxKind {
    Death,
    Fire,
}

struct Fx;

struct SpawnFXData {
    fire_handle: Handle<TextureAtlas>,
    death_handle: Handle<TextureAtlas>,
}

impl FromResources for SpawnFXData {
    fn from_resources(res: &Resources) -> Self {
        let asset_server = res.get_mut::<AssetServer>().unwrap();
        let mut texture_atlas_asset = res.get_mut::<Assets<TextureAtlas>>().unwrap();

        let fire = asset_server.load("spritesheet/effects/MagicBarrier_64x64.png");
        let fire_atlas = TextureAtlas::from_grid(fire, Vec2::new(64.0, 64.0), 33, 1);
        let fire_handle = texture_atlas_asset.add(fire_atlas);

        let death = asset_server.load("spritesheet/effects/explosion.png");
        let death_atlas = TextureAtlas::from_grid(death, Vec2::new(64.0, 64.0), 4, 4);
        let death_handle = texture_atlas_asset.add(death_atlas);
        Self {
            fire_handle: fire_handle.clone(),
            death_handle: death_handle.clone(),
        }
    }
}

fn spawn_fx_system(
    command: &mut Commands,
    data: Res<SpawnFXData>,
    events: Res<Events<FxSpawnEvent>>,
) {
    let mut reader = events.get_reader();

    for event in reader.iter(&events) {
        let texture_atlas = match event.kind {
            FxKind::Fire => &data.fire_handle,
            FxKind::Death => &data.death_handle,
        }.clone();
        let animation = match event.kind {
            FxKind::Fire => Animation::new(AnimationMode::Stop, (0..33).collect()),
            FxKind::Death => Animation::new(AnimationMode::Stop, (0..16).collect()),
        };
        let mut transform = event.transform.clone();
        transform.translation.z += 1.0;
        command.spawn(SpriteSheetBundle {
            texture_atlas: texture_atlas,
            transform: transform,
            sprite: TextureAtlasSprite {
                index: animation.current_frame(),
                ..Default::default()
            },
            ..Default::default()
        });
        if let Some(duration) = event.duration {
            command.with(AnimTimer::new(duration/animation.frame_count()as f32));
        }
        command.with(animation);
        command.with(Fx);
    }
}

fn remove_finished_fx(command: &mut Commands, query: Query<(Entity, &Animation), With<Fx>>) {
    for (e, anim) in query.iter() {
        if anim.is_stopped() {
            command.despawn(e);
        }
    }
}
