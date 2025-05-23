use clap::Parser;
use colored::*;
use image::{GenericImageView, Pixel, Rgb, RgbImage};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "bitify")]
#[command(about = "Convert images to colorful ASCII art")]
#[command(long_about = "
Bitify converts images to colorful ASCII art and saves them as PNG files with black backgrounds.

DENSITY PRESETS:
  low     - 10 chars  | Fast, chunky 8-bit look, good for pixel art
  medium  - 20 chars  | Balanced detail and performance  
  high    - 40 chars  | Fine detail, slower processing
  ultra   - 70 chars  | Maximum detail, complex textures
  extreme - 95 chars  | Ultra-fine detail, very slow

EXAMPLES:
  bitify image.jpg                    # Medium density (default)
  bitify -d low image.jpg             # Low density for retro look
  bitify -d ultra -w 120 image.jpg    # Ultra density with custom width
")]
struct Args {
    image_path: String,
    
    #[arg(short, long, default_value = "80")]
    #[arg(help = "Output width in characters (overrides density preset)")]
    width: u32,
    
    #[arg(short, long, default_value = "medium")]
    #[arg(value_parser = parse_density)]
    #[arg(help = "ASCII density preset: low, medium, high, ultra, extreme")]
    density: DensityPreset,
}

#[derive(Clone, Debug, PartialEq)]
enum DensityPreset {
    Low,
    Medium, 
    High,
    Ultra,
    Extreme,
}

fn parse_density(s: &str) -> Result<DensityPreset, String> {
    match s.to_lowercase().as_str() {
        "low" => Ok(DensityPreset::Low),
        "medium" => Ok(DensityPreset::Medium),
        "high" => Ok(DensityPreset::High),
        "ultra" => Ok(DensityPreset::Ultra),
        "extreme" => Ok(DensityPreset::Extreme),
        _ => Err(format!("Invalid density '{}'. Use: low, medium, high, ultra, extreme", s)),
    }
}

impl DensityPreset {
    fn get_chars(&self) -> &'static [char] {
        match self {
            DensityPreset::Low => &[' ', '.', ':', '+', '#', '@'],
            DensityPreset::Medium => &[' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'],
            DensityPreset::High => &[' ', '.', '\'', '`', '^', '"', ',', ':', ';', 'I', 'l', '!', 'i', '>', '<', '~', '+', '_', '-', '?', ']', '[', '}', '{', '1', ')', '(', '|', '\\', '/', 't', 'f', 'j', 'r', 'x', 'n', 'u', 'v', 'c', 'z', 'X', 'Y', 'U', 'J', 'C', 'L', 'Q', '0', 'O', 'Z', 'm', 'w', 'q', 'p', 'd', 'b', 'k', 'h', 'a', 'o', '*', '#', 'M', 'W', '&', '8', '%', 'B', '@'],
            DensityPreset::Ultra => &[' ', '.', '\'', '`', '^', '"', ',', ':', ';', 'I', 'l', '!', 'i', '>', '<', '~', '+', '_', '-', '?', ']', '[', '}', '{', '1', ')', '(', '|', '\\', '/', 't', 'f', 'j', 'r', 'x', 'n', 'u', 'v', 'c', 'z', 'X', 'Y', 'U', 'J', 'C', 'L', 'Q', '0', 'O', 'Z', 'm', 'w', 'q', 'p', 'd', 'b', 'k', 'h', 'a', 'o', '*', '#', 'M', 'W', '&', '8', '%', 'B', '@', '$'],
            DensityPreset::Extreme => &[' ', '.', '\'', '`', '^', '"', ',', ':', ';', 'I', 'l', '!', 'i', '>', '<', '~', '+', '_', '-', '?', ']', '[', '}', '{', '1', ')', '(', '|', '\\', '/', 't', 'f', 'j', 'r', 'x', 'n', 'u', 'v', 'c', 'z', 'X', 'Y', 'U', 'J', 'C', 'L', 'Q', '0', 'O', 'Z', 'm', 'w', 'q', 'p', 'd', 'b', 'k', 'h', 'a', 'o', '*', '#', 'M', 'W', '&', '8', '%', 'B', '@', '$', 'A', 'G', 'H', 'K', 'P', 'R', 'S', 'T', 'V', 'g', 's', 'y', 'e', 'F', 'D', 'N', '2', '3', '4', '5', '6', '7', '9', 'E'],
        }
    }
    
    fn get_default_width(&self) -> u32 {
        match self {
            DensityPreset::Low => 40,
            DensityPreset::Medium => 80,
            DensityPreset::High => 120,
            DensityPreset::Ultra => 150,
            DensityPreset::Extreme => 200,
        }
    }
}

