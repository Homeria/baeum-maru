use super::*;

fn input() -> RoomInput<'static> {
    RoomInput {
        name: "다목적실",
        building: "본관",
        floor: Some(3),
    }
}

#[test]
fn name_trims_spaces() {
    assert_eq!(RoomName::parse(" 다목적실 ").unwrap().as_str(), "다목적실");
}

#[test]
fn name_rejects_empty_and_too_long() {
    assert_eq!(RoomName::parse("  "), Err(RoomError::EmptyName));
    assert!(RoomName::parse(&"가".repeat(30)).is_ok());
    assert_eq!(
        RoomName::parse(&"가".repeat(31)),
        Err(RoomError::NameTooLong)
    );
}

#[test]
fn br_r1_building_is_required() {
    assert_eq!(BuildingName::parse(""), Err(RoomError::EmptyBuilding));
    assert_eq!(BuildingName::parse("   "), Err(RoomError::EmptyBuilding));
}

#[test]
fn building_trims_and_limits_length() {
    assert_eq!(BuildingName::parse(" 별관 ").unwrap().as_str(), "별관");
    assert!(BuildingName::parse(&"가".repeat(20)).is_ok());
    assert_eq!(
        BuildingName::parse(&"가".repeat(21)),
        Err(RoomError::BuildingTooLong)
    );
}

#[test]
fn floor_accepts_basement_2_to_5() {
    for ok in [-2, -1, 1, 2, 3, 4, 5] {
        assert_eq!(Floor::new(ok).unwrap().value(), i8::try_from(ok).unwrap());
    }
}

#[test]
fn floor_rejects_zero_and_out_of_range() {
    for bad in [0, -3, 6, 300, i32::MIN] {
        assert_eq!(Floor::new(bad), Err(RoomError::InvalidFloor), "{bad}");
    }
}

#[test]
fn floors_sort_from_basement_up() {
    let mut floors = [3, -1, 1, -2].map(|n| Floor::new(n).unwrap());
    floors.sort();

    assert_eq!(floors.map(Floor::value), [-2, -1, 1, 3]);
}

#[test]
fn br_r1_allows_room_without_floor() {
    let room = NewRoom::parse(RoomInput {
        floor: None,
        ..input()
    })
    .unwrap();

    assert_eq!(room.name().as_str(), "다목적실");
    assert_eq!(room.building().as_str(), "본관");
    assert_eq!(room.floor(), None);
}

#[test]
fn parse_keeps_floor() {
    let room = NewRoom::parse(input()).unwrap();

    assert_eq!(room.floor().map(Floor::value), Some(3));
}

#[test]
fn collects_every_error_in_field_order() {
    let errors = NewRoom::parse(RoomInput {
        name: "",
        building: " ",
        floor: Some(0),
    })
    .unwrap_err();

    assert_eq!(
        errors,
        [
            RoomError::EmptyName,
            RoomError::EmptyBuilding,
            RoomError::InvalidFloor
        ]
    );
}
