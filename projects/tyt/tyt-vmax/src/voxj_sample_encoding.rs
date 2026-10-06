use clap::ValueEnum;

/// Sample-block encoding for a `to-voxj` document.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum VoxjSampleEncoding {
    /// One raw JSON channel of material indices per layer. The most readable.
    #[value(name = "raw-json")]
    RawJson,

    /// One run-length-encoded JSON channel per layer.
    #[value(name = "rle-json")]
    RleJson,

    /// One bit-packed, base64-encoded channel per layer. The fastest to decode.
    #[value(name = "packed-base64")]
    PackedBase64,
}
