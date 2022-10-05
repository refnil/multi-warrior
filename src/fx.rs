use bevy::prelude::*;

use crate::anim::*;

pub struct FxPlugin;

impl Plugin for FxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Events<FxSpawnEvent>>()
            .init_resource::<SpawnFXData>()
            .add_system_to_stage(
                CoreStage::PostUpdate,
                Events::<FxSpawnEvent>::update_system,
            )
            .add_system(spawn_fx_system)
            .add_system(remove_finished_fx);
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

#[derive(Component)]
struct Fx;

struct SpawnFXData {
    fire_handle: Handle<TextureAtlas>,
    death_handle: Handle<TextureAtlas>,
}

impl FromWorld for SpawnFXData {
    fn from_world(world: &mut World) -> Self {

        let asset_server = world.get_resource::<AssetServer>().unwrap();
        let fire = asset_server.load("spritesheet/effects/MagicBarrier_64x64.png");
        let death = asset_server.load("spritesheet/effects/explosion.png");

        let mut texture_atlas_asset = world.get_resource_mut::<Assets<TextureAtlas>>().unwrap();

        let fire_atlas = TextureAtlas::from_grid(fire, Vec2::new(64.0, 64.0), 33, 1);
        let fire_handle = texture_atlas_asset.add(fire_atlas);
        let death_atlas = TextureAtlas::from_grid(death, Vec2::new(64.0, 64.0), 4, 4);
        let death_handle = texture_atlas_asset.add(death_atlas);
        Self {
            fire_handle: fire_handle.clone(),
            death_handle: death_handle.clone(),
        }
    }
}

fn spawn_fx_system(
    mut command: Commands,
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
        let mut bundle = command.spawn_bundle(SpriteSheetBundle {
            texture_atlas: texture_atlas,
            transform: transform,
            sprite: TextureAtlasSprite {
                index: animation.current_frame(),
                ..Default::default()
            },
            ..Default::default()
        });
        if let Some(duration) = event.duration {
            bundle.insert(AnimTimer::new(duration/animation.frame_count()as f32));
        }
        bundle.insert(animation);
        bundle.insert(Fx);
    }
}

fn remove_finished_fx(mut command: Commands, query: Query<(Entity, &Animation), With<Fx>>) {
    for (e, anim) in query.iter() {
        if anim.is_stopped() {
            command.entity(e).despawn();
        }
    }
}
