use super::Vector3;
use crate::scalar::{Real, Scalar, Signed};
use crate::trigonometry::{Trig, angles};
use std::fmt;
use std::ops::{Div, DivAssign, Index, IndexMut, Mul, MulAssign};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MatSize {
    pub rows: usize,
    pub columns: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatStoreType {
    RowMajor,
    ColumnMajor,
}

/// Range of normalized device coordinates on the depth axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NdcDepthRange {
    /// OpenGL: `[-1, 1]`.
    NegativeOneToOne,
    /// DirectX and Vulkan: `[0, 1]`.
    ZeroToOne,
}

/// Memory layout of a [`Mat`], chosen by a marker type.
///
/// The associated storage type is what allows the layout to differ per order on stable Rust, where `[T; R * C]` cannot
/// be written for generic `R` and `C`.
pub trait StoreOrder: Copy + Default + fmt::Debug + 'static {
    const STORE_TYPE: MatStoreType;

    type Storage<T: Copy, const R: usize, const C: usize>: Copy;

    fn splat<T: Copy, const R: usize, const C: usize>(value: T) -> Self::Storage<T, R, C>;
    fn get<T: Copy, const R: usize, const C: usize>(data: &Self::Storage<T, R, C>, row: usize, column: usize) -> &T;
    fn get_mut<T: Copy, const R: usize, const C: usize>(
        data: &mut Self::Storage<T, R, C>,
        row: usize,
        column: usize,
    ) -> &mut T;
    fn flat<T: Copy, const R: usize, const C: usize>(data: &Self::Storage<T, R, C>) -> &[T];
    fn flat_mut<T: Copy, const R: usize, const C: usize>(data: &mut Self::Storage<T, R, C>) -> &mut [T];
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RowMajor;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ColumnMajor;

impl StoreOrder for RowMajor {
    const STORE_TYPE: MatStoreType = MatStoreType::RowMajor;

    type Storage<T: Copy, const R: usize, const C: usize> = [[T; C]; R];

    #[inline]
    fn splat<T: Copy, const R: usize, const C: usize>(value: T) -> [[T; C]; R] {
        [[value; C]; R]
    }
    #[inline]
    fn get<T: Copy, const R: usize, const C: usize>(data: &[[T; C]; R], row: usize, column: usize) -> &T {
        &data[row][column]
    }
    #[inline]
    fn get_mut<T: Copy, const R: usize, const C: usize>(data: &mut [[T; C]; R], row: usize, column: usize) -> &mut T {
        &mut data[row][column]
    }
    #[inline]
    fn flat<T: Copy, const R: usize, const C: usize>(data: &[[T; C]; R]) -> &[T] {
        data.as_flattened()
    }
    #[inline]
    fn flat_mut<T: Copy, const R: usize, const C: usize>(data: &mut [[T; C]; R]) -> &mut [T] {
        data.as_flattened_mut()
    }
}

impl StoreOrder for ColumnMajor {
    const STORE_TYPE: MatStoreType = MatStoreType::ColumnMajor;

    type Storage<T: Copy, const R: usize, const C: usize> = [[T; R]; C];

    #[inline]
    fn splat<T: Copy, const R: usize, const C: usize>(value: T) -> [[T; R]; C] {
        [[value; R]; C]
    }
    #[inline]
    fn get<T: Copy, const R: usize, const C: usize>(data: &[[T; R]; C], row: usize, column: usize) -> &T {
        &data[column][row]
    }
    #[inline]
    fn get_mut<T: Copy, const R: usize, const C: usize>(data: &mut [[T; R]; C], row: usize, column: usize) -> &mut T {
        &mut data[column][row]
    }
    #[inline]
    fn flat<T: Copy, const R: usize, const C: usize>(data: &[[T; R]; C]) -> &[T] {
        data.as_flattened()
    }
    #[inline]
    fn flat_mut<T: Copy, const R: usize, const C: usize>(data: &mut [[T; R]; C]) -> &mut [T] {
        data.as_flattened_mut()
    }
}

/// A fixed-size `R` by `C` matrix. Indexing is always `(row, column)`, whatever the [`StoreOrder`].
pub struct Mat<const R: usize, const C: usize, T: Copy = f32, S: StoreOrder = RowMajor> {
    data: S::Storage<T, R, C>,
}

