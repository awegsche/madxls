use std::{ops::{AddAssign, Add, SubAssign}, fmt::Display};


#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct CursorPosition {
    absolute: usize,
    line: usize,
}

impl Display for CursorPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.absolute, self.line)
    }
}

impl AddAssign<usize> for CursorPosition {
    fn add_assign(&mut self, rhs: usize) {
        self.absolute += rhs;
    }
}

impl SubAssign<usize> for CursorPosition {
    fn sub_assign(&mut self, rhs: usize) {
        self.absolute -= rhs;
    }
}

impl Add<usize> for CursorPosition {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self {
            absolute: self.absolute + rhs,
            line: self.line,
        }
    }
}

impl Add<usize> for &CursorPosition {
    type Output = CursorPosition;

    fn add(self, rhs: usize) -> Self::Output {
        Self::Output {
            absolute: self.absolute + rhs,
            line: self.line,
        }
    }
}

impl PartialOrd for CursorPosition {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CursorPosition {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.absolute.cmp(&other.absolute)
    }
}

impl CursorPosition {
    pub fn new(absolute: usize, line: usize) -> Self {
        Self {
            absolute,
            line
        }
    }

    pub fn advance_line(&mut self) {
        self.line += 1;
    }

    pub fn character(&self, lines: &[usize]) -> usize {
        self.absolute - lines[self.line]
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub(crate) fn absolute(&self) -> usize {
        self.absolute
    }
}

impl Default for CursorPosition {
    fn default() -> Self {
        Self{
            absolute: 0,
            line: 0
        }
    }
}



