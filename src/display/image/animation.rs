//! animation handling stuff (gif/webp)
use {
    crate::display::image::source::ImageData,
    color_eyre::eyre::{Context, Result, bail},
    image::{
        AnimationDecoder, DynamicImage, ImageDecoder,
        codecs::{gif::GifDecoder, webp::WebPDecoder},
    },
    std::{
        fs::File,
        io::{BufRead, BufReader, Cursor, Seek},
        path::Path,
        thread,
        time::Duration,
    },
};

/// a single frame in an animation
#[derive(Debug, Clone)]
pub struct AnimationFrame {
    /// the frame's image data
    pub data: ImageData,
    /// duration to display this frame
    pub delay: Duration,
}

/// animated image with multiple frames
#[derive(Debug)]
pub struct AnimatedImage {
    /// all frames in the animation
    pub frames: Vec<AnimationFrame>,
    /// original width
    pub width: u32,
    /// original height
    pub height: u32,
    /// number of times to loop
    pub loop_count: u16,
}

impl AnimatedImage {
    /// load an animated GIF from a file path
    pub fn from_gif_path(path: &Path) -> Result<Self> {
        let file = File::open(path).context(format!("Failed to open gif: {}", path.display()))?;
        let reader = BufReader::new(file);
        Self::from_gif_reader(reader)
    }

    /// load an animated GIF from bytes
    pub fn from_gif_bytes(bytes: &[u8]) -> Result<Self> {
        let cursor = Cursor::new(bytes);
        Self::from_gif_reader(cursor)
    }

    /// calculate the number of rows the animation takes up
    fn terminal_line_count(&self) -> u32 {
        self.height.div_ceil(6)
    }

    /// play an animation in-place
    pub fn play_in_place<W: std::io::Write>(
        &self,
        encoder: &crate::display::image::encoder::SixelEncoder,
        writer: &mut W,
    ) -> Result<()> {
        let line_count = self.terminal_line_count();
        let mut is_first_frame = true;

        for frame in &self.frames {
            if !is_first_frame {
                write!(writer, "\x1B[{}A\x1B[G", line_count).context("failed to move cursor")?;
            }

            let sixel_data = encoder
                .encode(&frame.data)
                .context("failed to encode frame")?;
            write!(writer, "{}", sixel_data).context("failed to write sixel data")?;
            writer.flush().context("failed to flush output")?;
            thread::sleep(frame.delay);
            is_first_frame = false;
        }

        Ok(())
    }

    /// load an animated gif from any reader
    fn from_gif_reader<R>(reader: R) -> Result<Self>
    where
        R: std::io::Read + BufRead + Seek,
    {
        let decoder = GifDecoder::new(reader).context("failed to decode gif")?;
        let (width, height) = decoder.dimensions();
        let loop_count = 0;
        let frames: Vec<_> = decoder
            .into_frames()
            .collect::<Result<Vec<_>, _>>()
            .context("failed to decode gif frames")?;

        if frames.is_empty() {
            bail!("gif has no frames");
        }

        let animation_frames = frames
            .into_iter()
            .map(|frame| {
                let buffer = frame.buffer();
                let (w, h) = buffer.dimensions();
                let rgb_data = DynamicImage::ImageRgba8(buffer.clone())
                    .into_rgba8()
                    .into_raw();

                let delay = frame.delay().numer_denom_ms();
                let delay_ms = delay.0 as f32 / delay.1 as f32;
                let duration = Duration::from_millis(delay_ms.max(1.0) as u64);

                AnimationFrame {
                    data: ImageData::new(rgb_data, w as usize, h as usize),
                    delay: duration,
                }
            })
            .collect();

        Ok(Self {
            frames: animation_frames,
            width,
            height,
            loop_count,
        })
    }

    /// load an animated webp from a file path
    pub fn from_webp_path(path: &Path) -> Result<Self> {
        let file =
            File::open(path).with_context(|| format!("Failed to open WebP: {}", path.display()))?;
        let reader = BufReader::new(file);
        Self::from_webp_reader(reader)
    }

    /// load an animated webp from bytes
    pub fn from_webp_bytes(bytes: &[u8]) -> Result<Self> {
        let cursor = Cursor::new(bytes);
        Self::from_webp_reader(cursor)
    }

    /// load an animated webp from any reader
    fn from_webp_reader<R>(reader: R) -> Result<Self>
    where
        R: std::io::Read + Seek + BufRead,
    {
        let decoder = WebPDecoder::new(reader).context("failed to decode webp")?;
        let (width, height) = decoder.dimensions();
        let loop_count = 0;
        let frames: Vec<_> = decoder
            .into_frames()
            .collect::<Result<Vec<_>, _>>()
            .context("failed to decode frames")?;

        if frames.is_empty() {
            bail!("webp has no frames");
        }

        let animation_frames = frames
            .into_iter()
            .map(|frame| {
                let buffer = frame.buffer();
                let (w, h) = buffer.dimensions();
                let rgb_data = DynamicImage::ImageRgba8(buffer.clone())
                    .to_rgba8()
                    .into_raw();

                let delay = frame.delay().numer_denom_ms();
                let delay_ms = delay.0 as f32 / delay.1 as f32;
                let duration = Duration::from_millis(delay_ms.max(10.0) as u64);

                AnimationFrame {
                    data: ImageData::new(rgb_data, w as usize, h as usize),
                    delay: duration,
                }
            })
            .collect();

        Ok(Self {
            frames: animation_frames,
            width,
            height,
            loop_count,
        })
    }

    /// get the number of frames
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// get a specific frame by index
    pub fn get_frame(&self, index: usize) -> Option<&AnimationFrame> {
        self.frames.get(index)
    }

    /// get the total duration of the animation
    pub fn total_duration(&self) -> Duration {
        self.frames.iter().map(|f| f.delay).sum()
    }

    /// check if this should loop infinitely
    pub fn is_infinite_loop(&self) -> bool {
        self.loop_count == 0
    }

    /// apply a speed multiplier to all frame delays
    pub fn with_speed(mut self, speed: f32) -> Self {
        for frame in &mut self.frames {
            let new_delay_ms = frame.delay.as_millis() as f32 / speed;
            frame.delay = Duration::from_millis(new_delay_ms.max(1.0) as u64);
        }

        self
    }
}

/// check if a file is an animated format
pub fn is_animated_format(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext = ext.to_string_lossy().to_lowercase();
        matches!(ext.as_str(), "gif" | "webp")
    } else {
        false
    }
}

/// try to load an animation from a path
pub fn load_animated(path: &Path) -> Result<AnimatedImage> {
    if !path.exists() {
        bail!("File does not exist: {}", path.display());
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase());

    match ext.as_deref() {
        Some("gif") => AnimatedImage::from_gif_path(path),
        Some("webp") => AnimatedImage::from_webp_path(path),
        Some(ext) => bail!("Unsupported animation format: {}", ext),
        None => bail!("Could not determine file format: {}", path.display()),
    }
}
