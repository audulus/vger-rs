//! A cache of laid-out text, so text that is drawn or measured frame after frame is laid out
//! once instead of every time.

use fontdue::layout::{GlyphPosition, LinePosition};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// The layout of a string at a size, maximum width and device pixel ratio, in device pixels.
pub(crate) struct TextLayout {
    text: String,
    size: u32,
    max_width: Option<u32>,
    scale: u32,
    pub glyphs: Vec<GlyphPosition>,
    pub lines: Vec<LinePosition>,
}

/// Layouts used recently. A layout survives as long as it is used at least once per frame:
/// `next_frame` drops the layouts that weren't used since the previous call.
#[derive(Default)]
pub(crate) struct TextLayoutCache {
    current: HashMap<u64, Arc<TextLayout>>,
    previous: HashMap<u64, Arc<TextLayout>>,
}

fn key(text: &str, size: u32, max_width: Option<u32>, scale: u32) -> u64 {
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    size.hash(&mut hasher);
    max_width.hash(&mut hasher);
    scale.hash(&mut hasher);
    hasher.finish()
}

impl TextLayout {
    fn matches(&self, text: &str, size: u32, max_width: Option<u32>, scale: u32) -> bool {
        self.size == size && self.max_width == max_width && self.scale == scale && self.text == text
    }
}

impl TextLayoutCache {
    /// Returns the layout of `text`, calling `layout` to compute its glyphs and lines if it
    /// isn't cached.
    pub fn get(
        &mut self,
        text: &str,
        size: u32,
        max_width: Option<f32>,
        scale: f32,
        layout: impl FnOnce() -> (Vec<GlyphPosition>, Vec<LinePosition>),
    ) -> Arc<TextLayout> {
        let max_width = max_width.map(f32::to_bits);
        let scale = scale.to_bits();
        let k = key(text, size, max_width, scale);

        if let Some(found) = self.current.get(&k) {
            if found.matches(text, size, max_width, scale) {
                return found.clone();
            }
        }
        if let Some(found) = self.previous.remove(&k) {
            if found.matches(text, size, max_width, scale) {
                self.current.insert(k, found.clone());
                return found;
            }
        }

        // Not cached, or another string with the same key, which this replaces.
        let (glyphs, lines) = layout();
        let laid_out = Arc::new(TextLayout {
            text: text.to_owned(),
            size,
            max_width,
            scale,
            glyphs,
            lines,
        });
        self.current.insert(k, laid_out.clone());
        laid_out
    }

    /// Starts a frame: drops the layouts not used during the last one.
    pub fn next_frame(&mut self) {
        self.previous = std::mem::take(&mut self.current);
    }

    /// The number of cached layouts.
    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.current.len() + self.previous.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layout_of(
        calls: &mut usize,
    ) -> impl FnOnce() -> (Vec<GlyphPosition>, Vec<LinePosition>) + '_ {
        move || {
            *calls += 1;
            (vec![], vec![])
        }
    }

    #[test]
    fn test_reuses_a_layout() {
        let mut cache = TextLayoutCache::default();
        let mut calls = 0;
        cache.get("hello", 12, None, 2.0, layout_of(&mut calls));
        cache.get("hello", 12, None, 2.0, layout_of(&mut calls));
        assert_eq!(calls, 1);
    }

    #[test]
    fn test_size_width_and_scale_are_part_of_the_key() {
        let mut cache = TextLayoutCache::default();
        let mut calls = 0;
        cache.get("hello", 12, None, 2.0, layout_of(&mut calls));
        cache.get("hello", 13, None, 2.0, layout_of(&mut calls));
        cache.get("hello", 12, Some(40.0), 2.0, layout_of(&mut calls));
        cache.get("hello", 12, None, 1.0, layout_of(&mut calls));
        cache.get("hellp", 12, None, 2.0, layout_of(&mut calls));
        assert_eq!(calls, 5);
    }

    #[test]
    fn test_keeps_layouts_used_each_frame_and_drops_the_rest() {
        let mut cache = TextLayoutCache::default();
        let mut calls = 0;
        cache.get("kept", 12, None, 1.0, layout_of(&mut calls));
        cache.get("dropped", 12, None, 1.0, layout_of(&mut calls));

        cache.next_frame();
        cache.get("kept", 12, None, 1.0, layout_of(&mut calls));
        assert_eq!(calls, 2);

        cache.next_frame();
        assert_eq!(cache.len(), 1);
        cache.get("kept", 12, None, 1.0, layout_of(&mut calls));
        cache.get("dropped", 12, None, 1.0, layout_of(&mut calls));
        assert_eq!(calls, 3);
    }
}
