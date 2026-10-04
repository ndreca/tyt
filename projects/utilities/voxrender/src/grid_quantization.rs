/// The fraction bits of a fixed-point grid coordinate.
pub const GRID_FRACTION_BITS: u32 = 13;

/// No component of a quantized direction exceeds `1 << GRID_DIRECTION_BITS`.
pub const GRID_DIRECTION_BITS: u32 = 16;

/// The extra fraction bits of a view's per-pixel steps. Their rounding stays
/// under one unit across 8192 pixels.
pub const GRID_GUARD_BITS: u32 = 12;

/// A quantized point lies within `1 << GRID_RANGE_BITS` cells of its grid's
/// origin. The range keeps grid entry in 64-bit integers.
pub const GRID_RANGE_BITS: u32 = 32;

/// A shadow ray starts `2^-SHADOW_INSET_BITS` of a cell in from its face's
/// edges.
pub const SHADOW_INSET_BITS: u32 = 10;
