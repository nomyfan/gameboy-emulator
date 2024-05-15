import {
  BlobReader,
  BlobWriter,
  Entry,
  TextReader,
  TextWriter,
  Uint8ArrayReader,
  Uint8ArrayWriter,
  ZipReader,
  ZipWriter,
} from "@zip.js/zip.js";

export type IZipDataPrimitive = null | undefined | string | number | boolean;

export interface IZipDataObject {
  [key: string]:
    | IZipDataPrimitive
    | IZipDataObject
    | Array<IZipDataPrimitive | IZipDataObject>;
}

export interface IZipDataEntry {
  path: string;
  data: string | IZipDataObject | Blob | Uint8Array;
}

export async function zip(
  entries: IZipDataEntry[],
  options?: {
    level?: 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9;
    mimeType?: string;
  },
) {
  const writer = new ZipWriter(new BlobWriter(options?.mimeType), {
    level: options?.level,
  });

  for (const entry of entries) {
    if (typeof entry.data === "string") {
      await writer.add(entry.path, new TextReader(entry.data));
    } else if (entry.data instanceof Blob) {
      await writer.add(entry.path, new BlobReader(entry.data));
    } else if (entry.data instanceof Uint8Array) {
      await writer.add(entry.path, new Uint8ArrayReader(entry.data));
    } else if (typeof entry.data === "object" && entry.data) {
      const data = JSON.stringify(entry.data, null, 2);
      await writer.add(entry.path, new TextReader(data));
    } else {
      throw new Error("Unsupported data type " + typeof entry.data);
    }
  }

  return await writer.close();
}

export class ZipKvReader {
  private readonly entries: Map<string, Entry>;
  constructor(entries: Entry[]) {
    this.entries = new Map(entries.map((e) => [e.filename, e]));
  }

  async getBlob(name: string) {
    return this.entries.get(name)?.getData?.(new BlobWriter());
  }

  async getUint8Array(name: string) {
    return this.entries.get(name)?.getData?.(new Uint8ArrayWriter());
  }

  async getText(name: string) {
    return this.entries.get(name)?.getData?.(new TextWriter());
  }

  async getObject<T extends IZipDataObject>(name: string) {
    const text = await this.getText(name);
    return text ? (JSON.parse(text) as T) : undefined;
  }
}

export async function unzip(file: Blob) {
  const reader = new ZipReader(new BlobReader(file));
  const entries = await reader.getEntries();
  return new ZipKvReader(entries);
}
