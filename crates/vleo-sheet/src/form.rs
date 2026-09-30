//! The declaration form — what a sheet still needs, and why each answer matters.
//!
//! Nine questions, in the order a person is asked them. `xtask declare` prints
//! them for a terminal and the daemon serves them to the web face, so a person
//! filling a row in a browser is asked exactly what a person at a terminal is
//! asked. A second copy of this list in JavaScript would drift the first week
//! somebody added a field, and the drift would be invisible: both forms would
//! look complete.
//!
//! It lives here because this crate is the single implementation of what a
//! sheet *means*. `unfilled` is the function the generators refuse on, so the
//! form and the generator cannot disagree about what a finished sheet is.

use crate::model::Sheet;

mod arrays;
mod fields;
mod save;
mod text;

pub use arrays::*;
pub use fields::*;
pub use save::*;
pub use text::*;
