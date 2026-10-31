import sharp from "sharp";
import ico from "sharp-ico";
import * as fs from "node:fs";
import * as path from "node:path";

const root = path.resolve(
  import.meta.dirname,
  "..",
  "app",
  "gameboy",
  "public"
);

const mainIcon = "icon.svg";
// Same design shrunk into the central 80% safe zone, since platforms crop the
// outer area of maskable icons before applying their own shape.
const maskableIcon = "maskable-icon.svg";
const mainIconData = fs.readFileSync(path.join(root, mainIcon)).toString("base64");

// Follow the macOS app icon grid: an 824x824 rounded rect with a drop shadow
// centered on a 1024x1024 transparent canvas, so the installed app fits in the Dock.
const roundedIcon = Buffer.from(`
<svg width="1024" height="1024" viewBox="0 0 1024 1024" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <clipPath id="shape">
      <rect x="100" y="100" width="824" height="824" rx="185" />
    </clipPath>
    <filter id="shadow" x="-10%" y="-10%" width="120%" height="120%">
      <feGaussianBlur stdDeviation="10" />
      <feOffset dy="10" />
      <feComponentTransfer>
        <feFuncA type="linear" slope="0.3" />
      </feComponentTransfer>
    </filter>
  </defs>
  <rect x="100" y="100" width="824" height="824" rx="185" filter="url(#shadow)" />
  <image x="100" y="100" width="824" height="824" clip-path="url(#shape)"
    href="data:image/svg+xml;base64,${mainIconData}" />
</svg>`);

const options = [
  { size: 64, name: "pwa-64x64.png", input: roundedIcon },
  { size: 180, name: "apple-touch-icon-180x180.png" },
  { size: 192, name: "pwa-192x192.png", input: roundedIcon },
  { size: 512, name: "pwa-512x512.png", input: roundedIcon },
  { size: 512, name: "maskable-icon-512x512.png", input: path.join(root, maskableIcon) },
];

for (const { size, name, input = path.join(root, mainIcon) } of options) {
  await sharp(input)
    .resize(size)
    .toFile(path.join(root, name));
}

await ico.sharpsToIco(
  [sharp(path.join(root, mainIcon))],
  path.join(root, "favicon.ico"),
  { sizes: [48], resizeOptions: {} }
);
