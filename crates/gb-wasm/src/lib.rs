mod utils;

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

const COLORS: [[u8; 4]; 4] = [
    [0xFF, 0xFF, 0xFF, 0xFF],
    [0xAA, 0xAA, 0xAA, 0xFF],
    [0x55, 0x55, 0x55, 0xFF],
    [0x00, 0x00, 0x00, 0xFF],
];

#[wasm_bindgen(js_name = GameBoy)]
pub struct GameBoyHandle {
    gb: GameBoy,
}

struct ScaleImageData(Vec<u8>, u8);

impl ScaleImageData {
    fn new(width: usize, height: usize, scale: u8) -> ScaleImageData {
        let scaled_size = width * scale as usize * height * scale as usize * 4;
        ScaleImageData(vec![0; scaled_size], scale)
    }

    fn set(&mut self, x: usize, y: usize, color: &[u8; 4]) {
        let scale = self.1 as usize;
        let x_begin = x * scale;
        let x_end = x_begin + scale;
        let y_begin = y * scale;
        let y_end = y_begin + scale;

        for y in y_begin..y_end {
            let begin = (y * 160 * scale + x_begin) * 4;
            let end = (y * 160 * scale + x_end) * 4;
            self.0[begin..end].chunks_mut(4).for_each(|chunk| chunk.copy_from_slice(color));
        }
    }

    fn as_image_data(&self) -> ImageData {
        ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(&self.0),
            160 * self.1 as u32,
            144 * self.1 as u32,
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

        let mut scale_image = ScaleImageData::new(160, 144, scale);
        let frame_handle =
            Box::new(
                move |data: &BoxedArray<u8, 23040>,
                      #[cfg(debug_assertions)] _dbg_data: Option<(
                    &BoxedArray<u8, 0x2000>,
                    bool,
                )>| {
                    data.iter().enumerate().for_each(|(n, color_id)| {
                        let color = &COLORS[*color_id as usize];

                        let y = n / 160;
                        let x = n % 160;
                        scale_image.set(x, y, color);
                    });

                    let image_data = scale_image.as_image_data();
                    canvas_context.put_image_data(&image_data, 0.0, 0.0).unwrap();
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
    gb.set_handles(
        Some(
            Box::new(
                move |data: &BoxedArray<u8, 23040>,
                      #[cfg(debug_assertions)] _dbg_data: Option<(
                    &BoxedArray<u8, 0x2000>,
                    bool,
                )>| {
                    data.iter().enumerate().for_each(|(n, color_id)| {
                        let color = &COLORS[*color_id as usize];

                        let y = n / 160;
                        let x = n % 160;
                        scale_image.set(x, y, color);
                    });

                    let image_data = scale_image.as_image_data();
                    canvas_context.put_image_data(&image_data, 0.0, 0.0).unwrap();
                },
            ),
        ),
        None,
    );

    // Play n frames
    gb.continue_clocks(70224 * frame_at.unwrap_or(60));

    let mut encode_options = ImageEncodeOptions::new();
    encode_options.type_(&"image/jpeg").quality(1.0);
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
    console_error_panic_hook::set_once();
}
