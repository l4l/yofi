use std::io::BufReader;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, ensure, Context, Result};
use once_cell::unsync::Lazy;

pub struct Loaded {
    width: u32,
    height: u32,
    data: Vec<u32>,
}

pub enum IconInner {
    Pending(PathBuf),
    Failed,
    Loaded(Loaded),
}

pub struct Icon {
    inner: Lazy<Option<IconInner>, Box<dyn FnOnce() -> Option<IconInner>>>,
}

impl Icon {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        Self {
            inner: Lazy::new(Box::new(move || {
                let mut inner = IconInner::new(path);
                inner.load().map(|()| inner)
            })),
        }
    }

    pub fn as_image(&self) -> Option<raqote::Image> {
        Lazy::force(&self.inner)
            .as_ref()?
            .loaded()
            .map(|l| l.as_image())
    }
}

impl Default for IconInner {
    fn default() -> Self {
        Self::Failed
    }
}

impl IconInner {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self::Pending(path.into())
    }

    fn loaded(&self) -> Option<&Loaded> {
        if let Self::Loaded(l) = self {
            Some(l)
        } else {
            unreachable!()
        }
    }

    fn load(&mut self) -> Option<()> {
        let loaded = match self {
            Self::Pending(path) => Loaded::load(path),
            Self::Failed => return None,
            Self::Loaded(_) => return Some(()),
        };

        if let Some(loaded) = loaded {
            *self = Self::Loaded(loaded);
            Some(())
        } else {
            *self = Self::Failed;
            None
        }
    }
}

impl Loaded {
    pub fn load(path: impl AsRef<Path>) -> Option<Self> {
        let path = path.as_ref();
        let failed_to_load = |e| log::info!("failed to load icon at path `{:?}`: {}", path, e);
        match path.extension()?.to_str()? {
            "png" => Self::from_png_path(path).map_err(failed_to_load).ok(),
            "svg" => Self::from_svg_path(path).map_err(failed_to_load).ok(),
            ext => {
                log::info!("unsupported icon extension: {:?}", ext);
                None
            }
        }
    }

    fn from_png_path(path: impl AsRef<Path>) -> Result<Self> {
        let mut decoder = png::Decoder::new(BufReader::new(std::fs::File::open(path.as_ref())?));
        decoder.set_transformations(png::Transformations::normalize_to_color8());
        let mut reader = decoder
            .read_info()
            .map_err(|e| anyhow!("failed to read png info: {}", e))?;
        let mut buf = vec![0; reader.output_buffer_size()];
        let info = reader
            .next_frame(&mut buf)
            .map_err(|e| anyhow!("failed to read png frame: {}", e))?;
        let buf = &buf[..info.buffer_size()];

        let data = match info.color_type {
            png::ColorType::Rgb => {
                ensure!(buf.len() % 3 == 0, "corrupted icon file");

                buf.chunks(3)
                    .map(|chunk| {
                        let a = 0xffu32 << 24;
                        let r = u32::from(chunk[0]) << 16;
                        let g = u32::from(chunk[1]) << 8;
                        let b = u32::from(chunk[2]);

                        a | r | g | b
                    })
                    .collect()
            }
            png::ColorType::Rgba => rgba_to_argb(buf)?,
            png::ColorType::GrayscaleAlpha => {
                ensure!(buf.len() % 2 == 0, "corrupted icon file");

                buf.chunks(2)
                    .map(|chunk| {
                        let x = u32::from(chunk[0]);
                        let a = u32::from(chunk[1]) << 24;

                        a | (x << 16) | (x << 8) | x
                    })
                    .collect()
            }
            png::ColorType::Grayscale => buf
                .iter()
                .copied()
                .map(u32::from)
                .map(|chunk| {
                    let a = 0xffu32 << 24;
                    let r = chunk << 16;
                    let g = chunk << 8;
                    let b = chunk;

                    a | r | g | b
                })
                .collect(),
            png::ColorType::Indexed => unreachable!("image shall be converted"),
        };

        Ok(Self {
            width: info.width,
            height: info.height,
            data,
        })
    }

    fn from_svg_path(path: impl AsRef<Path>) -> Result<Self> {
        let opt = usvg::Options::default();
        let data = std::fs::read(path.as_ref())
            .with_context(|| format!("failed to open svg file: {:?}", path.as_ref()))?;
        let tree = usvg::Tree::from_data(&data, &opt.to_ref())
            .map_err(|e| anyhow!("svg open error: {}", e))?;

        let width = tree.svg_node().size.width().ceil() as u32;
        let height = tree.svg_node().size.height().ceil() as u32;
        let mut buf = tiny_skia::Pixmap::new(width, height).context("invalid pixmap size")?;
        resvg::render(
            &tree,
            usvg::FitTo::Original,
            tiny_skia::Transform::identity(),
            buf.as_mut(),
        )
        .ok_or_else(|| anyhow!("cannot render svg"))?;

        Ok(Self {
            width,
            height,
            data: rgba_to_argb(buf.data())?,
        })
    }

    fn as_image(&self) -> raqote::Image {
        raqote::Image {
            width: self.width as i32,
            height: self.height as i32,
            data: self.data.as_slice(),
        }
    }
}

fn rgba_to_argb(buf: &[u8]) -> Result<Vec<u32>> {
    ensure!(buf.len() % 4 == 0, "corrupted icon file");

    let data = buf
        .chunks(4)
        .map(|chunk| {
            let src = raqote::SolidSource::from_unpremultiplied_argb(
                chunk[3], chunk[0], chunk[1], chunk[2],
            );

            let a = u32::from(src.a) << 24;
            let r = u32::from(src.r) << 16;
            let g = u32::from(src.g) << 8;
            let b = u32::from(src.b);

            a | r | g | b
        })
        .collect();

    Ok(data)
}
