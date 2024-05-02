use crate::editor_plugin::EditorPlugin;
use bevy::app::{App, AppExit, Last, Startup};
use bevy::asset::Assets;
use bevy::math::Vec3;
use bevy::pbr::{PbrBundle, PointLight, PointLightBundle, StandardMaterial};
use bevy::prelude::{
    default, Camera3dBundle, Color, Commands, Component, Cuboid, EventReader, Mesh, ResMut,
    Transform,
};
use bevy::DefaultPlugins;
use dexterous_developer::{
    dexterous_developer_setup, hot_bevy_main, InitialPlugins, ReloadableApp, ReloadableAppContents,
    ReloadableElementsSetup,
};
use std::process::exit;

mod editor_plugin;
pub mod terminal;
mod code_editor;

#[hot_bevy_main]
pub fn bevy_main(initial_plugins: impl InitialPlugins) {
    let mut app = App::new();
    app.add_plugins(initial_plugins.initialize::<DefaultPlugins>());
    app.add_plugins(EditorPlugin::default());
    app.add_systems(Last, |mut app_exit: EventReader<AppExit>| {
        if !app_exit.is_empty() {
            exit(0);
        }
    });
    if std::env::var("ONLY_UI").is_ok() {
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Camera3dBundle::default());
        });
        app.run();
        return;
    }
    app.setup_reloadable_elements::<reloadable>();
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // cube
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            material: materials.add(Color::rgb_u8(21, 14, 25)),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
        GetRidOf,
    ));
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        GetRidOf,
    ));
}

#[derive(Component)]
struct GetRidOf;

#[dexterous_developer_setup]
fn reloadable(app: &mut ReloadableAppContents) {
    app.reset_setup::<GetRidOf, _>(setup);
}
