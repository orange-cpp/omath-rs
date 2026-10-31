use super::Error;
use crate::linear_algebra::{Mat, NdcDepthRange, StoreOrder, Triangle, Vector2, Vector3};
use crate::primitives::{Aabb, Obb};
use crate::scalar::{Real, Scalar, Signed};
use crate::trigonometry::{Angle, AngleRange, Clamped0To180, ViewAngles};
use std::cell::Cell;
use std::fmt;

/// A viewport size in pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ViewPort {
    pub width: f32,
    pub height: f32,
}

impl ViewPort {
    #[inline]
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    #[inline]
    pub const fn aspect_ratio(&self) -> f32 {
        self.width / self.height
    }
}

pub type FieldOfView = Angle<f32, Clamped0To180>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewPortClipping {
    /// Reject anything outside the viewport.
    Auto,
    /// Only reject positions outside the depth range, leaving `x` and `y` to the caller.
    Manual,
}

/// Which view matrix rows point opposite to the world direction of the same name.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CameraAxes {
    pub inverted_forward: bool,
    pub inverted_right: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScreenStart {
    TopLeftCorner,
    BottomLeftCorner,
}

/// A 4x4 matrix in the layout of the engine `E`.
pub type Mat4X4Of<E> = Mat<4, 4, <E as CameraEngine>::Scalar, <E as CameraEngine>::Order>;

/// The view angles of the engine `E`.
pub type ViewAnglesOf<E> = ViewAngles<
    <E as CameraEngine>::Scalar,
    <E as CameraEngine>::PitchRange,
    <E as CameraEngine>::YawRange,
    <E as CameraEngine>::RollRange,
>;

/// The conventions a [`Camera`] needs from an engine. Replaces the C++ `CameraEngineConcept` and the template
/// parameters of `Camera`.
///
/// The ranges are bounded by `Default` because `ViewAngles` derives it, which puts the bound on the marker types.
pub trait CameraEngine: 'static {
    type Scalar: Real;
    type Order: StoreOrder;
    type PitchRange: AngleRange + Default;
    type YawRange: AngleRange + Default;
    type RollRange: AngleRange + Default;

    /// Depth range of the engine's normalized device coordinates. The C++ default is `NegativeOneToOne`.
    const DEPTH_RANGE: NdcDepthRange = NdcDepthRange::NegativeOneToOne;
    const AXES: CameraAxes = CameraAxes {
        inverted_forward: false,
        inverted_right: false,
    };

    fn calc_look_at_angle(
        cam_origin: Vector3<Self::Scalar>,
        look_at: Vector3<Self::Scalar>,
    ) -> ViewAnglesOf<Self>;

    fn calc_view_matrix(
        angles: ViewAnglesOf<Self>,
        cam_origin: Vector3<Self::Scalar>,
    ) -> Mat4X4Of<Self>;

    fn calc_projection_matrix(
        fov: FieldOfView,
        view_port: ViewPort,
        near_plane: Self::Scalar,
        far_plane: Self::Scalar,
        ndc_depth_range: NdcDepthRange,
    ) -> Mat4X4Of<Self>;

    /// Inverse of the engine's rotation matrix. Engines that keep the default of `None` recover angles through
    /// [`CameraEngine::calc_look_at_angle`] instead, which loses roll unless it turns around the camera's forward axis.
    fn extract_rotation_angles(_rotation: &Mat4X4Of<Self>) -> Option<ViewAnglesOf<Self>> {
        None
    }

    /// Recovers the FOV from a projection matrix. The default handles a plain vertical FOV, where
    /// `m[1,1] == 1 / tan(fov / 2)`. Engines measuring it some other way must undo their own projection.
    fn calc_field_of_view(projection_matrix: &Mat4X4Of<Self>) -> FieldOfView {
        let f = projection_matrix.at(1, 1);
        let fov_radians = Self::Scalar::from_f64(2.0) * (Self::Scalar::ONE / f).atan();
        FieldOfView::from_radians(fov_radians.to_f64() as f32)
    }
}

