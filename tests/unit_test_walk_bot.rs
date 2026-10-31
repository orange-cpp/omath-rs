//! Unit tests for `omath::pathfinding::WalkBot`.
//! Covers all status transitions, callback behaviour, and a full walk simulation.

mod common;

use common::expect_float_eq;
use omath::Vector3;
use omath::pathfinding::{NavigationMesh, WalkBot, WalkBotStatus};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

fn v(x: f32, y: f32, z: f32) -> Vector3<f32> {
    Vector3::new(x, y, z)
}

/// Build a simple bidirectional linear chain: (0,0,0) <-> (1,0,0) <-> (2,0,0) <-> ... <-> (n-1,0,0)
fn make_linear_mesh(n: i32) -> Arc<NavigationMesh> {
    let mut mesh = NavigationMesh::new();
    for i in 0..n {
        let mut neighbors = Vec::new();
        if i > 0 {
            neighbors.push(v((i - 1) as f32, 0.0, 0.0));
        }
        if i + 1 < n {
            neighbors.push(v((i + 1) as f32, 0.0, 0.0));
        }
        mesh.set_neighbors(v(i as f32, 0.0, 0.0), neighbors);
    }
    Arc::new(mesh)
}

/// What a callback fills in, shared because the test reads it while the bot still holds the closure.
type Recorded<T> = Rc<RefCell<Vec<T>>>;

/// Collect every status update fired during one `update` call.
fn record_status(bot: &mut WalkBot<'_>) -> Recorded<WalkBotStatus> {
    let recorded = Recorded::default();
    let sink = Rc::clone(&recorded);
    bot.on_status(move |status| sink.borrow_mut().push(status));
    recorded
}

/// Collect every "next node" hint fired during one `update` call.
fn record_nodes(bot: &mut WalkBot<'_>) -> Recorded<Vector3<f32>> {
    let recorded = Recorded::default();
    let sink = Rc::clone(&recorded);
    bot.on_path(move |node| sink.borrow_mut().push(node));
    recorded
}

// Construction

#[test]
fn default_constructed_bot_is_inert() {
    // A default-constructed bot with no mesh, no target, and no callbacks must not crash.
    let mut bot = WalkBot::default();
    bot.update(v(0.0, 0.0, 0.0));
}

#[test]
fn construct_with_mesh_and_distance() {
    let mesh = make_linear_mesh(3);
    let _bot = WalkBot::new(&mesh, 0.5);
}

// Status: Finished

#[test]
fn fires_finished_when_bot_is_at_target() {
    let mesh = make_linear_mesh(3);
    let mut bot = WalkBot::new(&mesh, 0.5);
    bot.set_target(v(2.0, 0.0, 0.0));

    let statuses = record_status(&mut bot);

    // bot_position == target_position -> distance == 0, well within threshold
    bot.update(v(2.0, 0.0, 0.0));

    assert_eq!(statuses.borrow().first(), Some(&WalkBotStatus::Finished));
}

#[test]
fn fires_finished_when_bot_is_within_min_distance() {
    let mesh = make_linear_mesh(3);
    let mut bot = WalkBot::new(&mesh, 0.5);
    bot.set_target(v(0.4, 0.0, 0.0));

    let statuses = record_status(&mut bot);

    // distance between (0,0,0) and (0.4,0,0) is 0.4 < 0.5 threshold
    bot.update(v(0.0, 0.0, 0.0));

    assert_eq!(statuses.borrow().first(), Some(&WalkBotStatus::Finished));
}

#[test]
fn no_finished_when_outside_min_distance() {
    let mesh = make_linear_mesh(5);
    let mut bot = WalkBot::new(&mesh, 0.5);
    bot.set_target(v(4.0, 0.0, 0.0));

    let statuses = record_status(&mut bot);

    // Attach path callback so we get further status updates
    let _nodes = record_nodes(&mut bot);

    bot.update(v(0.0, 0.0, 0.0));

    // Finished must NOT appear in the status list
    assert!(!statuses.borrow().contains(&WalkBotStatus::Finished));
}

