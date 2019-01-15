#[macro_use]
mod common;

pub mod eager;
pub mod full;

pub use self::eager::{EagerStateMachine, TagHintSink};
pub use self::full::{FullStateMachine, LexemeSink};