/// What [`Camera::extract_projection_params`] recovers from a projection matrix.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProjectionParams<T> {
    pub fov: FieldOfView,
    pub aspect_ratio: T,
}

/// One plane `ax + by + cz + d >= 0` of the view frustum, where the inside is non-negative.
#[derive(Clone, Copy)]
struct FrustumPlane<T> {
    a: T,
    b: T,
    c: T,
    d: T,
}

/// A perspective camera following the conventions of the engine `E`.
///
/// The matrices are cached on first use and dropped by the setters. The caches sit in [`Cell`]s so that the getters can
/// take `&self`, which makes a camera `Send` but not `Sync`.
pub struct Camera<E: CameraEngine> {
    view_port: ViewPort,
    field_of_view: FieldOfView,
    far_plane_distance: E::Scalar,
    near_plane_distance: E::Scalar,
    view_angles: ViewAnglesOf<E>,
    origin: Vector3<E::Scalar>,

    view_projection_matrix: Cell<Option<Mat4X4Of<E>>>,
    projection_matrix: Cell<Option<Mat4X4Of<E>>>,
    view_matrix: Cell<Option<Mat4X4Of<E>>>,
    inv_view_projection_matrix: Cell<Option<Mat4X4Of<E>>>,
}

impl<E: CameraEngine> Clone for Camera<E> {
    fn clone(&self) -> Self {
        Self {
            view_port: self.view_port,
            field_of_view: self.field_of_view,
            far_plane_distance: self.far_plane_distance,
            near_plane_distance: self.near_plane_distance,
            view_angles: self.view_angles,
            origin: self.origin,
            view_projection_matrix: self.view_projection_matrix.clone(),
            projection_matrix: self.projection_matrix.clone(),
            view_matrix: self.view_matrix.clone(),
            inv_view_projection_matrix: self.inv_view_projection_matrix.clone(),
        }
    }
}

impl<E: CameraEngine> fmt::Debug for Camera<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Camera")
            .field("view_port", &self.view_port)
            .field("field_of_view", &self.field_of_view)
            .field("far_plane_distance", &self.far_plane_distance)
            .field("near_plane_distance", &self.near_plane_distance)
            .field("view_angles", &self.view_angles)
            .field("origin", &self.origin)
            .finish_non_exhaustive()
    }
}

impl<E: CameraEngine> Camera<E> {
    pub fn new(
        position: Vector3<E::Scalar>,
        view_angles: ViewAnglesOf<E>,
        view_port: ViewPort,
        fov: FieldOfView,
        near: E::Scalar,
        far: E::Scalar,
    ) -> Self {
        Self {
            view_port,
            field_of_view: fov,
            far_plane_distance: far,
            near_plane_distance: near,
            view_angles,
            origin: position,
            view_projection_matrix: Cell::new(None),
            projection_matrix: Cell::new(None),
            view_matrix: Cell::new(None),
            inv_view_projection_matrix: Cell::new(None),
        }
    }

    /// Recovers the FOV and aspect ratio from a perspective projection matrix built by any engine.
    ///
    /// Both NDC depth ranges share the same `m[0,0]` and `m[1,1]` layout, so this works for either. The FOV is the same
    /// kind the camera takes, so it can be passed back to [`Camera::set_field_of_view`].
    pub fn extract_projection_params(proj_matrix: &Mat4X4Of<E>) -> ProjectionParams<E::Scalar> {
        // m[0,0] == m[1,1] / aspect_ratio  =>  aspect = m[1,1] / m[0,0]
        ProjectionParams {
            fov: E::calc_field_of_view(proj_matrix),
            aspect_ratio: proj_matrix.at(1, 1) / proj_matrix.at(0, 0),
        }
    }

