// import { foo } from "http://localhost:5173/gb_wasm_bindings/gb_wasm.js";

import {
  new_gameboy,
  foo,
} from "http://localhost:5173/@fs/Users/nomyfan/develop/gameboy-emulator/app/gb_wasm/gb_wasm_bindings/gb_wasm.js";

self.onmessage = async function gbWorker(event) {
  const { data } = event;
  if (data.kind === "play") {
    const payload = data.payload;
    console.log("before", payload);
    const canvas = data.payload.canvas;
    const file = data.payload.file;

    // const { foo } = await import(
    //   "http://localhost:5173/@fs/Users/nomyfan/develop/gameboy-emulator/app/gb_wasm/gb_wasm_bindings/gb_wasm.js"
    // );

    // let count = 0;

    // const renderCallback = () => {
    //   console.log("raf" + count);
    //   count++;

    //   requestAnimationFrame(renderCallback);
    // };

    // requestAnimationFrame(renderCallback);

    new_gameboy(file, canvas);
    // foo();
    // const importObject = {
    //   imports: {
    //     bar: (message: string) => console.info("message from bar " + message),
    //   },
    // };

    // const source = await fetch("http://localhost:5173/gb_wasm.wasm")
    //   .then((response) => response.arrayBuffer())
    //   .then((bytes) =>
    //     WebAssembly.instantiate(bytes, { env: importObject.imports })
    //   );
    // const source = await WebAssembly.instantiateStreaming(
    //   fetch("http://localhost:5173/gb_wasm_bindings/gb_wasm.js"),
    //   importObject
    // );
    // const exports = source.instance.exports as {
    //   foo(name: string): void;
    // };
    // exports.foo("balab");
  }
};
