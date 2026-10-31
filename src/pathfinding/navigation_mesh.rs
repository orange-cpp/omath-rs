use crate::Vector3;
use crate::scalar::Scalar;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Write as _;
use std::str::SplitAsciiWhitespace;

/// Failure of a [`NavigationMesh`] operation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    /// The mesh has no vertices to pick the closest of.
    NoClosestVertex,
    /// The vertex is not in the mesh.
    VertexNotFound(Vector3<f32>),
    /// The serialized data ended in the middle of a vertex's neighbors.
    UnexpectedEndOfData,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoClosestVertex => f.write_str("failed to get closest point"),
            Self::VertexNotFound(vertex) => {
                write!(f, "vertex [{}, {}, {}] not found", vertex.x, vertex.y, vertex.z)
            }
            Self::UnexpectedEndOfData => f.write_str("deserialize: unexpected end of data"),
        }
    }
}

impl std::error::Error for Error {}

/// [`Vector3<f32>`] has no `Eq`, because of NaN, so maps are keyed by the bit patterns of the coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct VertexKey(u64, u64, u64);

impl From<Vector3<f32>> for VertexKey {
    fn from(vertex: Vector3<f32>) -> Self {
        Self(vertex.x.hash_bits(), vertex.y.hash_bits(), vertex.z.hash_bits())
    }
}

#[derive(Clone, Debug)]
struct VertexEntry {
    vertex: Vector3<f32>,
    neighbors: Vec<Vector3<f32>>,
}

/// A directed graph of vertices, each of which can carry an optional event tag (e.g. "jump", "teleport").
#[derive(Clone, Debug, Default)]
pub struct NavigationMesh {
    vertex_map: HashMap<VertexKey, VertexEntry>,
    vertex_events: HashMap<VertexKey, String>,
}

impl NavigationMesh {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds `vertex`, or replaces the neighbors it already has.
    pub fn set_neighbors(&mut self, vertex: Vector3<f32>, neighbors: Vec<Vector3<f32>>) {
        self.vertex_map.insert(vertex.into(), VertexEntry { vertex, neighbors });
    }

    /// Every vertex with its neighbors, in no particular order.
    pub fn iter(&self) -> impl Iterator<Item = (Vector3<f32>, &[Vector3<f32>])> {
        self.vertex_map.values().map(|entry| (entry.vertex, entry.neighbors.as_slice()))
    }

    pub fn contains_vertex(&self, vertex: Vector3<f32>) -> bool {
        self.vertex_map.contains_key(&vertex.into())
    }

    pub fn vertex_count(&self) -> usize {
        self.vertex_map.len()
    }

    pub fn get_closest_vertex(&self, point: Vector3<f32>) -> Result<Vector3<f32>, Error> {
        let mut closest = None;
        let mut closest_distance = f32::INFINITY;

        for entry in self.vertex_map.values() {
            let distance = entry.vertex.distance_to(point);

            if closest.is_none() || distance < closest_distance {
                closest = Some(entry.vertex);
                closest_distance = distance;
            }
        }

        closest.ok_or(Error::NoClosestVertex)
    }

    /// `None` when `vertex` is not in the mesh.
    pub fn get_neighbors(&self, vertex: Vector3<f32>) -> Option<&[Vector3<f32>]> {
        self.vertex_map.get(&vertex.into()).map(|entry| entry.neighbors.as_slice())
    }

    pub fn empty(&self) -> bool {
        self.vertex_map.is_empty()
    }

    pub fn set_event(&mut self, vertex: Vector3<f32>, event_id: &str) -> Result<(), Error> {
        if !self.contains_vertex(vertex) {
            return Err(Error::VertexNotFound(vertex));
        }

        self.vertex_events.insert(vertex.into(), event_id.to_owned());
        Ok(())
    }

    pub fn clear_event(&mut self, vertex: Vector3<f32>) {
        self.vertex_events.remove(&vertex.into());
    }

    pub fn get_event(&self, vertex: Vector3<f32>) -> Option<&str> {
        self.vertex_events.get(&vertex.into()).map(String::as_str)
    }

