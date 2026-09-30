use serde::{Deserialize, Serialize};

/// 2D Bounding Rectangle representing an on-screen UI element.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct LayoutRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl LayoutRect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    #[inline]
    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.x + self.w && py >= self.y && py <= self.y + self.h
    }

    #[inline]
    pub fn center(&self) -> (f32, f32) {
        (self.x + self.w * 0.5, self.y + self.h * 0.5)
    }

    #[inline]
    pub fn pad(&self, pad_x: f32, pad_y: f32) -> Self {
        Self {
            x: self.x + pad_x,
            y: self.y + pad_y,
            w: (self.w - pad_x * 2.0).max(0.0),
            h: (self.h - pad_y * 2.0).max(0.0),
        }
    }

    #[inline]
    pub fn as_tuple(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.w, self.h)
    }
}

/// Boundary exit event emitted when navigation attempts to move beyond a stack's bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavBoundaryExit {
    ExitTop,
    ExitBottom,
    ExitLeft,
    ExitRight,
}

/// Horizontal Stack Layout manager for distributing items horizontally (e.g. tabs, buttons, stat chips).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HStack {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub gap: f32,
    pub count: usize,
}

impl HStack {
    /// Creates a new horizontal stack distributing items evenly across total width `w`.
    pub fn new_equal(x: f32, y: f32, w: f32, h: f32, count: usize, gap: f32) -> Self {
        Self {
            x,
            y,
            w,
            h,
            gap,
            count: count.max(1),
        }
    }

    /// Computes the item width for equal distribution.
    #[inline]
    pub fn item_width(&self) -> f32 {
        if self.count <= 1 {
            self.w
        } else {
            let total_gaps = self.gap * (self.count as f32 - 1.0);
            ((self.w - total_gaps) / self.count as f32).max(1.0)
        }
    }

    /// Returns the bounding rectangle of the child element at index `idx`.
    pub fn item_rect(&self, idx: usize) -> LayoutRect {
        let item_w = self.item_width();
        let ix = self.x + (idx as f32) * (item_w + self.gap);
        LayoutRect::new(ix, self.y, item_w, self.h)
    }

    /// Returns bounding rectangles for all items in the stack.
    pub fn all_rects(&self) -> Vec<LayoutRect> {
        (0..self.count).map(|i| self.item_rect(i)).collect()
    }

    /// Hit-tests a point against all items, returning the matching index if any.
    pub fn hit_test(&self, px: f32, py: f32) -> Option<usize> {
        if py < self.y || py > self.y + self.h {
            return None;
        }
        for i in 0..self.count {
            if self.item_rect(i).contains(px, py) {
                return Some(i);
            }
        }
        None
    }

    /// Navigates left with optional wrap-around. Returns `(new_idx, Option<NavBoundaryExit>)`.
    pub fn nav_left(&self, current: usize, wrap: bool) -> (usize, Option<NavBoundaryExit>) {
        if self.count <= 1 {
            return (0, Some(NavBoundaryExit::ExitLeft));
        }
        if current == 0 {
            if wrap {
                (self.count - 1, None)
            } else {
                (0, Some(NavBoundaryExit::ExitLeft))
            }
        } else {
            (current - 1, None)
        }
    }

    /// Navigates right with optional wrap-around. Returns `(new_idx, Option<NavBoundaryExit>)`.
    pub fn nav_right(&self, current: usize, wrap: bool) -> (usize, Option<NavBoundaryExit>) {
        if self.count <= 1 {
            return (0, Some(NavBoundaryExit::ExitRight));
        }
        if current + 1 >= self.count {
            if wrap {
                (0, None)
            } else {
                (current, Some(NavBoundaryExit::ExitRight))
            }
        } else {
            (current + 1, None)
        }
    }
}

/// Vertical Stack Layout manager for distributing items vertically (e.g. track list, driver rosters, option rows).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VStack {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub item_h: f32,
    pub gap: f32,
}

impl VStack {
    /// Creates a new vertical stack with uniform item heights.
    pub fn new_uniform(x: f32, y: f32, w: f32, item_h: f32, gap: f32) -> Self {
        Self {
            x,
            y,
            w,
            item_h,
            gap,
        }
    }

    /// Returns the bounding rectangle of the child element at index `idx`.
    #[inline]
    pub fn item_rect(&self, idx: usize) -> LayoutRect {
        let iy = self.y + (idx as f32) * (self.item_h + self.gap);
        LayoutRect::new(self.x, iy, self.w, self.item_h)
    }

    /// Returns the bounding rectangle for windowed scrolling (relative to `start_idx`).
    #[inline]
    pub fn item_rect_windowed(&self, idx: usize, start_idx: usize) -> LayoutRect {
        let relative_idx = idx.saturating_sub(start_idx);
        let iy = self.y + (relative_idx as f32) * (self.item_h + self.gap);
        LayoutRect::new(self.x, iy, self.w, self.item_h)
    }

    /// Computes the total content height for `count` items.
    #[inline]
    pub fn total_height(&self, count: usize) -> f32 {
        if count == 0 {
            0.0
        } else {
            (count as f32) * self.item_h + ((count - 1) as f32) * self.gap
        }
    }

