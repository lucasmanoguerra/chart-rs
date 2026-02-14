use crate::error::{ChartError, ChartResult};

pub type TimePointIndex = i64;

const MIN_VISIBLE_BARS_COUNT: f64 = 2.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LogicalRange {
    pub from: f64,
    pub to: f64,
}

impl LogicalRange {
    #[must_use]
    pub fn left(self) -> f64 {
        self.from
    }

    #[must_use]
    pub fn right(self) -> f64 {
        self.to
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StrictRange {
    left: TimePointIndex,
    right: TimePointIndex,
}

impl StrictRange {
    #[must_use]
    pub fn new(left: TimePointIndex, right: TimePointIndex) -> Self {
        Self { left, right }
    }

    #[must_use]
    pub fn left(self) -> TimePointIndex {
        self.left
    }

    #[must_use]
    pub fn right(self) -> TimePointIndex {
        self.right
    }

    #[must_use]
    pub fn count(self) -> f64 {
        (self.right - self.left + 1) as f64
    }

    #[must_use]
    pub fn contains(self, index: TimePointIndex) -> bool {
        self.left <= index && index <= self.right
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimeScalePoint {
    pub time: f64,
    pub original_time: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeScaleOptions {
    pub right_offset: f64,
    pub right_offset_pixels: Option<f64>,
    pub bar_spacing: f64,
    pub min_bar_spacing: f64,
    pub max_bar_spacing: f64,
    pub fix_left_edge: bool,
    pub fix_right_edge: bool,
    pub lock_visible_time_range_on_resize: bool,
    pub right_bar_stays_on_scroll: bool,
}

impl Default for TimeScaleOptions {
    fn default() -> Self {
        Self {
            right_offset: 0.0,
            right_offset_pixels: None,
            bar_spacing: 6.0,
            min_bar_spacing: 0.5,
            max_bar_spacing: 0.0,
            fix_left_edge: false,
            fix_right_edge: false,
            lock_visible_time_range_on_resize: false,
            right_bar_stays_on_scroll: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct TransitionState {
    bar_spacing: f64,
    right_offset: f64,
}

#[derive(Debug, Clone)]
pub struct TimeScale {
    options: TimeScaleOptions,
    width: f64,
    base_index_or_null: Option<TimePointIndex>,
    right_offset: f64,
    points: Vec<TimeScalePoint>,
    bar_spacing: f64,
    scroll_start_point: Option<f64>,
    scale_start_point: Option<f64>,
    common_transition_start_state: Option<TransitionState>,
    visible_range: Option<LogicalRange>,
    visible_range_invalidated: bool,
}

impl Default for TimeScale {
    fn default() -> Self {
        Self::new(TimeScaleOptions::default())
    }
}

impl TimeScale {
    #[must_use]
    pub fn new(options: TimeScaleOptions) -> Self {
        Self {
            width: 0.0,
            base_index_or_null: None,
            right_offset: options.right_offset,
            points: Vec::new(),
            bar_spacing: options.bar_spacing,
            scroll_start_point: None,
            scale_start_point: None,
            common_transition_start_state: None,
            visible_range: None,
            visible_range_invalidated: true,
            options,
        }
    }

    #[must_use]
    pub fn options(&self) -> TimeScaleOptions {
        self.options
    }

    pub fn apply_options(&mut self, options: TimeScaleOptions) -> ChartResult<()> {
        self.options = options;
        if self.options.fix_left_edge {
            self.do_fix_left_edge()?;
        }
        if self.options.fix_right_edge {
            self.do_fix_right_edge();
        }
        self.set_bar_spacing(self.options.bar_spacing)?;
        if let Some(pixels) = self.options.right_offset_pixels {
            self.set_right_offset(pixels / self.bar_spacing)?;
        } else {
            self.set_right_offset(self.options.right_offset)?;
        }
        Ok(())
    }

    pub fn set_width(&mut self, new_width: f64) -> ChartResult<()> {
        if !new_width.is_finite() || new_width <= 0.0 {
            return Err(ChartError::InvalidData(
                "time scale width must be finite and > 0".to_owned(),
            ));
        }
        if (self.width - new_width).abs() <= f64::EPSILON {
            return Ok(());
        }

        let previous_visible_range = self.visible_logical_range();
        let old_width = self.width;
        self.width = new_width;
        self.visible_range_invalidated = true;

        if self.options.lock_visible_time_range_on_resize && old_width > 0.0 {
            self.bar_spacing = self.bar_spacing * new_width / old_width;
        }

        if self.options.fix_left_edge
            && let Some(range) = previous_visible_range
            && range.left() <= 0.0
        {
            let delta = old_width - new_width;
            self.right_offset -= (delta / self.bar_spacing).round() + 1.0;
            self.visible_range_invalidated = true;
        }

        self.correct_bar_spacing();
        self.correct_offset();
        Ok(())
    }

    #[must_use]
    pub fn width(&self) -> f64 {
        self.width
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.width == 0.0 || self.points.is_empty() || self.base_index_or_null.is_none()
    }

    #[must_use]
    pub fn has_points(&self) -> bool {
        !self.points.is_empty()
    }

    pub fn set_points(&mut self, points: Vec<TimeScalePoint>) {
        self.points = points;
        self.visible_range_invalidated = true;
        self.correct_offset();
    }

    pub fn set_base_index(&mut self, base_index: Option<TimePointIndex>) -> ChartResult<()> {
        self.base_index_or_null = base_index;
        self.visible_range_invalidated = true;
        self.correct_offset();
        self.do_fix_left_edge()?;
        Ok(())
    }

    #[must_use]
    pub fn base_index(&self) -> TimePointIndex {
        self.base_index_or_null.unwrap_or(0)
    }

    #[must_use]
    pub fn right_offset(&self) -> f64 {
        self.right_offset
    }

    pub fn set_right_offset(&mut self, offset: f64) -> ChartResult<()> {
        if !offset.is_finite() {
            return Err(ChartError::InvalidData(
                "time scale right offset must be finite".to_owned(),
            ));
        }
        self.right_offset = offset;
        self.visible_range_invalidated = true;
        self.correct_offset();
        Ok(())
    }

    #[must_use]
    pub fn bar_spacing(&self) -> f64 {
        self.bar_spacing
    }

    pub fn set_bar_spacing(&mut self, new_bar_spacing: f64) -> ChartResult<()> {
        if !new_bar_spacing.is_finite() || new_bar_spacing <= 0.0 {
            return Err(ChartError::InvalidData(
                "time scale bar spacing must be finite and > 0".to_owned(),
            ));
        }
        let old_bar_spacing = self.bar_spacing;
        self.bar_spacing = new_bar_spacing;
        self.correct_bar_spacing();

        if self.options.right_offset_pixels.is_some() && old_bar_spacing > 0.0 {
            self.right_offset = self.right_offset * old_bar_spacing / self.bar_spacing;
        }

        self.correct_offset();
        self.visible_range_invalidated = true;
        Ok(())
    }

    pub fn restore_default(&mut self) -> ChartResult<()> {
        self.visible_range_invalidated = true;
        self.set_bar_spacing(self.options.bar_spacing)?;
        let new_offset = if let Some(px) = self.options.right_offset_pixels {
            px / self.bar_spacing
        } else {
            self.options.right_offset
        };
        self.set_right_offset(new_offset)
    }

    pub fn set_visible_range(
        &mut self,
        strict_range: StrictRange,
        apply_default_offset: bool,
    ) -> ChartResult<()> {
        if self.width <= 0.0 {
            return Err(ChartError::InvalidData(
                "cannot set visible range before width".to_owned(),
            ));
        }
        let length = strict_range.count();
        if !length.is_finite() || length <= 0.0 {
            return Err(ChartError::InvalidData(
                "visible strict range must be non-empty".to_owned(),
            ));
        }
        let pixel_offset = if apply_default_offset {
            self.options.right_offset_pixels.unwrap_or(0.0)
        } else {
            0.0
        };
        self.set_bar_spacing((self.width - pixel_offset) / length)?;
        self.right_offset = strict_range.right() as f64 - self.base_index() as f64;
        if apply_default_offset {
            self.right_offset = if pixel_offset > 0.0 {
                pixel_offset / self.bar_spacing
            } else {
                self.options.right_offset
            };
        }
        self.correct_offset();
        self.visible_range_invalidated = true;
        Ok(())
    }

    pub fn set_logical_range(&mut self, range: LogicalRange) -> ChartResult<()> {
        let strict = StrictRange::new(range.from as TimePointIndex, range.to as TimePointIndex);
        self.set_visible_range(strict, false)
    }

    pub fn fit_content(&mut self) -> ChartResult<()> {
        let Some(first) = self.first_index() else {
            return Ok(());
        };
        let Some(last) = self.last_index() else {
            return Ok(());
        };
        let right_offset_bars = if self.options.right_offset_pixels.is_none() {
            self.options.right_offset
        } else {
            0.0
        };
        self.set_visible_range(
            StrictRange::new(first, last + right_offset_bars as TimePointIndex),
            true,
        )
    }

    pub fn index_to_coordinate(&self, index: TimePointIndex) -> ChartResult<f64> {
        if self.is_empty() {
            return Ok(0.0);
        }
        let base_index = self.base_index() as f64;
        let delta_from_right = base_index + self.right_offset - index as f64;
        Ok(self.width - (delta_from_right + 0.5) * self.bar_spacing - 1.0)
    }

    pub fn coordinate_to_index(&self, x: f64) -> ChartResult<TimePointIndex> {
        Ok(self.coordinate_to_float_index(x)?.ceil() as TimePointIndex)
    }

    pub fn coordinate_to_float_index(&self, x: f64) -> ChartResult<f64> {
        if !x.is_finite() {
            return Err(ChartError::InvalidData(
                "coordinate must be finite".to_owned(),
            ));
        }
        if self.bar_spacing <= 0.0 {
            return Err(ChartError::InvalidData(
                "bar spacing must be > 0".to_owned(),
            ));
        }
        let delta_from_right = (self.width - 1.0 - x) / self.bar_spacing;
        let index = self.base_index() as f64 + self.right_offset - delta_from_right;
        Ok((index * 1_000_000.0).round() / 1_000_000.0)
    }

    pub fn zoom(&mut self, zoom_point: f64, scale: f64) -> ChartResult<()> {
        if self.is_empty() || !scale.is_finite() || scale == 0.0 {
            return Ok(());
        }
        let clamped_zoom_point = zoom_point.clamp(1.0, self.width);
        let float_index_at_zoom_point = self.coordinate_to_float_index(clamped_zoom_point)?;
        let bar_spacing = self.bar_spacing;
        let new_bar_spacing = bar_spacing + scale * (bar_spacing / 10.0);
        self.set_bar_spacing(new_bar_spacing)?;
        if !self.options.right_bar_stays_on_scroll {
            let corrected = self.right_offset
                + (float_index_at_zoom_point
                    - self.coordinate_to_float_index(clamped_zoom_point)?);
            self.set_right_offset(corrected)?;
        }
        Ok(())
    }

    pub fn start_scale(&mut self, x: f64) {
        if self.scroll_start_point.is_some() {
            self.end_scroll();
        }
        if self.scale_start_point.is_some() || self.common_transition_start_state.is_some() {
            return;
        }
        if self.is_empty() {
            return;
        }
        self.scale_start_point = Some(x);
        self.save_common_transition_start_state();
    }

    pub fn scale_to(&mut self, x: f64) -> ChartResult<()> {
        let Some(start_state) = self.common_transition_start_state else {
            return Ok(());
        };
        let Some(scale_start) = self.scale_start_point else {
            return Ok(());
        };
        let start_length_from_right = (self.width - x).clamp(0.0, self.width);
        let current_length_from_right = (self.width - scale_start).clamp(0.0, self.width);
        if start_length_from_right == 0.0 || current_length_from_right == 0.0 {
            return Ok(());
        }
        self.set_bar_spacing(
            start_state.bar_spacing * start_length_from_right / current_length_from_right,
        )
    }

    pub fn end_scale(&mut self) {
        if self.scale_start_point.is_none() {
            return;
        }
        self.scale_start_point = None;
        self.clear_common_transition_start_state();
    }

    pub fn start_scroll(&mut self, x: f64) {
        if self.scroll_start_point.is_some() || self.common_transition_start_state.is_some() {
            return;
        }
        if self.is_empty() {
            return;
        }
        self.scroll_start_point = Some(x);
        self.save_common_transition_start_state();
    }

    pub fn scroll_to(&mut self, x: f64) {
        let Some(scroll_start_point) = self.scroll_start_point else {
            return;
        };
        let shift_in_logical = (scroll_start_point - x) / self.bar_spacing;
        let start = self
            .common_transition_start_state
            .unwrap_or(TransitionState {
                bar_spacing: self.bar_spacing,
                right_offset: self.right_offset,
            });
        self.right_offset = start.right_offset + shift_in_logical;
        self.visible_range_invalidated = true;
        self.correct_offset();
    }

    pub fn end_scroll(&mut self) {
        if self.scroll_start_point.is_none() {
            return;
        }
        self.scroll_start_point = None;
        self.clear_common_transition_start_state();
    }

    pub fn visible_logical_range(&mut self) -> Option<LogicalRange> {
        self.update_visible_range();
        self.visible_range
    }

    pub fn visible_strict_range(&mut self) -> Option<StrictRange> {
        self.update_visible_range();
        self.visible_range.map(|range| {
            StrictRange::new(
                range.left().floor() as TimePointIndex,
                range.right().ceil() as TimePointIndex,
            )
        })
    }

    #[must_use]
    pub fn first_index(&self) -> Option<TimePointIndex> {
        if self.points.is_empty() {
            None
        } else {
            Some(0)
        }
    }

    #[must_use]
    pub fn last_index(&self) -> Option<TimePointIndex> {
        if self.points.is_empty() {
            None
        } else {
            Some(self.points.len() as TimePointIndex - 1)
        }
    }

    fn update_visible_range(&mut self) {
        if !self.visible_range_invalidated {
            return;
        }
        self.visible_range_invalidated = false;
        if self.is_empty() {
            self.visible_range = None;
            return;
        }
        let new_bars_length = self.width / self.bar_spacing;
        let right_border = self.right_offset + self.base_index() as f64;
        let left_border = right_border - new_bars_length + 1.0;
        self.visible_range = Some(LogicalRange {
            from: left_border,
            to: right_border,
        });
    }

    fn correct_bar_spacing(&mut self) {
        let min = self.min_bar_spacing();
        let max = self.max_bar_spacing();
        let clamped = self.bar_spacing.clamp(min, max);
        if (clamped - self.bar_spacing).abs() > f64::EPSILON {
            self.bar_spacing = clamped;
            self.visible_range_invalidated = true;
        }
    }

    fn min_bar_spacing(&self) -> f64 {
        if self.options.fix_left_edge && self.options.fix_right_edge && !self.points.is_empty() {
            return self.width / self.points.len() as f64;
        }
        self.options.min_bar_spacing
    }

    fn max_bar_spacing(&self) -> f64 {
        if self.options.max_bar_spacing > 0.0 {
            self.options.max_bar_spacing
        } else {
            self.width * 0.5
        }
    }

    fn min_right_offset(&self) -> Option<f64> {
        let first = self.first_index()?;
        let base = self.base_index_or_null?;
        let bars_estimation = if self.options.fix_left_edge {
            self.width / self.bar_spacing
        } else {
            MIN_VISIBLE_BARS_COUNT.min(self.points.len() as f64)
        };
        Some(first as f64 - base as f64 - 1.0 + bars_estimation)
    }

    fn max_right_offset(&self) -> f64 {
        if self.options.fix_right_edge {
            0.0
        } else {
            self.width / self.bar_spacing - MIN_VISIBLE_BARS_COUNT.min(self.points.len() as f64)
        }
    }

    fn correct_offset(&mut self) {
        if let Some(min_right_offset) = self.min_right_offset()
            && self.right_offset < min_right_offset
        {
            self.right_offset = min_right_offset;
            self.visible_range_invalidated = true;
        }
        let max_right_offset = self.max_right_offset();
        if self.right_offset > max_right_offset {
            self.right_offset = max_right_offset;
            self.visible_range_invalidated = true;
        }
    }

    fn do_fix_left_edge(&mut self) -> ChartResult<()> {
        if !self.options.fix_left_edge {
            return Ok(());
        }
        let Some(first) = self.first_index() else {
            return Ok(());
        };
        let Some(visible) = self.visible_strict_range() else {
            return Ok(());
        };
        let delta = visible.left() - first;
        if delta < 0 {
            let left_edge_offset = self.right_offset - delta as f64 - 1.0;
            self.set_right_offset(left_edge_offset)?;
        }
        self.correct_bar_spacing();
        Ok(())
    }

    fn do_fix_right_edge(&mut self) {
        self.correct_offset();
        self.correct_bar_spacing();
    }

    fn save_common_transition_start_state(&mut self) {
        self.common_transition_start_state = Some(TransitionState {
            bar_spacing: self.bar_spacing,
            right_offset: self.right_offset,
        });
    }

    fn clear_common_transition_start_state(&mut self) {
        self.common_transition_start_state = None;
    }
}

#[cfg(test)]
mod tests {
    use super::{TimeScale, TimeScaleOptions};

    #[test]
    fn index_coordinate_and_coordinate_index_match_lightweight_formula() {
        let mut time_scale = TimeScale::new(TimeScaleOptions::default());
        time_scale.set_width(1000.0).expect("width");
        time_scale.set_points(
            (0..200)
                .map(|i| super::TimeScalePoint {
                    time: i as f64,
                    original_time: None,
                })
                .collect(),
        );
        time_scale.set_base_index(Some(199)).expect("base");
        time_scale.set_right_offset(0.0).expect("offset");
        time_scale.set_bar_spacing(6.0).expect("spacing");

        let x = time_scale
            .index_to_coordinate(199)
            .expect("index_to_coordinate");
        assert!((x - (1000.0 - (0.5 * 6.0) - 1.0)).abs() <= 1e-9);

        let logical = time_scale
            .coordinate_to_float_index(x)
            .expect("coordinate_to_float_index");
        assert!((logical - 198.5).abs() <= 1e-9);
    }

    #[test]
    fn zoom_preserves_anchor_when_right_bar_does_not_stay() {
        let mut time_scale = TimeScale::new(TimeScaleOptions::default());
        time_scale.set_width(800.0).expect("width");
        time_scale.set_points(
            (0..100)
                .map(|i| super::TimeScalePoint {
                    time: i as f64,
                    original_time: None,
                })
                .collect(),
        );
        time_scale.set_base_index(Some(99)).expect("base");
        time_scale.set_bar_spacing(5.0).expect("spacing");
        let anchor = 400.0;
        let before = time_scale
            .coordinate_to_float_index(anchor)
            .expect("anchor-before");
        time_scale.zoom(anchor, 0.5).expect("zoom");
        let after = time_scale
            .coordinate_to_float_index(anchor)
            .expect("anchor-after");
        assert!((before - after).abs() <= 1e-6);
    }
}