    pub fn calc_view_angles_from_view_matrix(view_matrix: &Mat4X4Of<E>) -> ViewAnglesOf<E> {
        // A view matrix's rotation block is V = M * R^T: R is the engine's rotation for the angles and M is the
        // engine's fixed mapping from world axes to view rows (including any row it flips), which is just the view
        // matrix of a camera with all angles at 0. M is orthonormal, so R = V^T * M, and the engine turns R back into
        // angles - roll included, whatever order it composes its rotations in.
        let reference = E::calc_view_matrix(ViewAnglesOf::<E>::default(), Vector3::default());

        let mut rotation = Mat4X4Of::<E>::zero();
        for row in 0..3 {
            for column in 0..3 {
                *rotation.at_mut(row, column) = view_matrix.at(0, row) * reference.at(0, column)
                    + view_matrix.at(1, row) * reference.at(1, column)
                    + view_matrix.at(2, row) * reference.at(2, column);
            }
        }
        *rotation.at_mut(3, 3) = E::Scalar::ONE;

        E::extract_rotation_angles(&rotation)
            .unwrap_or_else(|| Self::calc_view_angles_by_look_at(view_matrix))
    }

    pub fn calc_origin_from_view_matrix(view_matrix: &Mat4X4Of<E>) -> Vector3<E::Scalar> {
        // The view matrix is R * T(-origin), so the last column stores t = -R * origin.
        // Recovering origin: origin = -R^T * t
        let m = |row: usize, column: usize| view_matrix.at(row, column);
        let component = |column: usize| {
            -(m(0, column) * m(0, 3) + m(1, column) * m(1, 3) + m(2, column) * m(2, 3))
        };
        Vector3::new(component(0), component(1), component(2))
    }

    /// Fallback for engines without `extract_rotation_angles`: look-at recovers pitch and yaw from the forward row, then
    /// roll is solved around forward. This only works for engines that roll around the camera's own forward axis; for
    /// the others roll is left at 0 and only the viewing direction is recovered.
    fn calc_view_angles_by_look_at(view_matrix: &Mat4X4Of<E>) -> ViewAnglesOf<E> {
        let row = |matrix: &Mat4X4Of<E>, index: usize| {
            Vector3::new(
                matrix.at(index, 0),
                matrix.at(index, 1),
                matrix.at(index, 2),
            )
        };

        let forward_row = row(view_matrix, 2);
        let forward_vector = if E::AXES.inverted_forward {
            -forward_row
        } else {
            forward_row
        };
        let mut angles = E::calc_look_at_angle(Vector3::default(), forward_vector);

        // Look-at only pins the forward axis and leaves roll at 0. Roll is the angle, around forward, between the
        // matrix's up row and the up row of the same orientation without roll. Which way a positive roll turns is
        // engine-specific, so try both signs and keep the one that rebuilds the matrix's up row.
        let level = E::calc_view_matrix(angles, Vector3::default());
        let up = row(view_matrix, 1);
        let roll_radians = up.dot(row(&level, 0)).atan2(up.dot(row(&level, 1)));

        let rebuild_with_roll = |radians: E::Scalar| {
            let mut candidate = angles;
            candidate.roll = Angle::from_radians(radians);
            E::calc_view_matrix(candidate, Vector3::default())
        };

        let positive = rebuild_with_roll(roll_radians);
        let negative = rebuild_with_roll(-roll_radians);
        let positive_wins = up.dot(row(&positive, 1)) >= up.dot(row(&negative, 1));
        let rebuilt = if positive_wins { positive } else { negative };

        // Engines that apply roll around a world axis instead of the camera's forward axis move forward when rolling,
        // so no roll fits on top of the look-at angles. Keep roll at 0 for them.
        let forward_tolerance = E::Scalar::from_f64(1e-4);
        if forward_row.dot(row(&rebuilt, 2)) < E::Scalar::ONE - forward_tolerance {
            return angles;
        }

        angles.roll = Angle::from_radians(if positive_wins {
            roll_radians
        } else {
            -roll_radians
        });
        angles
    }

