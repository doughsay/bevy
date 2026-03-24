//! Test that inserting TerrainShadowMaskTexture on a camera doesn't crash
//! the PBR pipeline. This validates the Bevy fork's binding 32 integration.
//!
//! The test creates a dummy R8Unorm texture and inserts it as
//! TerrainShadowMaskTexture on the camera entity in the render world.
//! If the view bind group creation at binding 32 is correct, the scene
//! renders without wgpu validation errors.

use bevy::{
    prelude::*,
    pbr::TerrainShadowMaskTexture,
    render::{
        render_resource::*,
        renderer::RenderDevice,
        camera::ExtractedCamera,
        view::ExtractedView,
        texture::TextureCache,
        RenderApp, Render,
        render_resource::TextureDimension,
    },
    core_pipeline::prepass::{DepthPrepass, ViewPrepassTextures},
};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_systems(Startup, setup);

    // Add a render-world system that inserts TerrainShadowMaskTexture
    if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
        render_app.add_systems(
            Render,
            insert_dummy_shadow_mask
                .in_set(bevy::render::RenderSystems::PrepareAssets),
        );
    }

    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
    // cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));
    // directional light with shadows
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    // camera — DepthPrepass + Msaa::Off (matches the planetary engine setup)
    commands.spawn((
        Camera3d::default(),
        DepthPrepass,
        Msaa::Off,
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

/// Render-world system: creates a dummy R8Unorm shadow mask texture and
/// inserts TerrainShadowMaskTexture on views that have a depth prepass.
fn insert_dummy_shadow_mask(
    mut commands: Commands,
    mut texture_cache: ResMut<TextureCache>,
    render_device: Res<RenderDevice>,
    views: Query<
        (Entity, &ExtractedCamera),
        (With<ExtractedView>, With<ViewPrepassTextures>),
    >,
) {
    for (entity, camera) in &views {
        let Some(size) = camera.physical_viewport_size else {
            continue;
        };

        let cached = texture_cache.get(
            &render_device,
            TextureDescriptor {
                label: Some("test_terrain_shadow_mask"),
                size: Extent3d {
                    width: size.x,
                    height: size.y,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::R8Unorm,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::STORAGE_BINDING,
                view_formats: &[],
            },
        );

        commands.entity(entity).insert(TerrainShadowMaskTexture {
            texture_view: cached.default_view.clone(),
        });
    }
}
