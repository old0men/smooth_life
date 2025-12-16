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
pub const ALIVE: Color = Color::srgba(1.0, 1.0, 1.0, 0.999);

pub const TIME_PER_FRAME: u64 = 32;

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

    fn is_dead(&self) -> bool {
        if self.mortal_state == 0.0 {true} else {false}
    }

    fn is_alive(&self) -> bool {
        if self.mortal_state == 0.0 {false} else {true}
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

impl CellMap {
    fn update_mortal(&mut self, position: IVec2, mortal_state: f32) {
        match self.0.get_mut(&position) {
            Some(mut map_cell) => {
                map_cell.mortal_state = mortal_state;
            },
            _ => {}
        }
    }
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
        .insert_resource(CellMap::default())
        .add_systems(Startup, spawn_camera)
        .add_systems(Update, (
            spawn_space.run_if(input_pressed(MouseButton::Right)),
            spawn_cell.run_if(input_pressed(MouseButton::Left)),
            rules.run_if(input_pressed(KeyCode::Space)),
            smooth_update.run_if(input_pressed(KeyCode::Space)),
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

    let mut alive = 0;
    let mut dead = 0;
    let mut map_alive = 0;
    let mut map_dead = 0;

    for (cell, _) in query.iter() {
        if cell.mortal_state == 1.0 {
            alive += 1
        } else {
            dead += 1
        }
    }

    for (_, cell) in cell_map.0.iter_mut() {
        if cell.mortal_state == 1.0 {
            map_alive += 1
        } else {
            map_dead += 1
        }
    }
    println!("main alive: {}, dead: {}", alive, dead);
    println!("map alive: {}, dead: {}", map_alive, map_dead);
    println!("--------------------------");

    if let Some(click_position) = q_windows.cursor_position() {
        let screen = check_screen(*q_windows);
        let mut spawn_position: IVec2 = integer_position(Vec2::new(click_position.x, click_position.y), Vec2::new(screen.width, screen.height));

        for neighbor in FULL_NEIGHBORHOOD {
            match cell_map.0.get_mut(&spawn_position.add_tuple(neighbor)) {
                Some(mut cell) => {
                    if cell.is_dead() && neighbor == (0, 0) {
                        for (mut check_cell, handle) in query.iter_mut() {
                            if check_cell.position == cell.position {
                                if let Some(mut material) = materials.get_mut(handle) {
                                    material.color = Color::from(ALIVE);
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

fn spawn_space (
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut cell_map: ResMut<CellMap>,
    mut query: Query<(&mut Cell, &MeshMaterial2d<ColorMaterial>)>,
    q_windows: Single<&Window, With<PrimaryWindow>>
){

    let mut alive = 0;
    let mut dead = 0;
    let mut map_alive = 0;
    let mut map_dead = 0;

    for (cell, _) in query.iter() {
        if cell.mortal_state == 1.0 {
            alive += 1
        } else {
            dead += 1
        }
    }

    for (_, cell) in cell_map.0.iter_mut() {
        if cell.mortal_state == 1.0 {
            map_alive += 1
        } else {
            map_dead += 1
        }
    }
    println!("main alive: {}, dead: {}", alive, dead);
    println!("map alive: {}, dead: {}", map_alive, map_dead);
    println!("--------------------------");

    if let Some(click_position) = q_windows.cursor_position() {
        let screen = check_screen(*q_windows);
        let mut spawn_position: IVec2 = integer_position(Vec2::new(click_position.x, click_position.y), Vec2::new(screen.width, screen.height));

        for neighbor in FULL_NEIGHBORHOOD {
            match cell_map.0.get_mut(&spawn_position.add_tuple(neighbor)) {
                Some(cell) => {continue},
                None => {
                    commands.spawn((
                        Mesh2d(meshes.add(Rectangle::new(CELL_WIDTH, CELL_WIDTH))),
                        MeshMaterial2d(materials.add(Color::from(INVISIBLE))),
                        Transform::from_xyz((spawn_position.x + neighbor.0) as f32, (spawn_position.y + neighbor.1) as f32, 0.0),
                        Cell::spawn_dead(spawn_position.add_tuple(neighbor))
                    ));
                    cell_map.0.insert(spawn_position.add_tuple(neighbor), Cell::spawn_dead(spawn_position.add_tuple(neighbor)));
                }
            }
        }
    }
}


fn rules(
    mut query: Query<(&mut Cell, &MeshMaterial2d<ColorMaterial>)>,
    mut cell_map: ResMut<CellMap>,
) {
    println!("--------------------neighbor check-------------------");
    for (mut cell, _) in query.iter_mut() {
        cell.live_neighbors = 0.0;
        for neighbor_position in FULL_NEIGHBORHOOD {
            if neighbor_position != (0, 0) {
                match cell_map.0.get_mut(&(cell.position.add_tuple(neighbor_position))) {
                    Some(mut neighbor_cell) => {

                        cell.live_neighbors += neighbor_cell.mortal_state;
                    },
                    _ => {}
                }
            }
        }
        println!("neighors: {}", cell.live_neighbors);
    }
}

fn smooth_update (
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut query: Query<(&mut Cell, &MeshMaterial2d<ColorMaterial>)>,
    mut cell_map: ResMut<CellMap>,
) {
    println!("-------------update-----------------------");
    for (mut cell, handle) in query.iter_mut() {
        println!("live_neighbors: {}; {}", cell.live_neighbors, cell.live_neighbors/8.0);
        let mortal_state: f32 = cell.live_neighbors/8.0;

        if mortal_state < 0.15 && mortal_state > 0.07 {
            let growth: f32 = mortal_state+mortal_state*0.1;
            if let Some(mut material) = materials.get_mut(handle) {
                material.color = Color::srgba(1.0, 0.0, 0.0, growth);
            }
            cell.mortal_state = growth;
            cell_map.update_mortal(cell.position, cell.mortal_state);
        } else if mortal_state > 0.15 {
            let growth: f32 = mortal_state-mortal_state*0.3;
            if let Some(mut material) = materials.get_mut(handle) {
                material.color = Color::srgba(-1.0, 0.0, 0.0, growth);
            }
            cell.mortal_state = growth;
            cell_map.update_mortal(cell.position, cell.mortal_state);
        } else {
            if let Some(mut material) = materials.get_mut(handle) {
                material.color = Color::srgba(1.0, 1.0, 1.0, mortal_state);
            }
            cell.mortal_state = mortal_state;
            cell_map.update_mortal(cell.position, cell.mortal_state);
        }


    }
    let ten_millis = time::Duration::from_millis(60);
    thread::sleep(ten_millis);
}

fn update(
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut query: Query<(&mut Cell, &MeshMaterial2d<ColorMaterial>)>,
    mut cell_map: ResMut<CellMap>,
) {
    for (mut cell, handle) in query.iter_mut() {
        println!("live_neighbors: {}", cell.live_neighbors);
        if cell.live_neighbors < 4.0 && cell.live_neighbors > 1.0 && cell.is_alive() {
            if let Some(mut material) = materials.get_mut(handle) {
                material.color = Color::from(ALIVE)
            }
            cell.mortal_state = 1.0;
            cell_map.update_mortal(cell.position, cell.mortal_state);
        } else if cell.live_neighbors == 3.0 && cell.is_dead() {
            if let Some(mut material) = materials.get_mut(handle) {
                material.color = Color::from(ALIVE)
            }
            cell.mortal_state = 1.0;
            cell_map.update_mortal(cell.position, cell.mortal_state);
        } else {
            if let Some(mut material) = materials.get_mut(handle) {
                material.color = Color::from(INVISIBLE)
            }
            cell.mortal_state = 0.0;
            cell_map.update_mortal(cell.position, cell.mortal_state);
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