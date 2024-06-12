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

export function isPlainObject(object: unknown) {
  if (typeof object !== "object" || object === null) {
    return false;
  }

  const prototype = Object.getPrototypeOf(object);
  if (prototype === null || prototype === Object.prototype) {
    return true;
  }

  return object.constructor === Object;
}

// TODO: TS introduce `using` keyword, but need some configuration
export async function myUsing<T extends { free: () => void }>(
  resource: T,
  callback: (resource: T) => Promise<void>,
) {
  try {
    return await callback(resource);
  } finally {
    resource.free();
  }
}

export function flow<Args extends unknown[], R, U>(
  f: (...args: Args) => R,
  g: (arg: R) => U,
): (...args: Args) => U {
  return (...args: Args) => {
    return g(f(...args));
  };
}

export function flowAsync<Args extends unknown[], R, U>(
  f: (...args: Args) => Promise<R>,
  g: (arg: R) => Promise<U>,
): (...args: Args) => Promise<U> {
  return async (...args: Args) => {
    return await g(await f(...args));
  };
}

/**
 * Resolve at least `ms` milliseconds.
 * @param ms
 * @param action
 */
export function after<R>(ms: number, action: () => Promise<R>) {
  return new Promise<R>((resolve) => {
    const start = performance.now();
    action().then((ret) => {
      const delay = Math.max(0, ms - (performance.now() - start));
      if (delay < 1) {
        resolve(ret);
      } else {
        setTimeout(resolve, delay, ret);
      }
    });
  });
}
