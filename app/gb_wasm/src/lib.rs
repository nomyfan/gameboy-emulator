mod utils;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use gb::wasm::{Cartridge, GameBoy, Manifest};
use gb::FrameOutHandle;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::js_sys::Uint8ClampedArray;
use web_sys::{
    console, Blob, CanvasRenderingContext2d, HtmlCanvasElement, OffscreenCanvas,
    OffscreenCanvasRenderingContext2d, Performance,
};

const COLOR_PALETTES: [u32; 4] = [0xFFFFFF, 0xAAAAAA, 0x555555, 0x000000];

fn request_animation_frame(f: &Closure<dyn FnMut()>) -> i32 {
    web_sys::js_sys::global()
        .dyn_into::<web_sys::DedicatedWorkerGlobalScope>()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .unwrap()

    // web_sys::window().unwrap().request_animation_frame(f.as_ref().unchecked_ref()).unwrap()
}

#[wasm_bindgen]
pub async fn new_gameboy(rom: Blob, canvas: OffscreenCanvas) {
    let array_buffer_promise: JsFuture = rom.array_buffer().into();
    let array_buffer = array_buffer_promise.await.expect("TODO:");
    let rom = Uint8ClampedArray::new(&array_buffer).to_vec();
    let cart = Cartridge::try_from(rom).expect("TODO:");

    // TODO: audio
    let mut gb = GameBoy::new(Manifest { cart, sample_rate: None });

    // context.set_fill_style(&"rgb(122, 122, 122)".into());
    // context.fill_rect(0.0, 0.0, 160.0, 144.0);

    let frame = Arc::new(RefCell::new(0));

    let self_ = web_sys::js_sys::global().dyn_into::<web_sys::WorkerGlobalScope>().unwrap();
    // let self_ = web_sys::window().unwrap();

    // let on_frame: Box<FrameOutHandle> = Box::new({
    //     let frame = frame.clone();
    //     // let mut last = worker_self.performance().unwrap().now();
    //     move |data, #[cfg(debug_assertions)] dbg_data| {
    //         return;
    //         *frame.borrow_mut() += 1;
    //         let v = frame.borrow();

    //         // FIXME: update too frequently, maybe we should update buffer only and update canvas with requestAnimationFrame
    //         // let now = worker_self.performance().unwrap().now();
    //         // let duration = now - last;
    //         // last = now;
    //         // console::log_1(&format!("frame {} now {}", v, duration).into());
    //         let context = canvas
    //             .get_context("2d")
    //             .unwrap()
    //             .unwrap()
    //             .dyn_into::<OffscreenCanvasRenderingContext2d>()
    //             .unwrap();
    //         data.iter().enumerate().for_each(|(y, pixel)| {
    //             pixel.iter().enumerate().for_each(|(x, color_id)| {
    //                 let color = COLOR_PALETTES[*color_id as usize];
    //                 let r = (color >> 16) as u8;
    //                 let g = (color >> 8) as u8;
    //                 let b = color as u8;
    //                 // let r = 122;
    //                 // let g = 122;
    //                 // let b = 122;
    //                 context.set_fill_style(&format!("rgb({}, {}, {})", r, g, b).into());
    //                 context.fill_rect(x as f64, y as f64, 1.0, 1.0);
    //             });
    //         });
    //     }
    // });
    // let pull_command = Box::new(move || Ok(None));

    // gb.set_handles(on_frame, None);

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let mut start = self_.performance().unwrap().now();
    let mut last_frame_id = 0;

    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<OffscreenCanvasRenderingContext2d>()
        .unwrap();
    context.set_transform(3.0, 0.0, 0.0, 3.0, 0.0, 0.0).unwrap();

    *g.borrow_mut() = Some(Closure::new(move || {
        let end = self_.performance().unwrap().now();
        let duration = end - start;
        // console::log_1(&format!("duration {}", duration).into());
        // start = end;
        if duration > 16.666 {
            let _ = gb.play_with_clocks();
            let (data, frame_id) = gb.pull_frame();
            // console::log_1(&format!("frame {} {}", frame_id, last_frame_id).into());
            if frame_id != last_frame_id {
                last_frame_id = frame_id;

                data.iter().enumerate().for_each(|(y, pixel)| {
                    pixel.iter().enumerate().for_each(|(x, color_id)| {
                        let color = COLOR_PALETTES[*color_id as usize];
                        let r = (color >> 16) as u8;
                        let g = (color >> 8) as u8;
                        let b = color as u8;
                        // let r = 122;
                        // let g = 122;
                        // let b = 122;
                        context.set_fill_style(&format!("rgb({}, {}, {})", r, g, b).into());
                        context.fill_rect(x as f64, y as f64, 1.0, 1.0);
                    });
                });
            }
            start = end;
        }
        request_animation_frame(f.borrow().as_ref().unwrap());
    }));

    request_animation_frame(g.borrow().as_ref().unwrap());

    // let duration = gb.play_with_clocks();
    // console::log_1(&format!("duration {}", duration).into());

    // while (*frame.borrow()) < 10 {
    //     gb.play2();
    // }
    // console::log_1(&"done".into());

    // let context = canvas
    //     .get_context("2d")
    //     .unwrap()
    //     .unwrap()
    //     .dyn_into::<OffscreenCanvasRenderingContext2d>()
    //     .unwrap();
    // for y in 0..144 {
    //     for x in 0..160 {
    //         context.set_fill_style(&format!("red").into());
    //         context.fill_rect(x as f64, y as f64, 1.0, 1.0);
    //     }
    // }
}
