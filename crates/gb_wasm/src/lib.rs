use std::{cell::RefCell, rc::Rc};

use gb::GameBoySnapshot;
use gb::{buffer_size_from_sample_rate, Cartridge, GameBoy, Manifest};
use gb_shared::boxed::BoxedArray;
use gb_shared::command::{Command, JoypadCommand, JoypadKey};
use gb_shared::Snapshot;
use js_sys::Uint8ClampedArray;
use wasm_bindgen::{prelude::*, Clamped};
use web_sys::{
    js_sys, Blob, CanvasRenderingContext2d, HtmlCanvasElement, ImageData, ImageEncodeOptions,
    OffscreenCanvas, OffscreenCanvasRenderingContext2d, WritableStream,
};

#[wasm_bindgen(js_name = GameBoy)]
pub struct GameBoyHandle {
    gb: GameBoy,
    muted: Rc<RefCell<bool>>,
}

struct Frame {
    raw: Vec<u8>,
    canvas: OffscreenCanvas,
    context: OffscreenCanvasRenderingContext2d,
    width: u32,
    height: u32,
}

impl Frame {
    fn new(width: u32, height: u32) -> Frame {
        let canvas = OffscreenCanvas::new(width, height).unwrap();
        Self {
            raw: vec![0xFF; width as usize * height as usize * 4],
            width,
            height,
            context: canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<OffscreenCanvasRenderingContext2d>()
                .unwrap(),
            canvas,
        }
    }

    #[cfg(feature = "debug_frame")]
    fn set_with_rgb(&mut self, x: usize, y: usize, rgb: &[u8]) {
        let offset = (y * self.width as usize + x) * 4;
        self.raw[offset..offset + 3].copy_from_slice(rgb);
    }

    fn render_with_rgb(&mut self, rgb: &[u8]) {
        rgb.chunks(3).zip(self.raw.chunks_mut(4)).for_each(|(src, dst)| {
            dst[0] = src[0];
            dst[1] = src[1];
            dst[2] = src[2];
        });
        let image_data = self.as_image_data();
        self.context.put_image_data(&image_data, 0.0, 0.0).unwrap();
    }

    fn render_canvas_with_rgb(
        &mut self,
        rgb: &[u8],
        context: &CanvasRenderingContext2d,
        width: f64,
        height: f64,
    ) {
        self.render_with_rgb(rgb);
        context
            .draw_image_with_offscreen_canvas_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                &self.canvas,
                0.0,
                0.0,
                self.width as f64,
                self.height as f64,
                0.0,
                0.0,
                width,
                height,
            )
            .unwrap();
    }

    fn render_offscreen_canvas_with_rgb(
        &mut self,
        rgb: &[u8],
        context: &OffscreenCanvasRenderingContext2d,
        scale: f64,
    ) {
        self.render_with_rgb(rgb);
        context
            .draw_image_with_offscreen_canvas_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                &self.canvas,
                0.0,
                0.0,
                self.width as f64,
                self.height as f64,
                0.0,
                0.0,
                self.width as f64 * scale,
                self.height as f64 * scale,
            )
            .unwrap();
    }

    fn as_image_data(&self) -> ImageData {
        ImageData::new_with_u8_clamped_array_and_sh(Clamped(&self.raw), self.width, self.height)
            .unwrap()
    }
}

#[wasm_bindgen(js_class = GameBoy)]
impl GameBoyHandle {
    #[wasm_bindgen]
    pub fn __for_emitting_types_only__(_: JoypadKey) {}

    #[wasm_bindgen]
    pub fn create(
        rom: Uint8ClampedArray,
        canvas: HtmlCanvasElement,
        sav: Option<Vec<u8>>,
        sample_rate: Option<u32>,
        audio_stream: Option<WritableStream>,
        dbg_canvas: Option<HtmlCanvasElement>,
    ) -> GameBoyHandle {
        let rom = rom.to_vec();
        let cart = Cartridge::try_from(rom).unwrap();

        let mut gb = GameBoy::new(Manifest { cart, sample_rate });
        if let Some(sav) = sav {
            gb.resume_cartridge(&sav).unwrap();
        }

        let canvas_context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();
        canvas_context.set_image_smoothing_enabled(false);

        #[cfg(feature = "debug_frame")]
        let mut dbg_canvas_context = dbg_canvas.map(|canvas| {
            let context = canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<CanvasRenderingContext2d>()
                .unwrap();
            let frame = Frame::new(256 + 10 + 40, 256 + 10 + 256);
            context.set_image_smoothing_enabled(false);
            (context, frame)
        });

        let mut frame = Frame::new(160, 144);
        let frame_handle = Box::new(
            move |data: &BoxedArray<u8, 69120>, #[cfg(feature = "debug_frame")] dbg_data: &[u8]| {
                let width = canvas.width() as f64;
                let height = canvas.height() as f64;
                frame.render_canvas_with_rgb(data.as_ref(), &canvas_context, width, height);

                #[cfg(feature = "debug_frame")]
                if let Some((dbg_canvas_context, dbg_frame)) = dbg_canvas_context.as_mut() {
                    // The first tile map, 256x256
                    for (n, rgb) in dbg_data.chunks(3).take(256 * 256).enumerate() {
                        let y = n / 256;
                        let x = n % 256;
                        dbg_frame.set_with_rgb(x, y, rgb);
                    }

                    // The second tile map, 256x256
                    for (n, rgb) in dbg_data.chunks(3).skip(256 * 256).take(256 * 256).enumerate() {
                        let y = n / 256;
                        let x = n % 256;
                        dbg_frame.set_with_rgb(x, y + 256 + 10, rgb);
                    }

                    // Palette colors
                    dbg_data.chunks(3).skip(256 * 256 * 2).enumerate().for_each(|(n, rgb)| {
                        let x = 266 + (n % 4) * 10;
                        let y = (n / 4) * 10;
                        let gap = (n / 4) * 5;
                        for i in 0..10 {
                            for j in 0..10 {
                                dbg_frame.set_with_rgb(x + i, y + j + gap, rgb);
                            }
                        }
                    });

                    let image_data = dbg_frame.as_image_data();
                    dbg_canvas_context.put_image_data(&image_data, 0.0, 0.0).unwrap();
                }
            },
        );

