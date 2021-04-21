pub mod entry;
pub mod instance;
pub mod parse;
pub mod template;
pub mod validate;

#[derive(Debug, Clone, Copy)]
pub struct Amount(pub isize);

#[derive(Debug, Clone)]
pub struct Tag(pub String);

use std::fmt;
impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}E", self.0 / 100, self.0 % 100)
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
