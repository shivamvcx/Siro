const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

console.log("Building icons from icon.svg...");

const edgeExe = "C:\\Program Files (x86)\\Microsoft\\EdgeCore\\156.0.4314.50\\msedge.exe";

// Square SVG template centered on the visual content of icon.svg
const squareSvg = `<svg width="256" height="256" viewBox="-2 15 138 138" fill="none" xmlns="http://www.w3.org/2000/svg">
<g filter="url(#filter0_d_54_5)">
<rect y="17" width="128" height="128" rx="25" fill="url(#paint0_linear_54_5)"/>
</g>
<path d="M36 96.8929H52.0714V104.929H76.1786V88.8571H44.0357V80.8214H36V48.6786H44.0357V40.6429H84.2143V48.6786H92.25V64.75H76.1786V56.7143H52.0714V72.7857H84.2143V80.8214H92.25V112.964H84.2143V121H44.0357V112.964H36V96.8929Z" fill="white"/>
<defs>
<filter id="filter0_d_54_5" x="0" y="17" width="134" height="134" filterUnits="userSpaceOnUse" color-interpolation-filters="sRGB">
<feFlood flood-opacity="0" result="BackgroundImageFix"/>
<feColorMatrix in="SourceAlpha" type="matrix" values="0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 127 0" result="hardAlpha"/>
<feOffset dx="5" dy="5"/>
<feGaussianBlur stdDeviation="0.5"/>
<feComposite in2="hardAlpha" operator="out"/>
<feColorMatrix type="matrix" values="0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0.25 0"/>
<feBlend mode="normal" in2="BackgroundImageFix" result="effect1_dropShadow_54_5"/>
<feBlend mode="normal" in="SourceGraphic" in2="effect1_dropShadow_54_5" result="shape"/>
</filter>
<linearGradient id="paint0_linear_54_5" x1="8.5" y1="24.5" x2="120.5" y2="136.5" gradientUnits="userSpaceOnUse">
<stop stop-color="#4294FF"/>
<stop offset="1" stop-color="#C3DDFF"/>
</linearGradient>
</defs>
</svg>`;

const htmlContent = `<!DOCTYPE html>
<html>
<head>
<style>
  * { margin: 0; padding: 0; box-sizing: border-box; }
  html, body {
    width: 100vw;
    height: 100vh;
    background: transparent !important;
    overflow: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  svg {
    width: 100%;
    height: 100%;
  }
</style>
</head>
<body>
${squareSvg}
</body>
</html>`;

const tempHtmlPath = path.resolve(__dirname, 'render_icon.html');
fs.writeFileSync(tempHtmlPath, htmlContent, 'utf-8');

const targetDir = path.resolve(__dirname, '../target');
if (!fs.existsSync(targetDir)) fs.mkdirSync(targetDir, { recursive: true });

const sizes = [256, 128, 64, 48, 32, 16];
const pngBuffers = {};

for (const size of sizes) {
  const outPng = path.join(targetDir, `icon_${size}.png`);
  const cmd = `"${edgeExe}" --headless=new --default-background-color=00000000 --screenshot="${outPng}" --window-size=${size},${size} "file:///${tempHtmlPath.replace(/\\/g, '/')}"`;
  execSync(cmd, { stdio: 'pipe' });
  pngBuffers[size] = fs.readFileSync(outPng);
  console.log(`Generated ${size}x${size} PNG: ${pngBuffers[size].length} bytes`);
}

// Clean up temp html
try { fs.unlinkSync(tempHtmlPath); } catch (_) {}

// Construct Windows .ICO file with all resolutions
// Format:
// ICONDIR (6 bytes):
//   idReserved (2 bytes, 0)
//   idType (2 bytes, 1)
//   idCount (2 bytes, number of images)
// ICONDIRENTRY (16 bytes per image):
//   bWidth (1 byte, 0 for 256)
//   bHeight (1 byte, 0 for 256)
//   bColorCount (1 byte, 0)
//   bReserved (1 byte, 0)
//   wPlanes (2 bytes, 1)
//   wBitCount (2 bytes, 32)
//   dwBytesInRes (4 bytes)
//   dwImageOffset (4 bytes)
// Followed by raw PNG data for each image.

const numImages = sizes.length;
const headerSize = 6 + numImages * 16;
let currentOffset = headerSize;

const header = Buffer.alloc(headerSize);
header.writeUInt16LE(0, 0); // reserved
header.writeUInt16LE(1, 2); // ICO type
header.writeUInt16LE(numImages, 4);

sizes.forEach((size, idx) => {
  const entryOffset = 6 + idx * 16;
  const pngBuf = pngBuffers[size];
  
  header.writeUInt8(size === 256 ? 0 : size, entryOffset); // width
  header.writeUInt8(size === 256 ? 0 : size, entryOffset + 1); // height
  header.writeUInt8(0, entryOffset + 2); // color count
  header.writeUInt8(0, entryOffset + 3); // reserved
  header.writeUInt16LE(1, entryOffset + 4); // color planes
  header.writeUInt16LE(32, entryOffset + 6); // bits per pixel
  header.writeUInt32LE(pngBuf.length, entryOffset + 8); // image size
  header.writeUInt32LE(currentOffset, entryOffset + 12); // image offset

  currentOffset += pngBuf.length;
});

const icoBuffer = Buffer.concat([header, ...sizes.map(s => pngBuffers[s])]);

// Write to installer/app_icon.ico
const installerIco = path.resolve(__dirname, '../installer/app_icon.ico');
fs.writeFileSync(installerIco, icoBuffer);
console.log(`Saved multi-resolution ICO to: ${installerIco} (${icoBuffer.length} bytes)`);

// Ensure assets/ directory exists and save icon_256.png for runtime window icon
const assetsDir = path.resolve(__dirname, '../assets');
if (!fs.existsSync(assetsDir)) fs.mkdirSync(assetsDir, { recursive: true });

const assetPng = path.join(assetsDir, 'icon_256.png');
fs.writeFileSync(assetPng, pngBuffers[256]);
console.log(`Saved runtime 256x256 window icon to: ${assetPng}`);

const assetIco = path.join(assetsDir, 'icon.ico');
fs.writeFileSync(assetIco, icoBuffer);
console.log("Icon build completed successfully!");