fn main() {
    let args = Args::parse();
    
    let effective_width = if args.width == 80 && args.density != DensityPreset::Medium {
        args.density.get_default_width()
    } else {
        args.width
    };
    
    match process_image(&args.image_path, effective_width, &args.density) {
        Ok((ascii_art, ascii_data)) => {
            println!("{}", ascii_art);
            if let Err(e) = save_ascii_png(&ascii_data, &args.image_path, &args.density) {
                eprintln!("Warning: Failed to save ASCII art: {}", e);
            } else {
                println!("\n✨ ASCII art saved to ~/Bitify/ (density: {:?})", args.density);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}

#[derive(Clone)]
struct AsciiPixel {
    character: char,
    color: (u8, u8, u8),
}

fn process_image(image_path: &str, target_width: u32, density: &DensityPreset) -> Result<(String, Vec<Vec<AsciiPixel>>), Box<dyn std::error::Error>> {
    let img = image::open(image_path)?;
    let (width, height) = img.dimensions();
    
    let aspect_ratio = height as f32 / width as f32;
    let target_height = (target_width as f32 * aspect_ratio * 0.5) as u32;
    
    let resized = img.resize_exact(target_width, target_height, image::imageops::FilterType::Nearest);
    
    let ascii_chars = density.get_chars();
    let mut ascii_art = String::new();
    let mut ascii_data = Vec::new();
    
    for y in 0..target_height {
        let mut row = Vec::new();
        for x in 0..target_width {
            let pixel = resized.get_pixel(x, y);
            let rgba = pixel.to_rgba();
            
            let brightness = (rgba[0] as f32 * 0.299 + rgba[1] as f32 * 0.587 + rgba[2] as f32 * 0.114) / 255.0;
            
            let char_index = (brightness * (ascii_chars.len() - 1) as f32) as usize;
            let ascii_char = ascii_chars[char_index];
            
            let ascii_pixel = AsciiPixel {
                character: ascii_char,
                color: (rgba[0], rgba[1], rgba[2]),
            };
            
            row.push(ascii_pixel.clone());
            
            let colored_char = format!("{}", ascii_char)
                .truecolor(rgba[0], rgba[1], rgba[2]);
            
            ascii_art.push_str(&colored_char.to_string());
        }
        ascii_data.push(row);
        ascii_art.push('\n');
    }
    
    Ok((ascii_art, ascii_data))
}

fn save_ascii_png(ascii_data: &[Vec<AsciiPixel>], original_path: &str, density: &DensityPreset) -> Result<(), Box<dyn std::error::Error>> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let bitify_dir = home_dir.join("Bitify");
    
    fs::create_dir_all(&bitify_dir)?;
    
    let path_buf = PathBuf::from(original_path);
    let original_name = path_buf
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("image");
    
    let output_path = bitify_dir.join(format!("{}_{:?}_ascii.png", original_name, density));
    
    let char_width = 8;
    let char_height = 12;
    let img_width = ascii_data[0].len() as u32 * char_width;
    let img_height = ascii_data.len() as u32 * char_height;
    
    let mut img = RgbImage::from_pixel(img_width, img_height, Rgb([0, 0, 0]));
    
    for (row_idx, row) in ascii_data.iter().enumerate() {
        for (col_idx, ascii_pixel) in row.iter().enumerate() {
            let base_x = col_idx as u32 * char_width;
            let base_y = row_idx as u32 * char_height;
            
            let pattern = get_char_pattern(ascii_pixel.character);
            let color = Rgb([ascii_pixel.color.0, ascii_pixel.color.1, ascii_pixel.color.2]);
            
            for (py, row_pattern) in pattern.iter().enumerate() {
                for (px, &pixel_on) in row_pattern.iter().enumerate() {
                    if pixel_on {
                        let x = base_x + px as u32;
                        let y = base_y + py as u32;
                        if x < img_width && y < img_height {
                            img.put_pixel(x, y, color);
                        }
                    }
                }
            }
        }
    }
    
    img.save(output_path)?;
    Ok(())
}

fn get_char_pattern(ch: char) -> &'static [[bool; 8]; 12] {
    match ch {
        ' ' => &SPACE,
        '.' => &DOT,
        ':' => &COLON,
        '-' => &DASH,
        '=' => &EQUALS,
        '+' => &PLUS,
        '*' => &ASTERISK,
        '#' => &HASH,
        '%' => &PERCENT,
        '@' => &AT,
        '\'' => &DOT,
        '`' => &DOT,
        '^' => &CARET,
        '"' => &COLON,
        ',' => &DOT,
        ';' => &COLON,
        'I' => &PIPE_PATTERN,
        'l' => &PIPE_PATTERN,
        '!' => &PIPE_PATTERN,
        'i' => &DOT,
        '>' => &GREATER,
        '<' => &LESS,
        '~' => &TILDE,
        '_' => &UNDERSCORE,
        '?' => &QUESTION,
        ']' => &BRACKET_RIGHT,
        '[' => &BRACKET_LEFT,
        '}' => &BRACKET_RIGHT,
        '{' => &BRACKET_LEFT,
        '1' => &PIPE_PATTERN,
        ')' => &PAREN_RIGHT,
        '(' => &PAREN_LEFT,
        '|' => &PIPE_PATTERN,
        '\\' => &BACKSLASH,
        '/' => &SLASH,
        't' | 'f' | 'j' | 'r' | 'x' | 'n' | 'u' | 'v' | 'c' | 'z' => &SMALL_BLOCK,
        'X' | 'Y' | 'U' | 'J' | 'C' | 'L' | 'Q' | 'O' | 'Z' => &MEDIUM_BLOCK,
        '0' => &ZERO,
        'm' | 'w' | 'q' | 'p' | 'd' | 'b' | 'k' | 'h' | 'a' | 'o' => &SMALL_BLOCK,
        'M' | 'W' | 'B' | 'A' | 'G' | 'H' | 'K' | 'P' | 'R' | 'S' | 'T' | 'V' => &LARGE_BLOCK,
        '&' | '8' | '$' => &LARGE_BLOCK,
        'g' | 's' | 'y' | 'e' | 'F' | 'D' | 'N' => &MEDIUM_BLOCK,
        '2' | '3' | '4' | '5' | '6' | '7' | '9' | 'E' => &MEDIUM_BLOCK,
        _ => &SPACE,
    }
}

const SPACE: [[bool; 8]; 12] = [
    [false; 8],
    [false; 8],
    [false; 8],
    [false; 8],
    [false; 8],
    [false; 8],
    [false; 8],
    [false; 8],
    [false; 8],
    [false; 8],
    [false; 8],
    [false; 8],
];

const DOT: [[bool; 8]; 12] = [
    [false; 8],
    [false; 8],
    [false; 8],
    [false; 8],
    [false; 8],
    [false; 8],
    [false; 8],
    [false; 8],
    [false; 8],
    [false, false, false, true, true, false, false, false],
    [false, false, false, true, true, false, false, false],
    [false; 8],
];

const COLON: [[bool; 8]; 12] = [
    [false; 8],
    [false; 8],
    [false; 8],
    [false, false, false, true, true, false, false, false],
    [false, false, false, true, true, false, false, false],
    [false; 8],
    [false; 8],
    [false, false, false, true, true, false, false, false],
    [false, false, false, true, true, false, false, false],
    [false; 8],
    [false; 8],
    [false; 8],
];

const DASH: [[bool; 8]; 12] = [
    [false; 8],
    [false; 8],
    [false; 8],
    [false; 8],
    [false; 8],
    [false; 8],
    [false, true, true, true, true, true, true, false],
    [false; 8],
    [false; 8],
    [false; 8],
    [false; 8],
    [false; 8],
];

const EQUALS: [[bool; 8]; 12] = [
    [false; 8],
    [false; 8],
    [false; 8],
    [false; 8],
    [false, true, true, true, true, true, true, false],
    [false; 8],
    [false; 8],
    [false, true, true, true, true, true, true, false],
    [false; 8],
    [false; 8],
    [false; 8],
    [false; 8],
];

const PLUS: [[bool; 8]; 12] = [
    [false; 8],
    [false; 8],
    [false; 8],
    [false, false, false, true, true, false, false, false],
    [false, false, false, true, true, false, false, false],
    [false, true, true, true, true, true, true, false],
    [false, true, true, true, true, true, true, false],
    [false, false, false, true, true, false, false, false],
    [false, false, false, true, true, false, false, false],
    [false; 8],
    [false; 8],
    [false; 8],
];

const ASTERISK: [[bool; 8]; 12] = [
    [false; 8],
    [false; 8],
    [false, false, true, false, false, true, false, false],
    [false, false, false, true, true, false, false, false],
    [false, true, true, true, true, true, true, false],
    [false, false, false, true, true, false, false, false],
    [false, false, true, false, false, true, false, false],
    [false; 8],
    [false; 8],
    [false; 8],
    [false; 8],
    [false; 8],
];

const HASH: [[bool; 8]; 12] = [
    [false; 8],
    [false, false, true, false, true, false, false, false],
    [false, false, true, false, true, false, false, false],
    [false, true, true, true, true, true, true, false],
    [false, false, true, false, true, false, false, false],
    [false, false, true, false, true, false, false, false],
    [false, true, true, true, true, true, true, false],
    [false, false, true, false, true, false, false, false],
    [false, false, true, false, true, false, false, false],
    [false; 8],
    [false; 8],
    [false; 8],
];

const PERCENT: [[bool; 8]; 12] = [
    [false; 8],
    [false, true, true, false, false, false, true, false],
    [false, true, true, false, false, true, false, false],
    [false, false, false, false, true, false, false, false],
    [false, false, false, true, false, false, false, false],
    [false, false, true, false, false, false, false, false],
    [false, true, false, false, false, false, false, false],
    [false, true, false, false, true, true, false, false],
    [false, false, false, false, true, true, false, false],
    [false; 8],
    [false; 8],
    [false; 8],
];

const AT: [[bool; 8]; 12] = [
    [false, false, true, true, true, true, false, false],
    [false, true, false, false, false, false, true, false],
    [false, true, false, true, true, false, true, false],
    [false, true, true, false, false, true, true, false],
    [false, true, true, false, false, true, true, false],
    [false, true, true, false, false, true, true, false],
    [false, true, false, true, true, true, false, false],
    [false, true, false, false, false, false, false, false],
    [false, false, true, true, true, true, false, false],
    [false; 8],
    [false; 8],
    [false; 8],
];

const CARET: [[bool; 8]; 12] = [
    [false; 8],
    [false, false, false, true, true, false, false, false],
    [false, false, true, false, false, true, false, false],
    [false, true, false, false, false, false, true, false],
    [false; 8],
    [false; 8],
    [false; 8],
    [false; 8],
    [false; 8],
    [false; 8],
    [false; 8],
    [false; 8],
];

const GREATER: [[bool; 8]; 12] = [
    [false; 8],
    [false; 8],
    [false, false, true, false, false, false, false, false],
    [false, false, false, true, false, false, false, false],
    [false, false, false, false, true, false, false, false],
    [false, false, false, false, false, true, false, false],
    [false, false, false, false, true, false, false, false],
    [false, false, false, true, false, false, false, false],
    [false, false, true, false, false, false, false, false],
    [false; 8],
    [false; 8],
    [false; 8],
];

const LESS: [[bool; 8]; 12] = [
    [false; 8],
    [false; 8],
    [false, false, false, false, false, true, false, false],
    [false, false, false, false, true, false, false, false],
    [false, false, false, true, false, false, false, false],
    [false, false, true, false, false, false, false, false],
    [false, false, false, true, false, false, false, false],
    [false, false, false, false, true, false, false, false],
    [false, false, false, false, false, true, false, false],
    [false; 8],
    [false; 8],
    [false; 8],
];

const TILDE: [[bool; 8]; 12] = [
    [false; 8],
    [false; 8],
    [false; 8],
    [false; 8],
    [false, false, true, true, false, false, true, false],
    [false, true, false, false, true, true, false, false],
    [false; 8],
    [false; 8],
    [false; 8],
    [false; 8],
    [false; 8],
    [false; 8],
];

const UNDERSCORE: [[bool; 8]; 12] = [
    [false; 8],
    [false; 8],
    [false; 8],
    [false; 8],
    [false; 8],
    [false; 8],
    [false; 8],
    [false; 8],
    [false; 8],
    [false; 8],
    [false; 8],
    [true, true, true, true, true, true, true, true],
];

const QUESTION: [[bool; 8]; 12] = [
    [false; 8],
    [false, false, true, true, true, true, false, false],
    [false, true, false, false, false, false, true, false],
    [false, false, false, false, false, true, false, false],
    [false, false, false, false, true, false, false, false],
    [false, false, false, true, false, false, false, false],
    [false, false, false, true, false, false, false, false],
    [false; 8],
    [false, false, false, true, false, false, false, false],
    [false; 8],
    [false; 8],
    [false; 8],
];

const BRACKET_RIGHT: [[bool; 8]; 12] = [
    [false, true, true, true, false, false, false, false],
    [false, false, false, true, false, false, false, false],
    [false, false, false, true, false, false, false, false],
    [false, false, false, true, false, false, false, false],
    [false, false, false, true, false, false, false, false],
    [false, false, false, true, false, false, false, false],
    [false, false, false, true, false, false, false, false],
    [false, false, false, true, false, false, false, false],
    [false, false, false, true, false, false, false, false],
    [false, true, true, true, false, false, false, false],
    [false; 8],
    [false; 8],
];

const BRACKET_LEFT: [[bool; 8]; 12] = [
    [false, false, false, true, true, true, false, false],
    [false, false, false, true, false, false, false, false],
    [false, false, false, true, false, false, false, false],
    [false, false, false, true, false, false, false, false],
    [false, false, false, true, false, false, false, false],
    [false, false, false, true, false, false, false, false],
    [false, false, false, true, false, false, false, false],
    [false, false, false, true, false, false, false, false],
    [false, false, false, true, false, false, false, false],
    [false, false, false, true, true, true, false, false],
    [false; 8],
    [false; 8],
];

const PAREN_RIGHT: [[bool; 8]; 12] = [
    [false, false, true, false, false, false, false, false],
    [false, false, false, true, false, false, false, false],
    [false, false, false, false, true, false, false, false],
    [false, false, false, false, true, false, false, false],
    [false, false, false, false, true, false, false, false],
    [false, false, false, false, true, false, false, false],
    [false, false, false, false, true, false, false, false],
    [false, false, false, false, true, false, false, false],
    [false, false, false, true, false, false, false, false],
    [false, false, true, false, false, false, false, false],
    [false; 8],
    [false; 8],
];

const PAREN_LEFT: [[bool; 8]; 12] = [
    [false, false, false, false, true, false, false, false],
    [false, false, false, true, false, false, false, false],
    [false, false, true, false, false, false, false, false],
    [false, false, true, false, false, false, false, false],
    [false, false, true, false, false, false, false, false],
    [false, false, true, false, false, false, false, false],
    [false, false, true, false, false, false, false, false],
    [false, false, true, false, false, false, false, false],
    [false, false, false, true, false, false, false, false],
    [false, false, false, false, true, false, false, false],
    [false; 8],
    [false; 8],
];

const PIPE_PATTERN: [[bool; 8]; 12] = [
    [false, false, false, true, true, false, false, false],
    [false, false, false, true, true, false, false, false],
    [false, false, false, true, true, false, false, false],
    [false, false, false, true, true, false, false, false],
    [false, false, false, true, true, false, false, false],
    [false, false, false, true, true, false, false, false],
    [false, false, false, true, true, false, false, false],
    [false, false, false, true, true, false, false, false],
    [false, false, false, true, true, false, false, false],
    [false, false, false, true, true, false, false, false],
    [false; 8],
    [false; 8],
];

const BACKSLASH: [[bool; 8]; 12] = [
    [false, true, false, false, false, false, false, false],
    [false, true, false, false, false, false, false, false],
    [false, false, true, false, false, false, false, false],
    [false, false, true, false, false, false, false, false],
    [false, false, false, true, false, false, false, false],
    [false, false, false, true, false, false, false, false],
    [false, false, false, false, true, false, false, false],
    [false, false, false, false, true, false, false, false],
    [false, false, false, false, false, true, false, false],
    [false, false, false, false, false, true, false, false],
    [false; 8],
    [false; 8],
];

const SLASH: [[bool; 8]; 12] = [
    [false, false, false, false, false, true, false, false],
    [false, false, false, false, false, true, false, false],
    [false, false, false, false, true, false, false, false],
    [false, false, false, false, true, false, false, false],
    [false, false, false, true, false, false, false, false],
    [false, false, false, true, false, false, false, false],
    [false, false, true, false, false, false, false, false],
    [false, false, true, false, false, false, false, false],
    [false, true, false, false, false, false, false, false],
    [false, true, false, false, false, false, false, false],
    [false; 8],
    [false; 8],
];

const ZERO: [[bool; 8]; 12] = [
    [false, false, true, true, true, true, false, false],
    [false, true, false, false, false, false, true, false],
    [false, true, false, false, false, true, true, false],
    [false, true, false, false, true, false, true, false],
    [false, true, false, true, false, false, true, false],
    [false, true, true, false, false, false, true, false],
    [false, true, false, false, false, false, true, false],
    [false, false, true, true, true, true, false, false],
    [false; 8],
    [false; 8],
    [false; 8],
    [false; 8],
];

const SMALL_BLOCK: [[bool; 8]; 12] = [
    [false; 8],
    [false; 8],
    [false; 8],
    [false; 8],
    [false, false, true, true, true, true, false, false],
    [false, false, true, true, true, true, false, false],
    [false, false, true, true, true, true, false, false],
    [false, false, true, true, true, true, false, false],
    [false; 8],
    [false; 8],
    [false; 8],
    [false; 8],
];

const MEDIUM_BLOCK: [[bool; 8]; 12] = [
    [false; 8],
    [false; 8],
    [false, true, true, true, true, true, true, false],
    [false, true, true, true, true, true, true, false],
    [false, true, true, true, true, true, true, false],
    [false, true, true, true, true, true, true, false],
    [false, true, true, true, true, true, true, false],
    [false, true, true, true, true, true, true, false],
    [false; 8],
    [false; 8],
    [false; 8],
    [false; 8],
];

const LARGE_BLOCK: [[bool; 8]; 12] = [
    [false; 8],
    [true, true, true, true, true, true, true, true],
    [true, true, true, true, true, true, true, true],
    [true, true, true, true, true, true, true, true],
    [true, true, true, true, true, true, true, true],
    [true, true, true, true, true, true, true, true],
    [true, true, true, true, true, true, true, true],
    [true, true, true, true, true, true, true, true],
    [true, true, true, true, true, true, true, true],
    [true, true, true, true, true, true, true, true],
    [false; 8],
    [false; 8],
];