impl<const R: usize, const C: usize, T: Copy, S: StoreOrder> Clone for Mat<R, C, T, S> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<const R: usize, const C: usize, T: Copy, S: StoreOrder> Copy for Mat<R, C, T, S> {}

impl<const R: usize, const C: usize, T: Scalar, S: StoreOrder> Default for Mat<R, C, T, S> {
    #[inline]
    fn default() -> Self {
        Self::zero()
    }
}

impl<const R: usize, const C: usize, T: Copy + PartialEq, S: StoreOrder> PartialEq for Mat<R, C, T, S> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<const R: usize, const C: usize, T: Copy, S: StoreOrder> Mat<R, C, T, S> {
    #[inline]
    pub fn splat(value: T) -> Self {
        Self { data: S::splat(value) }
    }

    /// Builds a matrix from an element function of `(row, column)`.
    pub fn from_fn(mut element: impl FnMut(usize, usize) -> T) -> Self {
        let mut result = Self::splat(element(0, 0));
        for row in 0..R {
            for column in 0..C {
                *result.at_mut(row, column) = element(row, column);
            }
        }
        result
    }

    /// Builds a matrix from rows written in reading order, whatever the [`StoreOrder`].
    #[inline]
    pub fn from_rows(rows: [[T; C]; R]) -> Self {
        Self::from_fn(|row, column| rows[row][column])
    }

    /// Reads `R * C` elements that are already in this matrix's memory layout, or `None` for any other length.
    pub fn from_slice(raw_data: &[T]) -> Option<Self> {
        if raw_data.len() != R * C {
            return None;
        }
        let mut result = Self::splat(*raw_data.first()?);
        result.as_mut_slice().copy_from_slice(raw_data);
        Some(result)
    }

    #[inline]
    pub const fn row_count() -> usize {
        R
    }

    #[inline]
    pub const fn columns_count() -> usize {
        C
    }

    #[inline]
    pub const fn size() -> MatSize {
        MatSize { rows: R, columns: C }
    }

    #[inline]
    pub const fn store_ordering() -> MatStoreType {
        S::STORE_TYPE
    }

    /// Panics when out of range. The C++ version only checks in debug builds.
    #[inline]
    pub fn at(&self, row: usize, column: usize) -> T {
        *S::get::<T, R, C>(&self.data, row, column)
    }

    #[inline]
    pub fn at_mut(&mut self, row: usize, column: usize) -> &mut T {
        S::get_mut::<T, R, C>(&mut self.data, row, column)
    }

    /// The elements in memory order. Suitable for passing to a graphics API.
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        S::flat::<T, R, C>(&self.data)
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        S::flat_mut::<T, R, C>(&mut self.data)
    }

    #[inline]
    pub fn set(&mut self, value: T) {
        self.as_mut_slice().fill(value);
    }

    #[must_use]
    pub fn transposed(&self) -> Mat<C, R, T, S> {
        Mat::from_fn(|row, column| self.at(column, row))
    }

    /// Removes one row and one column. `R2` and `C2` must be `R - 1` and `C - 1`, which stable Rust cannot spell in the
    /// return type, so the mismatch is a compile-time error here instead.
    #[must_use]
    pub fn strip<const R2: usize, const C2: usize>(&self, row: usize, column: usize) -> Mat<R2, C2, T, S> {
        const {
            assert!(R2 + 1 == R && C2 + 1 == C, "strip must remove exactly one row and one column");
            assert!(R2 > 0 && C2 > 0, "cannot strip a matrix down to nothing");
        }
        Mat::from_fn(|r, c| self.at(if r < row { r } else { r + 1 }, if c < column { c } else { c + 1 }))
    }
}

impl<const R: usize, const C: usize, T: Scalar, S: StoreOrder> Mat<R, C, T, S> {
    #[inline]
    pub fn zero() -> Self {
        Self::splat(T::ZERO)
    }

    pub fn sum(&self) -> T {
        self.as_slice().iter().fold(T::ZERO, |sum, &value| sum + value)
    }

    #[inline]
    pub fn clear(&mut self) {
        self.set(T::ZERO);
    }

    /// Numeric cast of every element, with the same caveat as `Vector3::cast`.
    pub fn cast<U: Scalar>(&self) -> Mat<R, C, U, S> {
        Mat::from_fn(|row, column| U::from_f64(self.at(row, column).to_f64()))
    }

