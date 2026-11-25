//! Bundled fonts, so a figure renders the same on every machine.
//!
//! DejaVu Sans ([`SANS`]) sets text, DejaVu Sans Mono ([`MONO`]) sets text
//! whose columns line up, each in two weights, and New Computer Modern
//! Math sets math. Each constant carries the bytes of its font file, which
//! keep their own licenses (see this crate's `LICENSE`). rustd3-figure
//! loads them through its default `fonts` feature.
//!
//! ```
//! use rustd3_fonts::{ALL, DEJAVU_SANS, DEJAVU_SANS_MONO, MONO, NEW_CM_MATH, SANS};
//!
//! assert_eq!(DEJAVU_SANS.family(), SANS);
//! assert!(!DEJAVU_SANS.bold());
//! assert_eq!(DEJAVU_SANS.data()[..4], [0, 1, 0, 0]); // TrueType
//! assert_eq!(NEW_CM_MATH.data()[..4], *b"OTTO"); // OpenType with CFF outlines
//! assert_eq!(DEJAVU_SANS_MONO.family(), MONO);
//! assert_eq!(ALL.len(), 5);
//! ```

// A `const` that held `include_bytes!` directly would be copied into every
// crate that uses it; behind one static, a binary holds each file once.
macro_rules! bundled {
    ($file:literal) => {{
        static DATA: [u8; include_bytes!($file).len()] = *include_bytes!($file);
        &DATA
    }};
}

/// The family of the text faces, `"DejaVu Sans"`.
pub const SANS: &str = "DejaVu Sans";

/// The family of the monospaced faces, `"DejaVu Sans Mono"`.
pub const MONO: &str = "DejaVu Sans Mono";

/// One bundled font file.
#[derive(Debug, Clone, Copy)]
pub struct Font {
    family: &'static str,
    bold: bool,
    data: &'static [u8],
}

impl Font {
    /// The family name in the font's `name` table (name id 1).
    pub const fn family(&self) -> &'static str {
        self.family
    }

    /// Whether this is the bold weight of its family.
    pub const fn bold(&self) -> bool {
        self.bold
    }

    /// The font file's bytes, TrueType or OpenType.
    pub const fn data(&self) -> &'static [u8] {
        self.data
    }
}

/// DejaVu Sans 2.37, regular (`DejaVuSans.ttf`).
pub const DEJAVU_SANS: Font = Font {
    family: SANS,
    bold: false,
    data: bundled!("../DejaVuSans.ttf"),
};

/// DejaVu Sans 2.37, bold (`DejaVuSans-Bold.ttf`).
pub const DEJAVU_SANS_BOLD: Font = Font {
    family: SANS,
    bold: true,
    data: bundled!("../DejaVuSans-Bold.ttf"),
};

/// New Computer Modern Math, regular, with the OpenType `MATH` table that
/// math layout reads (`NewCMMath-Regular.otf`).
pub const NEW_CM_MATH: Font = Font {
    family: "NewComputerModernMath",
    bold: false,
    data: bundled!("../NewCMMath-Regular.otf"),
};

/// DejaVu Sans Mono 2.37, regular (`DejaVuSansMono.ttf`): every glyph is
/// as wide as every other, so padded columns of text line up.
pub const DEJAVU_SANS_MONO: Font = Font {
    family: MONO,
    bold: false,
    data: bundled!("../DejaVuSansMono.ttf"),
};

/// DejaVu Sans Mono 2.37, bold (`DejaVuSansMono-Bold.ttf`).
pub const DEJAVU_SANS_MONO_BOLD: Font = Font {
    family: MONO,
    bold: true,
    data: bundled!("../DejaVuSansMono-Bold.ttf"),
};

/// Every bundled font, the monospaced faces last, so that they come after
/// the others in a fallback order.
pub const ALL: [Font; 5] = [
    DEJAVU_SANS,
    DEJAVU_SANS_BOLD,
    NEW_CM_MATH,
    DEJAVU_SANS_MONO,
    DEJAVU_SANS_MONO_BOLD,
];
