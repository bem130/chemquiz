pub mod catalog;
pub mod catalog_manifest;
pub mod compound;
pub mod demo;
pub mod quiz;
#[cfg(target_arch = "wasm32")]
pub mod wasm;

pub use catalog::{Catalog, CatalogEntry, CatalogError, CatalogLoadError};
pub use catalog_manifest::{CatalogLeaf, CatalogManifest, CatalogNode};
pub use compound::Compound;
pub use demo::{DEMO_OPTION_COUNT, demo_catalog, demo_compounds};
pub use quiz::{QuizError, QuizItem, QuizMode, generate_quiz};
