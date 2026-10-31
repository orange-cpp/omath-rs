use super::{Collider, Simplex};
use crate::linear_algebra::Vector3;
use crate::scalar::Real;
use std::cmp::Ordering;
use std::collections::{BTreeSet, BinaryHeap};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EpaResult<T = f32> {
    /// From A to B.
    pub normal: Vector3<T>,
    pub penetration_vector: Vector3<T>,
    pub depth: T,
    pub iterations: usize,
    pub num_vertices: usize,
    pub num_faces: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EpaParams<T = f32> {
    pub max_iterations: usize,
    /// Absolute tolerance on distance growth.
    pub tolerance: T,
}

impl<T: Real> Default for EpaParams<T> {
    fn default() -> Self {
        Self {
            max_iterations: 64,
            tolerance: T::from_f64(1e-4f32 as f64),
        }
    }
}

#[derive(Clone, Copy)]
struct Face<T> {
    i0: usize,
    i1: usize,
    i2: usize,
    /// Unit outward normal.
    n: Vector3<T>,
    /// `n · v0`, ideally `>= 0` because the origin is inside.
    d: T,
    /// Tombstone, which avoids an O(n) compaction per iteration.
    deleted: bool,
}

/// A heap entry, ordered so that the max-heap [`BinaryHeap`] pops the smallest distance first.
struct HeapItem<T> {
    d: T,
    idx: usize,
}

impl<T: Real> Ord for HeapItem<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.d.partial_cmp(&self.d).unwrap_or(Ordering::Equal)
    }
}

impl<T: Real> PartialOrd for HeapItem<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Real> PartialEq for HeapItem<T> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl<T: Real> Eq for HeapItem<T> {}

/// Directed horizon edges. Ordered rather than hashed so that repeated solves stitch faces the same way.
type Boundary = BTreeSet<(usize, usize)>;

/// Expanding polytope algorithm: the penetration between `a` and `b`, from the tetrahedron GJK found around the origin.
///
/// `None` when no face survives. Panics when `simplex` holds fewer than 4 points; C++ has this as a precondition too.
pub fn epa_solve<C: Collider + ?Sized>(
    a: &C,
    b: &C,
    simplex: &Simplex<C::Scalar>,
    params: EpaParams<C::Scalar>,
) -> Option<EpaResult<C::Scalar>> {
    let mut vertexes: Vec<_> = simplex.iter().copied().collect();
    let mut faces = vec![
        make_face(&vertexes, 0, 1, 2),
        make_face(&vertexes, 0, 2, 3),
        make_face(&vertexes, 0, 3, 1),
        make_face(&vertexes, 1, 3, 2),
    ];

    let mut heap: BinaryHeap<_> = faces
        .iter()
        .enumerate()
        .map(|(idx, face)| HeapItem { d: face.d, idx })
        .collect();

    let mut iterations = 0;
    let mut boundary = Boundary::new();

    for it in 0..params.max_iterations {
        discard_stale_heap_entries(&faces, &mut heap);

        let Some(top) = heap.peek() else {
            break;
        };
        let face = faces[top.idx];

        let p = support_point(a, b, face.n);
        let p_dist = face.n.dot(p);

        // Converged: the new support can't push the face closer than the tolerance.
        if p_dist - face.d <= params.tolerance {
            return Some(make_result(&face, it + 1, vertexes.len(), faces.len()));
        }

        let new_idx = vertexes.len();
        vertexes.push(p);

        tombstone_visible_faces(&mut faces, &mut boundary, p);

        // Stitch new faces around the horizon and push them directly, with no heap rebuild.
        for &(edge_a, edge_b) in &boundary {
            let face = make_face(&vertexes, edge_a, edge_b, new_idx);
            heap.push(HeapItem {
                d: face.d,
                idx: faces.len(),
            });
            faces.push(face);
        }

        if !p.dot(p).is_finite() {
            break; // safety
        }

        iterations = it + 1;
    }

    let best = find_best_surviving_face(&faces)?;
    Some(make_result(best, iterations, vertexes.len(), faces.len()))
}

