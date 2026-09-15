use std::{
    fs,
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};

#[derive(Clone, PartialEq)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}
impl Image {
    pub fn write(&self, path: &Path) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut encoder = png::Encoder::new(
            BufWriter::new(fs::File::create(path).unwrap()),
            self.width,
            self.height,
        );
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(&self.pixels)
            .unwrap();
    }
    fn read(path: &Path) -> Self {
        let mut reader = png::Decoder::new(BufReader::new(fs::File::open(path).unwrap()))
            .read_info()
            .unwrap();
        let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut pixels).unwrap();
        assert_eq!(info.color_type, png::ColorType::Rgba);
        pixels.truncate(info.buffer_size());
        Self {
            width: info.width,
            height: info.height,
            pixels,
        }
    }
    pub fn crop(&self, x: u32, y: u32, width: u32, height: u32) -> Self {
        assert!(x + width <= self.width && y + height <= self.height);
        let mut pixels = Vec::new();
        for row in y..y + height {
            let start = ((row * self.width + x) * 4) as usize;
            pixels.extend_from_slice(&self.pixels[start..start + width as usize * 4]);
        }
        Self {
            width,
            height,
            pixels,
        }
    }
}

/// Strict RGBA comparison. Missing baselines fail unless updating explicitly;
/// updates are forbidden in CI. Artifacts remain outside the reference directory.
pub fn check(name: &str, actual: &Image) {
    assert!(
        name.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '/'))
    );
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let expected_path = root
        .join("tests/visual/references")
        .join(name)
        .with_extension("png");
    let artifacts = root.join("target/visual-report").join(name);
    actual.write(&artifacts.join("actual.png"));
    let update = std::env::var("UPDATE_VISUAL_REFERENCES").as_deref() == Ok("1");
    assert!(
        !update || std::env::var_os("CI").is_none(),
        "Reference updates are forbidden in CI"
    );
    if update {
        actual.write(&expected_path);
    }
    assert!(
        expected_path.exists(),
        "Missing visual reference: {}. Review and explicitly generate references; CI never accepts missing images.",
        expected_path.display()
    );
    let expected = Image::read(&expected_path);
    expected.write(&artifacts.join("expected.png"));
    let dimensions_match = actual.width == expected.width && actual.height == expected.height;
    let mut difference = actual.clone();
    let mut changed = 0;
    for (index, pixel) in difference
        .pixels
        .as_chunks_mut::<4>()
        .0
        .iter_mut()
        .enumerate()
    {
        let range = index * 4..index * 4 + 4;
        let same = dimensions_match && actual.pixels[range.clone()] == expected.pixels[range];
        if same {
            pixel.copy_from_slice(&[240, 240, 240, 255]);
        } else {
            pixel.copy_from_slice(&[220, 0, 90, 255]);
            changed += 1;
        }
    }
    difference.write(&artifacts.join("diff.png"));
    fs::write(artifacts.join("index.html"), format!("<!doctype html><meta charset=utf-8><title>{name}</title><style>body{{font:16px system-ui;margin:24px}}main{{display:flex;gap:16px}}figure{{margin:0;min-width:0}}img{{max-width:100%;border:1px solid #ccc}}figcaption{{margin:8px 0}}</style><h1>{name}</h1><p>{changed} changed pixels. Exact RGBA comparison.</p><main><figure><figcaption>Expected</figcaption><img src=expected.png></figure><figure><figcaption>Actual</figcaption><img src=actual.png></figure><figure><figcaption>Difference</figcaption><img src=diff.png></figure></main>")).unwrap();
    assert!(
        dimensions_match && changed == 0,
        "Visual mismatch ({changed} pixels): {}",
        artifacts.join("index.html").display()
    );
}