    /// Cofactor expansion along the topmost remaining row. Rows and columns still in play are bitmasks, so no
    /// intermediate matrices are built.
    fn determinant_of(&self, rows: u64, columns: u64) -> T
    where
        T: Signed,
    {
        let row = rows.trailing_zeros() as usize;
        let remaining_rows = rows & (rows - 1);

        if remaining_rows == 0 {
            return self.at(row, columns.trailing_zeros() as usize);
        }

        let mut determinant = T::ZERO;
        let mut negate = false;
        for column in 0..C {
            if columns & (1 << column) == 0 {
                continue;
            }
            let minor = self.determinant_of(remaining_rows, columns & !(1 << column));
            let cofactor = self.at(row, column) * if negate { -minor } else { minor };
            determinant += cofactor;
            negate = !negate;
        }
        determinant
    }

    /// Panics at compile time for non-square matrices.
    pub fn determinant(&self) -> T
    where
        T: Signed,
    {
        const { assert!(R == C, "determinant is only defined for square matrices") };
        self.determinant_of(Self::all_bits(R), Self::all_bits(C))
    }

    #[inline]
    const fn all_bits(count: usize) -> u64 {
        if count >= 64 { u64::MAX } else { (1 << count) - 1 }
    }

    pub fn minor(&self, row: usize, column: usize) -> T
    where
        T: Signed,
    {
        self.determinant_of(Self::all_bits(R) & !(1 << row), Self::all_bits(C) & !(1 << column))
    }

    pub fn alg_complement(&self, row: usize, column: usize) -> T
    where
        T: Signed,
    {
        let minor = self.minor(row, column);
        if (row + column) % 2 == 0 { minor } else { -minor }
    }
}

impl<const R: usize, const C: usize, T: Real, S: StoreOrder> Mat<R, C, T, S> {
    /// The inverse, or `None` for a singular matrix.
    pub fn inverted(&self) -> Option<Self> {
        const { assert!(R == C, "only square matrices can be inverted") };

        // 4x4 is by far the most common size here: every view, projection and view-projection matrix.
        if R == 4 {
            return self.inverted_4x4();
        }

        let determinant = self.determinant();

        if determinant.abs_value() < T::EPSILON {
            return None;
        }

        let transposed = self.transposed();
        Some(Self::from_fn(|row, column| transposed.alg_complement(row, column)) / determinant)
    }

    /// Closed form 4x4 inverse. The six 2x2 minors of the top two rows and the six of the bottom two rows are each
    /// shared between the determinant and four entries of the adjugate, so computing them once covers the whole matrix.
    fn inverted_4x4(&self) -> Option<Self> {
        let a = |row: usize, column: usize| self.at(row, column);

        let top_01 = a(0, 0) * a(1, 1) - a(1, 0) * a(0, 1);
        let top_02 = a(0, 0) * a(1, 2) - a(1, 0) * a(0, 2);
        let top_03 = a(0, 0) * a(1, 3) - a(1, 0) * a(0, 3);
        let top_12 = a(0, 1) * a(1, 2) - a(1, 1) * a(0, 2);
        let top_13 = a(0, 1) * a(1, 3) - a(1, 1) * a(0, 3);
        let top_23 = a(0, 2) * a(1, 3) - a(1, 2) * a(0, 3);

        let bot_01 = a(2, 0) * a(3, 1) - a(3, 0) * a(2, 1);
        let bot_02 = a(2, 0) * a(3, 2) - a(3, 0) * a(2, 2);
        let bot_03 = a(2, 0) * a(3, 3) - a(3, 0) * a(2, 3);
        let bot_12 = a(2, 1) * a(3, 2) - a(3, 1) * a(2, 2);
        let bot_13 = a(2, 1) * a(3, 3) - a(3, 1) * a(2, 3);
        let bot_23 = a(2, 2) * a(3, 3) - a(3, 2) * a(2, 3);

        let determinant =
            top_01 * bot_23 - top_02 * bot_13 + top_03 * bot_12 + top_12 * bot_03 - top_13 * bot_02 + top_23 * bot_01;

        if determinant.abs_value() < T::EPSILON {
            return None;
        }

        let inv = T::ONE / determinant;
        let adjugate = [
            [
                a(1, 1) * bot_23 - a(1, 2) * bot_13 + a(1, 3) * bot_12,
                -a(0, 1) * bot_23 + a(0, 2) * bot_13 - a(0, 3) * bot_12,
                a(3, 1) * top_23 - a(3, 2) * top_13 + a(3, 3) * top_12,
                -a(2, 1) * top_23 + a(2, 2) * top_13 - a(2, 3) * top_12,
            ],
            [
                -a(1, 0) * bot_23 + a(1, 2) * bot_03 - a(1, 3) * bot_02,
                a(0, 0) * bot_23 - a(0, 2) * bot_03 + a(0, 3) * bot_02,
                -a(3, 0) * top_23 + a(3, 2) * top_03 - a(3, 3) * top_02,
                a(2, 0) * top_23 - a(2, 2) * top_03 + a(2, 3) * top_02,
            ],
            [
                a(1, 0) * bot_13 - a(1, 1) * bot_03 + a(1, 3) * bot_01,
                -a(0, 0) * bot_13 + a(0, 1) * bot_03 - a(0, 3) * bot_01,
                a(3, 0) * top_13 - a(3, 1) * top_03 + a(3, 3) * top_01,
                -a(2, 0) * top_13 + a(2, 1) * top_03 - a(2, 3) * top_01,
            ],
            [
                -a(1, 0) * bot_12 + a(1, 1) * bot_02 - a(1, 2) * bot_01,
                a(0, 0) * bot_12 - a(0, 1) * bot_02 + a(0, 2) * bot_01,
                -a(3, 0) * top_12 + a(3, 1) * top_02 - a(3, 2) * top_01,
                a(2, 0) * top_12 - a(2, 1) * top_02 + a(2, 2) * top_01,
            ],
        ];

        Some(Self::from_fn(|row, column| adjugate[row][column] * inv))
    }
}

