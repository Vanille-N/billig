pub mod entry;
pub mod instance;
pub mod parse;
pub mod template;
pub mod validate;

#[derive(Debug, Clone, Copy)]
pub struct Amount(isize);

#[derive(Debug, Clone)]
pub struct Tag(String);
