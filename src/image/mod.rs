mod component;
mod cfa;
mod image;

pub use component::{Attribute, Color};
pub use cfa::CFA;
pub use self::image::{Metadata, Image, Component, Sensor, Rgb, Hsv};