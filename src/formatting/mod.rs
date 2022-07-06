//! Formatting
//!
//! Formatting can be used to declare a style,
//! or used within a document directly.

mod bold;
mod border;
mod borders;
mod character_property;
mod color;
mod dstrike;
mod indent_level;
mod italics;
mod justification;
mod numbering_id;
mod numbering_property;
mod outline;
mod paragraph_property;
mod size;
mod strike;
mod table_borders;
mod table_cell_property;
mod table_indent;
mod table_justification;
mod table_property;
mod table_row_property;
mod table_width;
mod underline;
mod fonts;
mod spacing;
mod line_rule;
mod indent;
mod widow_control;
mod lang;
mod table_margin;
mod margin;

// re-export
pub use self::{
    bold::*, border::*, borders::*, character_property::*, color::*, dstrike::*, fonts::*, indent::*, indent_level::*,
    italics::*, justification::*, lang::*, numbering_id::*, numbering_property::*, outline::*,
    paragraph_property::*, size::*, spacing::*, strike::*, table_borders::*, table_cell_property::*,
    table_indent::*, table_justification::*, table_property::*, table_row_property::*,
    table_width::*, underline::*, widow_control::*,
};
