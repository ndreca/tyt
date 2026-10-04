# Implementation decisions

Code-level choices a reviewer of the Rust would want explained, recorded as
they land.

## S1. Transparent output

- The transparent branch solves in `f32` after the sRGB transfer. The color
  needs no clamp because every channel over white lies between the
  transparency and one. palette's `u8` conversion saturates rounding noise.
- The plan said colorless transmittance moves by a level or two. A colorless
  transmittance keeps today's alpha, but its color moves toward the exact
  composite by up to 19 levels in a halo's core. The README now states the
  measured result. The test checks the alpha instead of the move.
- Against exact composites of the `glow` golden, the halos' mean error over
  white falls from 2.7 levels to 0.1. Over black it falls from 28.7 to 26.3.
- Only the `glass` and `glow` goldens moved, all four variants of each. The
  others hold byte for byte.
