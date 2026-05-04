pub mod taxonomy;
pub mod tagio;
pub mod scan;

pub use taxonomy::{Classification, Taxonomy, normalize};

#[cfg(test)]
mod tests;
