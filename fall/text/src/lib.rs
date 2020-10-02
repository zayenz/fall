extern crate itertools;
extern crate proptest;
extern crate rand;
extern crate serde;

pub mod prop;
mod text;
mod text_buf;
mod text_edit;
mod text_range;
mod text_slice;
mod text_unit;

pub use crate::text::Text;
pub use crate::text_buf::TextBuf;
pub use crate::text_edit::{TextEdit, TextEditBuilder, TextEditOp};
pub use crate::text_range::TextRange;
pub use crate::text_slice::TextSuffix;
pub use crate::text_unit::{tu, TextUnit};
