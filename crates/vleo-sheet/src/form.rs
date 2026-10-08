//! The declaration form — what a sheet still needs, and why each answer matters.
//!
//! Nine questions, in the order a person is asked them. The node form asks
//! them, and the daemon serves them to the web face, from this one list. A
//! second copy of it in JavaScript would drift the first week somebody added
//! a field, and the drift would be invisible: both forms would look complete.
//!
//! It lives here because this crate is the single implementation of what a
//! sheet *means*. `unfilled` is the function the generators refuse on, so the
//! form and the generator cannot disagree about what a finished sheet is.

use crate::model::Sheet;
use crate::{Error, ErrorKind};

// The parts, one file each, under src/form/.
mod ask;
mod blocks;
mod edit;
mod fields;
mod publish;
mod save;
mod view;
mod who;

pub use ask::*;
pub use blocks::*;
pub use edit::*;
pub use fields::*;
pub use publish::*;
pub use save::*;
pub use view::*;
pub use who::*;
