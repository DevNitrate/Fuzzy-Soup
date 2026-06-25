#![allow(dead_code)]

use bevy::{DefaultPlugins, app::{App, Startup}, asset::Assets, camera::Camera3d, camera_controller::free_camera::{FreeCamera, FreeCameraPlugin}, ecs::system::{Commands, ResMut}, math::primitives::Sphere, mesh::Mesh3d, pbr::{MeshMaterial3d, StandardMaterial}, prelude::Mesh, transform::components::Transform};

use crate::train::colmap::ColmapScene;

mod utils;
mod train;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FreeCameraPlugin)
        // .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let scene = ColmapScene::new("colmap/south-building/undistorted/");
    commands.spawn((Camera3d::default(), FreeCamera::default(), Transform::from_translation(scene.images[0].camera_pos).with_rotation(scene.images[0].camera_orientation)));

    println!("img path: {:?}", scene.images[0].image_path);
    let sphere = meshes.add(Sphere::new(0.001));
    let mat = StandardMaterial {
        unlit: true,
        ..Default::default()
    };
    let material = materials.add(mat);

    for p in scene.points {
        commands.spawn((
                Mesh3d(sphere.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_xyz(p.x, p.y, p.z)
        ));
    }

}