    pub fn look_at(&mut self, target: Vector3<E::Scalar>) {
        self.view_angles = E::calc_look_at_angle(self.origin, target);
        self.invalidate_view();
    }

    #[inline]
    pub fn calc_look_at_angles(&self, look_to: Vector3<E::Scalar>) -> ViewAnglesOf<E> {
        E::calc_look_at_angle(self.origin, look_to)
    }

    #[inline]
    pub fn get_forward(&self) -> Vector3<E::Scalar> {
        self.view_basis_row(2)
    }

    #[inline]
    pub fn get_right(&self) -> Vector3<E::Scalar> {
        self.view_basis_row(0)
    }

    #[inline]
    pub fn get_up(&self) -> Vector3<E::Scalar> {
        self.view_basis_row(1)
    }

    pub fn get_abs_forward(&self) -> Vector3<E::Scalar> {
        if E::AXES.inverted_forward {
            -self.get_forward()
        } else {
            self.get_forward()
        }
    }

    pub fn get_abs_right(&self) -> Vector3<E::Scalar> {
        if E::AXES.inverted_right {
            -self.get_right()
        } else {
            self.get_right()
        }
    }

    #[inline]
    pub fn get_abs_up(&self) -> Vector3<E::Scalar> {
        self.get_up()
    }

    pub fn calc_view_projection_matrix(&self) -> Mat4X4Of<E> {
        self.calc_projection_matrix() * self.calc_view_matrix()
    }

    pub fn calc_view_matrix(&self) -> Mat4X4Of<E> {
        E::calc_view_matrix(self.view_angles, self.origin)
    }

    pub fn calc_projection_matrix(&self) -> Mat4X4Of<E> {
        E::calc_projection_matrix(
            self.field_of_view,
            self.view_port,
            self.near_plane_distance,
            self.far_plane_distance,
            E::DEPTH_RANGE,
        )
    }

    pub fn get_view_projection_matrix(&self) -> Mat4X4Of<E> {
        if let Some(view_projection) = self.view_projection_matrix.get() {
            return view_projection;
        }

        let view_projection = self.get_projection_matrix() * self.get_view_matrix();
        self.view_projection_matrix.set(Some(view_projection));
        // Any cached inverse describes the matrix just replaced. Dropping it here rather than in every setter means a
        // new setter cannot forget to invalidate it, as long as it invalidates the view-projection matrix itself.
        self.inv_view_projection_matrix.set(None);
        view_projection
    }

    /// `None` when the view-projection matrix is not invertible. That is a degenerate camera (zero field of view, near
    /// equal to far), and it is the one case that re-inverts on every call instead of caching the failure.
    pub fn get_inv_view_projection_matrix(&self) -> Option<Mat4X4Of<E>> {
        let view_projection = self.get_view_projection_matrix();

        if self.inv_view_projection_matrix.get().is_none() {
            self.inv_view_projection_matrix
                .set(view_projection.inverted());
        }

        self.inv_view_projection_matrix.get()
    }

    pub fn get_view_matrix(&self) -> Mat4X4Of<E> {
        if let Some(view) = self.view_matrix.get() {
            return view;
        }

        let view = self.calc_view_matrix();
        self.view_matrix.set(Some(view));
        view
    }

    pub fn get_projection_matrix(&self) -> Mat4X4Of<E> {
        if let Some(projection) = self.projection_matrix.get() {
            return projection;
        }

        let projection = self.calc_projection_matrix();
        self.projection_matrix.set(Some(projection));
        projection
    }

    pub fn set_field_of_view(&mut self, fov: FieldOfView) {
        self.field_of_view = fov;
        self.invalidate_projection();
    }

    pub fn set_near_plane(&mut self, near_plane: E::Scalar) {
        self.near_plane_distance = near_plane;
        self.invalidate_projection();
    }

    pub fn set_far_plane(&mut self, far_plane: E::Scalar) {
        self.far_plane_distance = far_plane;
        self.invalidate_projection();
    }