impl<const R: usize, const C: usize, T: Copy, S: StoreOrder> Index<(usize, usize)> for Mat<R, C, T, S> {
    type Output = T;

    #[inline]
    fn index(&self, (row, column): (usize, usize)) -> &T {
        S::get::<T, R, C>(&self.data, row, column)
    }
}

impl<const R: usize, const C: usize, T: Copy, S: StoreOrder> IndexMut<(usize, usize)> for Mat<R, C, T, S> {
    #[inline]
    fn index_mut(&mut self, (row, column): (usize, usize)) -> &mut T {
        self.at_mut(row, column)
    }
}

impl<const R: usize, const C: usize, const O: usize, T: Scalar, S: StoreOrder> Mul<Mat<C, O, T, S>>
    for Mat<R, C, T, S>
{
    type Output = Mat<R, O, T, S>;

    /// The element loops are ordered by storage so that LLVM can vectorize them; this replaces the C++ AVX2 intrinsics.
    fn mul(self, other: Mat<C, O, T, S>) -> Mat<R, O, T, S> {
        let mut result = Mat::<R, O, T, S>::zero();
        match S::STORE_TYPE {
            MatStoreType::RowMajor => {
                for row in 0..R {
                    for column in 0..C {
                        let current = self.at(row, column);
                        for other_column in 0..O {
                            *result.at_mut(row, other_column) += current * other.at(column, other_column);
                        }
                    }
                }
            }
            MatStoreType::ColumnMajor => {
                for other_column in 0..O {
                    for column in 0..C {
                        let current = other.at(column, other_column);
                        for row in 0..R {
                            *result.at_mut(row, other_column) += self.at(row, column) * current;
                        }
                    }
                }
            }
        }
        result
    }
}

impl<const N: usize, T: Scalar, S: StoreOrder> MulAssign for Mat<N, N, T, S> {
    #[inline]
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}

impl<const R: usize, const C: usize, T: Scalar, S: StoreOrder> MulAssign<T> for Mat<R, C, T, S> {
    fn mul_assign(&mut self, value: T) {
        self.as_mut_slice().iter_mut().for_each(|element| *element *= value);
    }
}

impl<const R: usize, const C: usize, T: Scalar, S: StoreOrder> DivAssign<T> for Mat<R, C, T, S> {
    fn div_assign(&mut self, value: T) {
        self.as_mut_slice().iter_mut().for_each(|element| *element /= value);
    }
}

impl<const R: usize, const C: usize, T: Scalar, S: StoreOrder> Mul<T> for Mat<R, C, T, S> {
    type Output = Self;

    #[inline]
    fn mul(mut self, value: T) -> Self {
        self *= value;
        self
    }
}

impl<const R: usize, const C: usize, T: Scalar, S: StoreOrder> Div<T> for Mat<R, C, T, S> {
    type Output = Self;

