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
}

struct ScaleImageData {
    raw: Vec<u8>,
    scale: u8,
    width: usize,
    height: usize,
}

impl ScaleImageData {
    fn new(width: usize, height: usize, scale: u8) -> ScaleImageData {
        let scaled_size = width * scale as usize * height * scale as usize * 4;
        Self { raw: vec![0; scaled_size], scale, width, height }
    }

    fn set(&mut self, x: usize, y: usize, color: &[u8; 4]) {
        let scale = self.scale as usize;
        let x_begin = x * scale;
        let x_end = x_begin + scale;
        let y_begin = y * scale;
        let y_end = y_begin + scale;

        for y in y_begin..y_end {
            let begin = (y * self.width * scale + x_begin) * 4;
            let end = (y * self.width * scale + x_end) * 4;
            self.raw[begin..end].chunks_mut(4).for_each(|chunk| chunk.copy_from_slice(color));
        }
    }

    fn as_image_data(&self) -> ImageData {
        ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(&self.raw),
            self.width as u32 * self.scale as u32,
            self.height as u32 * self.scale as u32,
        )
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
        scale: Option<u8>,
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

        let scale = scale.unwrap_or(1);
        let canvas_context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();

        #[cfg(feature = "debug_frame")]
        let mut dbg_canvas_context = dbg_canvas.map(|canvas| {
            let context = canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<CanvasRenderingContext2d>()
                .unwrap();
            let scale_image = ScaleImageData::new(256 + 10 + 40, 256 + 10 + 256, 1);
            (context, scale_image)
        });

        let mut scale_image = ScaleImageData::new(160, 144, scale);
        let mut rgba = [0xFF, 0xFF, 0xFF, 0xFF];
        let frame_handle = Box::new(
            move |data: &BoxedArray<u8, 69120>, #[cfg(feature = "debug_frame")] dbg_data: &[u8]| {
                for (n, rgb) in data.chunks(3).enumerate() {
                    rgba[0] = rgb[0];
                    rgba[1] = rgb[1];
                    rgba[2] = rgb[2];

                    let y = n / 160;
                    let x = n % 160;
                    scale_image.set(x, y, &rgba);
                }

                let image_data = scale_image.as_image_data();
                canvas_context.put_image_data(&image_data, 0.0, 0.0).unwrap();

                #[cfg(feature = "debug_frame")]
                if let Some((canvas_context, scale_image)) = dbg_canvas_context.as_mut() {
                    // The first tile map, 256x256
                    for (n, rgb) in dbg_data.chunks(3).take(256 * 256).enumerate() {
                        rgba[0] = rgb[0];
                        rgba[1] = rgb[1];
                        rgba[2] = rgb[2];
                        let y = n / 256;
                        let x = n % 256;
                        scale_image.set(x, y, &rgba);
                    }

                    // The second tile map, 256x256
                    for (n, rgb) in dbg_data.chunks(3).skip(256 * 256).take(256 * 256).enumerate() {
                        rgba[0] = rgb[0];
                        rgba[1] = rgb[1];
                        rgba[2] = rgb[2];
                        let y = n / 256;
                        let x = n % 256;
                        scale_image.set(x, y + 256 + 10, &rgba);
                    }

                    // Palette colors
                    dbg_data.chunks(3).skip(256 * 256 * 2).enumerate().for_each(|(n, rgb)| {
                        rgba[0] = rgb[0];
                        rgba[1] = rgb[1];
                        rgba[2] = rgb[2];

                        let x = 266 + (n % 4) * 10;
                        let y = (n / 4) * 10;
                        let gap = (n / 4) * 5;
                        for i in 0..10 {
                            for j in 0..10 {
                                scale_image.set(x + i, y + j + gap, &rgba);
                            }
                        }
                    });

                    let image_data = scale_image.as_image_data();
                    canvas_context.put_image_data(&image_data, 0.0, 0.0).unwrap();
                }
            },
        );

        match (audio_stream, sample_rate) {
            (Some(stream), Some(sample_rate)) => {
                let stream_writer = stream.get_writer().unwrap();
                let sample_count = buffer_size_from_sample_rate(sample_rate);
                let audio_buffer = js_sys::Float32Array::new_with_length(sample_count * 2);

                gb.set_handles(
                    Some(frame_handle),
                    Some(Box::new(move |data| {
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

        GameBoyHandle { gb }
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
    let mut scale_image = ScaleImageData::new(160, 144, SCALE as u8);
    let mut rgba = [0xFF, 0xFF, 0xFF, 0xFF];
    gb.set_handles(
        Some(Box::new(move |data, #[cfg(feature = "debug_frame")] _| {
            data.chunks(3).enumerate().for_each(|(n, rgb)| {
                rgba[0] = rgb[0];
                rgba[1] = rgb[1];
                rgba[2] = rgb[2];

                let y = n / 160;
                let x = n % 160;
                scale_image.set(x, y, &rgba);
            });

            let image_data = scale_image.as_image_data();
            canvas_context.put_image_data(&image_data, 0.0, 0.0).unwrap();
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
