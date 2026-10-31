//! Behaviour of `NavigationMesh` and `Astar` that the ported C++ tests leave open. The serialized text pinned here is
//! what `std::ostream` writes for these numbers.

use omath::Vector3;
use omath::pathfinding::{Astar, Error, NavigationMesh};

fn v(x: f32, y: f32, z: f32) -> Vector3<f32> {
    Vector3::new(x, y, z)
}

fn serialized_vertex(x: f32, y: f32, z: f32) -> String {
    let mut nav = NavigationMesh::new();
    nav.set_neighbors(v(x, y, z), vec![]);
    nav.serialize()
}

#[test]
fn serialize_writes_the_cpp_line_format() {
    let mut nav = NavigationMesh::new();
    let a = v(1.0, 2.5, -3.0);
    nav.set_neighbors(a, vec![v(4.0, 0.25, 0.0), v(-0.0, 7.0, 8.0)]);
    nav.set_event(a, "jump").unwrap();

    assert_eq!(nav.serialize(), "1 2.5 -3 2 jump\n4 0.25 0\n-0 7 8\n");
    assert_eq!(serialized_vertex(1.0, 2.0, 3.0), "1 2 3 0 -\n");
}

#[test]
fn serialize_writes_numbers_the_way_ostream_does() {
    assert_eq!(serialized_vertex(0.1, 100000.0, 999999.0), "0.1 100000 999999 0 -\n");
    assert_eq!(serialized_vertex(1000000.0, 999999.5, 1234567.0), "1e+06 1e+06 1.23457e+06 0 -\n");
    assert_eq!(serialized_vertex(0.0001, 0.00001, -0.000012345), "0.0001 1e-05 -1.2345e-05 0 -\n");
    assert_eq!(serialized_vertex(3.1415927, 1.5e30, 1e-30), "3.14159 1.5e+30 1e-30 0 -\n");
    assert_eq!(serialized_vertex(f32::MAX, f32::INFINITY, -f32::INFINITY), "3.40282e+38 inf -inf 0 -\n");
}

#[test]
fn deserialize_reads_cpp_output() {
    let mut nav = NavigationMesh::new();
    nav.deserialize("1e+06 2.5 -3 1 jump\n0.1 1.5e+30 -0\n").unwrap();

    let vertex = v(1e6, 2.5, -3.0);
    assert_eq!(nav.get_neighbors(vertex), Some(&[v(0.1, 1.5e30, 0.0)][..]));
    assert_eq!(nav.get_event(vertex), Some("jump"));
}

#[test]
fn deserialize_fails_on_missing_neighbors() {
    let mut nav = NavigationMesh::new();

    assert_eq!(nav.deserialize("1 2 3 2 -\n4 5 6\n7 8\n"), Err(Error::UnexpectedEndOfData));
    assert_eq!(nav.deserialize("1 2 3 18446744073709551615 -\n"), Err(Error::UnexpectedEndOfData));
}

#[test]
fn deserialize_stops_at_a_vertex_line_it_cannot_read() {
    let mut nav = NavigationMesh::new();

    // Stream extraction does not read inf, and a number out of float range fails it too
    nav.deserialize("1 2 3 0 -\ninf 0 0 0 -\n7 8 9 0 -\n").unwrap();
    assert_eq!(nav.vertex_count(), 1);

    nav.deserialize("1 2 3 0 -\n1e50 0 0 0 -\n").unwrap();
    assert_eq!(nav.vertex_count(), 1);
    assert!(nav.contains_vertex(v(1.0, 2.0, 3.0)));
}

#[test]
fn neighbor_outside_the_mesh_is_a_dead_end() {
    let mut nav = NavigationMesh::new();
    let a = v(0.0, 0.0, 0.0);
    let b = v(1.0, 0.0, 0.0);
    let goal = v(5.0, 0.0, 0.0);
    nav.set_neighbors(a, vec![b]);
    nav.set_neighbors(goal, vec![]);

    assert_eq!(nav.get_neighbors(b), None);
    assert!(Astar::find_path(a, goal, &nav).is_empty());
}
