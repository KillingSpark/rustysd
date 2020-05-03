#[test]
fn test_unit_ordering() {
    let target1_str = format!(
        "
    [Unit]
    Description = {}
    Before = {}
    
    [Install]
    RequiredBy = {}
    ",
        "Target", "2.target", "2.target",
    );

    let parsed_file = crate::units::parse_file(&target1_str).unwrap();
    let target1_unit =
        crate::units::parse_target(parsed_file, &std::path::PathBuf::from("/path/to/1.target"))
            .unwrap();

    let target2_str = format!(
        "
    [Unit]
    Description = {}
    After = {}

    [Install]
    RequiredBy = {}
    ",
        "Target", "1.target", "3.target",
    );

    let parsed_file = crate::units::parse_file(&target2_str).unwrap();
    let target2_unit =
        crate::units::parse_target(parsed_file, &std::path::PathBuf::from("/path/to/2.target"))
            .unwrap();

    let target3_str = format!(
        "
    [Unit]
    Description = {}
    After = {}
    After = {}
    
    ",
        "Target", "1.target", "2.target"
    );

    let parsed_file = crate::units::parse_file(&target3_str).unwrap();
    let target3_unit =
        crate::units::parse_target(parsed_file, &std::path::PathBuf::from("/path/to/3.target"))
            .unwrap();

    let mut unit_table = std::collections::HashMap::new();

    use crate::units::Unit;
    use std::convert::TryInto;
    let target1_unit: Unit = target1_unit.try_into().unwrap();
    let target2_unit: Unit = target2_unit.try_into().unwrap();
    let target3_unit: Unit = target3_unit.try_into().unwrap();
    let id1 = target1_unit.id.clone();
    let id2 = target2_unit.id.clone();
    let id3 = target3_unit.id.clone();

    unit_table.insert(target1_unit.id.clone(), target1_unit);
    unit_table.insert(target2_unit.id.clone(), target2_unit);
    unit_table.insert(target3_unit.id.clone(), target3_unit);

    crate::units::fill_dependencies(&mut unit_table).unwrap();
    unit_table
        .values_mut()
        .for_each(|unit| unit.dedup_dependencies());
    crate::units::sanity_check_dependencies(&unit_table).unwrap();

    unit_table
        .values()
        .for_each(|unit| println!("{} {:?}", unit.id, unit.common.dependencies));

    // before/after 1.target
    assert!(unit_table
        .get(&id1)
        .unwrap()
        .common
        .dependencies
        .after
        .is_empty());
    assert!(
        unit_table
            .get(&id1)
            .unwrap()
            .common
            .dependencies
            .before
            .len()
            == 2
    );
    assert!(unit_table
        .get(&id1)
        .unwrap()
        .common
        .dependencies
        .before
        .contains(&id2));
    assert!(unit_table
        .get(&id1)
        .unwrap()
        .common
        .dependencies
        .before
        .contains(&id3));

    // before/after 2.target
    assert_eq!(
        unit_table
            .get(&id2)
            .unwrap()
            .common
            .dependencies
            .before
            .len(),
        1
    );
    assert!(unit_table
        .get(&id2)
        .unwrap()
        .common
        .dependencies
        .before
        .contains(&id3));
    assert_eq!(
        unit_table
            .get(&id2)
            .unwrap()
            .common
            .dependencies
            .after
            .len(),
        1
    );
    assert!(unit_table
        .get(&id2)
        .unwrap()
        .common
        .dependencies
        .after
        .contains(&id1));

    // before/after 3.target
    assert!(unit_table
        .get(&id3)
        .unwrap()
        .common
        .dependencies
        .before
        .is_empty());
    assert!(
        unit_table
            .get(&id3)
            .unwrap()
            .common
            .dependencies
            .after
            .len()
            == 2
    );
    assert!(unit_table
        .get(&id3)
        .unwrap()
        .common
        .dependencies
        .after
        .contains(&id2));
    assert!(unit_table
        .get(&id3)
        .unwrap()
        .common
        .dependencies
        .after
        .contains(&id1));

    // Test the collection of start subgraphs
    // add a new unrelated unit, that should never occur in any of these operations for {1,2,3}.target
    let target4_str = format!(
        "
    [Unit]
    Description = {}
    
    ",
        "Target"
    );
    let parsed_file = crate::units::parse_file(&target4_str).unwrap();
    let target4_unit =
        crate::units::parse_target(parsed_file, &std::path::PathBuf::from("/path/to/4.target"))
            .unwrap();
    let target4_unit: Unit = target4_unit.try_into().unwrap();
    let id4 = target4_unit.id.clone();
    unit_table.insert(target4_unit.id.clone(), target4_unit);

    // 3.target needs all units
    let mut ids_to_start = vec![id3.clone()];
    crate::units::collect_unit_start_subgraph(&mut ids_to_start, &unit_table);
    ids_to_start.sort();
    assert_eq!(ids_to_start, vec![id1.clone(), id2.clone(), id3.clone()]);

    // 2.target needs 1 and 2
    let mut ids_to_start = vec![id2.clone()];
    crate::units::collect_unit_start_subgraph(&mut ids_to_start, &unit_table);
    ids_to_start.sort();
    assert_eq!(ids_to_start, vec![id1.clone(), id2.clone()]);

    // 1.target needs only 1
    let mut ids_to_start = vec![id1.clone()];
    crate::units::collect_unit_start_subgraph(&mut ids_to_start, &unit_table);
    ids_to_start.sort();
    assert_eq!(ids_to_start, vec![id1.clone()]);

    // 4.target needs only 4
    let mut ids_to_start = vec![id4.clone()];
    crate::units::collect_unit_start_subgraph(&mut ids_to_start, &unit_table);
    ids_to_start.sort();
    assert_eq!(ids_to_start, vec![id4.clone()]);
}

