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
    /// Encodes `image` through [`tonemap`] and the sRGB transfer. A pixel's
    /// alpha is the larger of one minus its peak transmittance and the peak
    /// of its light, clamped to one. Its color is the light over that alpha.
    /// Under `background` the pixel is that color at that alpha plus the
    /// background scaled by the transmittance, clamped to one and written at
    /// full alpha.
    pub fn from_image(image: &RenderImage, background: Option<TySrgbU8>) -> Self {
        let backdrop = background.map(|color| color.into_format::<f32>().into_linear());
        let peak = |color: TyLinSrgbF32| color.red.max(color.green).max(color.blue);

        let pixels = image
            .pixels()
            .iter()
            .map(|pixel| {
                let alpha = (1.0 - peak(pixel.transmittance))
                    .max(peak(pixel.light))
                    .min(1.0);

                // The alpha is at least the light's peak, so a zero alpha
                // carries no light.
                let color = if alpha > 0.0 {
                    pixel.light / alpha
                } else {
                    pixel.light
                };

                let mapped = tonemap(color);

                let (color, alpha) = match backdrop {
                    Some(backdrop) => {
                        let over = mapped * alpha + backdrop * pixel.transmittance;

                        (
                            TyLinSrgbF32::new(
                                over.red.min(1.0),
                                over.green.min(1.0),
                                over.blue.min(1.0),
                            ),
                            1.0,
                        )
                    }

                    None => (mapped, alpha),
                };

                TySrgbaF32::from_linear(TyLinSrgbaF32::new(
                    color.red,
                    color.green,
                    color.blue,
                    alpha,
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
    use crate::{RenderImage, RenderOutput, RenderPixel, tonemap};
    use ty_math::{TyLinSrgbF32, TyLinSrgbaF32, TySrgbU8, TySrgbaF32, TySrgbaU8};

    const BLACK: TyLinSrgbF32 = TyLinSrgbF32::new(0.0, 0.0, 0.0);

    fn encoded(color: TyLinSrgbF32, alpha: f32) -> TySrgbaU8 {
        TySrgbaF32::from_linear(TyLinSrgbaF32::new(
            color.red,
            color.green,
            color.blue,
            alpha,
        ))
        .into_format::<u8, u8>()
    }

    #[test]
    fn hits_encode_through_the_tonemap_and_misses_take_the_background() {
        let mut image = RenderImage::new(2, 1);
        image.set_pixel(
            0,
            0,
            RenderPixel {
                light: TyLinSrgbF32::new(0.5, 0.25, 0.0),
                transmittance: BLACK,
            },
        );

        let expected = encoded(tonemap(TyLinSrgbF32::new(0.5, 0.25, 0.0)), 1.0);

        let transparent = RenderOutput::from_image(&image, None);
        assert_eq!(transparent.width(), 2);
        assert_eq!(transparent.height(), 1);
        assert_eq!(transparent.pixels(), [expected, TySrgbaU8::new(0, 0, 0, 0)]);
        assert_eq!(transparent.to_bytes().len(), 8);
        assert_eq!(&transparent.to_bytes()[4..], [0, 0, 0, 0]);

        let backed = RenderOutput::from_image(&image, Some(TySrgbU8::new(10, 20, 30)));
        assert_eq!(backed.pixels()[1], TySrgbaU8::new(10, 20, 30, 255));
    }

    #[test]
    fn a_halo_pixel_keeps_its_alpha_or_composites_over_the_background() {
        // A halo of (0.5, 0.25, 0) over a miss covers it by its peak.
        let mut image = RenderImage::new(1, 1);
        image.set_pixel(
            0,
            0,
            RenderPixel {
                light: TyLinSrgbF32::new(0.5, 0.25, 0.0),
                transmittance: TyLinSrgbF32::new(0.5, 0.5, 0.5),
            },
        );

        let mapped = tonemap(TyLinSrgbF32::new(1.0, 0.5, 0.0));

        let transparent = RenderOutput::from_image(&image, None);
        assert_eq!(transparent.pixels(), [encoded(mapped, 0.5)]);

        // Over black the color halves to the halo's light.
        let over_black = RenderOutput::from_image(&image, Some(TySrgbU8::new(0, 0, 0)));
        assert_eq!(over_black.pixels(), [encoded(mapped / 2.0, 1.0)]);

        // Over white the halo lifts the color toward white.
        let over_white = RenderOutput::from_image(&image, Some(TySrgbU8::new(255, 255, 255)));
        let [red, green, blue, alpha] = <[u8; 4]>::from(over_white.pixels()[0]);
        assert_eq!(alpha, 255);
        assert!(
            red > green && green > blue && blue > 128,
            "{red} {green} {blue}"
        );
    }

    #[test]
    fn glass_takes_the_larger_of_its_coverage_and_its_light_as_alpha() {
        let pane = TyLinSrgbF32::new(0.75, 0.75, 0.75);

        let mut image = RenderImage::new(2, 1);
        // A dim pane takes its coverage as the alpha and spreads its light
        // over it.
        image.set_pixel(
            0,
            0,
            RenderPixel {
                light: TyLinSrgbF32::new(0.125, 0.125, 0.125),
                transmittance: pane,
            },
        );
        // A highlight brighter than the coverage takes the light's peak as
        // the alpha.
        image.set_pixel(
            1,
            0,
            RenderPixel {
                light: TyLinSrgbF32::new(0.5, 0.25, 0.25),
                transmittance: pane,
            },
        );

        let output = RenderOutput::from_image(&image, None);
        assert_eq!(
            output.pixels(),
            [
                encoded(tonemap(TyLinSrgbF32::new(0.5, 0.5, 0.5)), 0.25),
                encoded(tonemap(TyLinSrgbF32::new(1.0, 0.5, 0.5)), 0.5),
            ]
        );
    }

    #[test]
    fn red_glass_over_white_is_red_and_clear_glass_over_white_stays_white() {
        let reflection = TyLinSrgbF32::new(0.04, 0.04, 0.04);
        let clear = TyLinSrgbF32::new(0.96, 0.96, 0.96);

        let mut image = RenderImage::new(3, 1);
        image.set_pixel(
            0,
            0,
            RenderPixel {
                light: reflection,
                transmittance: TyLinSrgbF32::new(0.96, 0.0, 0.0),
            },
        );
        image.set_pixel(
            1,
            0,
            RenderPixel {
                light: reflection,
                transmittance: clear,
            },
        );
        // A highlight on clear glass over white clamps to white.
        image.set_pixel(
            2,
            0,
            RenderPixel {
                light: TyLinSrgbF32::new(0.5, 0.5, 0.5),
                transmittance: clear,
            },
        );

        let output = RenderOutput::from_image(&image, Some(TySrgbU8::new(255, 255, 255)));

        let [red, green, blue, alpha] = <[u8; 4]>::from(output.pixels()[0]);
        assert_eq!(alpha, 255);
        assert!(
            red > 250 && green < 64 && green == blue,
            "{red} {green} {blue}"
        );

        let [red, green, blue, alpha] = <[u8; 4]>::from(output.pixels()[1]);
        assert_eq!(alpha, 255);
        assert!(
            red > 250 && red == green && green == blue,
            "{red} {green} {blue}"
        );

        assert_eq!(output.pixels()[2], TySrgbaU8::new(255, 255, 255, 255));
    }
}