    #[inline]
    fn div(mut self, value: T) -> Self {
        self /= value;
        self
    }
}

impl<const R: usize, const C: usize, T: Scalar, S: StoreOrder> fmt::Display for Mat<R, C, T, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[[")?;
        for row in 0..R {
            if row > 0 {
                f.write_str(" [")?;
            }
            for column in 0..C {
                write!(f, "{:9.3}", self.at(row, column).to_f64())?;
                if column != C - 1 {
                    f.write_str(", ")?;
                }
            }
            f.write_str(if row == R - 1 { "]]" } else { "]\n" })?;
        }
        Ok(())
    }
}

impl<const R: usize, const C: usize, T: Scalar, S: StoreOrder> fmt::Debug for Mat<R, C, T, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl<T: Scalar, S: StoreOrder> Mat<1, 4, T, S> {
    #[inline]
    pub fn row_from_vector(vector: Vector3<T>) -> Self {
        Self::from_rows([[vector.x, vector.y, vector.z, T::ONE]])
    }
}

impl<T: Scalar, S: StoreOrder> Mat<4, 1, T, S> {
    #[inline]
    pub fn column_from_vector(vector: Vector3<T>) -> Self {
        Self::from_rows([[vector.x], [vector.y], [vector.z], [T::ONE]])
    }
}

impl<T: Scalar, S: StoreOrder> Mat<4, 4, T, S> {
    pub fn to_screen(screen_width: T, screen_height: T) -> Self
    where
        T: Signed,
    {
        let (o, one) = (T::ZERO, T::ONE);
        let half = T::ONE + T::ONE;
        Self::from_rows([
            [screen_width / half, o, o, o],
            [o, -screen_height / half, o, o],
            [o, o, one, o],
            [screen_width / half, screen_height / half, o, one],
        ])
    }

    /// Applies the rotation part of a transform to a vector, touching only the upper left 3x3 block.
    #[inline]
    pub fn rotate_vector(&self, vector: Vector3<T>) -> Vector3<T> {
        Vector3::new(
            self.at(0, 0) * vector.x + self.at(0, 1) * vector.y + self.at(0, 2) * vector.z,
            self.at(1, 0) * vector.x + self.at(1, 1) * vector.y + self.at(1, 2) * vector.z,
            self.at(2, 0) * vector.x + self.at(2, 1) * vector.y + self.at(2, 2) * vector.z,
        )
    }

    pub fn translation(diff: Vector3<T>) -> Self {
        let (o, one) = (T::ZERO, T::ONE);
        Self::from_rows([[one, o, o, diff.x], [o, one, o, diff.y], [o, o, one, diff.z], [o, o, o, one]])
    }

    pub fn scale(scale: Vector3<T>) -> Self {
        let (o, one) = (T::ZERO, T::ONE);
        Self::from_rows([[scale.x, o, o, o], [o, scale.y, o, o], [o, o, scale.z, o], [o, o, o, one]])
    }

    #[inline]
    pub fn extract_origin(&self) -> Vector3<T> {
        Vector3::new(self.at(0, 3), self.at(1, 3), self.at(2, 3))
    }

    pub fn camera_view(forward: Vector3<T>, right: Vector3<T>, up: Vector3<T>, camera_origin: Vector3<T>) -> Self
    where
        T: Signed,
    {
        let (o, one) = (T::ZERO, T::ONE);
        // Closed form of the basis matrix times Mat::translation(-camera_origin).
        Self::from_rows([
            [right.x, right.y, right.z, -right.dot(camera_origin)],
            [up.x, up.y, up.z, -up.dot(camera_origin)],
            [forward.x, forward.y, forward.z, -forward.dot(camera_origin)],
            [o, o, o, one],
        ])
    }
}

impl<T: Real, S: StoreOrder> Mat<4, 4, T, S> {
    pub fn extract_scale(&self) -> Vector3<T> {
        let column_length = |column: usize| {
            let (x, y, z) = (self.at(0, column), self.at(1, column), self.at(2, column));
            (x * x + y * y + z * z).sqrt()
        };
        let or_one = |scale: T| if scale.abs_value() < T::EPSILON { T::ONE } else { scale };

        Vector3::new(or_one(column_length(0)), or_one(column_length(1)), or_one(column_length(2)))
    }

