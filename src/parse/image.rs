use png::{Decoder, Encoder, ColorType, BitDepth};
use png::HasParameters;

use std::fs::File;
use std::io::BufWriter;

#[derive(Clone, Copy)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Color {
        Color { r, g, b }
    }

    pub const fn red() -> Color {
        Color::from_rgb(0xff, 0, 0)
    }

    pub const fn green() -> Color {
        Color::from_rgb(0, 0xff, 0)
    }

    pub const fn blue() -> Color {
        Color::from_rgb(0, 0, 0xff)
    }

    pub const fn magenta() -> Color {
        Color::from_rgb(0xff, 0, 0xff)
    }

    pub const fn yellow() -> Color {
        Color::from_rgb(0xff, 0xff, 0)
    }

    pub fn is_darker_than(self, threshold: u8) -> bool {
        self.r < threshold && self.g < threshold && self.b < threshold
    }
}

#[derive(Clone)]
pub struct Image {
    data: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

impl Image {
    pub fn from_raw(data: Vec<u8>, width: usize, height: usize) -> Image {
        assert_eq!(data.len(), 3*width*height);
        Image { data, width, height }
    }

    pub fn from_fn(width: usize, height: usize, f: impl Fn(usize, usize) -> Color) -> Image {
        let mut data = Vec::with_capacity(width*height*3);

        for y in 0..height {
            for x in 0..width {
                let color = f(x, y);
                data.push(color.r);
                data.push(color.g);
                data.push(color.b);
            }
        }

        Image { data, width, height }
    }

    pub fn output_to_file(&self, name: &str) {
        let output_file = BufWriter::new(File::create(name).unwrap());

        let mut encoder = Encoder::new(output_file, self.width as u32, self.height as u32);
        encoder.set(png::BitDepth::Eight).set(png::ColorType::RGB);

        let mut writer = encoder.write_header().unwrap();

        writer.write_image_data(&self.data).unwrap();
    }

    pub fn read_from_file(name: &str) -> Image {
        let file = File::open(name).unwrap();

        let (info, mut reader) = Decoder::new(file).read_info().unwrap();
        let width = info.width as usize;
        let height = info.height as usize;

        assert_eq!(info.color_type, ColorType::RGB);
        assert_eq!(info.bit_depth, BitDepth::Eight);

        let mut data = vec![0; info.buffer_size()];
        reader.next_frame(&mut data).unwrap();

        Image { data, width, height }
    }

    pub fn set_color(&mut self, x: usize, y: usize, color: Color) {
        let index = 3*(y*self.width + x);
        self.data[index] = color.r;
        self.data[index+1] = color.g;
        self.data[index+2] = color.b;
    }

    pub fn get_color(&self, x: usize, y: usize) -> Color {
        let index = 3*(y*self.width + x);

        Color::from_rgb(self.data[index], self.data[index+1], self.data[index+2])
    }

    pub fn get_color_checked(&self, x: usize, y: usize) -> Option<Color> {
        if x < self.width && y < self.height {
            Some(self.get_color(x, y))
        } else {
            None
        }
    }
}
