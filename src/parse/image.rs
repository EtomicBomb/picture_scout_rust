use png::{Decoder, Encoder, ColorType, BitDepth};
use png::HasParameters;

use std::fs::File;
use std::io::BufWriter;
use rulinalg::matrix::Matrix;
use rulinalg::vector::Vector;

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

    pub const fn cyan() -> Color {
        Color::from_rgb(0, 0xff, 0xff)
    }

    pub fn is_darker_than(self, threshold: u8) -> bool {
        self.r < threshold && self.g < threshold && self.b < threshold
    }
}

#[derive(Clone)]
pub struct Image {
    data: Vec<u8>,
    pub base: usize,
    pub height: usize,
}

impl Image {
    pub fn from_raw(data: Vec<u8>, base: usize, height: usize) -> Image {
        assert_eq!(data.len(), 3*base*height);
        Image { data, base, height }
    }

    pub fn from_fn(base: usize, height: usize, f: impl Fn(usize, usize) -> Color) -> Image {
        let mut data = Vec::with_capacity(base*height*3);

        for y in 0..height {
            for x in 0..base {
                let color = f(x, y);
                data.push(color.r);
                data.push(color.g);
                data.push(color.b);
            }
        }

        Image { data, base, height }
    }

    pub fn output_to_file(&self, name: &str) {
        let output_file = BufWriter::new(File::create(name).unwrap());

        let mut encoder = Encoder::new(output_file, self.base as u32, self.height as u32);
        encoder.set(png::BitDepth::Eight).set(png::ColorType::RGB);

        let mut writer = encoder.write_header().unwrap();

        writer.write_image_data(&self.data).unwrap();
    }

    pub fn read_from_file(name: &str) -> Image {
        let file = File::open(name).unwrap();

        let (info, mut reader) = Decoder::new(file).read_info().unwrap();
        let base = info.width as usize;
        let height = info.height as usize;

        assert_eq!(info.color_type, ColorType::RGB);
        assert_eq!(info.bit_depth, BitDepth::Eight);

        let mut data = vec![0; info.buffer_size()];
        reader.next_frame(&mut data).unwrap();

        Image { data, base, height }
    }

    pub fn set_color(&mut self, x: usize, y: usize, color: Color) {
        let index = 3*(y*self.base + x);
        self.data[index] = color.r;
        self.data[index+1] = color.g;
        self.data[index+2] = color.b;
    }

    pub fn get_color(&self, x: usize, y: usize) -> Color {
        let index = 3*(y*self.base + x);

        Color::from_rgb(self.data[index], self.data[index+1], self.data[index+2])
    }

    pub fn get_color_checked(&self, x: usize, y: usize) -> Option<Color> {
        if x < self.base && y < self.height {
            Some(self.get_color(x, y))
        } else {
            None
        }
    }

    pub fn perspective_transform(&self, src: &[(f64, f64)], dst: &[(f64, f64)], new_base: usize, new_height: usize) -> Image {
        // https://www.pyimagesearch.com/2014/08/25/4-point-opencv-getperspective-transform-example/
        // https://docs.opencv.org/2.4/modules/imgproc/doc/geometric_transformations.html?highlight=getperspectivetransform#void%20warpPerspective(InputArray%20src,%20OutputArray%20dst,%20InputArray%20M,%20Size%20dsize,%20int%20flags,%20int%20borderMode,%20const%20Scalar&%20borderValue)
        // https://github.com/opencv/opencv/blob/11b020b9f9e111bddd40bffe3b1759aa02d966f0/modules/imgproc/src/imgwarp.cpp


        let m = get_perspective_shift_matrix(src, dst);

        // actually produce a new image with our transformation
        Image::from_fn(new_base, new_height, |x: usize, y: usize| {
            let x = x as f64;
            let y = y as f64;

            let new_x = (m[[0,0]]*x + m[[0,1]]*y + m[[0,2]]) / (m[[2,0]]*x + m[[2,1]]*y + m[[2,2]]);
            let new_y = (m[[1,0]]*x + m[[1,1]]*y + m[[1,2]]) / (m[[2,0]]*x + m[[2,1]]*y + m[[2,2]]);

            self.get_color_checked(new_x as usize, new_y as usize)
                .unwrap_or(Color::magenta()) // our debug value
        })
    }
}



fn get_perspective_shift_matrix(src: &[(f64, f64)], dst: &[(f64, f64)]) -> Matrix<f64> {
    assert_eq!(src.len(), 4);
    assert_eq!(dst.len(), 4);

    let mut a = Matrix::zeros(8, 8);
    let mut b = Vector::zeros(8);
    for i in 0..4 {
        let t = src[i].0;
        a[[i+4,3]] = t;
        a[[i,0]] = t;

        let t = src[i].1;
        a[[i+4,4]] = t;
        a[[i,1]] = t;

        a[[i+4,5]] = 1.0;
        a[[i,2]] = 1.0;

        a[[i,6]] = -src[i].0*dst[i].0;
        a[[i,7]] = -src[i].1*dst[i].0;
        a[[i+4,6]] = -src[i].0*dst[i].1;
        a[[i+4,7]] = -src[i].1*dst[i].1;

        b[i] = dst[i].0;
        b[i+4] = dst[i].1;
    }

    // calculate the transformation matrix m such that a*m = b
    let cs = a.solve(b).unwrap();
    let mut data = cs.into_vec();
    data.push(1.0);
    Matrix::<f64>::new(3, 3, data).inverse().unwrap()
}
