use bevy::color::palettes::basic::*;
use bevy::input::common_conditions::{input_just_pressed, input_pressed};
use bevy::prelude::*;
use bevy::reflect::utility::GenericTypeCell;
use bevy::utils::HashMap;
use bevy::window::PrimaryWindow;
use bevy::window::WindowEvent::KeyboardInput;

const CELL_WIDTH: f32 = 20.0;
const SMALL_STAR:[(i32, i32); 5] = [(0, 20), (20, 0), (0, -20), (-20, 0), (0, 0)];

pub const INVISIBLE: Color = Color::srgba(1.0, 1.0, 1.0, 1.0);

#[derive(Component, Clone, Copy, Debug)]
struct Cell {
    mortal_state: f32,
    position: IVec2,
    live_neighbors: f32,
}

impl Cell {

    fn new(mortal_state: f32, position: IVec2, live_neighbors: f32) -> Cell {
        Cell {
            mortal_state,
            position,
            live_neighbors,
        }
    }
    fn spawn_live(position: IVec2) -> Cell {
        Cell {
            mortal_state: 1.0,
            position,
            live_neighbors: 0.0,
        }
    }

    fn spawn_dead(position: IVec2) -> Cell {
        Cell {
            mortal_state: 0.0,
            position,
            live_neighbors: 0.0,
        }
    }

    fn mortal_update(&mut self, mortal_state: f32) {
        self.mortal_state = mortal_state
    }

    fn neighborhood_update (&mut self, neighbors: f32) {
        self.live_neighbors = neighbors;
    }
}

#[derive(Resource, Default)]
struct CellMap(HashMap<IVec2, Cell>);

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
        .insert_resource(CellMap::default())
        .add_systems(Startup, spawn_camera)
        .add_systems(Update, (
            neighborhood_check,
            rules.run_if(input_pressed(KeyCode::Space)),
            spawn_cell.run_if(input_pressed(MouseButton::Left)),
            clear_terminal
        ))
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


fn spawn_cell(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut cell_map: ResMut<CellMap>,
    q_windows: Single<&Window, With<PrimaryWindow>>
) {
    if let Some(click_position) = q_windows.cursor_position() {
        let screen = check_screen(*q_windows);
        let spawn_position:IVec2 = integer_position(Vec2::new(click_position.x, click_position.y), Vec2::new(screen.width, screen.height));

        for (key, value) in cell_map.0.iter() {
            println!("list on click: {key:?}; {value:?}")
        }

        match cell_map.0.get(&spawn_position) {
            Some(cell) => {
                if cell.mortal_state == 0.0 {

                    cell_map.0.remove(&spawn_position);
                    cell_map.0.insert(spawn_position, Cell::spawn_live(spawn_position));

                    for (key, value) in cell_map.0.iter() {
                        println!("animate cell: {key:?}; {value:?}")
                    }

                }
            },
            None => {
                commands.spawn((
                    Mesh2d(meshes.add(Rectangle::new(CELL_WIDTH, CELL_WIDTH))),
                    MeshMaterial2d(materials.add(Color::from(WHITE))),
                    Transform::from_xyz(spawn_position.x as f32, spawn_position.y as f32, 0.0),
                    Cell::spawn_live(spawn_position)
                ));

                cell_map.0.insert(IVec2::new(spawn_position.x, spawn_position.y), Cell::spawn_live(spawn_position));

                for dead_neighbor in SMALL_STAR {
                    if dead_neighbor != (0, 0) {
                        let dead_neighbor_position = IVec2::new(spawn_position.x + dead_neighbor.0, spawn_position.y + dead_neighbor.1);
                        commands.spawn((
                            Mesh2d(meshes.add(Rectangle::new(CELL_WIDTH, CELL_WIDTH))),
                            MeshMaterial2d(materials.add(Color::from(INVISIBLE))),
                            Transform::from_xyz(spawn_position.x as f32, spawn_position.y as f32, 0.0),
                            Cell::spawn_dead(dead_neighbor_position)
                        ));

                        cell_map.0.insert(dead_neighbor_position, Cell::spawn_dead(dead_neighbor_position));
                    }
                }
            }
        }
    }
}

fn neighborhood_check(
    mut query: Query<&mut Cell>,
    mut cell_map: ResMut<CellMap>,
) {

    let mut cell_count = 0;

    for mut cell in query.iter_mut() {
        cell.live_neighbors = 0.0;

        for neighbor in SMALL_STAR {

            let key_value = IVec2::new(cell.position.x + neighbor.0, cell.position.y + neighbor.1);
            match cell_map.0.get(&key_value) {
                Some(neighbor) => {
                    if neighbor.position == cell.position {
                        println!("neighbor_check: {:?}", cell);
                        cell_map.0.remove(&cell.position);
                        cell_map.0.insert(cell.position, Cell::new(cell.mortal_state, cell.position, cell.live_neighbors));
                    } else {
                        cell.live_neighbors += neighbor.mortal_state
                    }
                },
                None => {

                }
            }
        }

        cell_count += 1;
    }
    println!("cell count: {}", cell_count);
}

fn rules(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Cell, &MeshMaterial2d<ColorMaterial>)>,
    mut cell_map: ResMut<CellMap>,
    mut materials: ResMut<Assets<ColorMaterial>>
) {
    for (entity, cell, handle) in query.iter_mut() {



        match cell_map.0.get(&cell.position) {
            Some(hash_cell) => {
                if let Some(mut mat) = materials.get_mut(handle) {
                    mat.color = Color::srgba(1.0, 1.0, 0.0, hash_cell.mortal_state);
                    println!("{:?}=?{:?}", hash_cell.mortal_state, cell.mortal_state)

                }
            },
            None => {}
        }
    }
}

fn integer_position(click_position: Vec2, screen: Vec2) -> IVec2 {

    let centered_position = Vec2::new(click_position.x - screen.x, screen.y - click_position.y);
    let ratio = Vec2::new((centered_position.x/CELL_WIDTH).round(), (centered_position.y/CELL_WIDTH).round());

    let x_position: f32 = ratio.x * CELL_WIDTH;
    let y_position: f32 = ratio.y * CELL_WIDTH;

    IVec2::new(x_position as i32, y_position as i32)

}


fn clear_terminal(){
    print!("\x1B[2J\x1B[1;1H");
}
/*

pub fn check_electrons(
    mut commands: Commands,
    mut query: Query<(&MeshMaterial2d<ColorMaterial>, &mut Transform, &mut Particle)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut rng = rand::rng();
    for (handle, position, mut particle, ) in query.iter_mut() {
        if particle.cycles < 1 {
            if particle.particle_type == ParticleType::Electron {
                let distance_to_center = position.translation.distance(Vec3::ZERO);
                if wave_function(distance_to_center, rng.clone()) {
                    println!("------------------------------------------------{distance_to_center}");
                    if let Some(mut mat) = materials.get_mut(handle) {
                        mat.color = Color::from(GRAY)
                    }
                }
            }
            println!("cycle")
        }
        particle.cycles += 1;
    }
}

 */