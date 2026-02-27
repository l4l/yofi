use yofi::config::Config;
use yofi::mode::Mode;
use yofi::state::State;
use yofi::window::Params;

pub enum Action {
    Type(&'static str),
    NextItem,
}

pub fn test_entries() -> Vec<String> {
    [
        "Firefox",
        "Chromium",
        "Terminal",
        "Files",
        "Settings",
        "Calculator",
        "Text Editor",
        "Music Player",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

fn unpremultiply_to_rgba(buffer: &[u32]) -> Vec<u8> {
    let mut rgba = Vec::with_capacity(buffer.len() * 4);
    for &px in buffer {
        let a = (px >> 24) & 0xff;
        let r = (px >> 16) & 0xff;
        let g = (px >> 8) & 0xff;
        let b = px & 0xff;
        if a == 0 {
            rgba.extend_from_slice(&[0, 0, 0, 0]);
        } else {
            rgba.push((r * 255 / a) as u8);
            rgba.push((g * 255 / a) as u8);
            rgba.push((b * 255 / a) as u8);
            rgba.push(a as u8);
        }
    }
    rgba
}

fn save_png(path: &str, width: u32, height: u32, rgba: &[u8]) {
    let file = std::fs::File::create(path).expect("failed to create PNG file");
    let mut encoder = png::Encoder::new(file, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().expect("failed to write PNG header");
    writer
        .write_image_data(rgba)
        .expect("failed to write PNG data");
}

fn make_diff_image(expected: &[u8], actual: &[u8], width: u32, height: u32) -> Vec<u8> {
    let mut diff = Vec::with_capacity(expected.len());
    for (exp, act) in expected.chunks_exact(4).zip(actual.chunks_exact(4)) {
        if exp == act {
            // Matching pixel: dim grayscale
            let gray = ((exp[0] as u16 + exp[1] as u16 + exp[2] as u16) / 3 / 3) as u8;
            diff.extend_from_slice(&[gray, gray, gray, 255]);
        } else {
            // Differing pixel: bright red
            diff.extend_from_slice(&[255, 0, 0, 255]);
        }
    }
    // Pad if sizes differ (dimension mismatch)
    let total = (width as usize) * (height as usize) * 4;
    diff.resize(total, 0);
    diff
}

fn load_png_rgba(path: &str) -> (u32, u32, Vec<u8>) {
    let file = std::fs::File::open(path).unwrap_or_else(|_| {
        panic!("fixture not found: {path}\nRun with YOFI_BLESS=1 to generate reference snapshots")
    });
    let decoder = png::Decoder::new(file);
    let mut reader = decoder.read_info().unwrap();
    let mut rgba = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut rgba).unwrap();
    rgba.truncate(info.buffer_size());
    (info.width, info.height, rgba)
}

pub fn run_regression(name: &str, entries: Vec<String>, actions: &[Action]) {
    let config = Config::default();

    let mode = Mode::dialog_from_lines(entries);
    let mut state = State::new(mode);

    for action in actions {
        match action {
            Action::Type(s) => state.append_to_input(s),
            Action::NextItem => state.next_item(),
        }
    }

    let params: Params = config.param();
    let scale = params.scale.unwrap_or(1);
    let w = (params.width * u32::from(scale)) as i32;
    let h = (params.height * u32::from(scale)) as i32;
    let mut buffer = vec![0u32; (w * h) as usize];
    yofi::render_to_buffer(&config, &mut state, scale, w, h, &mut buffer);

    let actual_rgba = unpremultiply_to_rgba(&buffer);
    let w = w as u32;
    let h = h as u32;

    let fixture = format!("tests/fixtures/{name}.png");
    let new_file = format!("tests/fixtures/{name}.new.png");
    let diff_file = format!("tests/fixtures/{name}.diff.png");

    if std::env::var("YOFI_BLESS").is_ok() {
        save_png(&fixture, w, h, &actual_rgba);
        eprintln!("blessed: {fixture}");
    } else {
        let (rw, rh, reference_rgba) = load_png_rgba(&fixture);
        assert!(
            rw == w && rh == h,
            "dimension mismatch for '{name}': expected {rw}x{rh}, got {w}x{h}"
        );
        if actual_rgba != reference_rgba {
            save_png(&new_file, w, h, &actual_rgba);
            let diff = make_diff_image(&reference_rgba, &actual_rgba, w, h);
            save_png(&diff_file, w, h, &diff);
            panic!(
                "pixel mismatch for '{name}'\n  \
                 new: {new_file}\n  \
                 diff: {diff_file}\n  \
                 Run with YOFI_BLESS=1 to update"
            );
        }
    }
}