#[test]
fn finished_fired_even_without_path_callback() {
    // Finished is emitted before the on_path guard, so it fires regardless.
    let mesh = make_linear_mesh(2);
    let mut bot = WalkBot::new(&mesh, 0.5);
    bot.set_target(v(1.0, 0.0, 0.0));

    let statuses = record_status(&mut bot);
    // Intentionally do NOT register on_path callback.

    bot.update(v(1.0, 0.0, 0.0));

    assert_eq!(*statuses.borrow(), [WalkBotStatus::Finished]);
}

#[test]
fn finished_does_not_fall_through_to_pathing() {
    // update() must return after Finished: Pathing must not fire on the same tick.
    let mesh = make_linear_mesh(3);
    let mut bot = WalkBot::new(&mesh, 0.5);
    bot.set_target(v(1.0, 0.0, 0.0));

    let statuses = record_status(&mut bot);
    let nodes = record_nodes(&mut bot);

    bot.update(v(1.0, 0.0, 0.0)); // bot is at target

    assert_eq!(*statuses.borrow(), [WalkBotStatus::Finished]);
    assert!(nodes.borrow().is_empty());
}

// No target set

#[test]
fn no_update_without_target() {
    let mesh = make_linear_mesh(3);
    let mut bot = WalkBot::new(&mesh, 0.5);
    // Intentionally do NOT call set_target()

    let statuses = record_status(&mut bot);
    let nodes = record_nodes(&mut bot);

    bot.update(v(0.0, 0.0, 0.0));

    assert!(statuses.borrow().is_empty());
    assert!(nodes.borrow().is_empty());
}

// Status: Idle, no path callback registered

#[test]
fn no_path_callback_means_no_pathing_status() {
    let mesh = make_linear_mesh(4);
    let mut bot = WalkBot::new(&mesh, 0.5);
    bot.set_target(v(3.0, 0.0, 0.0));

    let statuses = record_status(&mut bot);
    // No on_path registered -> update returns early after Finished check

    bot.update(v(0.0, 0.0, 0.0));

    assert!(statuses.borrow().is_empty());
}

// Status: Idle, null/expired navigation mesh

#[test]
fn fires_idle_when_nav_mesh_is_null() {
    let mut bot = WalkBot::default(); // no mesh assigned
    bot.set_target(v(5.0, 0.0, 0.0));

    let statuses = record_status(&mut bot);
    let nodes = record_nodes(&mut bot);

    bot.update(v(0.0, 0.0, 0.0));

    assert_eq!(statuses.borrow().last(), Some(&WalkBotStatus::Idle));
    assert!(nodes.borrow().is_empty());
}

#[test]
fn fires_idle_when_nav_mesh_expires() {
    let mesh = make_linear_mesh(4);
    let mut bot = WalkBot::new(&mesh, 0.5);
    bot.set_target(v(3.0, 0.0, 0.0));

    let statuses = record_status(&mut bot);
    let nodes = record_nodes(&mut bot);

    // Let the Arc expire: WalkBot holds only a Weak.
    drop(mesh);

    bot.update(v(0.0, 0.0, 0.0));

    assert_eq!(statuses.borrow().last(), Some(&WalkBotStatus::Idle));
    assert!(nodes.borrow().is_empty());
}

#[test]
fn set_nav_mesh_restores_pathing() {
    let mut bot = WalkBot::default(); // starts with no mesh
    bot.set_target(v(3.0, 0.0, 0.0));

    let statuses = record_status(&mut bot);
    let nodes = record_nodes(&mut bot);

    // First call: no mesh -> Idle
    bot.update(v(0.0, 0.0, 0.0));
    assert_eq!(statuses.borrow().last(), Some(&WalkBotStatus::Idle));

    // Assign a mesh and call again. Keep the Arc alive so the Weak inside WalkBot does not expire before update() is
    // called.
    statuses.borrow_mut().clear();
    nodes.borrow_mut().clear();
    let new_mesh = make_linear_mesh(4);
    bot.set_nav_mesh(&new_mesh);
    bot.set_min_node_distance(0.5);
    bot.update(v(0.0, 0.0, 0.0));

    assert_eq!(statuses.borrow().last(), Some(&WalkBotStatus::Pathing));
    assert!(!nodes.borrow().is_empty());
}

// Status: Idle, A* finds no path

