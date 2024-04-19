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
