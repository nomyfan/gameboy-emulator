import { xxhash32 } from "hash-wasm";

export async function hashFile(file: File) {
  const buffer = await file.arrayBuffer();
  const data = new Uint8Array(buffer);
  return await xxhash32(data);
}

export async function blobFromCanvas(
  canvas: HTMLCanvasElement,
  type?: string,
  quality?: number,
) {
  return new Promise<Blob>((resolve, reject) => {
    canvas.toBlob(
      (blob) => {
        if (blob) {
          resolve(blob);
        } else {
          reject(new Error("Failed to take snapshot"));
        }
      },
      type,
      quality,
    );
  });
}
