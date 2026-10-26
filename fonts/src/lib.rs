//! Bundled fonts, so a figure renders the same on every machine.
//!
//! DejaVu Sans in two weights sets text and New Computer Modern Math sets
//! math. Each constant carries the bytes of its font file, which keep
//! their own licences (see this crate's `LICENSE`).

// A `const` that held `include_bytes!` directly would be copied into every
// crate that uses it; behind one static, a binary holds each file once.
macro_rules! bundled {
    ($file:literal) => {{
        static DATA: [u8; include_bytes!($file).len()] = *include_bytes!($file);
        &DATA
    }};
}

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
    family: "DejaVu Sans",
    bold: false,
    data: bundled!("../DejaVuSans.ttf"),
};

/// DejaVu Sans 2.37, bold (`DejaVuSans-Bold.ttf`).
pub const DEJAVU_SANS_BOLD: Font = Font {
    family: "DejaVu Sans",
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

/// Every bundled font.
pub const ALL: [Font; 3] = [DEJAVU_SANS, DEJAVU_SANS_BOLD, NEW_CM_MATH];
