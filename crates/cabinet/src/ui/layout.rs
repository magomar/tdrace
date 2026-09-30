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

/// Uniform 2D Grid Layout manager with orthogonal traversal and boundary detection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GridLayout {
    pub bounds: LayoutRect,
    pub rows: usize,
    pub columns: usize,
    pub cell_w: f32,
    pub cell_h: f32,
    pub gap_x: f32,
    pub gap_y: f32,
    pub wrap: bool,
    pub selected_idx: usize,
}

impl GridLayout {
    /// Creates a new grid layout calculating cell dimensions from total bounds and gaps.
    pub fn new(
        bounds: LayoutRect,
        rows: usize,
        columns: usize,
        gap_x: f32,
        gap_y: f32,
        wrap: bool,
    ) -> Self {
        let rows = rows.max(1);
        let columns = columns.max(1);
        let total_gaps_x = gap_x * (columns as f32 - 1.0);
        let total_gaps_y = gap_y * (rows as f32 - 1.0);
        let cell_w = ((bounds.w - total_gaps_x) / columns as f32).max(1.0);
        let cell_h = ((bounds.h - total_gaps_y) / rows as f32).max(1.0);

        Self {
            bounds,
            rows,
            columns,
            cell_w,
            cell_h,
            gap_x,
            gap_y,
            wrap,
            selected_idx: 0,
        }
    }

    /// Total number of cells in the grid.
    #[inline]
    pub fn total_cells(&self) -> usize {
        self.rows * self.columns
    }

    /// Converts a linear index into (row, column).
    #[inline]
    pub fn row_col_of(&self, idx: usize) -> (usize, usize) {
        let safe_idx = idx.min(self.total_cells().saturating_sub(1));
        (safe_idx / self.columns, safe_idx % self.columns)
    }

    /// Converts a (row, column) into a linear index.
    #[inline]
    pub fn index_of(&self, row: usize, col: usize) -> usize {
        let r = row.min(self.rows.saturating_sub(1));
        let c = col.min(self.columns.saturating_sub(1));
        r * self.columns + c
    }

    /// Returns the bounding rectangle of a cell at (row, column).
    pub fn cell_rect(&self, row: usize, col: usize) -> LayoutRect {
        let x = self.bounds.x + (col as f32) * (self.cell_w + self.gap_x);
        let y = self.bounds.y + (row as f32) * (self.cell_h + self.gap_y);
        LayoutRect::new(x, y, self.cell_w, self.cell_h)
    }

    /// Returns the bounding rectangle of a cell at linear index `idx`.
    #[inline]
    pub fn item_rect(&self, idx: usize) -> LayoutRect {
        let (row, col) = self.row_col_of(idx);
        self.cell_rect(row, col)
    }

    /// Hit-tests a point against all cells, returning the matching index if any.
    pub fn hit_test(&self, px: f32, py: f32) -> Option<usize> {
        if !self.bounds.contains(px, py) {
            return None;
        }
        for i in 0..self.total_cells() {
            if self.item_rect(i).contains(px, py) {
                return Some(i);
            }
        }
        None
    }

    /// Navigates up.
    pub fn nav_up(&mut self) -> Option<NavBoundaryExit> {
        let (row, col) = self.row_col_of(self.selected_idx);
        if row == 0 {
            if self.wrap {
                self.selected_idx = self.index_of(self.rows - 1, col);
                None
            } else {
                Some(NavBoundaryExit::ExitTop)
            }
        } else {
            self.selected_idx = self.index_of(row - 1, col);
            None
        }
    }

    /// Navigates down.
    pub fn nav_down(&mut self) -> Option<NavBoundaryExit> {
        let (row, col) = self.row_col_of(self.selected_idx);
        if row + 1 >= self.rows {
            if self.wrap {
                self.selected_idx = self.index_of(0, col);
                None
            } else {
                Some(NavBoundaryExit::ExitBottom)
            }
        } else {
            self.selected_idx = self.index_of(row + 1, col);
            None
        }
    }