#[test]
fn test_circle() {
    let target1_str = format!(
        "
    [Unit]
    Description = {}
    After = {}
    ",
        "Target", "3.target"
    );

    let parsed_file = crate::units::parse_file(&target1_str).unwrap();
    let target1_unit =
        crate::units::parse_target(parsed_file, &std::path::PathBuf::from("/path/to/1.target"))
            .unwrap();

    let target2_str = format!(
        "
    [Unit]
    Description = {}
    After = {}
    ",
        "Target", "1.target"
    );

    let parsed_file = crate::units::parse_file(&target2_str).unwrap();
    let target2_unit =
        crate::units::parse_target(parsed_file, &std::path::PathBuf::from("/path/to/2.target"))
            .unwrap();

    let target3_str = format!(
        "
    [Unit]
    Description = {}
    After = {}
    ",
        "Target", "2.target"
    );

    let parsed_file = crate::units::parse_file(&target3_str).unwrap();
    let target3_unit =
        crate::units::parse_target(parsed_file, &std::path::PathBuf::from("/path/to/3.target"))
            .unwrap();

    use crate::units::Unit;
    use std::convert::TryInto;
    let mut unit_table = std::collections::HashMap::new();
    let target1_unit: Unit = target1_unit.try_into().unwrap();
    let target2_unit: Unit = target2_unit.try_into().unwrap();
    let target3_unit: Unit = target3_unit.try_into().unwrap();
    let target1_id = target1_unit.id.clone();
    let target2_id = target2_unit.id.clone();
    let target3_id = target3_unit.id.clone();
    unit_table.insert(target1_unit.id.clone(), target1_unit);
    unit_table.insert(target2_unit.id.clone(), target2_unit);
    unit_table.insert(target3_unit.id.clone(), target3_unit);

    crate::units::fill_dependencies(&mut unit_table).unwrap();
    unit_table
        .values_mut()
        .for_each(|unit| unit.dedup_dependencies());

    if let Err(crate::units::SanityCheckError::CirclesFound(circles)) =
        crate::units::sanity_check_dependencies(&unit_table)
    {
        if circles.len() == 1 {
            let circle = &circles[0];
            assert_eq!(circle.len(), 3);
            assert!(circle.contains(&target1_id));
            assert!(circle.contains(&target2_id));
            assert!(circle.contains(&target3_id));
        } else {
            panic!("more than one circle found but there is only one");
        }
    } else {
        panic!("No circle found but there is one");
    }
}
