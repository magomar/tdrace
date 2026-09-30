use macroquad::shapes::draw_circle;
use serde::{Deserialize, Serialize};

use crate::ui::scaler::UiScaler;
use crate::ui::theme::Palette;

/// Pagination dots indicator for carousel and multi-page menus.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageDots {
    pub page: usize,
    pub total_pages: usize,
}

impl PageDots {
    pub fn new(page: usize, total_pages: usize) -> Self {
        let total = total_pages.max(1);
        Self {
            page: page.min(total - 1),
            total_pages: total,
        }
    }

    pub fn next_page(&mut self) -> bool {
        if self.page + 1 < self.total_pages {
            self.page += 1;
            true
        } else {
            false
        }
    }

    pub fn prev_page(&mut self) -> bool {
        if self.page > 0 {
            self.page -= 1;
            true
        } else {
            false
        }
    }

    pub fn set_page(&mut self, page: usize) -> bool {
        if page < self.total_pages {
            self.page = page;
            true
        } else {
            false
        }
    }

    /// Renders horizontal pagination dots centered around `(center_x, center_y)`.
    pub fn draw(
        &self,
        scaler: &UiScaler,
        center_x: f32,
        center_y: f32,
        dot_radius: f32,
        dot_gap: f32,
    ) {
        if self.total_pages <= 1 {
            return;
        }

        let r = scaler.s(dot_radius);
        let gap = scaler.s(dot_gap);
        let total_w = (self.total_pages as f32) * (r * 2.0) + ((self.total_pages - 1) as f32) * gap;
        let start_x = center_x - total_w * 0.5 + r;

        for i in 0..self.total_pages {
            let cx = start_x + (i as f32) * (r * 2.0 + gap);
            let color = if i == self.page {
                Palette::NEON_GOLD
            } else {
                Palette::UI_CARD_BORDER
            };
            draw_circle(cx, center_y, r, color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_dots_navigation_and_bounds() {
        let mut dots = PageDots::new(0, 4);
        assert_eq!(dots.page, 0);
        assert_eq!(dots.total_pages, 4);

        // Prev at 0 returns false
        assert!(!dots.prev_page());
        assert_eq!(dots.page, 0);

        // Next advances
        assert!(dots.next_page());
        assert_eq!(dots.page, 1);

        assert!(dots.set_page(3));
        assert_eq!(dots.page, 3);

        // Next at end returns false
        assert!(!dots.next_page());
        assert_eq!(dots.page, 3);
    }
}
