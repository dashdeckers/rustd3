//! poppler's tools on a PDF in memory: `pdftoppm` renders it back,
//! `pdftotext` extracts its text, `pdffonts` lists its fonts. Each reads
//! the PDF from a temporary file; a tool that cannot run fails the test with a message
//! naming poppler-utils.

use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

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

/// The text `pdftotext` extracts, as UTF-8 with `\n` line ends (Windows
/// builds default to the system code page and `\r\n`).
pub fn text(pdf: &[u8]) -> String {
    String::from_utf8(run("pdftotext", &["-enc", "UTF-8", "-", "-"], pdf))
        .expect("pdftotext writes UTF-8")
        .replace("\r\n", "\n")
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

/// What `tool` writes to stdout with `args`, the PDF given as a file
/// where `args` says `"-"`: a temporary file, since the tools read a PDF
/// by seeking, which stdin on Windows does not allow.
fn run(tool: &str, args: &[&str], pdf: &[u8]) -> Vec<u8> {
    static COUNT: AtomicUsize = AtomicUsize::new(0);
    let path = std::env::temp_dir().join(format!(
        "rustd3-poppler-{}-{}.pdf",
        std::process::id(),
        COUNT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::write(&path, pdf).expect("the PDF is written to a temporary file");
    let file = path.to_str().expect("a UTF-8 temporary path");
    // The first `-` is the input; `pdftotext`'s second is its output, stdout.
    let mut input = Some(file);
    let args: Vec<&str> = args
        .iter()
        .map(|a| match (*a, input.take()) {
            ("-", Some(file)) => file,
            (_, taken) => {
                input = input.or(taken);
                a
            }
        })
        .collect();
    let output = Command::new(tool)
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap_or_else(|e| {
            panic!("{tool} cannot run ({e}): the PDF tests need poppler-utils (pdftoppm, pdftotext, pdffonts)")
        });
    let _ = std::fs::remove_file(&path);
    assert!(
        output.status.success(),
        "{tool} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}
