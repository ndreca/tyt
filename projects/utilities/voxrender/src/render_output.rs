use crate::{RenderImage, tonemap};
use ty_math::{TyLinSrgbF32, TyLinSrgbaF32, TySrgbU8, TySrgbaF32, TySrgbaU8};

/// An 8-bit sRGB image with straight alpha that a PNG can store. Rows run top
/// to bottom, and pixels run left to right within a row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderOutput {
    width: u32,

    height: u32,

    pixels: Vec<TySrgbaU8>,
}

impl RenderOutput {
    /// Encodes `image` through [`tonemap`] and the sRGB transfer. A pixel no
    /// ray hit becomes `background` at full alpha, or transparent black when
    /// `background` is `None`.
    pub fn from_image(image: &RenderImage, background: Option<TySrgbU8>) -> Self {
        let pixels = image
            .pixels()
            .iter()
            .map(|pixel| {
                if pixel.alpha == 0.0 {
                    return background.map_or(TySrgbaU8::new(0, 0, 0, 0), |color| {
                        TySrgbaU8::new(color.red, color.green, color.blue, 255)
                    });
                }

                let mapped = tonemap(TyLinSrgbF32::new(pixel.red, pixel.green, pixel.blue));

                TySrgbaF32::from_linear(TyLinSrgbaF32::new(
                    mapped.red,
                    mapped.green,
                    mapped.blue,
                    pixel.alpha,
                ))
                .into_format::<u8, u8>()
            })
            .collect();

        RenderOutput {
            width: image.width(),
            height: image.height(),
            pixels,
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

    /// Every pixel in row-major order.
    pub fn pixels(&self) -> &[TySrgbaU8] {
        &self.pixels
    }

    /// Every pixel's four bytes in row-major order, red first.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.pixels
            .iter()
            .flat_map(|pixel| <[u8; 4]>::from(*pixel))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::{RenderImage, RenderOutput, tonemap};
    use ty_math::{TyLinSrgbF32, TyLinSrgbaF32, TySrgbU8, TySrgbaF32, TySrgbaU8};

    #[test]
    fn hits_encode_through_the_tonemap_and_misses_take_the_background() {
        let mut image = RenderImage::new(2, 1);
        image.set_pixel(0, 0, TyLinSrgbaF32::new(0.5, 0.25, 0.0, 1.0));

        let mapped = tonemap(TyLinSrgbF32::new(0.5, 0.25, 0.0));
        let expected = TySrgbaF32::from_linear(TyLinSrgbaF32::new(
            mapped.red,
            mapped.green,
            mapped.blue,
            1.0,
        ))
        .into_format::<u8, u8>();

        let transparent = RenderOutput::from_image(&image, None);
        assert_eq!(transparent.width(), 2);
        assert_eq!(transparent.height(), 1);
        assert_eq!(transparent.pixels(), [expected, TySrgbaU8::new(0, 0, 0, 0)]);
        assert_eq!(transparent.to_bytes().len(), 8);
        assert_eq!(&transparent.to_bytes()[4..], [0, 0, 0, 0]);

        let backed = RenderOutput::from_image(&image, Some(TySrgbU8::new(10, 20, 30)));
        assert_eq!(backed.pixels()[1], TySrgbaU8::new(10, 20, 30, 255));
    }
}
