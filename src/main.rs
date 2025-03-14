use bevy::color::palettes::basic::{BLACK, WHITE};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

const CELL_WIDTH: f32 = 5.0;

struct Grid {

}

#[derive(Component)]
struct Cell {
    mortal_state: f32,
}

struct Screen {
    width: f32,
    height: f32
}

impl Screen {
    fn new(width: f32, height: f32) -> Self { Screen { width, height } }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (spawn_camera, grid))
        //.add_systems(Update, smooth_life)
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn check_screen(window: &Window) -> Screen {
    let width = window.resolution.width()/2.0;
    let height = window.resolution.height()/2.0;
    Screen::new(width, height)
}

fn grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    q_windows: Single<&Window, With<PrimaryWindow>>,
) {

    let screen = check_screen(*q_windows);

    let cell_count_x = (screen.width / CELL_WIDTH) as i32;
    let mut x = 0;
    let cell_count_y = (screen.height / CELL_WIDTH) as i32;
    let mut y = 0;


    fn color(x: i32, color1: Srgba, color2: Srgba) -> Srgba {
        if x % 2 != 0 {
            color1
        } else {
            color2
        }
    }

    while x != cell_count_x*2 || y != cell_count_y*2 {
        if x < cell_count_x*2 {
            println!("x: {} range: {}", x, cell_count_x);
            commands.spawn((
                Mesh2d(meshes.add(Rectangle::default())),
                MeshMaterial2d(materials.add(Color::from(color))),
                Transform::from_xyz(2.5-screen.width + CELL_WIDTH*x as f32, screen.height - CELL_WIDTH*y as f32, 0.0)
                    .with_scale(Vec3::splat(5.0)),
                Cell {
                    mortal_state: 0.0,
                }
            ));
            x += 1;
        } else {
            x = 0;
            y += 1;
        }
    }
    println!("done")
}
/*
fn smooth_life(
    mut query: Query<(&mut Cell, &Transform, &mut Handle<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    handles: Query<&Handle<StandardMaterial>>
) {
    for cell in query.iter_mut() {
        if cell.1.translation % 2.0 != Vec3::ZERO {
            println!("handle: {:?}", handles)
        }
    }
}*/
