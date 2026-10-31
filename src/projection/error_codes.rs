use std::fmt;

/// Failure of a projection between world and screen space.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum Error {
    WorldPositionIsOutOfScreenBounds,
    InvViewProjMatDetEqZero,
    PerspectiveDividerLessEqZero,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::WorldPositionIsOutOfScreenBounds => "world position is out of screen bounds",
            Self::InvViewProjMatDetEqZero => "view-projection matrix is not invertible",
            Self::PerspectiveDividerLessEqZero => {
                "perspective divider is less than or equal to zero"
            }
        })
    }
}

impl std::error::Error for Error {}