    #[inline]
    fn singularity_epsilon() -> T {
        T::EPSILON * T::from_f64(8.0)
    }

    pub fn extract_rotation_zyx(&self) -> Vector3<T> {
        let scale = self.extract_scale();
        let m00 = self.at(0, 0) / scale.x;
        let m01 = self.at(0, 1) / scale.y;
        let m02 = self.at(0, 2) / scale.z;
        let m10 = self.at(1, 0) / scale.x;
        let m20 = self.at(2, 0) / scale.x;
        let m21 = self.at(2, 1) / scale.y;
        let m22 = self.at(2, 2) / scale.z;

        let y = angles::radians_to_degrees((-m20).clamped(-T::ONE, T::ONE).asin());

        // At y = +/-90 degrees cos(y) is 0, so the x and z rotations act around the same axis and only x - z (or x + z)
        // is recoverable. Pin z to 0 and fold the whole rotation into x, using the surviving m01/m02 pair.
        if T::ONE - m20.abs_value() < Self::singularity_epsilon() {
            let x = if m20 < T::ZERO { m01.atan2(m02) } else { (-m01).atan2(-m02) };
            return Vector3::new(angles::radians_to_degrees(x), y, T::ZERO);
        }

        Vector3::new(angles::radians_to_degrees(m21.atan2(m22)), y, angles::radians_to_degrees(m10.atan2(m00)))
    }

    /// Inverse of [`Mat::rotation_yxz`]: `(x, y, z)` in degrees with `x` in `[-90, 90]`.
    pub fn extract_rotation_yxz(&self) -> Vector3<T> {
        let scale = self.extract_scale();
        let m00 = self.at(0, 0) / scale.x;
        let m02 = self.at(0, 2) / scale.z;
        let m10 = self.at(1, 0) / scale.x;
        let m11 = self.at(1, 1) / scale.y;
        let m12 = self.at(1, 2) / scale.z;
        let m20 = self.at(2, 0) / scale.x;
        let m22 = self.at(2, 2) / scale.z;

        let x = angles::radians_to_degrees((-m12).clamped(-T::ONE, T::ONE).asin());

        // At x = +/-90 degrees the y and z rotations act around the same axis. Pin z to 0 and fold it all into y.
        if T::ONE - m12.abs_value() < Self::singularity_epsilon() {
            return Vector3::new(x, angles::radians_to_degrees((-m20).atan2(m00)), T::ZERO);
        }

        Vector3::new(x, angles::radians_to_degrees(m02.atan2(m22)), angles::radians_to_degrees(m10.atan2(m11)))
    }

    /// Inverse of [`Mat::rotation_zxy`]: `(x, y, z)` in degrees with `x` in `[-90, 90]`.
    pub fn extract_rotation_zxy(&self) -> Vector3<T> {
        let scale = self.extract_scale();
        let m00 = self.at(0, 0) / scale.x;
        let m01 = self.at(0, 1) / scale.y;
        let m10 = self.at(1, 0) / scale.x;
        let m11 = self.at(1, 1) / scale.y;
        let m20 = self.at(2, 0) / scale.x;
        let m21 = self.at(2, 1) / scale.y;
        let m22 = self.at(2, 2) / scale.z;

        let x = angles::radians_to_degrees(m21.clamped(-T::ONE, T::ONE).asin());

        // At x = +/-90 degrees the z and y rotations act around the same axis. Pin y to 0 and fold it all into z.
        if T::ONE - m21.abs_value() < Self::singularity_epsilon() {
            return Vector3::new(x, T::ZERO, angles::radians_to_degrees(m10.atan2(m00)));
        }

        Vector3::new(x, angles::radians_to_degrees((-m20).atan2(m22)), angles::radians_to_degrees((-m01).atan2(m11)))
    }

    pub fn rotation_axis_x(angle: impl Trig<T>) -> Self {
        let (o, one, sin, cos) = (T::ZERO, T::ONE, angle.sin(), angle.cos());
        Self::from_rows([[one, o, o, o], [o, cos, -sin, o], [o, sin, cos, o], [o, o, o, one]])
    }

    pub fn rotation_axis_y(angle: impl Trig<T>) -> Self {
        let (o, one, sin, cos) = (T::ZERO, T::ONE, angle.sin(), angle.cos());
        Self::from_rows([[cos, o, sin, o], [o, one, o, o], [-sin, o, cos, o], [o, o, o, one]])
    }

