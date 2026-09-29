use ty_math::TyLinSrgbaF32;

/// A linear-light RGBA image, rows top to bottom and pixels left to right
/// within a row.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderImage {
    width: u32,

    height: u32,

    pixels: Vec<TyLinSrgbaF32>,
}

impl RenderImage {
    /// A transparent black image of `width` by `height` pixels.
    pub fn new(width: u32, height: u32) -> Self {
        let count = width as usize * height as usize;

        RenderImage {
            width,
            height,
            pixels: vec![TyLinSrgbaF32::new(0.0, 0.0, 0.0, 0.0); count],
        }
    }

    /// The width in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// The height in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// The pixel at column `x` of row `y`, or `None` outside the image.
    pub fn pixel(&self, x: u32, y: u32) -> Option<TyLinSrgbaF32> {
        self.index(x, y).map(|index| self.pixels[index])
    }

    /// Writes `pixel` at column `x` of row `y`. `x` and `y` are within the
    /// image.
    pub fn set_pixel(&mut self, x: u32, y: u32, pixel: TyLinSrgbaF32) {
        let index = self.index(x, y).expect("the pixel is within the image");

        self.pixels[index] = pixel;
    }

    /// Every pixel in row-major order.
    pub fn pixels(&self) -> &[TyLinSrgbaF32] {
        &self.pixels
    }

    fn index(&self, x: u32, y: u32) -> Option<usize> {
        (x < self.width && y < self.height).then(|| y as usize * self.width as usize + x as usize)
    }
}

#[cfg(test)]
mod tests {
    use crate::RenderImage;
    use ty_math::TyLinSrgbaF32;

    #[test]
    fn pixels_address_by_column_and_row_within_the_image() {
        let mut image = RenderImage::new(3, 2);

        assert_eq!(image.pixels().len(), 6);
        assert_eq!(
            image.pixel(2, 1),
            Some(TyLinSrgbaF32::new(0.0, 0.0, 0.0, 0.0))
        );
        assert_eq!(image.pixel(3, 1), None);
        assert_eq!(image.pixel(2, 2), None);

        let red = TyLinSrgbaF32::new(1.0, 0.0, 0.0, 1.0);

        image.set_pixel(2, 1, red);

        assert_eq!(image.pixel(2, 1), Some(red));
        assert_eq!(image.pixels()[5], red);
    }
}
