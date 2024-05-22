use bevy::prelude::*;
use bevy_openxr::{add_xr_plugins, init::OxrInitPlugin, types::OxrExtensions};

#[bevy_main]
fn main() {
    App::new()
        .add_plugins(add_xr_plugins(DefaultPlugins).set(OxrInitPlugin {
            app_info: default(),
            exts: {
                let mut exts = OxrExtensions::default();
                exts.enable_fb_passthrough();
                exts.enable_hand_tracking();
                exts.enable_scene();
                exts
            },
            blend_modes: default(),
            backends: default(),
            formats: default(),
            resolutions: default(),
            synchronous_pipeline_compilation: default(),
        }))
        .add_plugins(bevy_xr_utils::hand_gizmos::HandGizmosPlugin)
        .add_systems(Startup, setup)
        .insert_resource(ClearColor(Color::NONE))
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // the scene meshes should be loaded in here(?)
}

fn test() {
    // running query scene anchors
}