    pub fn rotation_axis_z(angle: impl Trig<T>) -> Self {
        let (o, one, sin, cos) = (T::ZERO, T::ONE, angle.sin(), angle.cos());
        Self::from_rows([[cos, -sin, o, o], [sin, cos, o, o], [o, o, one, o], [o, o, o, one]])
    }

    /// Closed form of `rotation_axis_z(z) * rotation_axis_y(y) * rotation_axis_x(x)`.
    pub fn rotation_zyx(z: impl Trig<T>, y: impl Trig<T>, x: impl Trig<T>) -> Self {
        let (sin_z, cos_z, sin_y, cos_y, sin_x, cos_x) = (z.sin(), z.cos(), y.sin(), y.cos(), x.sin(), x.cos());
        let (o, one) = (T::ZERO, T::ONE);

        Self::from_rows([
            [cos_z * cos_y, cos_z * sin_y * sin_x - sin_z * cos_x, cos_z * sin_y * cos_x + sin_z * sin_x, o],
            [sin_z * cos_y, sin_z * sin_y * sin_x + cos_z * cos_x, sin_z * sin_y * cos_x - cos_z * sin_x, o],
            [-sin_y, cos_y * sin_x, cos_y * cos_x, o],
            [o, o, o, one],
        ])
    }

    /// Closed form of `rotation_axis_y(y) * rotation_axis_x(x) * rotation_axis_z(z)`: yaw * pitch * roll for Y-up
    /// engines, with the roll turning around the camera's own forward (Z) axis.
    pub fn rotation_yxz(y: impl Trig<T>, x: impl Trig<T>, z: impl Trig<T>) -> Self {
        let (sin_y, cos_y, sin_x, cos_x, sin_z, cos_z) = (y.sin(), y.cos(), x.sin(), x.cos(), z.sin(), z.cos());
        let (o, one) = (T::ZERO, T::ONE);

        Self::from_rows([
            [cos_y * cos_z + sin_y * sin_x * sin_z, -cos_y * sin_z + sin_y * sin_x * cos_z, sin_y * cos_x, o],
            [cos_x * sin_z, cos_x * cos_z, -sin_x, o],
            [-sin_y * cos_z + cos_y * sin_x * sin_z, sin_y * sin_z + cos_y * sin_x * cos_z, cos_y * cos_x, o],
            [o, o, o, one],
        ])
    }

    /// Closed form of `rotation_axis_z(z) * rotation_axis_x(x) * rotation_axis_y(y)`: yaw * pitch * roll for Z-up
    /// engines whose forward is Y, with the roll turning around the camera's own forward (Y) axis.
    pub fn rotation_zxy(z: impl Trig<T>, x: impl Trig<T>, y: impl Trig<T>) -> Self {
        let (sin_z, cos_z, sin_x, cos_x, sin_y, cos_y) = (z.sin(), z.cos(), x.sin(), x.cos(), y.sin(), y.cos());
        let (o, one) = (T::ZERO, T::ONE);

        Self::from_rows([
            [cos_z * cos_y - sin_z * sin_x * sin_y, -sin_z * cos_x, cos_z * sin_y + sin_z * sin_x * cos_y, o],
            [sin_z * cos_y + cos_z * sin_x * sin_y, cos_z * cos_x, sin_z * sin_y - cos_z * sin_x * cos_y, o],
            [-cos_x * sin_y, sin_x, cos_x * cos_y, o],
            [o, o, o, one],
        ])
    }

    /// `z_sign` is `1` for left-handed and `-1` for right-handed. Both handedness variants differ only there.
    fn perspective(x_axis: T, y_axis: T, near: T, far: T, z_sign: T, depth_range: NdcDepthRange) -> Self {
        let o = T::ZERO;
        let (z_scale, z_offset) = match depth_range {
            NdcDepthRange::ZeroToOne => (z_sign * far / (far - near), -(near * far) / (far - near)),
            NdcDepthRange::NegativeOneToOne => {
                (z_sign * (far + near) / (far - near), -(T::from_f64(2.0) * near * far) / (far - near))
            }
        };

        Self::from_rows([[x_axis, o, o, o], [o, y_axis, o, o], [o, o, z_scale, z_offset], [o, o, z_sign, o]])
    }

