use std::fmt;

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub struct CartesianP {
    pub x: i64,
    pub y: i64,
}

impl CartesianP {
    pub fn from(x: i64, y: i64) -> Self {
        CartesianP { x, y }
    }
}

impl fmt::Display for CartesianP {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_cartesian_point() {
        let p = CartesianP::from(-23, 38);
        assert_eq!(p, CartesianP { x: -23, y: 38 });
        assert_eq!(format!("{p}"), "(-23, 38)");
    }
}