        let muted = Rc::new(RefCell::new(false));
        match (audio_stream, sample_rate) {
            (Some(stream), Some(sample_rate)) => {
                let stream_writer = stream.get_writer().unwrap();
                let sample_count = buffer_size_from_sample_rate(sample_rate);
                let audio_buffer = js_sys::Float32Array::new_with_length(sample_count * 2);
                let muted = muted.clone();

                gb.set_handles(
                    Some(frame_handle),
                    Some(Box::new(move |data| {
                        if *muted.borrow() {
                            return;
                        }
                        let len = data.len().min(sample_count as usize);
                        for (i, (left, right)) in data.iter().take(len).enumerate() {
                            audio_buffer.set_index(i as u32 * 2, *left);
                            audio_buffer.set_index(i as u32 * 2 + 1, *right);
                        }

                        let slice = audio_buffer.slice(0, (len * 2) as u32);

                        let _ = stream_writer.write_with_chunk(&slice.into());
                    })),
                )
            }
            _ => {
                gb.set_handles(Some(frame_handle), None);
            }
        }

        GameBoyHandle { gb, muted }
    }

    #[wasm_bindgen(js_name = continue)]
    pub fn r#continue(&mut self, clocks: Option<u32>) {
        self.gb.continue_clocks(clocks.unwrap_or(70224)); // 70224 clocks per frame
    }

    #[wasm_bindgen(js_name = suspendCartridge)]
    pub fn suspend_cartridge(&mut self) -> Option<Vec<u8>> {
        self.gb.suspend_cartridge()
    }

    #[wasm_bindgen(js_name = changeKeyState)]
    pub fn change_key_state(&mut self, state: u8) {
        self.gb.exec_command(Command::Joypad(JoypadCommand::State(state)));
    }

    #[wasm_bindgen(js_name = takeSnapshot)]
    pub fn take_snapshot(&self) -> js_sys::Uint8Array {
        let snapshot = self.gb.take_snapshot();
        let bytes: Vec<u8> = Vec::try_from(&snapshot).unwrap();

        js_sys::Uint8Array::from(bytes.as_slice())
    }

    #[wasm_bindgen(js_name = restoreSnapshot)]
    pub fn restore_snapshot(&mut self, snapshot: js_sys::Uint8Array) -> Result<(), JsError> {
        match GameBoySnapshot::try_from(snapshot.to_vec().as_slice()) {
            Ok(snapshot) => {
                if snapshot.cart_checksum() != self.gb.cart_checksum() {
                    return Err(JsError::new("[ESS2]The snapshot doesn't match the game"));
                }
                self.gb.restore_snapshot(snapshot);

                Ok(())
            }
            Err(_) => Err(JsError::new("[ESS1]Snapshot is broken")),
        }
    }

    #[wasm_bindgen]
    pub fn mute(&mut self, muted: bool) {
        *self.muted.borrow_mut() = muted;
    }
}

#[wasm_bindgen]
pub struct GameBoyMetadata {
    name: String,
    cover: Blob,
}

#[wasm_bindgen]
impl GameBoyMetadata {
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn cover(&self) -> Blob {
        self.cover.clone()
    }
}

#[wasm_bindgen(js_name = obtainMetadata)]
pub async fn obtain_metadata(rom: Uint8ClampedArray, frame_at: Option<u32>) -> GameBoyMetadata {
    let rom = rom.to_vec();
    let cart = Cartridge::try_from(rom).unwrap();

    let name = {
        let mut title = cart.header.title;
        title[15] = 0;
        let title = title.into_iter().take_while(|x| *x != 0).collect::<Vec<u8>>();
        std::str::from_utf8(&title).unwrap().to_owned()
    };

    let mut gb = GameBoy::new(Manifest { cart, sample_rate: None });
    const SCALE: u32 = 2;
    let canvas = OffscreenCanvas::new(160 * SCALE, 144 * SCALE).unwrap();
    let canvas_context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<OffscreenCanvasRenderingContext2d>()
        .unwrap();
    canvas_context.set_image_smoothing_enabled(false);
    let mut frame = Frame::new(160, 144);
    gb.set_handles(
        Some(Box::new(move |data, #[cfg(feature = "debug_frame")] _| {
            frame.render_offscreen_canvas_with_rgb(data.as_ref(), &canvas_context, SCALE as f64);
        })),
        None,
    );

    // Play n frames
    gb.continue_clocks(70224 * frame_at.unwrap_or(60));

    let mut encode_options = ImageEncodeOptions::new();
    encode_options.type_("image/jpeg").quality(1.0);
    let cover = wasm_bindgen_futures::JsFuture::from(
        canvas.convert_to_blob_with_options(&encode_options).unwrap(),
    )
    .await
    .unwrap()
    .dyn_into::<Blob>()
    .unwrap();

    GameBoyMetadata { name, cover }
}

#[wasm_bindgen]
pub fn init_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once()
}

#[wasm_bindgen]
pub fn init_log(max_level: &str, filters: &str) {
    gb_console_log::init(max_level, filters)
}