    pub fn set_view_angles(&mut self, view_angles: ViewAnglesOf<E>) {
        self.view_angles = view_angles;
        self.invalidate_view();
    }

    pub fn set_origin(&mut self, origin: Vector3<E::Scalar>) {
        self.origin = origin;
        self.invalidate_view();
    }

    pub fn set_view_port(&mut self, view_port: ViewPort) {
        self.view_port = view_port;
        self.invalidate_projection();
    }

    #[inline]
    pub fn get_field_of_view(&self) -> FieldOfView {
        self.field_of_view
    }

    #[inline]
    pub fn get_near_plane(&self) -> E::Scalar {
        self.near_plane_distance
    }

    #[inline]
    pub fn get_far_plane(&self) -> E::Scalar {
        self.far_plane_distance
    }

    #[inline]
    pub fn get_view_angles(&self) -> ViewAnglesOf<E> {
        self.view_angles
    }

    #[inline]
    pub fn get_origin(&self) -> Vector3<E::Scalar> {
        self.origin
    }

    /// Same as [`Camera::world_to_screen_with`] from the top left corner.
    #[inline]
    pub fn world_to_screen(
        &self,
        world_position: Vector3<E::Scalar>,
    ) -> Result<Vector3<E::Scalar>, Error> {
        self.world_to_screen_with(world_position, ScreenStart::TopLeftCorner)
    }

    pub fn world_to_screen_with(
        &self,
        world_position: Vector3<E::Scalar>,
        screen_start: ScreenStart,
    ) -> Result<Vector3<E::Scalar>, Error> {
        self.project_to_screen(world_position, ViewPortClipping::Auto, screen_start)
    }

    /// Same as [`Camera::world_to_screen_unclipped_with`] from the top left corner.
    #[inline]
    pub fn world_to_screen_unclipped(
        &self,
        world_position: Vector3<E::Scalar>,
    ) -> Result<Vector3<E::Scalar>, Error> {
        self.world_to_screen_unclipped_with(world_position, ScreenStart::TopLeftCorner)
    }

    pub fn world_to_screen_unclipped_with(
        &self,
        world_position: Vector3<E::Scalar>,
        screen_start: ScreenStart,
    ) -> Result<Vector3<E::Scalar>, Error> {
        self.project_to_screen(world_position, ViewPortClipping::Manual, screen_start)
    }

    pub fn is_culled_by_frustum(&self, triangle: &Triangle<Vector3<E::Scalar>>) -> bool {
        let zero = E::Scalar::ZERO;

        // Transform to clip space (before perspective divide)
        let view_projection = self.get_view_projection_matrix();
        let c0 = Self::transform_point(&view_projection, triangle.vertex1);
        let c1 = Self::transform_point(&view_projection, triangle.vertex2);
        let c2 = Self::transform_point(&view_projection, triangle.vertex3);

        // If all vertices are behind the camera (w <= 0), trivially reject
        if c0[3] <= zero && c1[3] <= zero && c2[3] <= zero {
            return true;
        }

        let beyond_positive =
            |axis: usize| c0[axis] > c0[3] && c1[axis] > c1[3] && c2[axis] > c2[3];
        let beyond_negative =
            |axis: usize| c0[axis] < -c0[3] && c1[axis] < -c1[3] && c2[axis] < -c2[3];

        // Clip volume in clip space:
        // -w <= x <= w
        // -w <= y <= w
        // z_min <= z <= w  (z_min = -w for [-1,1], 0 for [0,1])
        if (0..2).any(|axis| beyond_negative(axis) || beyond_positive(axis)) || beyond_positive(2) {
            return true;
        }

        match E::DEPTH_RANGE {
            NdcDepthRange::ZeroToOne => c0[2] < zero && c1[2] < zero && c2[2] < zero,
            NdcDepthRange::NegativeOneToOne => beyond_negative(2),
        }
    }