    /// Navigates left.
    pub fn nav_left(&mut self) -> Option<NavBoundaryExit> {
        let (row, col) = self.row_col_of(self.selected_idx);
        if col == 0 {
            if self.wrap {
                self.selected_idx = self.index_of(row, self.columns - 1);
                None
            } else {
                Some(NavBoundaryExit::ExitLeft)
            }
        } else {
            self.selected_idx = self.index_of(row, col - 1);
            None
        }
    }

    /// Navigates right.
    pub fn nav_right(&mut self) -> Option<NavBoundaryExit> {
        let (row, col) = self.row_col_of(self.selected_idx);
        if col + 1 >= self.columns {
            if self.wrap {
                self.selected_idx = self.index_of(row, 0);
                None
            } else {
                Some(NavBoundaryExit::ExitRight)
            }
        } else {
            self.selected_idx = self.index_of(row, col + 1);
            None
        }
    }
}

/// Wrapping Flow Layout manager for tags, chips, and badges with row wrapping and 2D traversal.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlowLayout {
    pub bounds: LayoutRect,
    pub item_widths: Vec<f32>,
    pub item_height: f32,
    pub row_gap: f32,
    pub col_gap: f32,
    pub wrap_width: f32,
    pub selected_idx: usize,
}

impl FlowLayout {
    /// Creates a new flow layout with specified item widths.
    pub fn new(
        bounds: LayoutRect,
        item_widths: Vec<f32>,
        item_height: f32,
        row_gap: f32,
        col_gap: f32,
    ) -> Self {
        let wrap_width = bounds.w;
        Self {
            bounds,
            item_widths,
            item_height,
            row_gap,
            col_gap,
            wrap_width,
            selected_idx: 0,
        }
    }

    /// Computes layout rectangles and row assignments for all items.
    pub fn compute_layout(&self) -> Vec<(LayoutRect, usize)> {
        let mut result = Vec::with_capacity(self.item_widths.len());
        let mut cur_x = self.bounds.x;
        let mut cur_y = self.bounds.y;
        let mut cur_row = 0;

        for &w in &self.item_widths {
            if cur_x + w > self.bounds.x + self.wrap_width && cur_x > self.bounds.x {
                cur_x = self.bounds.x;
                cur_y += self.item_height + self.row_gap;
                cur_row += 1;
            }
            let rect = LayoutRect::new(cur_x, cur_y, w, self.item_height);
            result.push((rect, cur_row));
            cur_x += w + self.col_gap;
        }

        result
    }

    /// Returns the bounding rectangle of the item at `idx`.
    pub fn item_rect(&self, idx: usize) -> LayoutRect {
        let layout = self.compute_layout();
        if let Some((rect, _)) = layout.get(idx) {
            *rect
        } else {
            LayoutRect::default()
        }
    }

    /// Hit-tests a point against all flow items, returning the matching index.
    pub fn hit_test(&self, px: f32, py: f32) -> Option<usize> {
        let layout = self.compute_layout();
        for (i, (rect, _)) in layout.iter().enumerate() {
            if rect.contains(px, py) {
                return Some(i);
            }
        }
        None
    }

    /// Navigates to the next item (Right), wrapping across rows or exiting at the end.
    pub fn nav_right(&mut self) -> Option<NavBoundaryExit> {
        if self.item_widths.is_empty() {
            return Some(NavBoundaryExit::ExitRight);
        }
        if self.selected_idx + 1 < self.item_widths.len() {
            self.selected_idx += 1;
            None
        } else {
            Some(NavBoundaryExit::ExitBottom)
        }
    }

    /// Navigates to the previous item (Left), wrapping across rows or exiting at the start.
    pub fn nav_left(&mut self) -> Option<NavBoundaryExit> {
        if self.item_widths.is_empty() {
            return Some(NavBoundaryExit::ExitLeft);
        }
        if self.selected_idx > 0 {
            self.selected_idx -= 1;
            None
        } else {
            Some(NavBoundaryExit::ExitTop)
        }
    }

