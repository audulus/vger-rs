# vger-rs

![build status](https://github.com/audulus/vger-rs/actions/workflows/rust.yml/badge.svg)
[![dependency status](https://deps.rs/repo/github/audulus/vger-rs/status.svg)](https://deps.rs/repo/github/audulus/vger-rs)

vger is a vector graphics renderer which renders a limited set of primitives, but does so almost entirely on the GPU. Port of [VGER](https://github.com/audulus/vger) to Rust. Used in [rui](https://github.com/audulus/rui).

## Status

- ✅ Quadratic bezier strokes 
- ✅ Round Rectangles
- ✅ Circles
- ✅ Line segments (need square ends for Audulus)
- ✅ Arcs
- ✅ Text (Audulus only uses one font, but could add support for more if anyone is interested)
- ✅ Multi-line text
- ✅ Path Fills.
- ✅ Scissoring
- ❌ Images

## Why?

I was previously using nanovg for Audulus, which was consuming too much CPU for the immediate-mode UI. nanovg is certainly more full featured, but for Audulus, vger maintains 120fps while nanovg falls to 30fps on my 120Hz iPad because of CPU-side path tessellation, and other overhead. vger renders analytically without tessellation, leaning heavily on the fragment shader.

## How it works

vger draws one or more quads for each primitive and computes the actual primitive shape in the fragment function with an [SDF](https://en.wikipedia.org/wiki/Signed_distance_function). For path fills, vger splits paths into horizontal slabs (see [path.rs](https://github.com/audulus/vger-rs/blob/main/src/path.rs)) to reduce the number of tests in the fragment function.

The bezier path fill case is somewhat original. To avoid having to solve quadratic equations (which has numerical issues), the fragment function uses a sort-of reverse Loop-Blinn. To determine if a point is inside or outside, vger tests against the lines formed between the endpoints of each bezier curve, flipping inside/outside for each intersection with a +x ray from the point. Then vger tests the point against the area between the bezier segment and the line, flipping inside/outside again if inside. This avoids the pre-computation of [Loop-Blinn](https://www.microsoft.com/en-us/research/wp-content/uploads/2005/01/p1000-loop.pdf), and the AA issues of [Kokojima](https://dl.acm.org/doi/10.1145/1179849.1179997).

## References

[Text Rendering Hates You](https://faultlore.com/blah/text-hates-you/)

[Adventures in Text Rendering](https://www.warp.dev/blog/adventures-text-rendering-kerning-glyph-atlases)

[Vector Graphics on GPU](https://gasiulis.name/vector-graphics-on-gpu/)

[GPU UIs at 120 FPS](https://zed.dev/blog/videogame)
