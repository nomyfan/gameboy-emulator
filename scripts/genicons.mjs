import sharp from "sharp";
import ico from "sharp-ico";
import * as path from "node:path";

const root = path.resolve(
  import.meta.dirname,
  "..",
  "app",
  "gameboy",
  "public"
);

const mainIcon = "maskable-icon.svg";

const options = [
  { size: 64, name: "pwa-64x64.png" },
  { size: 180, name: "apple-touch-icon-180x180.png" },
  { size: 192, name: "pwa-192x192.png" },
  { size: 512, name: "pwa-512x512.png" },
  { size: 512, name: "maskable-icon-512x512.png" },
];

for (const { size, name } of options) {
  await sharp(path.join(root, mainIcon))
    .resize(size)
    .toFile(path.join(root, name));
}

await ico.sharpsToIco(
  [sharp(path.join(root, mainIcon))],
  path.join(root, "favicon.ico"),
  { sizes: [48], resizeOptions: {} }
);
