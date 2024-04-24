import sharp from "sharp";
import ico from "sharp-ico";

const options = [
  { size: 64, name: "pwa-64x64.png" },
  { size: 180, name: "apple-touch-icon-180x180.png" },
  { size: 192, name: "pwa-192x192.png" },
  { size: 512, name: "pwa-512x512.png" },
];

for (const { size, name } of options) {
  await sharp("./public/maskable-icon-512x512.png")
    .resize(size)
    .toFile(`./public/${name}`);
}

await ico.sharpsToIco(
  [sharp("./public/maskable-icon-512x512.png")],
  "./public/favicon.ico",
  {
    sizes: [48],
    resizeOptions: {},
  },
);
