use super::a_star::Astar;
use super::navigation_mesh::NavigationMesh;
use crate::Vector3;
use std::sync::{Arc, Weak};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WalkBotStatus {
    Idle,
    Pathing,
    Finished,
}

/// Steers towards a target over a [`NavigationMesh`], which it only holds weakly, reporting through closures.
pub struct WalkBot<'a> {
    nav_mesh: Weak<NavigationMesh>,
    on_next_path_node: Option<Box<dyn FnMut(Vector3<f32>) + 'a>>,
    on_status_update: Option<Box<dyn FnMut(WalkBotStatus) + 'a>>,
    last_visited: Option<Vector3<f32>>,
    target: Option<Vector3<f32>>,
    min_node_distance: f32,
}

impl Default for WalkBot<'_> {
    fn default() -> Self {
        Self {
            nav_mesh: Weak::new(),
            on_next_path_node: None,
            on_status_update: None,
            last_visited: None,
            target: None,
            min_node_distance: 1.0,
        }
    }
}

impl<'a> WalkBot<'a> {
    /// The C++ default for `min_node_distance`, which [`WalkBot::default`] also uses, is `1.0`.
    pub fn new(mesh: &Arc<NavigationMesh>, min_node_distance: f32) -> Self {
        Self { nav_mesh: Arc::downgrade(mesh), min_node_distance, ..Self::default() }
    }

    pub fn set_nav_mesh(&mut self, mesh: &Arc<NavigationMesh>) {
        self.nav_mesh = Arc::downgrade(mesh);
    }

    pub fn set_min_node_distance(&mut self, distance: f32) {
        self.min_node_distance = distance;
    }

    pub fn set_target(&mut self, target: Vector3<f32>) {
        self.target = Some(target);
    }

    /// Clears navigation state so the bot can be re-routed without stale visited-node memory.
    pub fn reset(&mut self) {
        self.last_visited = None;
    }

    /// Call every game tick with the current bot world position.
    pub fn update(&mut self, bot_position: Vector3<f32>) {
        let Some(target) = self.target else {
            return;
        };

        if target.distance_to(bot_position) <= self.min_node_distance {
            self.report_status(WalkBotStatus::Finished);
            return;
        }

        if self.on_next_path_node.is_none() {
            return;
        }

        let Some(nav_mesh) = self.nav_mesh.upgrade() else {
            self.report_status(WalkBotStatus::Idle);
            return;
        };

        let path = Astar::find_path(bot_position, target, &nav_mesh);
        let Some(&nearest) = path.first() else {
            self.report_status(WalkBotStatus::Idle);
            return;
        };

        // Record the nearest node as visited once we are close enough to it.
        if nearest.distance_to(bot_position) <= self.min_node_distance {
            self.last_visited = Some(nearest);
        }

        // If the nearest node was already visited, advance to the next one so we never oscillate back to a node we
        // just left. If the bot was displaced (blown back), nearest will be an unvisited node, so we route to it first
        // before continuing forward.
        let next_node = match path.get(1) {
            Some(&next) if self.last_visited == Some(nearest) => next,
            _ => nearest,
        };

        if let Some(on_next_path_node) = &mut self.on_next_path_node {
            on_next_path_node(next_node);
        }

        self.report_status(WalkBotStatus::Pathing);
    }

    pub fn on_path(&mut self, callback: impl FnMut(Vector3<f32>) + 'a) {
        self.on_next_path_node = Some(Box::new(callback));
    }

    pub fn on_status(&mut self, callback: impl FnMut(WalkBotStatus) + 'a) {
        self.on_status_update = Some(Box::new(callback));
    }

    fn report_status(&mut self, status: WalkBotStatus) {
        if let Some(on_status_update) = &mut self.on_status_update {
            on_status_update(status);
        }
    }
}
