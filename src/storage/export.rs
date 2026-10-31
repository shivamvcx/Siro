use crate::model::board::Board;
use crate::model::element::*;
use crate::model::planner::PlannerData;
use image::{ImageBuffer, Rgba};
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub struct ExportManager;

impl ExportManager {
    /// Exports the Bill of Materials (BOM) to CSV format
    pub fn export_bom_to_csv(planner: &PlannerData, path: &Path) -> Result<(), String> {
        let mut file = File::create(path).map_err(|e| format!("CSV creation error: {}", e))?;
        writeln!(
            file,
            "\"Item / Part\",\"Part Number\",\"Specification\",\"Quantity\",\"Unit Cost ($)\",\"Line Total ($)\",\"Acquired\",\"Supplier URL\",\"Notes\""
        )
        .map_err(|e| format!("CSV header write error: {}", e))?;

        for item in &planner.bom {
            writeln!(
                file,
                "\"{}\",\"{}\",\"{}\",{},{:.2},{:.2},\"{}\",\"{}\",\"{}\"",
                escape_csv(&item.name),
                escape_csv(&item.part_number),
                escape_csv(&item.spec_value),
                item.quantity,
                item.unit_cost,
                item.line_total(),
                if item.acquired { "Yes" } else { "No" },
                escape_csv(&item.supplier_url),
                escape_csv(&item.footprint_or_notes)
            )
            .map_err(|e| format!("CSV row write error: {}", e))?;
        }

        // Summary row
        writeln!(
            file,
            "\"TOTAL BUDGET\",,\"\",,,{:.2},,,",
            planner.total_cost()
        )
        .map_err(|e| format!("CSV summary write error: {}", e))?;

        Ok(())
    }

    /// Exports the board as an SVG vector file
    pub fn export_board_to_svg(board: &Board, path: &Path) -> Result<(), String> {
        let bounds = board.content_bounds();
        let width = bounds.width().max(100.0);
        let height = bounds.height().max(100.0);
        let min_x = bounds.min.x;
        let min_y = bounds.min.y;

        let mut svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{} {} {} {}" width="{}" height="{}">"#,
            min_x, min_y, width, height, width, height
        );
        svg.push_str("\n<style>\n");
        svg.push_str("  .bg { fill: #181a20; }\n");
        svg.push_str("  .txt { font-family: sans-serif; font-size: 14px; fill: #ffffff; }\n");
        svg.push_str("</style>\n");
        svg.push_str(&format!(
            r#"<rect x="{}" y="{}" width="{}" height="{}" class="bg"/>"#,
            min_x, min_y, width, height
        ));
        svg.push('\n');

