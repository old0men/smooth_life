use bevy::asset::Assets;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::{check_screen, integer_position, CellMap, IVec2Extensions, SMALL_STAR, Cell};

fn spawn_cell(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut cell_map: ResMut<CellMap>,
    q_windows: Single<&Window, With<PrimaryWindow>>
) {
    if let Some(click_position) = q_windows.cursor_position() {
        let screen = check_screen(*q_windows);
        let mut spawn_position:IVec2 = integer_position(Vec2::new(click_position.x, click_position.y), Vec2::new(screen.width, screen.height));

        for neighbor in SMALL_STAR {
            match cell_map.0.get(&spawn_position.add_tuple(neighbor)) {
                Some(cell) => {
                    if cell.mortal_state == 0.0 && neighbor == (0, 0) {
                        cell_map.0.remove(&spawn_position.add_tuple(neighbor));
                        cell_map.0.insert(spawn_position, Cell::spawn_live(spawn_position));
                    }
                },
                None => {
                    if neighbor != (0, 0) {
                        // spawn neighbor with mortal = 0.0
                        cell_map.0.insert(spawn_position.add_tuple(neighbor), Cell::spawn_dead(spawn_position.add_tuple(neighbor)));
                    } else {
                        // spawn host with mortal = 1.0
                        cell_map.0.insert(spawn_position, Cell::spawn_live(spawn_position));
                    }
                }
            }
        }
    }
}