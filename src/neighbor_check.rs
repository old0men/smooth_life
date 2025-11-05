use bevy::math::IVec2;
use bevy::prelude::{Query, ResMut};
use crate::{Cell, CellMap, FULL_NEIGHBORHOOD};

fn neighborhood_check(
    mut query: Query<&mut Cell>,
    mut cell_map: ResMut<CellMap>,
) {

    let mut cell_count = 0;
    let mut alive = 0;
    let mut dead = 0;

    for mut cell in query.iter_mut() {
        cell.live_neighbors = 0.0;

        for neighbor in FULL_NEIGHBORHOOD {

            let key_value = IVec2::new(cell.position.x + neighbor.0, cell.position.y + neighbor.1);
            match cell_map.0.get(&key_value) {
                Some(neighbor_cell) => {
                    if neighbor == (0, 0) {
                        cell.mortal_state = neighbor_cell.mortal_state;
                        println!("neighbor_check: {:?}##################################################################", cell);
                        cell_map.0.remove(&cell.position);
                        cell_map.0.insert(cell.position, Cell::new(cell.mortal_state, cell.position, cell.live_neighbors));
                    } else {
                        cell.live_neighbors += neighbor_cell.mortal_state
                    }
                },
                None => {

                }
            }
        }

        cell_count += 1;
        if cell.mortal_state == 0.0{
            dead += 1;
        } else {
            alive += 1;
        }
    }
    println!("cell count: {}", cell_count);
    println!("alive: {}", alive);
    println!("dead: {}", dead);
}