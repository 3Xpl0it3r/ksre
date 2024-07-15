#[derive(Debug)]
pub enum Error {
    EmptyTree,
    KeyNotFound,
    PageLoadErr,
    Generic,
}
