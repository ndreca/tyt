use crate::{
    Error, Result,
    dependencies::object::{EncodePng, PngChannels, PngImage},
    operations::object::Transfer,
};
use voxrender::RenderOutput;

/// Encodes `image` as an 8-bit RGBA PNG with the sRGB transfer stamped.
pub fn encode_render_png<D: EncodePng>(dependencies: &D, image: &RenderOutput) -> Result<Vec<u8>> {
    dependencies
        .encode_png(&PngImage {
            width: image.width(),
            height: image.height(),
            channels: PngChannels::Rgba,
            transfer: Transfer::Srgb,
            samples: image.to_bytes(),
        })
        .map_err(Error::Png)
}

#[cfg(test)]
mod tests {
    use crate::{
        Error,
        dependencies::{
            DependenciesImpl,
            object::{EncodePng, PngImage},
        },
        operations::object::encode_render_png,
    };
    use png::{ColorType, Decoder, Info, SrgbRenderingIntent};
    use std::{io::Cursor, result::Result as StdResult};
    use ty_math::TySrgbU8;
    use voxrender::{RenderImage, RenderOutput};

    /// Refuses every image.
    struct Refuse;

    impl EncodePng for Refuse {
        fn encode_png(&self, _: &PngImage) -> StdResult<Vec<u8>, String> {
            Err("no".to_owned())
        }
    }

    /// The samples and header of the PNG `bytes`.
    fn decode(bytes: &[u8]) -> (Vec<u8>, Info<'static>) {
        let mut reader = Decoder::new(Cursor::new(bytes)).read_info().unwrap();
        let mut buffer = vec![0u8; reader.output_buffer_size().unwrap()];
        let output = reader.next_frame(&mut buffer).unwrap();
        buffer.truncate(output.buffer_size());
        (buffer, reader.info().clone())
    }

    #[test]
    fn the_png_holds_the_pixels_with_the_srgb_transfer_stamped() {
        let image = RenderOutput::from_image(&RenderImage::new(2, 1), Some(TySrgbU8::new(1, 2, 3)));

        let png = encode_render_png(&DependenciesImpl, &image).unwrap();

        let (samples, info) = decode(&png);
        assert_eq!((info.width, info.height), (2, 1));
        assert_eq!(info.color_type, ColorType::Rgba);
        assert_eq!(info.srgb, Some(SrgbRenderingIntent::Perceptual));
        assert_eq!(samples, [1, 2, 3, 255, 1, 2, 3, 255]);
    }

    #[test]
    fn a_refused_png_errors() {
        let image = RenderOutput::from_image(&RenderImage::new(1, 1), None);

        assert!(matches!(
            encode_render_png(&Refuse, &image),
            Err(Error::Png(_))
        ));
    }
}