    pub fn is_aabb_culled_by_frustum(&self, aabb: &Aabb<E::Scalar>) -> bool {
        let zero = E::Scalar::ZERO;

        // For each plane, find the AABB corner most in the direction of the plane normal (the "positive vertex"). If
        // it is outside, the entire AABB is outside.
        self.extract_frustum_planes()
            .iter()
            .any(|&FrustumPlane { a, b, c, d }| {
                let px = if a >= zero { aabb.max.x } else { aabb.min.x };
                let py = if b >= zero { aabb.max.y } else { aabb.min.y };
                let pz = if c >= zero { aabb.max.z } else { aabb.min.z };

                a * px + b * py + c * pz + d < zero
            })
    }

    pub fn is_obb_culled_by_frustum(&self, obb: &Obb<E::Scalar>) -> bool {
        // For each plane, project the OBB extents onto the plane normal to get the effective radius, then test the
        // center's signed distance against it.
        self.extract_frustum_planes()
            .iter()
            .any(|&FrustumPlane { a, b, c, d }| {
                let normal = Vector3::new(a, b, c);

                let center_distance = normal.dot(obb.center) + d;
                let radius = obb.half_extents.x * normal.dot(obb.axis_x).abs_value()
                    + obb.half_extents.y * normal.dot(obb.axis_y).abs_value()
                    + obb.half_extents.z * normal.dot(obb.axis_z).abs_value();

                center_distance + radius < E::Scalar::ZERO
            })
    }

    pub fn world_to_view_coordinates(
        &self,
        world_coordinates: Vector3<E::Scalar>,
    ) -> Vector3<E::Scalar> {
        let [x, y, z, _] = Self::transform_point(&self.get_view_matrix(), world_coordinates);
        Vector3::new(x, y, z)
    }

    /// The C++ default for `clipping` is [`ViewPortClipping::Auto`].
    pub fn world_to_view_port(
        &self,
        world_position: Vector3<E::Scalar>,
        clipping: ViewPortClipping,
    ) -> Result<Vector3<E::Scalar>, Error> {
        Self::clip_to_view_port(
            Self::transform_point(&self.get_view_projection_matrix(), world_position),
            clipping,
        )
    }

    pub fn view_port_to_world(&self, ndc: Vector3<E::Scalar>) -> Result<Vector3<E::Scalar>, Error> {
        let inv_view_proj = self
            .get_inv_view_projection_matrix()
            .ok_or(Error::InvViewProjMatDetEqZero)?;
        Self::unproject(&inv_view_proj, ndc)
    }

    /// Same as [`Camera::screen_to_world_with`] from the top left corner.
    #[inline]
    pub fn screen_to_world(
        &self,
        screen_pos: Vector3<E::Scalar>,
    ) -> Result<Vector3<E::Scalar>, Error> {
        self.screen_to_world_with(screen_pos, ScreenStart::TopLeftCorner)
    }

    pub fn screen_to_world_with(
        &self,
        screen_pos: Vector3<E::Scalar>,
        screen_start: ScreenStart,
    ) -> Result<Vector3<E::Scalar>, Error> {
        self.view_port_to_world(self.screen_to_ndc(screen_pos, screen_start))
    }

    /// [`Camera::screen_to_world`] for a position without depth, which is taken to be at `z = 1`.
    #[inline]
    pub fn screen_to_world_2d(
        &self,
        screen_pos: Vector2<E::Scalar>,
    ) -> Result<Vector3<E::Scalar>, Error> {
        self.screen_to_world_2d_with(screen_pos, ScreenStart::TopLeftCorner)
    }

    pub fn screen_to_world_2d_with(
        &self,
        screen_pos: Vector2<E::Scalar>,
        screen_start: ScreenStart,
    ) -> Result<Vector3<E::Scalar>, Error> {
        self.screen_to_world_with(
            Vector3::new(screen_pos.x, screen_pos.y, E::Scalar::ONE),
            screen_start,
        )
    }