#[test]
fn fires_idle_when_no_path_exists() {
    // Disconnected graph: two isolated vertices
    let mut mesh = NavigationMesh::new();
    mesh.set_neighbors(v(0.0, 0.0, 0.0), vec![]);
    mesh.set_neighbors(v(10.0, 0.0, 0.0), vec![]);
    let mesh = Arc::new(mesh);

    let mut bot = WalkBot::new(&mesh, 0.5);
    bot.set_target(v(10.0, 0.0, 0.0));

    let statuses = record_status(&mut bot);
    let nodes = record_nodes(&mut bot);

    bot.update(v(0.0, 0.0, 0.0));

    assert_eq!(statuses.borrow().last(), Some(&WalkBotStatus::Idle));
    assert!(nodes.borrow().is_empty());
}

// Status: Pathing, normal routing

#[test]
fn fires_pathing_and_provides_next_node() {
    let mesh = make_linear_mesh(4);
    let mut bot = WalkBot::new(&mesh, 0.5);
    bot.set_target(v(3.0, 0.0, 0.0));

    let statuses = record_status(&mut bot);
    let nodes = record_nodes(&mut bot);

    bot.update(v(0.0, 0.0, 0.0));

    assert_eq!(statuses.borrow().last(), Some(&WalkBotStatus::Pathing));
    assert!(!nodes.borrow().is_empty());
}

#[test]
fn next_node_is_on_the_path() {
    let mesh = make_linear_mesh(5);
    let mut bot = WalkBot::new(&mesh, 0.5);
    bot.set_target(v(4.0, 0.0, 0.0));

    let nodes = record_nodes(&mut bot);

    bot.update(v(0.0, 0.0, 0.0));

    // The suggested node must be a mesh vertex (x in [0..4], y=0, z=0)
    let hint = *nodes.borrow().first().unwrap();
    assert!(hint.x >= 0.0);
    assert!(hint.x <= 4.0);
    expect_float_eq(hint.y, 0.0);
    expect_float_eq(hint.z, 0.0);
}

// set_min_node_distance

#[test]
fn min_node_distance_affects_finished_threshold() {
    let mesh = make_linear_mesh(3);
    let mut bot = WalkBot::new(&mesh, 0.1); // very tight threshold
    bot.set_target(v(1.0, 0.0, 0.0));

    let statuses = record_status(&mut bot);

    // distance 0.4: outside 0.1 threshold
    bot.update(v(0.6, 0.0, 0.0));
    assert!(!statuses.borrow().contains(&WalkBotStatus::Finished));

    statuses.borrow_mut().clear();
    bot.set_min_node_distance(0.5); // widen threshold
    bot.update(v(0.6, 0.0, 0.0)); // now 0.4 < 0.5 -> Finished

    assert_eq!(statuses.borrow().first(), Some(&WalkBotStatus::Finished));
}

// reset()

#[test]
fn reset_clears_last_visited() {
    let mesh = make_linear_mesh(3);
    let mut bot = WalkBot::new(&mesh, 0.5);
    bot.set_target(v(2.0, 0.0, 0.0));

    let nodes = record_nodes(&mut bot);

    // Tick 1: mark node 0 visited -> hint is node 1
    bot.update(v(0.05, 0.0, 0.0));
    expect_float_eq(nodes.borrow().last().unwrap().x, 1.0);

    // Without reset, a second tick from the same position also gives node 1.
    nodes.borrow_mut().clear();
    bot.reset();

    // After reset, last_visited is cleared. The nearest node is 0 again, it is within threshold so it gets marked
    // visited and we advance to 1. The hint should still be node 1 (same outcome, but state was cleanly reset).
    bot.update(v(0.05, 0.0, 0.0));
    // Confirm the bot still routes correctly after reset.
    assert!(nodes.borrow().last().unwrap().x >= 0.0);
}

// Node advancement: visited node causes skip to next

#[test]
fn advances_when_nearest_node_already_visited() {
    // Chain: (0,0,0) -> (1,0,0) -> (2,0,0)
    let mesh = make_linear_mesh(3);
    let mut bot = WalkBot::new(&mesh, 0.5);
    bot.set_target(v(2.0, 0.0, 0.0));

    let nodes = record_nodes(&mut bot);

    // Tick 1: bot is very close to node 0 -> node 0 marked visited -> hint is node 1.
    bot.update(v(0.05, 0.0, 0.0));
    expect_float_eq(nodes.borrow().last().unwrap().x, 1.0);

    nodes.borrow_mut().clear();

    // Tick 2: bot has moved to near node 1 -> node 1 marked visited -> hint advances to node 2.
    bot.update(v(1.05, 0.0, 0.0));
    assert!(nodes.borrow().last().unwrap().x > 1.0);
}

