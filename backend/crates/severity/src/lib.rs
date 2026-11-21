mod batch;
mod classifier;
mod query;
mod template;

pub use batch::BatchError;
pub use classifier::Classifier;
pub use query::query_classified;
pub use template::{normalize, template_hash};
