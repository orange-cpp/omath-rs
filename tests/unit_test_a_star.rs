//! Extra unit tests for the project's A* implementation.

use omath::Vector3;
use omath::pathfinding::{Astar, NavigationMesh};

fn v(x: f32, y: f32, z: f32) -> Vector3<f32> {
    Vector3::new(x, y, z)
}

/// 0 -> 1 -> 2 -> ... -> length-1 (directed)
fn make_linear_chain(length: i32) -> NavigationMesh {
    let mut nav = NavigationMesh::new();
    for i in 0..length {
        let neighbors = if i + 1 < length {
            vec![v((i + 1) as f32, 0.0, 0.0)]
        } else {
            vec![]
        };
        nav.set_neighbors(v(i as f32, 0.0, 0.0), neighbors);
    }
    nav
}

// Basic reachability

#[test]
fn trivial_neighbor() {
    let mut nav = NavigationMesh::new();
    let v1 = v(0.0, 0.0, 0.0);
    let v2 = v(1.0, 0.0, 0.0);
    nav.set_neighbors(v1, vec![v2]);
    nav.set_neighbors(v2, vec![v1]);

    let path = Astar::find_path(v1, v2, &nav);
    assert_eq!(path.len(), 2);
    assert_eq!(path.first(), Some(&v1));
    assert_eq!(path.last(), Some(&v2));
}

#[test]
fn start_equals_goal() {
    let mut nav = NavigationMesh::new();
    let vertex = v(1.0, 1.0, 0.0);
    nav.set_neighbors(vertex, vec![]);

    let path = Astar::find_path(vertex, vertex, &nav);
    assert_eq!(path.len(), 1);
    assert_eq!(path.first(), Some(&vertex));
}

#[test]
fn blocked_no_path_between_two_vertices() {
    let mut nav = NavigationMesh::new();
    let left = v(0.0, 0.0, 0.0);
    let right = v(2.0, 0.0, 0.0);
    // both vertices present but no connections
    nav.set_neighbors(left, vec![]);
    nav.set_neighbors(right, vec![]);

    let path = Astar::find_path(left, right, &nav);
    // disconnected vertices -> empty result
    assert!(path.is_empty());
}

#[test]
fn longer_path_avoids_block() {
    let mut nav = NavigationMesh::new();
    // build 3x3 grid of vertices, block center (1,1)
    let idx = |x: i32, y: i32| v(x as f32, y as f32, 0.0);
    for y in 0..3 {
        for x in 0..3 {
            if x == 1 && y == 1 {
                continue; // center is omitted (blocked)
            }
            let mut neigh = Vec::new();
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, ny) = (x + dx, y + dy);
                if !(0..3).contains(&nx) || !(0..3).contains(&ny) {
                    continue;
                }
                if nx == 1 && ny == 1 {
                    continue; // neighbor is the blocked center
                }
                neigh.push(idx(nx, ny));
            }
            nav.set_neighbors(idx(x, y), neigh);
        }
    }

    let start = idx(0, 1);
    let goal = idx(2, 1);
    let path = Astar::find_path(start, goal, &nav);
    assert!(!path.is_empty());
    assert_eq!(path.last(), Some(&goal));
}

#[test]
fn trivial_direct_neighbor_path() {
    let mut nav = NavigationMesh::new();
    // create two vertices directly connected
    let v1 = v(0.0, 0.0, 0.0);
    let v2 = v(1.0, 0.0, 0.0);
    nav.set_neighbors(v1, vec![v2]);
    nav.set_neighbors(v2, vec![v1]);

    let path = Astar::find_path(v1, v2, &nav);
    assert_eq!(path.len(), 2);
    assert_eq!(path.first(), Some(&v1));
    assert_eq!(path.last(), Some(&v2));
}

#[test]
fn no_path_when_disconnected() {
    let mut nav = NavigationMesh::new();
    let v1 = v(0.0, 0.0, 0.0);
    let v2 = v(10.0, 0.0, 0.0);
    // nav has only v1
    nav.set_neighbors(v1, vec![]);

    let path = Astar::find_path(v1, v2, &nav);
    // When the nav mesh contains only the start vertex, the closest vertex for both start and end will be the same
    // vertex. In that case Astar returns a single-element path with the start vertex.
    assert_eq!(path.len(), 1);
    assert_eq!(path.first(), Some(&v1));
}

#[test]
fn empty_nav_returns_no_path() {
    let nav = NavigationMesh::new();

    let path = Astar::find_path(v(0.0, 0.0, 0.0), v(1.0, 0.0, 0.0), &nav);
    assert!(path.is_empty());
}

#[test]
fn finding_right_path() {
    let mut mesh = NavigationMesh::new();

    mesh.set_neighbors(v(0.0, 0.0, 0.0), vec![v(0.0, 1.0, 0.0)]);
    mesh.set_neighbors(v(0.0, 1.0, 0.0), vec![v(0.0, 2.0, 0.0)]);
    mesh.set_neighbors(v(0.0, 2.0, 0.0), vec![v(0.0, 3.0, 0.0)]);
    mesh.set_neighbors(v(0.0, 3.0, 0.0), vec![]);
    let _ = Astar::find_path(Vector3::default(), v(0.0, 3.0, 0.0), &mesh);
}

