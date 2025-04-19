//! This example demonstrates the built-in 3d shapes in Bevy.
//! The scene includes a patterned texture and a rotation for visualizing the normals and UVs.
//!
//! You can toggle wireframes with the space bar except on wasm. Wasm does not support
//! `POLYGON_MODE_LINE` on the gpu.

use std::f32::consts::{FRAC_PI_2, PI};

use bevy::{
    color::palettes::basic::SILVER, core_pipeline::fxaa::Fxaa, input::mouse::MouseWheel, prelude::*
};
use bevy_simpletoon::plugin::SimpletoonPlugin;
use bevy_simpletoon::plugin::SimpletoonSettings;
use bevy::input::mouse::MouseMotion;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            SimpletoonPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                rotate,
                move_camera,
                zoom_camera
            ),
        )
        .run();
}

/// A marker component for our shapes so we can query them separately from the ground plane
#[derive(Component)]
struct Shape;

const SHAPES_X_EXTENT: f32 = 14.0;
const EXTRUSION_X_EXTENT: f32 = 16.0;
const Z_EXTENT: f32 = 5.0;

fn mat_from(c: Color) -> StandardMaterial {
    return StandardMaterial {
        perceptual_roughness: 0.8,
        base_color: c,
        ..default()
    };
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {


    let mats = [
        materials.add(mat_from(Color::srgb_u8(255, 100, 255))),
        materials.add(mat_from(Color::srgb_u8(22, 144, 100))),
        materials.add(mat_from(Color::srgb_u8(124, 144, 255))),
        materials.add(mat_from(Color::srgb_u8(200, 100, 255))),
        materials.add(mat_from(Color::srgb_u8(255, 0, 255))),
        materials.add(mat_from(Color::srgb_u8(244, 0, 30))),
        materials.add(mat_from(Color::srgb_u8(0, 144, 0))),
        materials.add(mat_from(Color::srgb_u8(0, 0, 255))),
        materials.add(mat_from(Color::srgb_u8(222, 144, 144))),        
    ];

    let shapes = [
        meshes.add(Cuboid::default()),
        meshes.add(Tetrahedron::default()),
        meshes.add(Capsule3d::default()),
        meshes.add(Torus::default()),
        meshes.add(Cylinder::default()),
        meshes.add(Cone::default()),
        meshes.add(ConicalFrustum::default()),
        meshes.add(Sphere::default().mesh().ico(5).unwrap()),
        meshes.add(Sphere::default().mesh().uv(32, 18)),
    ];

    let extrusions = [
        meshes.add(Extrusion::new(Rectangle::default(), 1.)),
        meshes.add(Extrusion::new(Capsule2d::default(), 1.)),
        meshes.add(Extrusion::new(Annulus::default(), 1.)),
        meshes.add(Extrusion::new(Circle::default(), 1.)),
        meshes.add(Extrusion::new(Ellipse::default(), 1.)),
        meshes.add(Extrusion::new(RegularPolygon::default(), 1.)),
        meshes.add(Extrusion::new(Triangle2d::default(), 1.)),
    ];

    let num_shapes = shapes.len();

    for (i, shape) in shapes.into_iter().enumerate() {
        commands.spawn((
            Mesh3d(shape),
            MeshMaterial3d(mats[i].clone()),
            Transform::from_xyz(
                -SHAPES_X_EXTENT / 2. + i as f32 / (num_shapes - 1) as f32 * SHAPES_X_EXTENT,
                2.0,
                Z_EXTENT / 2.,
            )
            .with_rotation(Quat::from_rotation_x(-PI / 4.)),
            Shape,
        ));
    }

    let num_extrusions = extrusions.len();

    for (i, shape) in extrusions.into_iter().enumerate() {
        commands.spawn((
            Mesh3d(shape),
            MeshMaterial3d(mats[i].clone()),
            Transform::from_xyz(
                -EXTRUSION_X_EXTENT / 2.
                    + i as f32 / (num_extrusions - 1) as f32 * EXTRUSION_X_EXTENT,
                2.0,
                -Z_EXTENT / 2.,
            )
            .with_rotation(Quat::from_rotation_x(-PI / 4.)),
            Shape,
        ));
    }

    let mut t = Transform::from_rotation(Quat::from_rotation_x(-PI / 4.0)); 
    t.rotate_y(FRAC_PI_2);
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: 2000.0,
            ..default() 
        },
        t
    ));
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 2000.0,
    });
 
    // ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0).subdivisions(10))),
        MeshMaterial3d(materials.add(mat_from(Color::from(SILVER)))),
    ));

    commands.spawn((
        Camera3d::default(),
        SimpletoonSettings::default(),
        Msaa::Off,
        Fxaa::default(),
        Transform::from_xyz(0.0, 7., 14.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        CameraController::default(),
    ));

    commands.insert_resource(ClearColor(Color::srgb_u8(135, 206, 235)));

}

fn rotate(mut query: Query<&mut Transform, With<Shape>>, time: Res<Time>) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_secs() / 2.);
    }
}

pub fn move_camera(
    mut camera_query: Query<(&mut Transform, &CameraController)>,
    mut mouse_motion_events: EventReader<MouseMotion>
) {
    let (mut ct, cam_con) = camera_query.single_mut();


    for event in mouse_motion_events.read() {
        let (yaw, pitch, roll) = ct.rotation.to_euler(EulerRot::YXZ);
        ct.rotation = Quat::from_euler(EulerRot::YXZ, 
            yaw - event.delta.x * cam_con.x_speed, 
            (pitch - (event.delta.y * cam_con.y_speed)).clamp(cam_con.min_pitch, cam_con.max_pitch), 
            roll
        );
    }
    ct.translation = Vec3::ZERO - ct.forward() * cam_con.distance;

}
pub fn zoom_camera(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut camera_query: Query<&mut CameraController, With<Camera>>,
) {
    for mouse_wheel_event in mouse_wheel_events.read() {
        for mut cam_con in &mut camera_query {

            cam_con.distance -= mouse_wheel_event.y * cam_con.zoom_speed;
        }
    }
}
#[derive(Component)]
pub struct CameraController {
    pub min_pitch: f32,
    pub max_pitch: f32,
    pub x_speed: f32,
    pub y_speed: f32,
    pub distance: f32,
    pub zoom_speed: f32
}

impl Default for CameraController {
    fn default() -> Self {
        Self { 
            min_pitch: -1.0, 
            max_pitch: 0.0, 
            x_speed: 0.0015, 
            y_speed: 0.0015, 
            distance: 30.0,
            zoom_speed: 2.0
        }
    }
}