    fn vertical_fov_axes(field_of_view: T, aspect_ratio: T) -> (T, T) {
        let fov_half_tan = (angles::degrees_to_radians(field_of_view) / T::from_f64(2.0)).tan();
        (T::ONE / (aspect_ratio * fov_half_tan), T::ONE / fov_half_tan)
    }

    /// For engines that report the FOV as horizontal (UE's `FMinimalViewInfo::FOV`, Quake-family `fov_x`, etc.).
    fn horizontal_fov_axes(horizontal_fov: T, aspect_ratio: T) -> (T, T) {
        let inv_tan_half_hfov = T::ONE / (angles::degrees_to_radians(horizontal_fov) / T::from_f64(2.0)).tan();
        (inv_tan_half_hfov, inv_tan_half_hfov * aspect_ratio)
    }

    pub fn perspective_left_handed_vertical_fov(
        field_of_view: T,
        aspect_ratio: T,
        near: T,
        far: T,
        depth_range: NdcDepthRange,
    ) -> Self {
        let (x_axis, y_axis) = Self::vertical_fov_axes(field_of_view, aspect_ratio);
        Self::perspective(x_axis, y_axis, near, far, T::ONE, depth_range)
    }

    pub fn perspective_right_handed_vertical_fov(
        field_of_view: T,
        aspect_ratio: T,
        near: T,
        far: T,
        depth_range: NdcDepthRange,
    ) -> Self {
        let (x_axis, y_axis) = Self::vertical_fov_axes(field_of_view, aspect_ratio);
        Self::perspective(x_axis, y_axis, near, far, -T::ONE, depth_range)
    }

    pub fn perspective_left_handed_horizontal_fov(
        horizontal_fov: T,
        aspect_ratio: T,
        near: T,
        far: T,
        depth_range: NdcDepthRange,
    ) -> Self {
        let (x_axis, y_axis) = Self::horizontal_fov_axes(horizontal_fov, aspect_ratio);
        Self::perspective(x_axis, y_axis, near, far, T::ONE, depth_range)
    }

    pub fn perspective_right_handed_horizontal_fov(
        horizontal_fov: T,
        aspect_ratio: T,
        near: T,
        far: T,
        depth_range: NdcDepthRange,
    ) -> Self {
        let (x_axis, y_axis) = Self::horizontal_fov_axes(horizontal_fov, aspect_ratio);
        Self::perspective(x_axis, y_axis, near, far, -T::ONE, depth_range)
    }

    /// `z_sign` is `1` for left-handed and `-1` for right-handed.
    fn ortho(left: T, right: T, bottom: T, top: T, near: T, far: T, z_sign: T, depth_range: NdcDepthRange) -> Self {
        let (o, one) = (T::ZERO, T::ONE);
        let two = T::from_f64(2.0);
        let (z_scale, z_offset) = match depth_range {
            NdcDepthRange::ZeroToOne => (z_sign / (far - near), -near / (far - near)),
            NdcDepthRange::NegativeOneToOne => (z_sign * two / (far - near), -(far + near) / (far - near)),
        };

        Self::from_rows([
            [two / (right - left), o, o, -(right + left) / (right - left)],
            [o, two / (top - bottom), o, -(top + bottom) / (top - bottom)],
            [o, o, z_scale, z_offset],
            [o, o, o, one],
        ])
    }

    pub fn ortho_left_handed(
        left: T,
        right: T,
        bottom: T,
        top: T,
        near: T,
        far: T,
        depth_range: NdcDepthRange,
    ) -> Self {
        Self::ortho(left, right, bottom, top, near, far, T::ONE, depth_range)
    }

    pub fn ortho_right_handed(
        left: T,
        right: T,
        bottom: T,
        top: T,
        near: T,
        far: T,
        depth_range: NdcDepthRange,
    ) -> Self {
        Self::ortho(left, right, bottom, top, near, far, -T::ONE, depth_range)
    }

    pub fn look_at_left_handed(eye: Vector3<T>, center: Vector3<T>, up: Vector3<T>) -> Self {
        let f = (center - eye).normalized();
        let s = f.cross(up).normalized();
        let u = s.cross(f);
        Self::camera_view(f, s, u, eye)
    }

    pub fn look_at_right_handed(eye: Vector3<T>, center: Vector3<T>, up: Vector3<T>) -> Self {
        let f = (center - eye).normalized();
        let s = f.cross(up).normalized();
        let u = s.cross(f);
        Self::camera_view(-f, s, u, eye)
    }
}