        for elem in &board.elements {
            match elem {
                BoardElement::Stroke(s) => {
                    if s.points.len() >= 2 {
                        let mut d = format!("M {} {}", s.points[0].pos.x, s.points[0].pos.y);
                        for p in &s.points[1..] {
                            d.push_str(&format!(" L {} {}", p.pos.x, p.pos.y));
                        }
                        let [r, g, b, a] = s.color.to_array();
                        let opacity = a as f32 / 255.0;
                        svg.push_str(&format!(
                            r#"<path d="{}" stroke="rgb({},{},{})" stroke-opacity="{}" stroke-width="{}" fill="none" stroke-linecap="round" stroke-linejoin="round"/>"#,
                            d, r, g, b, opacity, s.width
                        ));
                        svg.push('\n');
                    }
                }
                BoardElement::Shape(s) => {
                    if s.vertices.len() >= 2 {
                        let [sr, sg, sb, _] = s.stroke_color.to_array();
                        let [fr, fg, fb, fa] = s.fill_color.to_array();
                        let fill_opacity = fa as f32 / 255.0;

                        match s.kind {
                            ShapeKind::Rectangle | ShapeKind::RoundedRectangle => {
                                let rect = egui::Rect::from_two_pos(s.vertices[0], s.vertices[1]);
                                let rx = if s.kind == ShapeKind::RoundedRectangle { 8.0 } else { 0.0 };
                                svg.push_str(&format!(
                                    r#"<rect x="{}" y="{}" width="{}" height="{}" rx="{}" fill="rgb({},{},{})" fill-opacity="{}" stroke="rgb({},{},{})" stroke-width="{}"/>"#,
                                    rect.min.x, rect.min.y, rect.width(), rect.height(), rx, fr, fg, fb, fill_opacity, sr, sg, sb, s.stroke_width
                                ));
                                svg.push('\n');
                            }
                            ShapeKind::Ellipse => {
                                let rect = egui::Rect::from_two_pos(s.vertices[0], s.vertices[1]);
                                svg.push_str(&format!(
                                    r#"<ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="rgb({},{},{})" fill-opacity="{}" stroke="rgb({},{},{})" stroke-width="{}"/>"#,
                                    rect.center().x, rect.center().y, rect.width() / 2.0, rect.height() / 2.0, fr, fg, fb, fill_opacity, sr, sg, sb, s.stroke_width
                                ));
                                svg.push('\n');
                            }
                            ShapeKind::Line | ShapeKind::Arrow => {
                                svg.push_str(&format!(
                                    r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="rgb({},{},{})" stroke-width="{}"/>"#,
                                    s.vertices[0].x, s.vertices[0].y, s.vertices[1].x, s.vertices[1].y, sr, sg, sb, s.stroke_width
                                ));
                                svg.push('\n');
                            }
                            ShapeKind::Diamond | ShapeKind::Triangle => {
                                let pts_str = s
                                    .vertices
                                    .iter()
                                    .map(|v| format!("{},{}", v.x, v.y))
                                    .collect::<Vec<_>>()
                                    .join(" ");
                                svg.push_str(&format!(
                                    r#"<polygon points="{}" fill="rgb({},{},{})" fill-opacity="{}" stroke="rgb({},{},{})" stroke-width="{}"/>"#,
                                    pts_str, fr, fg, fb, fill_opacity, sr, sg, sb, s.stroke_width
                                ));
                                svg.push('\n');
                            }
                        }
                    }
                }
                BoardElement::Wire(w) => {
                    let [r, g, b, _] = w.color.to_color32().to_array();
                    let mid_x = (w.start.pos.x + w.end.pos.x) / 2.0;
                    svg.push_str(&format!(
                        r#"<path d="M {} {} L {} {} L {} {} L {} {}" stroke="rgb({},{},{})" stroke-width="{}" fill="none"/>"#,
                        w.start.pos.x, w.start.pos.y, mid_x, w.start.pos.y, mid_x, w.end.pos.y, w.end.pos.x, w.end.pos.y, r, g, b, w.width
                    ));
                    svg.push('\n');
                }
                BoardElement::Table(t) => {
                    svg.push_str(&format!(
                        r##"<rect x="{}" y="{}" width="{}" height="{}" fill="#1c1e24" stroke="#464b5a" stroke-width="1.5" rx="3"/>"##,
                        t.pos.x, t.pos.y, t.total_width(), t.total_height()
                    ));
                    svg.push('\n');
                    let mut cur_y = t.pos.y;
                    for (r_idx, row) in t.cells.iter().enumerate() {
                        let mut cur_x = t.pos.x;
                        for (c_idx, cell) in row.iter().enumerate() {
                            let cw = t.col_widths.get(c_idx).copied().unwrap_or(100.0);
                            let color = if r_idx == 0 { "#00dcff" } else { "#ffffff" };
                            svg.push_str(&format!(
                                r#"<text x="{}" y="{}" class="txt" fill="{}">{}</text>"#,
                                cur_x + 6.0, cur_y + 18.0, color, escape_xml(cell)
                            ));
                            cur_x += cw;
                        }
                        cur_y += t.row_height();
                    }
                    svg.push('\n');
                }
                BoardElement::Component(c) => {
                    svg.push_str(&format!(
                        r##"<rect x="{}" y="{}" width="{}" height="{}" fill="#22252c" stroke="#555a68" stroke-width="1.5" rx="4"/>"##,
                        c.pos.x, c.pos.y, c.width, c.height
                    ));
                    svg.push_str(&format!(
                        r#"<text x="{}" y="{}" class="txt" text-anchor="middle">{} ({})</text>"#,
                        c.pos.x + c.width / 2.0, c.pos.y + 20.0, escape_xml(&c.designator), escape_xml(&c.value)
                    ));
                    svg.push('\n');
                }
                BoardElement::Text(t) => {
                    svg.push_str(&format!(
                        r#"<text x="{}" y="{}" class="txt">{}</text>"#,
                        t.pos.x, t.pos.y + 15.0, escape_xml(&t.text)
                    ));
                    svg.push('\n');
                }
                _ => {}
            }
        }

        svg.push_str("</svg>\n");

        let mut file = File::create(path).map_err(|e| format!("SVG creation error: {}", e))?;
        file.write_all(svg.as_bytes()).map_err(|e| format!("SVG write error: {}", e))?;

