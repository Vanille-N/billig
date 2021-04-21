pub mod entry;
pub mod instance;
pub mod parse;
pub mod template;
pub mod validate;

#[derive(Debug)]
pub struct Amount(isize);

#[derive(Debug)]
pub struct Tag(String);
