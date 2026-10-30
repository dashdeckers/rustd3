//! poppler's tools on a PDF in memory: `pdftoppm` renders it back,
//! `pdftotext` extracts its text, `pdffonts` lists its fonts. Each reads
//! the PDF on stdin; a tool that cannot run fails the test with a message
//! naming poppler-utils. Shared with rustd3-examples' tests.

use std::io::Write;
use std::process::{Command, Stdio};

use rustd3_figure::raster::Image;

/// A font as `pdffonts` lists it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Font {
    /// The font's name, a subset's six-letter tag included.
    pub name: String,
    /// Whether the file embeds it.
    pub embedded: bool,
    /// Whether it is a subset.
    pub subset: bool,
}

/// The page rendered by `pdftoppm` at `dpi` (96 is one device px per list
/// px), as opaque RGBA.
pub fn render(pdf: &[u8], dpi: f64) -> Image {
    let png = run(
        "pdftoppm",
        &["-r", &dpi.to_string(), "-png", "-singlefile", "-"],
        pdf,
    );
    let mut reader = png::Decoder::new(std::io::Cursor::new(png))
        .read_info()
        .expect("pdftoppm writes a PNG");
    let mut buffer = vec![0; reader.output_buffer_size().expect("a size")];
    let frame = reader.next_frame(&mut buffer).expect("one frame");
    buffer.truncate(frame.buffer_size());
    let rgba = match frame.color_type {
        png::ColorType::Rgb => buffer
            .as_chunks::<3>()
            .0
            .iter()
            .flat_map(|p| [p[0], p[1], p[2], 0xff])
            .collect(),
        png::ColorType::Rgba => buffer,
        other @ (png::ColorType::Grayscale
        | png::ColorType::GrayscaleAlpha
        | png::ColorType::Indexed) => panic!("pdftoppm wrote {other:?} pixels"),
    };
    Image {
        width: frame.width,
        height: frame.height,
        rgba,
    }
}

/// The text `pdftotext` extracts.
pub fn text(pdf: &[u8]) -> String {
    String::from_utf8(run("pdftotext", &["-", "-"], pdf)).expect("pdftotext writes UTF-8")
}

/// The fonts `pdffonts` lists.
pub fn fonts(pdf: &[u8]) -> Vec<Font> {
    let out = String::from_utf8(run("pdffonts", &["-"], pdf)).expect("pdffonts writes UTF-8");
    // name, type (one or more words), encoding, emb, sub, uni, object, generation
    out.lines()
        .skip(2)
        .map(|line| {
            let words: Vec<&str> = line.split_whitespace().collect();
            let n = words.len();
            assert!(n >= 7, "a pdffonts row: {line}");
            Font {
                name: words[0].to_string(),
                embedded: words[n - 5] == "yes",
                subset: words[n - 4] == "yes",
            }
        })
        .collect()
}

/// What `tool` writes to stdout with `args` and `pdf` on stdin.
fn run(tool: &str, args: &[&str], pdf: &[u8]) -> Vec<u8> {
    let mut child = Command::new(tool)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| {
            panic!("{tool} cannot run ({e}): the PDF tests need poppler-utils (pdftoppm, pdftotext, pdffonts)")
        });
    let mut stdin = child.stdin.take().expect("a piped stdin");
    let input = pdf.to_vec();
    let writer = std::thread::spawn(move || stdin.write_all(&input));
    let output = child.wait_with_output().expect("the tool runs");
    writer
        .join()
        .expect("the writer thread")
        .expect("the PDF reaches the tool");
    assert!(
        output.status.success(),
        "{tool} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}
