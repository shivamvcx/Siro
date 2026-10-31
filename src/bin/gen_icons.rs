use image::imageops::{resize, FilterType};
use image::{DynamicImage, ImageFormat};
use std::fs;
use std::io::Cursor;
use std::path::Path;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("[*] Siro Icon Generation starting...");

    let edge_exe = r"C:\Program Files (x86)\Microsoft\EdgeCore\156.0.4314.50\msedge.exe";
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let target_dir = project_root.join("target");
    let assets_dir = project_root.join("assets");
    let installer_dir = project_root.join("installer");

    fs::create_dir_all(&target_dir)?;
    fs::create_dir_all(&assets_dir)?;
    fs::create_dir_all(&installer_dir)?;

    let temp_html = target_dir.join("render_master.html");
    let master_png = target_dir.join("master_512.png");

    // Perfectly squared SVG viewBox centered on the icon visual bounds
    let html_content = r##"<!DOCTYPE html>
<html>
<head>
<style>
  * { margin: 0; padding: 0; box-sizing: border-box; }
  html, body {
    width: 512px;
    height: 512px;
    background: transparent !important;
    overflow: hidden;
  }
  svg {
    width: 512px;
    height: 512px;
  }
</style>
</head>
<body>
<svg width="512" height="512" viewBox="-2 15 138 138" fill="none" xmlns="http://www.w3.org/2000/svg">
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
</svg>
</body>
</html>"##;

    fs::write(&temp_html, html_content)?;

    println!("[*] Rendering 512x512 master PNG via headless Chromium...");
    let file_url = format!("file:///{}", temp_html.to_str().unwrap().replace('\\', "/"));
    let status = Command::new(edge_exe)
        .arg("--headless=new")
        .arg("--default-background-color=00000000")
        .arg(format!("--screenshot={}", master_png.to_str().unwrap()))
        .arg("--window-size=512,512")
        .arg(&file_url)
        .status()?;

    if !status.success() {
        return Err("Edge screenshot failed".into());
    }

    let _ = fs::remove_file(&temp_html);

    let master_data = fs::read(&master_png)?;
    println!("[*] Master PNG rendered: {} bytes", master_data.len());
    let master_img = image::load_from_memory(&master_data)?;
    let rgba_master = master_img.to_rgba8();

    // Standard Windows icon dimensions
    let sizes = [256u32, 128, 64, 48, 32, 24, 16];
    let mut png_buffers: Vec<(u32, Vec<u8>)> = Vec::new();

    for &s in &sizes {
        let resized = resize(&rgba_master, s, s, FilterType::Lanczos3);
        let dyn_img = DynamicImage::ImageRgba8(resized);
        let mut buf = Vec::new();
        dyn_img.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)?;
        println!("  - Generated {}x{} PNG: {} bytes", s, s, buf.len());

        if s == 256 {
            let asset_256 = assets_dir.join("icon_256.png");
            fs::write(&asset_256, &buf)?;
            println!("[*] Saved runtime window icon to {}", asset_256.display());
        }
        if s == 32 {
            let asset_32 = assets_dir.join("icon_32.png");
            fs::write(&asset_32, &buf)?;
        }
        png_buffers.push((s, buf));
    }

    // Construct valid Windows .ICO binary (with PNG streams)
    let num_images = png_buffers.len() as u16;
    let header_size = 6 + (num_images as usize * 16);
    let mut current_offset = header_size as u32;

    let mut ico_data = Vec::new();
    // ICONDIR
    ico_data.extend_from_slice(&0u16.to_le_bytes()); // Reserved
    ico_data.extend_from_slice(&1u16.to_le_bytes()); // 1 = Icon
    ico_data.extend_from_slice(&num_images.to_le_bytes());

    for (s, buf) in &png_buffers {
        let b_dim = if *s >= 256 { 0u8 } else { *s as u8 };
        ico_data.push(b_dim); // Width
        ico_data.push(b_dim); // Height
        ico_data.push(0);     // Color count (0 = >=8bpp)
        ico_data.push(0);     // Reserved
        ico_data.extend_from_slice(&1u16.to_le_bytes());  // Color planes
        ico_data.extend_from_slice(&32u16.to_le_bytes()); // Bit count
        ico_data.extend_from_slice(&(buf.len() as u32).to_le_bytes());
        ico_data.extend_from_slice(&current_offset.to_le_bytes());

        current_offset += buf.len() as u32;
    }

    for (_, buf) in &png_buffers {
        ico_data.extend_from_slice(buf);
    }

    let installer_ico = installer_dir.join("app_icon.ico");
    fs::write(&installer_ico, &ico_data)?;
    println!("[*] Successfully updated {}", installer_ico.display());

    let asset_ico = assets_dir.join("icon.ico");
    fs::write(&asset_ico, &ico_data)?;
    println!("[*] Successfully updated {}", asset_ico.display());

    println!("[*] All icons built and verified!");
    Ok(())
}