    /// Navigates up to the nearest chip in the row above.
    pub fn nav_up(&mut self) -> Option<NavBoundaryExit> {
        let layout = self.compute_layout();
        if self.selected_idx >= layout.len() {
            return Some(NavBoundaryExit::ExitTop);
        }
        let (cur_rect, cur_row) = layout[self.selected_idx];
        if cur_row == 0 {
            return Some(NavBoundaryExit::ExitTop);
        }
        let target_row = cur_row - 1;
        let cur_center_x = cur_rect.x + cur_rect.w * 0.5;

        // Find closest item in target_row
        let mut best_idx = None;
        let mut best_dist = f32::MAX;
        for (i, (r, row)) in layout.iter().enumerate() {
            if *row == target_row {
                let center_x = r.x + r.w * 0.5;
                let dist = (center_x - cur_center_x).abs();
                if dist < best_dist {
                    best_dist = dist;
                    best_idx = Some(i);
                }
            }
        }

        if let Some(idx) = best_idx {
            self.selected_idx = idx;
            None
        } else {
            Some(NavBoundaryExit::ExitTop)
        }
    }

    /// Navigates down to the nearest chip in the row below.
    pub fn nav_down(&mut self) -> Option<NavBoundaryExit> {
        let layout = self.compute_layout();
        if self.selected_idx >= layout.len() {
            return Some(NavBoundaryExit::ExitBottom);
        }
        let (cur_rect, cur_row) = layout[self.selected_idx];
        let target_row = cur_row + 1;
        let cur_center_x = cur_rect.x + cur_rect.w * 0.5;

        let mut best_idx = None;
        let mut best_dist = f32::MAX;
        for (i, (r, row)) in layout.iter().enumerate() {
            if *row == target_row {
                let center_x = r.x + r.w * 0.5;
                let dist = (center_x - cur_center_x).abs();
                if dist < best_dist {
                    best_dist = dist;
                    best_idx = Some(i);
                }
            }
        }

        if let Some(idx) = best_idx {
            self.selected_idx = idx;
            None
        } else {
            Some(NavBoundaryExit::ExitBottom)
        }
    }
}

/// Responsive two-column spatial container with focus handoff.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SplitPane {
    pub bounds: LayoutRect,
    pub left_ratio: f32,   // e.g. 0.42 (42% left, remainder right minus gap)
    pub col_gap: f32,
    pub active_pane: usize, // 0 = Left, 1 = Right
}

impl SplitPane {
    /// Creates a new two-column split pane container.
    pub fn new(bounds: LayoutRect, left_ratio: f32, col_gap: f32) -> Self {
        Self {
            bounds,
            left_ratio: left_ratio.clamp(0.05, 0.95),
            col_gap,
            active_pane: 0,
        }
    }

    /// Returns the bounding rectangle of the left pane.
    pub fn left_rect(&self) -> LayoutRect {
        let total_avail = (self.bounds.w - self.col_gap).max(0.0);
        let left_w = total_avail * self.left_ratio;
        LayoutRect::new(self.bounds.x, self.bounds.y, left_w, self.bounds.h)
    }

    /// Returns the bounding rectangle of the right pane.
    pub fn right_rect(&self) -> LayoutRect {
        let total_avail = (self.bounds.w - self.col_gap).max(0.0);
        let left_w = total_avail * self.left_ratio;
        let right_x = self.bounds.x + left_w + self.col_gap;
        let right_w = total_avail - left_w;
        LayoutRect::new(right_x, self.bounds.y, right_w, self.bounds.h)
    }

    /// Returns the bounding rectangle for a given pane index (0 = Left, 1 = Right).
    pub fn pane_rect(&self, pane: usize) -> LayoutRect {
        if pane == 0 {
            self.left_rect()
        } else {
            self.right_rect()
        }
    }

