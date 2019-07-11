use crate::parse::image::{Image};

pub struct BooleanMatrix {
    data: Vec<bool>,
    width: usize,
    height: usize,
}

impl BooleanMatrix {
    pub fn base_height(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn all_false(width: usize, height: usize) -> BooleanMatrix {
        let data = vec![false; width*height];

        BooleanMatrix { data, width, height }
    }

    pub fn set(&mut self, x: usize, y: usize) {
        let index = self.get_index(x, y);
        self.data[index] = true;
    }

    pub fn is_set(&self, x: usize, y: usize) -> bool {
        let index = self.get_index(x, y);
        self.data[index]
    }

    /// Produces a boolean matrix with true corresponding to a dark color, and false meaning light.
    pub fn from_image(image: &Image, dark_threshold: u8) -> BooleanMatrix {
        let mut matrix = BooleanMatrix::all_false(image.base, image.height);

        for y in 0..image.height {
            for x in 0..image.base {
                let color = image.get_color(x, y);

                if color.is_darker_than(dark_threshold) {
                    matrix.set(x, y);
                }
            }
        }

        matrix
    }

    /// Produces an image with true corresponding to black, and false meaning white
    pub fn as_image(&self) -> Image {
        let mut image_data = Vec::with_capacity(self.data.len()*3);

        for &b in self.data.iter() {
            let color = if b { 0 } else { 255 };

            image_data.push(color);
            image_data.push(color);
            image_data.push(color);
        }

        Image::from_raw(image_data, self.width, self.height)
    }

    fn get_index(&self, x: usize, y: usize) -> usize {
        y*self.width + x
    }
}