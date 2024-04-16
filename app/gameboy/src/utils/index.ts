import { xxhash32 } from "hash-wasm";

export async function hashFile(file: File) {
  const buffer = await file.arrayBuffer();
  const data = new Uint8Array(buffer);
  return await xxhash32(data);
}
