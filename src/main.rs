use ab_glyph::ScaleFont;
use ab_glyph::{Font, FontRef, PxScale, point};
use image::{ImageBuffer, Rgba, RgbaImage};
use rand::Rng;

fn main() {
    // 内嵌字体 - 替换为您的实际字体文件
    let font = FontRef::try_from_slice(include_bytes!("../fonts/PinRuShouXieTi-1.ttf"))
        .expect("Failed to load font");

    // let font = FontRef::try_from_slice(include_bytes!("../fonts/NotoSansSC-Regular.ttf"))
    //     .expect("Failed to load font");
    // 示例文本
    let examples = vec![
        ("Hello, World!", "English"),
        // ("你好，世界！", "Chinese"),
        // ("1234567890", "Numbers"),
        // ("Hello, 世界123！", "Mixed"),
    ];

    for (text, description) in examples {
        let (w, h) = (400, 300);
        let mut img: RgbaImage = ImageBuffer::new(w, h);

        // 创建随机渐变背景
        let color1 = generate_random_color();
        let color2 = generate_random_color();
        create_gradient_background(&mut img, color1, color2);

        // 获取背景中心区域的平均颜色
        let bg_center_color = sample_center_color(&img, w, h);

        // 根据背景色选择合适的前景色
        let fg_color = choose_contrasting_color(bg_center_color);
        println!("Rendering: {}", description);
        render_text(&mut img, &font, text, fg_color, w, h);
        let filename = format!(
            "output_{}.webp",
            description.to_lowercase().replace(" ", "_")
        );
        img.save(&filename).unwrap();
        println!("Saved to {}", filename);
    }
}

fn render_text(img: &mut RgbaImage, font: &FontRef, text: &str, fg_color: [u8; 3], w: u32, h: u32) {
    let scale = PxScale::from(48.0);
    let scaled_font = font.as_scaled(scale);
    println!("scaled_font.h_advance(font.glyph_id('a')) = {}", text);
    // 计算文本的包围盒
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    let mut x_pos = 0.0;
    for c in text.chars() {
        let glyph_id = font.glyph_id(c);
        let mut glyph = glyph_id.with_scale(scale);
        glyph.position = point(x_pos, 0.0);

        if let Some(outlined) = font.outline_glyph(glyph.clone()) {
            let bounds = outlined.px_bounds();
            min_x = min_x.min(bounds.min.x);
            min_y = min_y.min(bounds.min.y);
            max_x = max_x.max(bounds.max.x);
            max_y = max_y.max(bounds.max.y);
        }

        x_pos += scaled_font.h_advance(glyph_id);
    }

    // 计算文本的实际宽度和高度
    let text_width = max_x - min_x;
    let text_height = max_y - min_y;

    // 计算起始位置（真正居中）
    let start_x = (w as f32 - text_width) / 2.0 - min_x;
    let start_y = (h as f32 - text_height) / 2.0 + scaled_font.ascent();

    // 渲染每个字符
    let mut x_pos = start_x;
    for c in text.chars() {
        let glyph_id = font.glyph_id(c);
        let mut glyph = glyph_id.with_scale(scale);
        glyph.position = point(x_pos, start_y);

        if let Some(outlined) = font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();
            outlined.draw(|x, y, c| {
                let px = x as i32 + bounds.min.x as i32;
                let py = y as i32 + bounds.min.y as i32;

                if px >= 0 && py >= 0 {
                    let px = px as u32;
                    let py = py as u32;

                    if px < w && py < h {
                        // 应用抗锯齿
                        let alpha = (c * 255.0) as u8;
                        println!("alpha = {},c:{}", alpha, c);
                        // if alpha > 0 {
                        //     let pixel = Rgba([fg_color[0], fg_color[1], fg_color[2], alpha]);
                        //     img.put_pixel(px, py, pixel);
                        // }
                    }
                }
            });
        }

        // 前进到下一个字符位置
        x_pos += scaled_font.h_advance(glyph_id);
    }
}

/// 生成随机颜色
fn generate_random_color() -> [u8; 3] {
    let mut rng = rand::rng();
    [rng.random(), rng.random(), rng.random()]
}

/// 创建渐变背景
fn create_gradient_background(img: &mut RgbaImage, color1: [u8; 3], color2: [u8; 3]) {
    for y in 0..img.height() {
        for x in 0..img.width() {
            // 计算渐变因子
            let factor =
                (x as f32 / img.width() as f32) * 0.5 + (y as f32 / img.height() as f32) * 0.5;

            // 插值计算颜色
            let r = (color1[0] as f32 * (1.0 - factor) + color2[0] as f32 * factor) as u8;
            let g = (color1[1] as f32 * (1.0 - factor) + color2[1] as f32 * factor) as u8;
            let b = (color1[2] as f32 * (1.0 - factor) + color2[2] as f32 * factor) as u8;

            img.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }
}

/// 采样中心区域颜色
fn sample_center_color(img: &RgbaImage, w: u32, h: u32) -> [u8; 3] {
    let center_x = w / 2;
    let center_y = h / 2;

    // 采样中心区域 3x3 像素
    let mut r_sum = 0;
    let mut g_sum = 0;
    let mut b_sum = 0;
    let mut count = 0;

    for y in center_y.saturating_sub(1)..=center_y.saturating_add(1) {
        for x in center_x.saturating_sub(1)..=center_x.saturating_add(1) {
            if let Some(pixel) = img.get_pixel_checked(x, y) {
                r_sum += pixel[0] as u32;
                g_sum += pixel[1] as u32;
                b_sum += pixel[2] as u32;
                count += 1;
            }
        }
    }

    if count > 0 {
        [
            (r_sum / count) as u8,
            (g_sum / count) as u8,
            (b_sum / count) as u8,
        ]
    } else {
        [128, 128, 128] // 默认灰色
    }
}

/// 选择对比度高的前景色
fn choose_contrasting_color(bg_color: [u8; 3]) -> [u8; 3] {
    // 计算背景亮度 (0-255)
    let brightness =
        (bg_color[0] as u32 * 299 + bg_color[1] as u32 * 587 + bg_color[2] as u32 * 114) / 1000;

    // 根据背景亮度选择前景色
    if brightness > 128 {
        [0, 0, 0] // 深色文本 (黑色)
    } else {
        [255, 255, 255] // 浅色文本 (白色)
    }
}
