pub mod parse;
pub mod entry;
pub mod validate;
pub mod template;
pub mod instance;

#[derive(Debug)]
pub struct Amount(isize);

#[derive(Debug)]
pub struct Tag(String);