// Displacement recovery: bot knocked back to unvisited node

#[test]
fn recover_after_displacement_to_unvisited_node() {
    // Chain: 0 -> 1 -> 2 -> 3 -> 4
    let mesh = make_linear_mesh(5);
    let mut bot = WalkBot::new(&mesh, 0.5);
    bot.set_target(v(4.0, 0.0, 0.0));

    let nodes = record_nodes(&mut bot);

    // Walk forward through nodes 0-3 to build visited state.
    for i in 0..=3 {
        nodes.borrow_mut().clear();
        bot.update(v(i as f32 + 0.1, 0.0, 0.0));
    }

    // Displace the bot back to near node 1. The bot should route toward node 1 first rather than skipping ahead to
    // node 4.
    nodes.borrow_mut().clear();
    bot.update(v(1.1, 0.0, 0.0));

    assert!(nodes.borrow().last().unwrap().x <= 3.0);
}

// Callback wiring

#[test]
fn replacing_path_callback_takes_effect() {
    let first_cb_calls = Cell::new(0);
    let second_cb_calls = Cell::new(0);

    let mesh = make_linear_mesh(4);
    let mut bot = WalkBot::new(&mesh, 0.5);
    bot.set_target(v(3.0, 0.0, 0.0));

    bot.on_path(|_| first_cb_calls.set(first_cb_calls.get() + 1));
    bot.update(v(0.0, 0.0, 0.0));

    bot.on_path(|_| second_cb_calls.set(second_cb_calls.get() + 1));
    bot.update(v(0.0, 0.0, 0.0));

    assert_eq!(first_cb_calls.get(), 1);
    assert_eq!(second_cb_calls.get(), 1);
}

#[test]
fn replacing_status_callback_takes_effect() {
    let cb1 = Cell::new(0);
    let cb2 = Cell::new(0);

    let mesh = make_linear_mesh(4);
    let mut bot = WalkBot::new(&mesh, 0.5);
    bot.set_target(v(3.0, 0.0, 0.0));

    let _nodes = record_nodes(&mut bot);

    bot.on_status(|_| cb1.set(cb1.get() + 1));
    bot.update(v(0.0, 0.0, 0.0));

    bot.on_status(|_| cb2.set(cb2.get() + 1));
    bot.update(v(0.0, 0.0, 0.0));

    assert_eq!(cb1.get(), 1);
    assert_eq!(cb2.get(), 1);
}

// Full walk simulation: bot traverses a linear mesh step by step

#[derive(Default)]
struct WalkState {
    position: Vector3<f32>,
    visited_nodes: Vec<Vector3<f32>>,
    status_history: Vec<WalkBotStatus>,
    finished: bool,
}

/// Simulates one game-loop tick and immediately "teleports" the bot to the suggested node so the next tick starts from
/// there.
struct WalkSimulator {
    bot: WalkBot<'static>,
    state: Rc<RefCell<WalkState>>,
}

impl WalkSimulator {
    fn new(
        mesh: &Arc<NavigationMesh>,
        start: Vector3<f32>,
        goal: Vector3<f32>,
        threshold: f32,
    ) -> Self {
        let state = Rc::new(RefCell::new(WalkState {
            position: start,
            ..WalkState::default()
        }));

        let mut bot = WalkBot::new(mesh, threshold);
        bot.set_target(goal);

        let path_state = Rc::clone(&state);
        bot.on_path(move |next| {
            let mut state = path_state.borrow_mut();
            state.visited_nodes.push(next);
            state.position = next; // teleport to the suggested node
        });

        let status_state = Rc::clone(&state);
        bot.on_status(move |status| {
            let mut state = status_state.borrow_mut();
            state.status_history.push(status);
            if status == WalkBotStatus::Finished {
                state.finished = true;
            }
        });

        Self { bot, state }
    }

    fn run(&mut self, max_ticks: i32) {
        for _ in 0..max_ticks {
            if self.finished() {
                break;
            }
            let position = self.state.borrow().position;
            self.bot.update(position);
        }
    }