// Directed edges

#[test]
fn directed_edge_forward_path_exists() {
    // A -> B only; path from A to B should succeed
    let mut nav = NavigationMesh::new();
    let a = v(0.0, 0.0, 0.0);
    let b = v(1.0, 0.0, 0.0);
    nav.set_neighbors(a, vec![b]);
    nav.set_neighbors(b, vec![]); // no edge back

    let path = Astar::find_path(a, b, &nav);
    assert!(!path.is_empty());
    assert_eq!(path.last(), Some(&b));
}

#[test]
fn directed_edge_reverse_path_missing() {
    // A -> B only; path from B to A should fail
    let mut nav = NavigationMesh::new();
    let a = v(0.0, 0.0, 0.0);
    let b = v(1.0, 0.0, 0.0);
    nav.set_neighbors(a, vec![b]);
    nav.set_neighbors(b, vec![]);

    let path = Astar::find_path(b, a, &nav);
    assert!(path.is_empty());
}

// Vertex snapping

#[test]
fn off_mesh_start_snaps_to_nearest_vertex() {
    let mut nav = NavigationMesh::new();
    let v1 = v(0.0, 0.0, 0.0);
    let v2 = v(10.0, 0.0, 0.0);
    nav.set_neighbors(v1, vec![v2]);
    nav.set_neighbors(v2, vec![v1]);

    // Start is slightly off v1 but closer to it than to v2
    let off_start = v(0.1, 0.0, 0.0);
    let path = Astar::find_path(off_start, v2, &nav);
    assert!(!path.is_empty());
    assert_eq!(path.last(), Some(&v2));
}

#[test]
fn off_mesh_end_snaps_to_nearest_vertex() {
    let mut nav = NavigationMesh::new();
    let v1 = v(0.0, 0.0, 0.0);
    let v2 = v(10.0, 0.0, 0.0);
    nav.set_neighbors(v1, vec![v2]);
    nav.set_neighbors(v2, vec![v1]);

    // Goal is slightly off v2 but closer to it than to v1
    let off_goal = v(9.9, 0.0, 0.0);
    let path = Astar::find_path(v1, off_goal, &nav);
    assert!(!path.is_empty());
    assert_eq!(path.last(), Some(&v2));
}

// Cycle handling

#[test]
fn cyclic_graph_finds_path_without_looping() {
    // Triangle: A <-> B <-> C <-> A
    let mut nav = NavigationMesh::new();
    let a = v(0.0, 0.0, 0.0);
    let b = v(1.0, 0.0, 0.0);
    let c = v(0.5, 1.0, 0.0);
    nav.set_neighbors(a, vec![b, c]);
    nav.set_neighbors(b, vec![a, c]);
    nav.set_neighbors(c, vec![a, b]);

    let path = Astar::find_path(a, c, &nav);
    assert!(!path.is_empty());
    assert_eq!(path.last(), Some(&c));
}

#[test]
fn self_loop_vertex_does_not_break_search() {
    // Vertex with itself as a neighbor
    let mut nav = NavigationMesh::new();
    let a = v(0.0, 0.0, 0.0);
    let b = v(1.0, 0.0, 0.0);
    nav.set_neighbors(a, vec![a, b]); // self-loop on a
    nav.set_neighbors(b, vec![a]);

    let path = Astar::find_path(a, b, &nav);
    assert!(!path.is_empty());
    assert_eq!(path.last(), Some(&b));
}

// Longer chains

#[test]
fn linear_chain_reaches_end() {
    const K_LENGTH: i32 = 10;
    let nav = make_linear_chain(K_LENGTH);

    let start = v(0.0, 0.0, 0.0);
    let goal = v((K_LENGTH - 1) as f32, 0.0, 0.0);

    let path = Astar::find_path(start, goal, &nav);
    assert!(!path.is_empty());
    assert_eq!(path.last(), Some(&goal));
}

#[test]
fn linear_chain_midpoint_reachable() {
    let nav = make_linear_chain(6);

    let start = v(0.0, 0.0, 0.0);
    let mid = v(3.0, 0.0, 0.0);

    let path = Astar::find_path(start, mid, &nav);
    assert!(!path.is_empty());
    assert_eq!(path.last(), Some(&mid));
}

// Serialize -> pathfind integration

#[test]
fn pathfind_after_serialize_deserialize() {
    let mut nav = NavigationMesh::new();
    let a = v(0.0, 0.0, 0.0);
    let b = v(1.0, 0.0, 0.0);
    let c = v(2.0, 0.0, 0.0);
    nav.set_neighbors(a, vec![b]);
    nav.set_neighbors(b, vec![a, c]);
    nav.set_neighbors(c, vec![b]);

    let mut nav2 = NavigationMesh::new();
    nav2.deserialize(&nav.serialize()).unwrap();

    let path = Astar::find_path(a, c, &nav2);
    assert!(!path.is_empty());
    assert_eq!(path.last(), Some(&c));
}