    /// NDC have `x` to the right and `y` up, both in `[-1, 1]`.
    ///
    /// ```text
    /// +------------------------>
    /// | (0, 0)
    /// |
    /// v
    /// ```
    pub fn ndc_to_screen_position_from_top_left_corner(
        &self,
        ndc: Vector3<E::Scalar>,
    ) -> Vector3<E::Scalar> {
        let (one, two) = (E::Scalar::ONE, E::Scalar::from_f64(2.0));
        Vector3::new(
            (ndc.x + one) / two * self.width(),
            (ndc.y / -two + E::Scalar::from_f64(0.5)) * self.height(),
            ndc.z,
        )
    }

    /// ```text
    /// ^
    /// |
    /// | (0, 0)
    /// +------------------------>
    /// ```
    pub fn ndc_to_screen_position_from_bottom_left_corner(
        &self,
        ndc: Vector3<E::Scalar>,
    ) -> Vector3<E::Scalar> {
        let (one, two) = (E::Scalar::ONE, E::Scalar::from_f64(2.0));
        Vector3::new(
            (ndc.x + one) / two * self.width(),
            (ndc.y / two + E::Scalar::from_f64(0.5)) * self.height(),
            ndc.z,
        )
    }

    /// The C++ default for `screen_start` is [`ScreenStart::TopLeftCorner`].
    pub fn screen_to_ndc(
        &self,
        screen_pos: Vector3<E::Scalar>,
        screen_start: ScreenStart,
    ) -> Vector3<E::Scalar> {
        let (one, two) = (E::Scalar::ONE, E::Scalar::from_f64(2.0));
        let ndc_x = screen_pos.x / self.width() * two - one;
        let ndc_y = match screen_start {
            ScreenStart::TopLeftCorner => one - screen_pos.y / self.height() * two,
            ScreenStart::BottomLeftCorner => {
                (screen_pos.y / self.height() - E::Scalar::from_f64(0.5)) * two
            }
        };
        Vector3::new(ndc_x, ndc_y, screen_pos.z)
    }

    #[inline]
    fn width(&self) -> E::Scalar {
        E::Scalar::from_f64(f64::from(self.view_port.width))
    }

    #[inline]
    fn height(&self) -> E::Scalar {
        E::Scalar::from_f64(f64::from(self.view_port.height))
    }

    /// Gribb-Hartmann: extracts the 6 frustum planes from the view-projection matrix. With rows `r0..r3`:
    ///
    /// ```text
    /// left   = r3 + r0
    /// right  = r3 - r0
    /// bottom = r3 + r1
    /// top    = r3 - r1
    /// far    = r3 - r2
    /// near   = r3 + r2 for [-1, 1], r2 for [0, 1]
    /// ```
    fn extract_frustum_planes(&self) -> [FrustumPlane<E::Scalar>; 6] {
        let m = self.get_view_projection_matrix();
        let extract_plane = |sign: E::Scalar, row: usize| {
            let element = |column: usize| m.at(3, column) + sign * m.at(row, column);
            FrustumPlane {
                a: element(0),
                b: element(1),
                c: element(2),
                d: element(3),
            }
        };
        let (plus, minus) = (E::Scalar::ONE, -E::Scalar::ONE);

        let near = match E::DEPTH_RANGE {
            NdcDepthRange::ZeroToOne => FrustumPlane {
                a: m.at(2, 0),
                b: m.at(2, 1),
                c: m.at(2, 2),
                d: m.at(2, 3),
            },
            NdcDepthRange::NegativeOneToOne => extract_plane(plus, 2),
        };

        [
            extract_plane(plus, 0),
            extract_plane(minus, 0),
            extract_plane(plus, 1),
            extract_plane(minus, 1),
            extract_plane(minus, 2),
            near,
        ]
    }