fn make_result<T: Real>(
    face: &Face<T>,
    iterations: usize,
    num_vertices: usize,
    num_faces: usize,
) -> EpaResult<T> {
    EpaResult {
        normal: face.n,
        penetration_vector: face.n * face.d,
        depth: face.d,
        iterations,
        num_vertices,
        num_faces,
    }
}

fn visible_from<T: Real>(face: &Face<T>, p: Vector3<T>) -> bool {
    face.n.dot(p) - face.d > T::from_f64(1e-7)
}

fn add_edge_boundary(boundary: &mut Boundary, a: usize, b: usize) {
    // The opposite edge being there means an internal edge shared by two visible faces. Otherwise it is on the horizon.
    if !boundary.remove(&(b, a)) {
        boundary.insert((a, b));
    }
}

fn make_face<T: Real>(vertexes: &[Vector3<T>], i0: usize, mut i1: usize, mut i2: usize) -> Face<T> {
    let tiny = T::from_f64(1e-30);

    let a0 = vertexes[i0];
    let a1 = vertexes[i1];
    let a2 = vertexes[i2];

    let mut n = (a1 - a0).cross(a2 - a0);
    if n.dot(n) <= tiny {
        n = any_perp_vec(a1 - a0); // degenerate guard
    }
    // Make the normal point away from the origin.
    if n.dot(a0) < T::ZERO {
        std::mem::swap(&mut i1, &mut i2);
        n = -n;
    }
    n = n * (T::ONE / n.dot(n).max_of(tiny).sqrt());

    Face {
        i0,
        i1,
        i2,
        n,
        d: n.dot(a0),
        deleted: false,
    }
}

fn support_point<C: Collider + ?Sized>(
    a: &C,
    b: &C,
    dir: Vector3<C::Scalar>,
) -> Vector3<C::Scalar> {
    a.find_abs_furthest_vertex_position(dir) - b.find_abs_furthest_vertex_position(-dir)
}

fn near_zero_vec<T: Real>(v: Vector3<T>) -> bool {
    let eps = T::from_f64(1e-7f32 as f64);
    v.dot(v) <= eps * eps
}

fn any_perp_vec<T: Real>(v: Vector3<T>) -> Vector3<T> {
    let (zero, one) = (T::ZERO, T::ONE);
    let axes = [
        Vector3::new(one, zero, zero),
        Vector3::new(zero, one, zero),
        Vector3::new(zero, zero, one),
    ];

    axes.into_iter()
        .map(|axis| v.cross(axis))
        .find(|d| !near_zero_vec(*d))
        .unwrap_or(axes[0])
}

fn find_best_surviving_face<T: Real>(faces: &[Face<T>]) -> Option<&Face<T>> {
    let mut best: Option<&Face<T>> = None;
    for face in faces {
        if !face.deleted && best.is_none_or(|best| face.d < best.d) {
            best = Some(face);
        }
    }
    best
}

fn tombstone_visible_faces<T: Real>(faces: &mut [Face<T>], boundary: &mut Boundary, p: Vector3<T>) {
    boundary.clear();
    for face in faces {
        if !face.deleted && visible_from(face, p) {
            face.deleted = true;
            add_edge_boundary(boundary, face.i0, face.i1);
            add_edge_boundary(boundary, face.i1, face.i2);
            add_edge_boundary(boundary, face.i2, face.i0);
        }
    }
}

fn discard_stale_heap_entries<T: Real>(faces: &[Face<T>], heap: &mut BinaryHeap<HeapItem<T>>) {
    while let Some(top) = heap.peek() {
        if !faces[top.idx].deleted && faces[top.idx].d == top.d {
            break;
        }
        heap.pop();
    }
}
