mod common;

use common::expect_near;
use omath::Vector3;
use omath::pathfinding::{Error, NavigationMesh};

fn v(x: f32, y: f32, z: f32) -> Vector3<f32> {
    Vector3::new(x, y, z)
}

fn round_trip(nav: &NavigationMesh) -> NavigationMesh {
    let mut nav2 = NavigationMesh::new();
    nav2.deserialize(&nav.serialize()).unwrap();
    nav2
}

#[test]
fn serialize_deserialize_round_trip() {
    let mut nav = NavigationMesh::new();
    let a = v(0.0, 0.0, 0.0);
    let b = v(1.0, 0.0, 0.0);
    let c = v(0.0, 1.0, 0.0);

    nav.set_neighbors(a, vec![b, c]);
    nav.set_neighbors(b, vec![a]);
    nav.set_neighbors(c, vec![a]);

    let data = nav.serialize();
    let mut nav2 = NavigationMesh::new();
    assert!(nav2.deserialize(&data).is_ok());

    assert_eq!(nav2.vertex_count(), nav.vertex_count());
    assert_eq!(nav2.get_neighbors(a).unwrap().len(), 2);
}

#[test]
fn get_closest_vertex_when_empty() {
    let nav = NavigationMesh::new();
    let res = nav.get_closest_vertex(v(5.0, 5.0, 5.0));
    assert!(res.is_err());
}

#[test]
fn serialize_empty_mesh() {
    let nav = NavigationMesh::new();
    assert!(nav.serialize().is_empty());
}

#[test]
fn deserialize_empty_string() {
    let mut nav = NavigationMesh::new();
    assert!(nav.deserialize("").is_ok());
    assert!(nav.empty());
}

#[test]
fn serialize_produces_human_readable_text() {
    let mut nav = NavigationMesh::new();
    nav.set_neighbors(v(1.0, 2.0, 3.0), vec![v(4.0, 5.0, 6.0)]);

    let data = nav.serialize();

    // Must contain the vertex and neighbor coords as plain text
    for digit in ["1", "2", "3", "4", "5", "6"] {
        assert!(data.contains(digit));
    }
}

#[test]
fn deserialize_restores_neighbor_values() {
    let mut nav = NavigationMesh::new();
    let vertex = v(1.0, 2.0, 3.0);
    let n1 = v(4.0, 5.0, 6.0);
    let n2 = v(7.0, 8.0, 9.0);
    nav.set_neighbors(vertex, vec![n1, n2]);

    let nav2 = round_trip(&nav);

    assert!(nav2.contains_vertex(vertex));
    let neighbors = nav2.get_neighbors(vertex).unwrap();
    assert_eq!(neighbors.len(), 2);
    assert_eq!(neighbors[0], n1);
    assert_eq!(neighbors[1], n2);
}

#[test]
fn deserialize_overwrites_previous_data() {
    let mut nav = NavigationMesh::new();
    let vertex = v(1.0, 0.0, 0.0);
    nav.set_neighbors(vertex, vec![]);

    // Load a different mesh into the same object
    let mut other = NavigationMesh::new();
    let a = v(10.0, 20.0, 30.0);
    other.set_neighbors(a, vec![]);

    nav.deserialize(&other.serialize()).unwrap();

    assert_eq!(nav.vertex_count(), 1);
    assert!(!nav.contains_vertex(vertex));
    assert!(nav.contains_vertex(a));
}

#[test]
fn round_trip_negative_and_fractional_coords() {
    let mut nav = NavigationMesh::new();
    let vertex = v(-1.5, 0.25, -3.75);
    let n = v(100.0, -200.0, 0.001);
    nav.set_neighbors(vertex, vec![n]);

    let nav2 = round_trip(&nav);

    assert!(nav2.contains_vertex(vertex));
    let neighbors = nav2.get_neighbors(vertex).unwrap();
    assert_eq!(neighbors.len(), 1);
    expect_near(neighbors[0].x, n.x, 1e-3_f32);
    expect_near(neighbors[0].y, n.y, 1e-3_f32);
    expect_near(neighbors[0].z, n.z, 1e-3_f32);
}

