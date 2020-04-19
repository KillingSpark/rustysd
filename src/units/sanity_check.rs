use crate::units::*;
use std::collections::HashMap;

#[derive(Debug, Eq, PartialEq)]
pub enum SanityCheckError {
    Generic(String),
    CirclesFound(Vec<Vec<UnitId>>),
}

/// Currently only checks that the units form a DAG so the recursive startup sequence does not hang itself.
/// There might be more to be checked but this is probably the most essential one.
pub fn sanity_check_dependencies(
    unit_table: &HashMap<UnitId, Unit>,
) -> Result<(), SanityCheckError> {
    let mut root_ids = Vec::new();
    for unit in unit_table.values() {
        if unit.common.dependencies.after.len() == 0 {
            root_ids.push(unit.id.clone());
        }
    }
    // check whether there are cycles in the startup sequence
    let mut finished_ids = HashMap::new();
    let mut not_finished_ids: HashMap<_, _> =
        unit_table.keys().cloned().map(|id| (id, ())).collect();
    let mut circles = Vec::new();

    loop {
        //if no nodes left -> no cycles
        let root_id = if not_finished_ids.len() == 0 {
            break;
        } else {
            // find new node that has no incoming edges anymore
            let root_id = not_finished_ids
                .keys()
                .filter(|id| {
                    let unit = unit_table.get(id).unwrap();
                    let in_degree = unit.common.dependencies.after.iter().fold(0, |acc, id| {
                        if finished_ids.contains_key(id) {
                            acc
                        } else {
                            acc + 1
                        }
                    });
                    in_degree == 0
                })
                .nth(0);
            if let Some(id) = root_id {
                id.clone()
            } else {
                // make sensible error-message
                circles.push(not_finished_ids.keys().cloned().collect());
                break;
            }
        };

        // stores the current DFS path to detect cycles in the directed graph (only using "before" edges)
        let mut visited_ids = Vec::new();
        if let Err(SanityCheckError::CirclesFound(new_circles)) = search_backedge(
            &root_id,
            unit_table,
            &mut visited_ids,
            &mut finished_ids,
            &mut not_finished_ids,
        ) {
            circles.extend(new_circles)
        };
    }
    if circles.is_empty() {
        Ok(())
    } else {
        Err(SanityCheckError::CirclesFound(circles))
    }
}

fn search_backedge(
    id: &UnitId,
    unit_table: &HashMap<UnitId, Unit>,
    visited_ids: &mut Vec<UnitId>,
    finished_ids: &mut HashMap<UnitId, ()>,
    not_finished_ids: &mut HashMap<UnitId, ()>,
) -> Result<(), SanityCheckError> {
    if finished_ids.contains_key(id) {
        return Ok(());
    }

    if visited_ids.contains(id) {
        let mut circle_start_idx = 0;
        for _ in 0..visited_ids.len() {
            if visited_ids[circle_start_idx] == *id {
                break;
            }
            circle_start_idx += 1;
        }
        let circle_ids = visited_ids[circle_start_idx..].to_vec();
        for circleid in &circle_ids {
            finished_ids.insert(circleid.clone(), ());
            not_finished_ids.remove(circleid);
        }

        return Err(SanityCheckError::CirclesFound(vec![circle_ids]));
    }
    visited_ids.push(id.clone());

    let unit = unit_table.get(id).unwrap();
    for next_id in &unit.common.dependencies.before {
        let res = search_backedge(
            next_id,
            unit_table,
            visited_ids,
            finished_ids,
            not_finished_ids,
        );
        if res.is_err() {
            return res;
        }
    }
    visited_ids.pop();
    finished_ids.insert(id.clone(), ());
    not_finished_ids.remove(id);

    Ok(())
}