        Ok(())
    }

    /// Exports the board as a high-resolution PNG image
    pub fn export_board_to_png(board: &Board, path: &Path) -> Result<(), String> {
        let bounds = board.content_bounds();
        let w = (bounds.width().max(400.0) as u32).min(4096);
        let h = (bounds.height().max(400.0) as u32).min(4096);

        let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(w, h);
        // Dark background
        for pixel in img.pixels_mut() {
            *pixel = Rgba([24, 26, 32, 255]);
        }

        // Draw elements onto image buffer
        for elem in &board.elements {
            match elem {
                BoardElement::Stroke(s) => {
                    let [r, g, b, _] = s.color.to_array();
                    let color = Rgba([r, g, b, 255]);
                    for pair in s.points.windows(2) {
                        let x0 = ((pair[0].pos.x - bounds.min.x) as i32).clamp(0, w as i32 - 1);
                        let y0 = ((pair[0].pos.y - bounds.min.y) as i32).clamp(0, h as i32 - 1);
                        let x1 = ((pair[1].pos.x - bounds.min.x) as i32).clamp(0, w as i32 - 1);
                        let y1 = ((pair[1].pos.y - bounds.min.y) as i32).clamp(0, h as i32 - 1);
                        draw_line_on_image(&mut img, x0, y0, x1, y1, color);
                    }
                }
                BoardElement::Wire(wire) => {
                    let [r, g, b, _] = wire.color.to_color32().to_array();
                    let color = Rgba([r, g, b, 255]);
                    let mid_x = (wire.start.pos.x + wire.end.pos.x) / 2.0;
                    let p1 = (wire.start.pos.x, wire.start.pos.y);
                    let p2 = (mid_x, wire.start.pos.y);
                    let p3 = (mid_x, wire.end.pos.y);
                    let p4 = (wire.end.pos.x, wire.end.pos.y);
                    for &(a, b) in &[(&p1, &p2), (&p2, &p3), (&p3, &p4)] {
                        let x0 = ((a.0 - bounds.min.x) as i32).clamp(0, w as i32 - 1);
                        let y0 = ((a.1 - bounds.min.y) as i32).clamp(0, h as i32 - 1);
                        let x1 = ((b.0 - bounds.min.x) as i32).clamp(0, w as i32 - 1);
                        let y1 = ((b.1 - bounds.min.y) as i32).clamp(0, h as i32 - 1);
                        draw_line_on_image(&mut img, x0, y0, x1, y1, color);
                    }
                }
                _ => {
                    let aabb = elem.aabb();
                    let min_x = ((aabb.min.x - bounds.min.x) as i32).clamp(0, w as i32 - 1) as u32;
                    let max_x = ((aabb.max.x - bounds.min.x) as i32).clamp(0, w as i32 - 1) as u32;
                    let min_y = ((aabb.min.y - bounds.min.y) as i32).clamp(0, h as i32 - 1) as u32;
                    let max_y = ((aabb.max.y - bounds.min.y) as i32).clamp(0, h as i32 - 1) as u32;

                    let color = match elem {
                        BoardElement::Component(_) => Rgba([40, 190, 80, 255]),
                        BoardElement::Shape(_) => Rgba([0, 180, 240, 255]),
                        BoardElement::Text(t) if t.is_sticky => Rgba([255, 230, 100, 255]),
                        BoardElement::Text(_) => Rgba([220, 225, 235, 255]),
                        BoardElement::Table(_) => Rgba([70, 75, 90, 255]),
                        _ => Rgba([140, 150, 170, 255]),
                    };

                    for y in min_y..=max_y {
                        for x in min_x..=max_x {
                            if x == min_x || x == max_x || y == min_y || y == max_y {
                                img.put_pixel(x, y, color);
                            }
                        }
                    }
                }
            }
        }

        img.save(path).map_err(|e| format!("PNG save error: {}", e))?;
        Ok(())
    }
}

fn escape_csv(s: &str) -> String {
    s.replace('"', "\"\"")
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn draw_line_on_image(img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, mut x0: i32, mut y0: i32, x1: i32, y1: i32, color: Rgba<u8>) {
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let (w, h) = (img.width() as i32, img.height() as i32);

    loop {
        if x0 >= 0 && x0 < w && y0 >= 0 && y0 < h {
            img.put_pixel(x0 as u32, y0 as u32, color);
        }
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}