#[test]
fn get_closest_vertex_returns_nearest() {
    let mut nav = NavigationMesh::new();
    let a = v(0.0, 0.0, 0.0);
    let b = v(10.0, 0.0, 0.0);
    nav.set_neighbors(a, vec![]);
    nav.set_neighbors(b, vec![]);

    assert_eq!(nav.get_closest_vertex(v(1.0, 0.0, 0.0)), Ok(a));
}

#[test]
fn vertex_with_no_neighbors_round_trip() {
    let mut nav = NavigationMesh::new();
    let vertex = v(5.0, 5.0, 5.0);
    nav.set_neighbors(vertex, vec![]);

    let nav2 = round_trip(&nav);

    assert!(nav2.contains_vertex(vertex));
    assert!(nav2.get_neighbors(vertex).unwrap().is_empty());
}

// Vertex events

#[test]
fn set_event_on_non_existent_vertex_throws() {
    let mut nav = NavigationMesh::new();
    let vertex = v(99.0, 99.0, 99.0);
    assert_eq!(
        nav.set_event(vertex, "jump"),
        Err(Error::VertexNotFound(vertex))
    );
}

#[test]
fn event_not_set_by_default() {
    let mut nav = NavigationMesh::new();
    let vertex = v(0.0, 0.0, 0.0);
    nav.set_neighbors(vertex, vec![]);

    assert_eq!(nav.get_event(vertex), None);
}

#[test]
fn set_and_get_event() {
    let mut nav = NavigationMesh::new();
    let vertex = v(1.0, 0.0, 0.0);
    nav.set_neighbors(vertex, vec![]);
    nav.set_event(vertex, "jump").unwrap();

    assert_eq!(nav.get_event(vertex), Some("jump"));
}

#[test]
fn overwrite_event() {
    let mut nav = NavigationMesh::new();
    let vertex = v(1.0, 0.0, 0.0);
    nav.set_neighbors(vertex, vec![]);
    nav.set_event(vertex, "jump").unwrap();
    nav.set_event(vertex, "teleport").unwrap();

    assert_eq!(nav.get_event(vertex), Some("teleport"));
}

#[test]
fn clear_event() {
    let mut nav = NavigationMesh::new();
    let vertex = v(1.0, 0.0, 0.0);
    nav.set_neighbors(vertex, vec![]);
    nav.set_event(vertex, "jump").unwrap();
    nav.clear_event(vertex);

    assert_eq!(nav.get_event(vertex), None);
}

#[test]
fn event_round_trip_serialization() {
    let mut nav = NavigationMesh::new();
    let a = v(0.0, 0.0, 0.0);
    let b = v(1.0, 0.0, 0.0);
    nav.set_neighbors(a, vec![b]);
    nav.set_neighbors(b, vec![]);
    nav.set_event(b, "jump").unwrap();

    let nav2 = round_trip(&nav);

    assert_eq!(nav2.get_event(a), None);
    assert_eq!(nav2.get_event(b), Some("jump"));
}

#[test]
fn multiple_events_round_trip() {
    let mut nav = NavigationMesh::new();
    let a = v(0.0, 0.0, 0.0);
    let b = v(1.0, 0.0, 0.0);
    let c = v(2.0, 0.0, 0.0);
    nav.set_neighbors(a, vec![]);
    nav.set_neighbors(b, vec![]);
    nav.set_neighbors(c, vec![]);
    nav.set_event(a, "spawn").unwrap();
    nav.set_event(c, "teleport").unwrap();

    let nav2 = round_trip(&nav);

    assert_eq!(nav2.get_event(a), Some("spawn"));
    assert_eq!(nav2.get_event(b), None);
    assert_eq!(nav2.get_event(c), Some("teleport"));
}

#[test]
fn deserialize_clears_old_events() {
    let mut nav = NavigationMesh::new();
    let vertex = v(0.0, 0.0, 0.0);
    nav.set_neighbors(vertex, vec![]);
    nav.set_event(vertex, "jump").unwrap();

    // Deserialize a mesh that has no events
    let mut empty_events = NavigationMesh::new();
    empty_events.set_neighbors(vertex, vec![]);

    nav.deserialize(&empty_events.serialize()).unwrap();
    assert_eq!(nav.get_event(vertex), None);
}