    /// Hit-tests a point against all items up to `count`.
    pub fn hit_test(&self, px: f32, py: f32, count: usize) -> Option<usize> {
        if px < self.x || px > self.x + self.w {
            return None;
        }
        for i in 0..count {
            if self.item_rect(i).contains(px, py) {
                return Some(i);
            }
        }
        None
    }

    /// Hit-tests a point in a windowed / scrollable list.
    pub fn hit_test_windowed(
        &self,
        px: f32,
        py: f32,
        start_idx: usize,
        visible_count: usize,
        total_count: usize,
    ) -> Option<usize> {
        if px < self.x || px > self.x + self.w {
            return None;
        }
        let end_idx = (start_idx + visible_count).min(total_count);
        for i in start_idx..end_idx {
            if self.item_rect_windowed(i, start_idx).contains(px, py) {
                return Some(i);
            }
        }
        None
    }

    /// Computes the visible window `(start_idx, end_idx)` centered around `selected_idx`.
    pub fn visible_range(
        total_count: usize,
        visible_count: usize,
        selected_idx: usize,
    ) -> (usize, usize) {
        if total_count <= visible_count {
            (0, total_count)
        } else {
            let half = visible_count / 2;
            let start = selected_idx.saturating_sub(half).min(total_count - visible_count);
            let end = (start + visible_count).min(total_count);
            (start, end)
        }
    }

    /// Navigates up. When at index 0 and `wrap` is false, returns `(0, Some(NavBoundaryExit::ExitTop))`,
    /// enabling seamless focus transition to tabs/filters above.
    pub fn nav_up(&self, current: usize, total: usize, wrap: bool) -> (usize, Option<NavBoundaryExit>) {
        if total <= 1 {
            return (0, Some(NavBoundaryExit::ExitTop));
        }
        if current == 0 {
            if wrap {
                (total - 1, None)
            } else {
                (0, Some(NavBoundaryExit::ExitTop))
            }
        } else {
            (current - 1, None)
        }
    }

    /// Navigates down. When at bottom and `wrap` is false, returns `(total - 1, Some(NavBoundaryExit::ExitBottom))`.
    pub fn nav_down(&self, current: usize, total: usize, wrap: bool) -> (usize, Option<NavBoundaryExit>) {
        if total <= 1 {
            return (0, Some(NavBoundaryExit::ExitBottom));
        }
        if current + 1 >= total {
            if wrap {
                (0, None)
            } else {
                (current, Some(NavBoundaryExit::ExitBottom))
            }
        } else {
            (current + 1, None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hstack_equal_distribution() {
        let stack = HStack::new_equal(100.0, 50.0, 500.0, 30.0, 5, 10.0);
        assert_eq!(stack.item_width(), 92.0);
        let rect0 = stack.item_rect(0);
        assert_eq!(rect0.x, 100.0);
        assert_eq!(rect0.w, 92.0);

        let rect1 = stack.item_rect(1);
        assert_eq!(rect1.x, 202.0);

        assert_eq!(stack.hit_test(150.0, 60.0), Some(0));
        assert_eq!(stack.hit_test(210.0, 60.0), Some(1));
        assert_eq!(stack.hit_test(50.0, 60.0), None);
    }

    #[test]
    fn test_hstack_navigation_and_boundary() {
        let stack = HStack::new_equal(0.0, 0.0, 300.0, 20.0, 3, 5.0);
        assert_eq!(stack.nav_left(0, false), (0, Some(NavBoundaryExit::ExitLeft)));
        assert_eq!(stack.nav_left(0, true), (2, None));
        assert_eq!(stack.nav_right(2, false), (2, Some(NavBoundaryExit::ExitRight)));
        assert_eq!(stack.nav_right(2, true), (0, None));
    }

    #[test]
    fn test_vstack_layout_and_navigation() {
        let stack = VStack::new_uniform(10.0, 20.0, 200.0, 40.0, 5.0);
        assert_eq!(stack.item_rect(0), LayoutRect::new(10.0, 20.0, 200.0, 40.0));
        assert_eq!(stack.item_rect(1), LayoutRect::new(10.0, 65.0, 200.0, 40.0));
        assert_eq!(stack.total_height(3), 3.0 * 40.0 + 2.0 * 5.0);

        // Navigation boundary exit to allow escaping to tabs above
        assert_eq!(stack.nav_up(0, 5, false), (0, Some(NavBoundaryExit::ExitTop)));
        assert_eq!(stack.nav_up(2, 5, false), (1, None));
        assert_eq!(stack.nav_down(4, 5, false), (4, Some(NavBoundaryExit::ExitBottom)));
        assert_eq!(stack.nav_down(2, 5, false), (3, None));
    }

    #[test]
    fn test_vstack_windowed_scrolling() {
        let (start, end) = VStack::visible_range(10, 4, 0);
        assert_eq!((start, end), (0, 4));

        let (start, end) = VStack::visible_range(10, 4, 5);
        assert_eq!((start, end), (3, 7));

        let (start, end) = VStack::visible_range(10, 4, 9);
        assert_eq!((start, end), (6, 10));
    }
}