    /// One line per vertex, `x y z neighbor_count event_id`, where `event_id` is `-` when no event is set, followed by
    /// one `nx ny nz` line per neighbor. Numbers are written the way a C++ `std::ostream` writes them by default, so
    /// the bytes match what the C++ library writes. Vertices come in no particular order, as they do there.
    pub fn serialize(&self) -> String {
        let mut out = String::new();

        for (key, entry) in &self.vertex_map {
            let event = self.vertex_events.get(key).map_or("-", String::as_str);

            write_vector(&mut out, entry.vertex);
            writeln!(out, " {} {event}", entry.neighbors.len()).expect("writing to a String cannot fail");

            for &neighbor in &entry.neighbors {
                write_vector(&mut out, neighbor);
                out.push('\n');
            }
        }

        out
    }

    /// Replaces the mesh with what [`serialize`](Self::serialize) wrote.
    ///
    /// Reading stops without an error at the first vertex line that does not parse, keeping what came before it, the
    /// way the C++ stream extraction does. Only running out of neighbors is an error.
    pub fn deserialize(&mut self, raw: &str) -> Result<(), Error> {
        self.vertex_map.clear();
        self.vertex_events.clear();

        let mut tokens = raw.split_ascii_whitespace();

        while let Some((vertex, neighbors_count, event)) = read_vertex_line(&mut tokens) {
            // No reserving from `neighbors_count`, which is untrusted: the loop below runs out of data first.
            let mut neighbors = Vec::new();
            for _ in 0..neighbors_count {
                neighbors.push(read_vector(&mut tokens).ok_or(Error::UnexpectedEndOfData)?);
            }

            self.vertex_map.entry(vertex.into()).or_insert(VertexEntry { vertex, neighbors });

            if event != "-" {
                self.vertex_events.entry(vertex.into()).or_insert_with(|| event.to_owned());
            }
        }

        Ok(())
    }
}

fn read_vertex_line<'a>(tokens: &mut SplitAsciiWhitespace<'a>) -> Option<(Vector3<f32>, usize, &'a str)> {
    let vertex = read_vector(tokens)?;
    let neighbors_count = tokens.next()?.parse().ok()?;
    let event = tokens.next()?;

    Some((vertex, neighbors_count, event))
}

fn read_vector(tokens: &mut SplitAsciiWhitespace<'_>) -> Option<Vector3<f32>> {
    Some(Vector3::new(read_float(tokens)?, read_float(tokens)?, read_float(tokens)?))
}

/// `str::parse` also takes `inf` and `nan`, which C++ stream extraction fails on, and saturates where it fails.
fn read_float(tokens: &mut SplitAsciiWhitespace<'_>) -> Option<f32> {
    let token = tokens.next()?;

    if !token.bytes().all(|byte| byte.is_ascii_digit() || matches!(byte, b'+' | b'-' | b'.' | b'e' | b'E')) {
        return None;
    }

    token.parse().ok().filter(|value: &f32| value.is_finite())
}

fn write_vector(out: &mut String, vector: Vector3<f32>) {
    write!(out, "{} {} {}", ostream_float(vector.x), ostream_float(vector.y), ostream_float(vector.z))
        .expect("writing to a String cannot fail");
}

/// What `std::ostream << float` writes with default settings, which is `printf("%g")`: six significant digits, no
/// trailing zeros, and an exponent of at least two digits once the number leaves the range fixed notation is used for.
fn ostream_float(value: f32) -> String {
    const PRECISION: i32 = 6;

    if value.is_nan() {
        return "nan".to_owned();
    }
    if value.is_infinite() {
        return if value < 0.0 { "-inf" } else { "inf" }.to_owned();
    }
    if value == 0.0 {
        return if value.is_sign_negative() { "-0" } else { "0" }.to_owned();
    }

    // The exponent has to be the one after rounding to six digits: 999999.5 is 1e+06
    let scientific = format!("{:.*e}", (PRECISION - 1) as usize, value);
    let (mantissa, exponent) = scientific.split_once('e').expect("LowerExp always writes an exponent");
    let exponent: i32 = exponent.parse().expect("LowerExp writes the exponent as an integer");

    if (-4..PRECISION).contains(&exponent) {
        let fixed = format!("{:.*}", (PRECISION - 1 - exponent) as usize, value);
        return trim_fraction(&fixed).to_owned();
    }

    let sign = if exponent < 0 { '-' } else { '+' };
    format!("{}e{sign}{:02}", trim_fraction(mantissa), exponent.abs())
}

fn trim_fraction(number: &str) -> &str {
    if number.contains('.') { number.trim_end_matches('0').trim_end_matches('.') } else { number }
}
