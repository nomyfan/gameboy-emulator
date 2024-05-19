import { buf } from "crc-32";
import { xxHash32 } from "js-xxhash";

export { default as cloneDeep } from "lodash.clonedeep";

export function hash(data: Uint8Array): string;
export function hash(data: Blob): Promise<string>;
export function hash(data: Uint8Array | Blob): string | Promise<string> {
  if (data instanceof Blob) {
    return data.arrayBuffer().then((buf) => hash(new Uint8Array(buf)));
  }
  return xxHash32(data).toString(16);
}

export function crc32(data: Uint8Array): string;
export function crc32(data: Blob): Promise<string>;
export function crc32(data: Uint8Array | Blob): string | Promise<string> {
  if (data instanceof Blob) {
    return data.arrayBuffer().then((buf) => crc32(new Uint8Array(buf)));
  }
  return (buf(data) >>> 0).toString(16);
}

export async function canvasToBlob(
  canvas: HTMLCanvasElement | OffscreenCanvas,
  type?: string,
  quality?: number,
) {
  if (canvas instanceof OffscreenCanvas) {
    return await canvas.convertToBlob({ type, quality });
  }

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

export function join<T>(
  array: T[],
  separator: (index: number, value: T) => T,
): T[] {
  const newArray: T[] = [];
  for (let i = 0; i < array.length - 1; i++) {
    newArray.push(array[i], separator(i, array[i]));
  }

  if (array.length > 0) {
    newArray.push(array[array.length - 1]);
  }

  return newArray;
}
