mod spawn_cell;
mod neighbor_check;

use std::ops::Add;
use bevy::color::palettes::basic::*;
use bevy::input::common_conditions::{input_just_pressed, input_pressed};
use bevy::prelude::*;
use bevy::reflect::utility::GenericTypeCell;
use bevy::utils::HashMap;
use bevy::window::PrimaryWindow;
use bevy::window::WindowEvent::KeyboardInput;
use std::{thread, time};

const CELL_WIDTH: f32 = 20.0;
const SMALL_STAR:[(i32, i32); 5] = [(0, CELL_WIDTH as i32), (CELL_WIDTH as i32, 0), (0, -CELL_WIDTH as i32), (-CELL_WIDTH as i32, 0), (0, 0)];
const FULL_NEIGHBORHOOD: [(i32, i32); 9] = [(-CELL_WIDTH as i32, CELL_WIDTH as i32), (0, CELL_WIDTH as i32), (CELL_WIDTH as i32, CELL_WIDTH as i32), (CELL_WIDTH as i32, 0), (CELL_WIDTH as i32, -CELL_WIDTH as i32), (0, -CELL_WIDTH as i32), (-CELL_WIDTH as i32, -CELL_WIDTH as i32), (-CELL_WIDTH as i32, 0), (0, 0)];
pub const INVISIBLE: Color = Color::srgba(1.0, 1.0, 1.0, 0.03);
pub const ALIVE: Color = Color::srgba(1.0, 1.0, 1.0, 1.0);

pub const TIME_PER_FRAME: u64 = 150;

#[derive(Component, Clone, Copy, Debug)]
pub struct Cell {
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

pub trait IVec2Extensions {
    fn add_tuple(&mut self, tuple: (i32, i32)) -> IVec2;
}

impl IVec2Extensions for IVec2 {
    fn add_tuple(&mut self, tuple: (i32, i32)) -> IVec2{
        IVec2::new(self.x + tuple.0, self.y + tuple.1)
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
            spawn_cell.run_if(input_pressed(MouseButton::Left)),
            rules.run_if(input_pressed(KeyCode::Space)),
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
    mut query: Query<(&mut Cell, &MeshMaterial2d<ColorMaterial>)>,
    q_windows: Single<&Window, With<PrimaryWindow>>
) {
    if let Some(click_position) = q_windows.cursor_position() {
        let screen = check_screen(*q_windows);
        let mut spawn_position:IVec2 = integer_position(Vec2::new(click_position.x, click_position.y), Vec2::new(screen.width, screen.height));

        for neighbor in FULL_NEIGHBORHOOD {
            match cell_map.0.get_mut(&spawn_position.add_tuple(neighbor)) {
                Some(mut cell) => {

                    if cell.mortal_state == 0.0 && neighbor == (0, 0) {

                        for (mut check_cell, handle) in query.iter_mut() {
                            if check_cell.position == cell.position {
                                if let Some(mut material) = materials.get_mut(handle) {
                                    material.color = ALIVE;
                                }
                                check_cell.mortal_state = 1.0;
                            }
                        }
                        cell.mortal_state = 1.0;
                    }
                },
                None => {
                    if neighbor != (0, 0) {
                        commands.spawn((
                            Mesh2d(meshes.add(Rectangle::new(CELL_WIDTH, CELL_WIDTH))),
                            MeshMaterial2d(materials.add(Color::from(INVISIBLE))),
                            Transform::from_xyz((spawn_position.x + neighbor.0) as f32, (spawn_position.y + neighbor.1) as f32, 0.0),
                            Cell::spawn_dead(spawn_position.add_tuple(neighbor))
                        ));
                        cell_map.0.insert(spawn_position.add_tuple(neighbor), Cell::spawn_dead(spawn_position.add_tuple(neighbor)));
                    } else {
                        commands.spawn((
                            Mesh2d(meshes.add(Rectangle::new(CELL_WIDTH, CELL_WIDTH))),
                            MeshMaterial2d(materials.add(Color::from(ALIVE))),
                            Transform::from_xyz(spawn_position.x as f32, spawn_position.y as f32, 0.0),
                            Cell::spawn_live(spawn_position)
                        ));
                        cell_map.0.insert(spawn_position, Cell::spawn_live(spawn_position));
                    }
                }
            }
        }
    }
}


fn rules(
    mut commands: Commands,
    mut query: Query<(&mut Cell, &MeshMaterial2d<ColorMaterial>)>,
    mut cell_map: ResMut<CellMap>,
    mut materials: ResMut<Assets<ColorMaterial>>
) {


    for (mut curr_cell, _) in query.iter_mut() {

        for neighbor in FULL_NEIGHBORHOOD {

            let neighbor_position = IVec2::new(curr_cell.position.x + neighbor.0, curr_cell.position.y + neighbor.1);
            match cell_map.0.get_mut(&neighbor_position) {
                Some(mut next_cell) => {
                    if neighbor == (0, 0) {
                        println!("cell: {}; state: {}; neighbors: {}",curr_cell.position, curr_cell.mortal_state, curr_cell.live_neighbors);
                        println!("-----------next: {}", next_cell.mortal_state);
                        next_cell.live_neighbors = curr_cell.live_neighbors;

                    } else {
                        curr_cell.live_neighbors += next_cell.mortal_state; //loading next
                        //println!("{}", next_cell.mortal_state)
                    }
                },
                None => {

                }
            }
        }
    }
    println!("----------");

    // visual updates
    for (mut cell, handle) in query.iter_mut() {
        match cell_map.0.get(&cell.position) {
            Some(next_cell) => {
                if next_cell.live_neighbors > 1.0 && next_cell.live_neighbors < 4.0 && next_cell.mortal_state == 1.0{
                    if let Some(mut curr_mat) = materials.get_mut(handle) {
                        curr_mat.color = Color::srgba(1.0, 1.0, 1.0, 1.0);
                    }
                    cell.mortal_state = 1.0;
                    cell.live_neighbors = 0.0;

                } else if next_cell.live_neighbors == 3.0 && next_cell.mortal_state == 0.0 {
                    if let Some(mut curr_mat) = materials.get_mut(handle) {
                        curr_mat.color = Color::srgba(0.0, 1.0, 0.0, 1.0);
                    }
                    cell.mortal_state = 1.0;
                    cell.live_neighbors = 0.0;

                } else {
                    if let Some(mut curr_mat) = materials.get_mut(handle) {
                        curr_mat.color = Color::srgba(1.0, 0.0, 0.0, 0.0);
                    }
                    cell.mortal_state = 0.0;
                    cell.live_neighbors = 0.0;

                }
            }
            None => {}
        }
    }

    let ten_millis = time::Duration::from_millis(TIME_PER_FRAME);

    thread::sleep(ten_millis);
}

fn integer_position(click_position: Vec2, screen: Vec2) -> IVec2 {

    let centered_position = Vec2::new(click_position.x - screen.x, screen.y - click_position.y);
    let ratio = Vec2::new((centered_position.x/CELL_WIDTH).round(), (centered_position.y/CELL_WIDTH).round());

    let x_position: f32 = ratio.x * CELL_WIDTH;
    let y_position: f32 = ratio.y * CELL_WIDTH;

    IVec2::new(x_position as i32, y_position as i32)

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