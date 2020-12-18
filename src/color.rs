#[derive(Debug, PartialEq)]
pub enum Color {
	Red,
	Green,
	Blue
}

use std::fmt;

impl fmt::Display for Color {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Color::Red => write!(f, "red"),
			Color::Green => write!(f, "green"),
			Color::Blue => write!(f, "blue")
		}
	}
}

impl From<Color> for usize {
	fn from(c: Color) -> usize{
		match c {
			Color::Red => {
				0
			},
			Color::Green => {
				1
			},
			Color::Blue => {
				2
			}
		}
	}
}