    /// Returns the currently active pane rectangle.
    pub fn active_rect(&self) -> LayoutRect {
        self.pane_rect(self.active_pane)
    }

    /// Hit-tests a point against left and right panes.
    pub fn hit_test(&self, px: f32, py: f32) -> Option<usize> {
        if self.left_rect().contains(px, py) {
            Some(0)
        } else if self.right_rect().contains(px, py) {
            Some(1)
        } else {
            None
        }
    }

    /// Navigates left. If in the right pane, switches to left pane; otherwise emits ExitLeft.
    pub fn nav_left(&mut self) -> Option<NavBoundaryExit> {
        if self.active_pane == 1 {
            self.active_pane = 0;
            None
        } else {
            Some(NavBoundaryExit::ExitLeft)
        }
    }

    /// Navigates right. If in the left pane, switches to right pane; otherwise emits ExitRight.
    pub fn nav_right(&mut self) -> Option<NavBoundaryExit> {
        if self.active_pane == 0 {
            self.active_pane = 1;
            None
        } else {
            Some(NavBoundaryExit::ExitRight)
        }
    }
}

/// Proportional scroll indicator representing visible ratio and scroll position.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScrollIndicator {
    pub visible_ratio: f32, // viewport / content
    pub offset_ratio: f32,  // scroll position / content
}

impl ScrollIndicator {
    /// Creates a new scroll indicator directly from ratios.
    pub fn new(visible_ratio: f32, offset_ratio: f32) -> Self {
        Self {
            visible_ratio: visible_ratio.clamp(0.0, 1.0),
            offset_ratio: offset_ratio.clamp(0.0, 1.0),
        }
    }

    /// Creates a scroll indicator from item counts.
    pub fn from_counts(total: usize, visible: usize, offset: usize) -> Self {
        if total == 0 {
            return Self::new(1.0, 0.0);
        }
        let visible_ratio = (visible as f32 / total as f32).min(1.0);
        let max_offset = total.saturating_sub(visible);
        let offset_ratio = if max_offset == 0 {
            0.0
        } else {
            (offset as f32 / max_offset as f32).clamp(0.0, 1.0)
        };
        Self::new(visible_ratio, offset_ratio)
    }

    /// Returns true if content exceeds the viewport and requires scrolling.
    pub fn is_scrollable(&self) -> bool {
        self.visible_ratio < 1.0
    }

