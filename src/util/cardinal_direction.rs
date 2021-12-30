
use self::CardinalDirection::*;

#[derive(Copy, Clone)]
pub enum CardinalDirection {
    Up,
    Down,
    Left,
    Right,
}

impl CardinalDirection {
    /// Turn counter-clock-wise
    #[inline]
    pub fn ccw(self) -> CardinalDirection {
        match self {
            Up => Left,
            Down => Right,
            Left => Down,
            Right => Up,
        }
    }

    /// Turn clock-wise
    #[inline]
    pub fn cw(self) -> CardinalDirection {
        match self {
            Up => Right,
            Down => Left,
            Left => Up,
            Right => Down,
        }
    }

    /// Turn 180 Degrees
    #[inline]
    pub fn inv(self) -> CardinalDirection {
        match self {
            Up => Down,
            Down => Up,
            Left => Right,
            Right => Left,
        }
    }

    pub fn index(self) -> usize {
        match self {
            Up => 0,
            Down => 1,
            Left => 2,
            Right => 3,
        }
    }

    pub fn from_index(ind: usize) -> Option<CardinalDirection> {
        match ind {
            0 => Some(Up),
            1 => Some(Down),
            2 => Some(Left),
            3 => Some(Right),
            _ => None,
        }
    }
}
