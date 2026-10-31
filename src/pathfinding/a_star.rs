use super::navigation_mesh::{NavigationMesh, VertexKey};
use crate::Vector3;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

#[derive(Clone, Copy)]
struct OpenListNode {
    position: Vector3<f32>,
    f_cost: f32,
}

impl PartialEq for OpenListNode {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for OpenListNode {}

impl PartialOrd for OpenListNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OpenListNode {
    /// Reversed: [`BinaryHeap`] pops its largest element, and the open list has to pop the cheapest.
    fn cmp(&self, other: &Self) -> Ordering {
        other.f_cost.total_cmp(&self.f_cost)
    }
}

#[derive(Clone, Copy)]
struct PathNode {
    came_from: Option<Vector3<f32>>,
    g_cost: f32,
}

/// A* over a [`NavigationMesh`].
pub struct Astar;

impl Astar {
    /// The shortest path between the vertices closest to `start` and `end`, both included. Empty when the mesh is
    /// empty or no path exists.
    ///
    /// A neighbor that is not itself a vertex of the mesh counts as a dead end.
    #[must_use]
    pub fn find_path(start: Vector3<f32>, end: Vector3<f32>, nav_mesh: &NavigationMesh) -> Vec<Vector3<f32>> {
        let (Ok(start_vertex), Ok(end_vertex)) = (nav_mesh.get_closest_vertex(start), nav_mesh.get_closest_vertex(end))
        else {
            return Vec::new();
        };

        let mut closed_list = HashMap::<VertexKey, PathNode>::new();
        let mut node_data = HashMap::<VertexKey, PathNode>::new();
        let mut open_list = BinaryHeap::new();

        node_data.insert(start_vertex.into(), PathNode { came_from: None, g_cost: 0.0 });
        open_list.push(OpenListNode { position: start_vertex, f_cost: start_vertex.distance_to(end_vertex) });

        while let Some(OpenListNode { position: current, .. }) = open_list.pop() {
            let current_key = VertexKey::from(current);

            if closed_list.contains_key(&current_key) {
                continue;
            }

            let Some(&current_node) = node_data.get(&current_key) else {
                continue;
            };

            closed_list.insert(current_key, current_node);

            if current == end_vertex {
                return Self::reconstruct_final_path(&closed_list, current);
            }

            for &neighbor in nav_mesh.get_neighbors(current).unwrap_or_default() {
                let neighbor_key = VertexKey::from(neighbor);

                if closed_list.contains_key(&neighbor_key) {
                    continue;
                }

                let tentative_g_cost = current_node.g_cost + neighbor.distance_to(current);

                if node_data.get(&neighbor_key).is_none_or(|node| tentative_g_cost < node.g_cost) {
                    node_data.insert(neighbor_key, PathNode { came_from: Some(current), g_cost: tentative_g_cost });
                    open_list.push(OpenListNode {
                        position: neighbor,
                        f_cost: tentative_g_cost + neighbor.distance_to(end_vertex),
                    });
                }
            }
        }

        Vec::new()
    }

    fn reconstruct_final_path(closed_list: &HashMap<VertexKey, PathNode>, current: Vector3<f32>) -> Vec<Vector3<f32>> {
        let mut path = Vec::new();
        let mut next = Some(current);

        while let Some(vertex) = next {
            path.push(vertex);
            next = closed_list.get(&vertex.into()).and_then(|node| node.came_from);
        }

        path.reverse();
        path
    }
}