    /// Computes the thumb bar rectangle within a track area.
    pub fn thumb_rect(&self, track: LayoutRect) -> LayoutRect {
        let min_thumb_h = 10.0;
        let thumb_h = (track.h * self.visible_ratio).clamp(min_thumb_h, track.h);
        let max_travel = (track.h - thumb_h).max(0.0);
        let thumb_y = track.y + max_travel * self.offset_ratio;
        LayoutRect::new(track.x, thumb_y, track.w, thumb_h)
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

    #[test]
    fn test_grid_layout_boundary_exits_and_wrapping() {
        let bounds = LayoutRect::new(0.0, 0.0, 400.0, 300.0);
        let mut grid = GridLayout::new(bounds, 3, 4, 10.0, 10.0, false);
        assert_eq!(grid.total_cells(), 12);
        assert_eq!(grid.selected_idx, 0);

        // When focus is at row 0 and wrap is false, nav_up emits ExitTop without overflow
        assert_eq!(grid.nav_up(), Some(NavBoundaryExit::ExitTop));
        assert_eq!(grid.selected_idx, 0);

        // nav_down moves down 1 row (index 4)
        assert_eq!(grid.nav_down(), None);
        assert_eq!(grid.selected_idx, 4);

        // move to column 3 (index 7)
        grid.selected_idx = 7;
        assert_eq!(grid.nav_right(), Some(NavBoundaryExit::ExitRight));

        // When wrap is enabled, Right from column 3 wraps to column 0 of that row
        grid.wrap = true;
        assert_eq!(grid.nav_right(), None);
        assert_eq!(grid.selected_idx, 4);

        // When wrap is enabled, Up from row 0 wraps to bottom row
        grid.selected_idx = 2; // row 0, col 2
        assert_eq!(grid.nav_up(), None);
        assert_eq!(grid.selected_idx, 10); // row 2, col 2
    }

    #[test]
    fn test_flow_layout_wrap_traversal_and_exits() {
        let bounds = LayoutRect::new(0.0, 0.0, 100.0, 200.0);
        // 6 chips of width 40, col_gap 10: 2 chips per row (40+10+40=90 <= 100). Wraps across 3 rows!
        let widths = vec![40.0, 40.0, 40.0, 40.0, 40.0, 40.0];
        let mut flow = FlowLayout::new(bounds, widths, 25.0, 10.0, 10.0);

        let layout = flow.compute_layout();
        assert_eq!(layout.len(), 6);
        assert_eq!(layout[0].1, 0);
        assert_eq!(layout[1].1, 0);
        assert_eq!(layout[2].1, 1);
        assert_eq!(layout[3].1, 1);
        assert_eq!(layout[4].1, 2);
        assert_eq!(layout[5].1, 2);

        // Right from end of row 0 (index 1) wraps to first chip of row 1 (index 2)
        flow.selected_idx = 1;
        assert_eq!(flow.nav_right(), None);
        assert_eq!(flow.selected_idx, 2);

        // Nav down moves to nearest chip in row below
        flow.selected_idx = 0;
        assert_eq!(flow.nav_down(), None);
        assert_eq!(flow.selected_idx, 2);

        // Nav up from row 0 emits ExitTop
        flow.selected_idx = 0;
        assert_eq!(flow.nav_up(), Some(NavBoundaryExit::ExitTop));

        // Right from final chip emits ExitBottom
        flow.selected_idx = 5;
        assert_eq!(flow.nav_right(), Some(NavBoundaryExit::ExitBottom));
    }

    #[test]
    fn test_split_pane_focus_handoff_and_exits() {
        let bounds = LayoutRect::new(0.0, 0.0, 1000.0, 600.0);
        let mut split = SplitPane::new(bounds, 0.4, 20.0);

        let left = split.left_rect();
        let right = split.right_rect();
        assert_eq!(left.w, 980.0 * 0.4);
        assert_eq!(right.x, left.w + 20.0);
        assert_eq!(right.w, 980.0 * 0.6);

        assert_eq!(split.active_pane, 0);
        // Left from pane 0 exits left
        assert_eq!(split.nav_left(), Some(NavBoundaryExit::ExitLeft));

        // Right from pane 0 hands off to pane 1
        assert_eq!(split.nav_right(), None);
        assert_eq!(split.active_pane, 1);

        // Right from pane 1 exits right
        assert_eq!(split.nav_right(), Some(NavBoundaryExit::ExitRight));

        // Left from pane 1 hands off back to pane 0
        assert_eq!(split.nav_left(), None);
        assert_eq!(split.active_pane, 0);
    }

    #[test]
    fn test_scroll_indicator_ratios_and_thumb() {
        let ind = ScrollIndicator::from_counts(100, 25, 0);
        assert!(ind.is_scrollable());
        assert_eq!(ind.visible_ratio, 0.25);
        assert_eq!(ind.offset_ratio, 0.0);

        let track = LayoutRect::new(200.0, 50.0, 10.0, 200.0);
        let thumb0 = ind.thumb_rect(track);
        assert_eq!(thumb0.y, 50.0);
        assert_eq!(thumb0.h, 50.0);

        let ind_end = ScrollIndicator::from_counts(100, 25, 75);
        assert_eq!(ind_end.offset_ratio, 1.0);
        let thumb_end = ind_end.thumb_rect(track);
        assert_eq!(thumb_end.y, 200.0); // 50.0 + (200.0 - 50.0) * 1.0 = 200.0
    }
}