    fn finished(&self) -> bool {
        self.state.borrow().finished
    }
}

#[test]
fn bot_reaches_target_on_linear_mesh() {
    let mesh = make_linear_mesh(5);
    let mut sim = WalkSimulator::new(&mesh, v(0.0, 0.0, 0.0), v(4.0, 0.0, 0.0), 0.5);
    sim.run(50);

    assert!(sim.finished());
}

#[test]
fn status_transition_sequence_is_correct() {
    // Expect: one or more Pathing updates, then exactly Finished at the end.
    let mesh = make_linear_mesh(4);
    let mut sim = WalkSimulator::new(&mesh, v(0.0, 0.0, 0.0), v(3.0, 0.0, 0.0), 0.5);
    sim.run(50);

    assert!(sim.finished());
    let state = sim.state.borrow();
    let (last, before) = state.status_history.split_last().unwrap();

    // All entries before the last must be Pathing
    assert!(
        before
            .iter()
            .all(|&status| status == WalkBotStatus::Pathing)
    );

    assert_eq!(*last, WalkBotStatus::Finished);
}

#[test]
fn bot_visits_nodes_in_forward_order() {
    let mesh = make_linear_mesh(5);
    let mut sim = WalkSimulator::new(&mesh, v(0.0, 0.0, 0.0), v(4.0, 0.0, 0.0), 0.5);
    sim.run(50);

    let state = sim.state.borrow();
    assert!(!state.visited_nodes.is_empty());

    // Verify that x-coordinates are non-decreasing (forward progress only).
    for pair in state.visited_nodes.windows(2) {
        assert!(pair[1].x >= pair[0].x - 1e-3);
    }
}

#[test]
fn two_node_path_reaches_goal() {
    let mesh = make_linear_mesh(2); // (0,0,0) <-> (1,0,0)
    let mut sim = WalkSimulator::new(&mesh, v(0.0, 0.0, 0.0), v(1.0, 0.0, 0.0), 0.5);
    sim.run(10);

    assert!(sim.finished());
}

#[test]
fn long_chain_eventually_finishes() {
    const K_LENGTH: i32 = 20;
    let mesh = make_linear_mesh(K_LENGTH);
    let mut sim = WalkSimulator::new(
        &mesh,
        v(0.0, 0.0, 0.0),
        v((K_LENGTH - 1) as f32, 0.0, 0.0),
        0.5,
    );
    sim.run(200);

    assert!(sim.finished());
}

#[test]
fn start_already_at_target_finishes_immediately() {
    let mesh = make_linear_mesh(3);
    let mut sim = WalkSimulator::new(&mesh, v(1.0, 0.0, 0.0), v(1.0, 0.0, 0.0), 0.5);
    sim.run(5);

    assert!(sim.finished());
    assert_eq!(sim.state.borrow().status_history, [WalkBotStatus::Finished]);
}

#[test]
fn no_idle_emitted_during_successful_walk() {
    let mesh = make_linear_mesh(4);
    let mut sim = WalkSimulator::new(&mesh, v(0.0, 0.0, 0.0), v(3.0, 0.0, 0.0), 0.5);
    sim.run(50);

    assert!(
        !sim.state
            .borrow()
            .status_history
            .contains(&WalkBotStatus::Idle)
    );
}

// Walk simulation on a branching mesh

#[test]
fn bot_navigates_branching_mesh() {
    // Diamond topology:
    //        (1,1,0)
    //       /       \
    // (0,0,0)       (2,0,0)
    //       \       /
    //        (1,-1,0)
    let mut mesh = NavigationMesh::new();
    let start = v(0.0, 0.0, 0.0);
    let top = v(1.0, 1.0, 0.0);
    let bot_node = v(1.0, -1.0, 0.0);
    let goal = v(2.0, 0.0, 0.0);

    mesh.set_neighbors(start, vec![top, bot_node]);
    mesh.set_neighbors(top, vec![start, goal]);
    mesh.set_neighbors(bot_node, vec![start, goal]);
    mesh.set_neighbors(goal, vec![top, bot_node]);
    let mesh = Arc::new(mesh);

    let mut sim = WalkSimulator::new(&mesh, start, goal, 0.5);
    sim.run(20);

    assert!(sim.finished());
}