    fn is_ndc_out_of_bounds(ndc: Vector3<E::Scalar>) -> bool {
        let one = E::Scalar::ONE;
        let eps = E::Scalar::EPSILON;

        // x and y are always in [-1, 1]
        ndc.x < -one - eps
            || ndc.x > one + eps
            || ndc.y < -one - eps
            || ndc.y > one + eps
            || Self::is_ndc_z_value_out_of_bounds(ndc.z)
    }

    fn is_ndc_z_value_out_of_bounds(z_ndc: E::Scalar) -> bool {
        let one = E::Scalar::ONE;
        let eps = E::Scalar::EPSILON;
        let lowest = match E::DEPTH_RANGE {
            NdcDepthRange::NegativeOneToOne => -one,
            NdcDepthRange::ZeroToOne => E::Scalar::ZERO,
        };
        z_ndc < lowest - eps || z_ndc > one + eps
    }

    /// Homogeneous point transform: `matrix * (x, y, z, 1)`, written out because this is the per-entity hot path.
    #[inline]
    fn transform_point(matrix: &Mat4X4Of<E>, point: Vector3<E::Scalar>) -> [E::Scalar; 4] {
        std::array::from_fn(|row| {
            matrix.at(row, 0) * point.x
                + matrix.at(row, 1) * point.y
                + matrix.at(row, 2) * point.z
                + matrix.at(row, 3)
        })
    }

    /// Perspective divide plus clipping of a clip-space point produced by `transform_point`.
    fn clip_to_view_port(
        clip: [E::Scalar; 4],
        clipping: ViewPortClipping,
    ) -> Result<Vector3<E::Scalar>, Error> {
        let w = clip[3];
        if w <= E::Scalar::EPSILON {
            return Err(Error::PerspectiveDividerLessEqZero);
        }

        let ndc = Vector3::new(clip[0] / w, clip[1] / w, clip[2] / w);

        let clipped = match clipping {
            ViewPortClipping::Auto => Self::is_ndc_out_of_bounds(ndc),
            ViewPortClipping::Manual => Self::is_ndc_z_value_out_of_bounds(ndc.z),
        };

        if clipped {
            Err(Error::WorldPositionIsOutOfScreenBounds)
        } else {
            Ok(ndc)
        }
    }

    fn unproject(
        inv_view_proj: &Mat4X4Of<E>,
        ndc: Vector3<E::Scalar>,
    ) -> Result<Vector3<E::Scalar>, Error> {
        let world = Self::transform_point(inv_view_proj, ndc);
        let w = world[3];

        if w.abs_value() < E::Scalar::EPSILON {
            return Err(Error::PerspectiveDividerLessEqZero);
        }

        Ok(Vector3::new(world[0] / w, world[1] / w, world[2] / w))
    }

    fn project_to_screen(
        &self,
        world_position: Vector3<E::Scalar>,
        clipping: ViewPortClipping,
        screen_start: ScreenStart,
    ) -> Result<Vector3<E::Scalar>, Error> {
        let ndc = self.world_to_view_port(world_position, clipping)?;

        Ok(match screen_start {
            ScreenStart::TopLeftCorner => self.ndc_to_screen_position_from_top_left_corner(ndc),
            ScreenStart::BottomLeftCorner => {
                self.ndc_to_screen_position_from_bottom_left_corner(ndc)
            }
        })
    }

    /// Row of the view matrix's rotation block: 0 = right, 1 = up, 2 = forward.
    fn view_basis_row(&self, row: usize) -> Vector3<E::Scalar> {
        let view_matrix = self.get_view_matrix();
        Vector3::new(
            view_matrix.at(row, 0),
            view_matrix.at(row, 1),
            view_matrix.at(row, 2),
        )
    }

    /// Dropping the view-projection matrix is enough for its inverse: `get_view_projection_matrix` clears the inverse
    /// whenever it rebuilds.
    fn invalidate_view(&mut self) {
        self.view_matrix.set(None);
        self.view_projection_matrix.set(None);
    }

    fn invalidate_projection(&mut self) {
        self.projection_matrix.set(None);
        self.view_projection_matrix.set(None);
    }
}
