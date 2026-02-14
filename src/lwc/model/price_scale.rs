use crate::error::{ChartError, ChartResult};

use super::StrictRange;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PriceScaleMode {
    #[default]
    Normal,
    Logarithmic,
    Percentage,
    IndexedTo100,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PriceScaleState {
    pub auto_scale: bool,
    pub is_inverted: bool,
    pub mode: PriceScaleMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PriceScaleStateChange {
    pub auto_scale: Option<bool>,
    pub is_inverted: Option<bool>,
    pub mode: Option<PriceScaleMode>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PriceScaleMargins {
    pub top: f64,
    pub bottom: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PriceScaleOptions {
    pub auto_scale: bool,
    pub mode: PriceScaleMode,
    pub invert_scale: bool,
    pub scale_margins: PriceScaleMargins,
    pub ensure_edge_tick_marks_visible: bool,
}

impl Default for PriceScaleOptions {
    fn default() -> Self {
        Self {
            auto_scale: true,
            mode: PriceScaleMode::Normal,
            invert_scale: false,
            scale_margins: PriceScaleMargins {
                top: 0.2,
                bottom: 0.1,
            },
            ensure_edge_tick_marks_visible: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PriceRange {
    min: f64,
    max: f64,
}

impl PriceRange {
    #[must_use]
    pub fn new(min: f64, max: f64) -> Self {
        Self { min, max }
    }

    #[must_use]
    pub fn min(self) -> f64 {
        self.min
    }

    #[must_use]
    pub fn max(self) -> f64 {
        self.max
    }

    #[must_use]
    pub fn length(self) -> f64 {
        self.max - self.min
    }

    #[must_use]
    pub fn is_empty(self) -> bool {
        self.min == self.max || self.min.is_nan() || self.max.is_nan()
    }

    #[must_use]
    pub fn merge(self, other: PriceRange) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    pub fn scale_around_center(&mut self, coeff: f64) {
        if !coeff.is_finite() || self.length() == 0.0 {
            return;
        }
        let center = (self.max + self.min) * 0.5;
        let max_delta = (self.max - center) * coeff;
        let min_delta = (self.min - center) * coeff;
        self.max = center + max_delta;
        self.min = center + min_delta;
    }

    pub fn shift(&mut self, delta: f64) {
        if !delta.is_finite() {
            return;
        }
        self.max += delta;
        self.min += delta;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AutoScaleMargins {
    pub above: f64,
    pub below: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AutoScaleInfo {
    pub price_range: Option<PriceRange>,
    pub margins: Option<AutoScaleMargins>,
}

pub trait AutoScaleSource {
    fn visible(&self) -> bool;
    fn first_value(&self) -> Option<f64>;
    fn autoscale_info(&self, visible_bars: StrictRange) -> Option<AutoScaleInfo>;
    fn min_move(&self) -> f64 {
        1.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct LogFormula {
    logical_offset: f64,
    coord_offset: f64,
}

impl Default for LogFormula {
    fn default() -> Self {
        Self {
            logical_offset: 4.0,
            coord_offset: 0.0001,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PriceScale {
    id: String,
    options: PriceScaleOptions,
    height: f64,
    internal_height_cache: Option<f64>,
    price_range: Option<PriceRange>,
    price_range_snapshot: Option<PriceRange>,
    invalidated_for_range: Option<StrictRange>,
    invalidated_for_range_valid: bool,
    is_custom_price_range: bool,
    margin_above: f64,
    margin_below: f64,
    scale_start_point: Option<f64>,
    scroll_start_point: Option<f64>,
    log_formula: LogFormula,
    min_move_override: Option<f64>,
}

impl PriceScale {
    #[must_use]
    pub fn new(id: impl Into<String>, options: PriceScaleOptions) -> Self {
        Self {
            id: id.into(),
            options,
            height: 0.0,
            internal_height_cache: None,
            price_range: None,
            price_range_snapshot: None,
            invalidated_for_range: None,
            invalidated_for_range_valid: false,
            is_custom_price_range: false,
            margin_above: 0.0,
            margin_below: 0.0,
            scale_start_point: None,
            scroll_start_point: None,
            log_formula: LogFormula::default(),
            min_move_override: None,
        }
    }

    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    #[must_use]
    pub fn options(&self) -> PriceScaleOptions {
        self.options
    }

    pub fn apply_options(&mut self, options: PriceScaleOptions) -> ChartResult<()> {
        if !(0.0..=1.0).contains(&options.scale_margins.top) {
            return Err(ChartError::InvalidData(
                "price scale top margin must be in [0,1]".to_owned(),
            ));
        }
        if !(0.0..=1.0).contains(&options.scale_margins.bottom) {
            return Err(ChartError::InvalidData(
                "price scale bottom margin must be in [0,1]".to_owned(),
            ));
        }
        if options.scale_margins.top + options.scale_margins.bottom > 1.0 {
            return Err(ChartError::InvalidData(
                "sum of price scale margins must be <= 1".to_owned(),
            ));
        }
        self.options = options;
        self.invalidate_internal_height_cache();
        Ok(())
    }

    #[must_use]
    pub fn mode(&self) -> PriceScaleState {
        PriceScaleState {
            auto_scale: self.options.auto_scale,
            is_inverted: self.options.invert_scale,
            mode: self.options.mode,
        }
    }

    pub fn set_mode(&mut self, change: PriceScaleStateChange) {
        let old_mode = self.mode();
        if let Some(auto_scale) = change.auto_scale {
            self.options.auto_scale = auto_scale;
        }
        if let Some(mode) = change.mode {
            self.options.mode = mode;
            if matches!(
                mode,
                PriceScaleMode::Percentage | PriceScaleMode::IndexedTo100
            ) {
                self.options.auto_scale = true;
            }
            self.invalidated_for_range_valid = false;
        }
        if old_mode.mode == PriceScaleMode::Logarithmic && self.options.mode != old_mode.mode {
            if let Some(pr) = self.price_range {
                if let Some(raw) = convert_price_range_from_log(Some(pr), self.log_formula) {
                    self.price_range = Some(raw);
                } else {
                    self.options.auto_scale = true;
                }
            }
        }
        if self.options.mode == PriceScaleMode::Logarithmic && self.options.mode != old_mode.mode {
            self.price_range = convert_price_range_to_log(self.price_range, self.log_formula);
        }
        if let Some(inverted) = change.is_inverted {
            self.options.invert_scale = inverted;
        }
    }

    #[must_use]
    pub fn is_auto_scale(&self) -> bool {
        self.options.auto_scale
    }

    #[must_use]
    pub fn is_custom_price_range(&self) -> bool {
        self.is_custom_price_range
    }

    #[must_use]
    pub fn is_log(&self) -> bool {
        self.options.mode == PriceScaleMode::Logarithmic
    }

    #[must_use]
    pub fn is_percentage(&self) -> bool {
        self.options.mode == PriceScaleMode::Percentage
    }

    #[must_use]
    pub fn is_indexed_to_100(&self) -> bool {
        self.options.mode == PriceScaleMode::IndexedTo100
    }

    #[must_use]
    pub fn is_inverted(&self) -> bool {
        self.options.invert_scale
    }

    pub fn set_height(&mut self, value: f64) {
        if (self.height - value).abs() <= f64::EPSILON {
            return;
        }
        self.height = value;
        self.invalidate_internal_height_cache();
    }

    #[must_use]
    pub fn height(&self) -> f64 {
        self.height
    }

    #[must_use]
    pub fn internal_height(&mut self) -> f64 {
        if let Some(cached) = self.internal_height_cache {
            return cached;
        }
        let value = self.height - self.top_margin_px() - self.bottom_margin_px();
        self.internal_height_cache = Some(value);
        value
    }

    #[must_use]
    pub fn price_range(&mut self) -> Option<PriceRange> {
        self.make_sure_valid();
        self.price_range
    }

    pub fn set_price_range(&mut self, range: Option<PriceRange>) {
        if self.price_range == range {
            return;
        }
        self.price_range = range;
    }

    pub fn set_custom_price_range(&mut self, range: Option<PriceRange>) {
        self.set_price_range(range);
        self.is_custom_price_range = range.is_some();
    }

    #[must_use]
    pub fn is_empty(&mut self) -> bool {
        self.make_sure_valid();
        self.height == 0.0 || self.price_range.is_none_or(|r| r.is_empty())
    }

    pub fn set_min_move_override(&mut self, min_move: Option<f64>) {
        self.min_move_override = min_move;
    }

    #[must_use]
    pub fn min_move(&self) -> f64 {
        self.min_move_override.unwrap_or(1.0)
    }

    pub fn recalculate_price_range(
        &mut self,
        visible_bars: StrictRange,
        sources: &[&dyn AutoScaleSource],
    ) {
        self.invalidated_for_range = Some(visible_bars);
        self.invalidated_for_range_valid = false;
        self.recalculate_price_range_impl(sources);
    }

    pub fn price_to_coordinate(&mut self, price: f64, base_value: f64) -> ChartResult<f64> {
        let logical = if self.is_percentage() {
            to_percent(price, base_value)
        } else if self.is_indexed_to_100() {
            to_indexed_to_100(price, base_value)
        } else {
            price
        };
        self.logical_to_coordinate(logical)
    }

    pub fn coordinate_to_price(&mut self, coordinate: f64, base_value: f64) -> ChartResult<f64> {
        let logical = self.coordinate_to_logical(coordinate)?;
        Ok(self.logical_to_price(logical, base_value))
    }

    #[must_use]
    pub fn logical_to_price(&self, logical: f64, base_value: f64) -> f64 {
        if self.is_percentage() {
            from_percent(logical, base_value)
        } else if self.is_indexed_to_100() {
            from_indexed_to_100(logical, base_value)
        } else {
            logical
        }
    }

    pub fn logical_to_coordinate(&mut self, mut logical: f64) -> ChartResult<f64> {
        self.make_sure_valid();
        if self.is_empty() {
            return Ok(0.0);
        }
        if self.is_log() && logical != 0.0 {
            logical = to_log(logical, self.log_formula);
        }
        let range = self
            .price_range
            .ok_or_else(|| ChartError::InvalidData("price range is not available".to_owned()))?;
        let inv_coordinate = self.bottom_margin_px()
            + (self.internal_height() - 1.0) * (logical - range.min()) / range.length();
        Ok(self.inverted_coordinate(inv_coordinate))
    }

    pub fn coordinate_to_logical(&mut self, coordinate: f64) -> ChartResult<f64> {
        self.make_sure_valid();
        if self.is_empty() {
            return Ok(0.0);
        }
        let range = self
            .price_range
            .ok_or_else(|| ChartError::InvalidData("price range is not available".to_owned()))?;
        let inv_coordinate = self.inverted_coordinate(coordinate);
        let logical = range.min()
            + range.length()
                * ((inv_coordinate - self.bottom_margin_px()) / (self.internal_height() - 1.0));
        if self.is_log() {
            Ok(from_log(logical, self.log_formula))
        } else {
            Ok(logical)
        }
    }

    pub fn start_scale(&mut self, x: f64) {
        if self.is_percentage() || self.is_indexed_to_100() {
            return;
        }
        if self.scale_start_point.is_some() || self.price_range_snapshot.is_some() {
            return;
        }
        self.make_sure_valid();
        if self.is_empty() {
            return;
        }
        self.scale_start_point = Some(self.height - x);
        self.price_range_snapshot = self.price_range;
    }

    pub fn scale_to(&mut self, mut x: f64) {
        if self.is_percentage() || self.is_indexed_to_100() {
            return;
        }
        let Some(scale_start) = self.scale_start_point else {
            return;
        };
        self.options.auto_scale = false;
        x = self.height - x;
        if x < 0.0 {
            x = 0.0;
        }
        let mut coeff = (scale_start + (self.height - 1.0) * 0.2) / (x + (self.height - 1.0) * 0.2);
        coeff = coeff.max(0.1);
        if let Some(mut range) = self.price_range_snapshot {
            range.scale_around_center(coeff);
            self.price_range = Some(range);
        }
    }

    pub fn end_scale(&mut self) {
        if self.is_percentage() || self.is_indexed_to_100() {
            return;
        }
        self.scale_start_point = None;
        self.price_range_snapshot = None;
    }

    pub fn start_scroll(&mut self, x: f64) {
        if self.options.auto_scale {
            return;
        }
        if self.scroll_start_point.is_some() || self.price_range_snapshot.is_some() {
            return;
        }
        self.make_sure_valid();
        if self.is_empty() {
            return;
        }
        self.scroll_start_point = Some(x);
        self.price_range_snapshot = self.price_range;
    }

    pub fn scroll_to(&mut self, x: f64) {
        if self.options.auto_scale {
            return;
        }
        let Some(scroll_start) = self.scroll_start_point else {
            return;
        };
        let Some(current_range) = self.price_range else {
            return;
        };
        let mut pixel_delta = x - scroll_start;
        if self.is_inverted() {
            pixel_delta *= -1.0;
        }
        let price_units_per_pixel = current_range.length() / (self.internal_height() - 1.0);
        let price_delta = pixel_delta * price_units_per_pixel;
        if let Some(mut snapshot) = self.price_range_snapshot {
            snapshot.shift(price_delta);
            self.price_range = Some(snapshot);
        }
    }

    pub fn end_scroll(&mut self) {
        if self.options.auto_scale {
            return;
        }
        self.scroll_start_point = None;
        self.price_range_snapshot = None;
    }

    #[must_use]
    pub fn has_visible_edge_marks(&self) -> bool {
        self.options.ensure_edge_tick_marks_visible && self.options.auto_scale
    }

    #[must_use]
    pub fn edge_marks_padding(&self) -> f64 {
        6.0
    }

    fn recalculate_price_range_impl(&mut self, sources: &[&dyn AutoScaleSource]) {
        if self.is_custom_price_range && !self.options.auto_scale {
            return;
        }
        let Some(visible_bars) = self.invalidated_for_range else {
            return;
        };
        let mut price_range: Option<PriceRange> = None;
        let mut margin_above: f64 = 0.0;
        let mut margin_below: f64 = 0.0;

        for source in sources {
            if !source.visible() {
                continue;
            }
            let Some(first_value) = source.first_value() else {
                continue;
            };
            let Some(info) = source.autoscale_info(visible_bars) else {
                continue;
            };
            let Some(mut source_range) = info.price_range else {
                continue;
            };
            source_range = match self.options.mode {
                PriceScaleMode::Logarithmic => {
                    convert_price_range_to_log(Some(source_range), self.log_formula)
                        .unwrap_or(source_range)
                }
                PriceScaleMode::Percentage => to_percent_range(source_range, first_value),
                PriceScaleMode::IndexedTo100 => to_indexed_to_100_range(source_range, first_value),
                PriceScaleMode::Normal => source_range,
            };
            price_range = Some(if let Some(acc) = price_range {
                acc.merge(source_range)
            } else {
                source_range
            });
            if let Some(margins) = info.margins {
                margin_above = margin_above.max(margins.above);
                margin_below = margin_below.max(margins.below);
            }
        }

        if self.has_visible_edge_marks() {
            margin_above = margin_above.max(self.edge_marks_padding());
            margin_below = margin_below.max(self.edge_marks_padding());
        }
        if (margin_above - self.margin_above).abs() > f64::EPSILON
            || (margin_below - self.margin_below).abs() > f64::EPSILON
        {
            self.margin_above = margin_above;
            self.margin_below = margin_below;
            self.invalidate_internal_height_cache();
        }

        if let Some(mut range) = price_range {
            if (range.min() - range.max()).abs() <= f64::EPSILON {
                let extend = 5.0 * self.min_move();
                if self.is_log() {
                    if let Some(raw) = convert_price_range_from_log(Some(range), self.log_formula) {
                        range = raw;
                    }
                }
                range = PriceRange::new(range.min() - extend, range.max() + extend);
                if self.is_log() {
                    if let Some(log_range) =
                        convert_price_range_to_log(Some(range), self.log_formula)
                    {
                        range = log_range;
                    }
                }
            }
            if self.is_log() {
                if let Some(raw) = convert_price_range_from_log(Some(range), self.log_formula) {
                    let new_formula = log_formula_for_price_range(Some(raw));
                    if !log_formulas_are_same(new_formula, self.log_formula) {
                        self.log_formula = new_formula;
                        if let Some(log_range) =
                            convert_price_range_to_log(Some(raw), self.log_formula)
                        {
                            range = log_range;
                        }
                        if let Some(snapshot_raw) = convert_price_range_from_log(
                            self.price_range_snapshot,
                            self.log_formula,
                        ) {
                            self.price_range_snapshot =
                                convert_price_range_to_log(Some(snapshot_raw), self.log_formula);
                        }
                    }
                }
            }
            self.price_range = Some(range);
        } else if self.price_range.is_none() {
            self.price_range = Some(PriceRange::new(-0.5, 0.5));
            self.log_formula = log_formula_for_price_range(None);
        }
        self.invalidated_for_range_valid = true;
    }

    fn make_sure_valid(&mut self) {
        if !self.invalidated_for_range_valid {
            self.invalidated_for_range_valid = true;
        }
    }

    fn invalidate_internal_height_cache(&mut self) {
        self.internal_height_cache = None;
    }

    fn inverted_coordinate(&self, coordinate: f64) -> f64 {
        if self.is_inverted() {
            coordinate
        } else {
            self.height - 1.0 - coordinate
        }
    }

    fn top_margin_px(&self) -> f64 {
        if self.is_inverted() {
            self.options.scale_margins.bottom * self.height + self.margin_below
        } else {
            self.options.scale_margins.top * self.height + self.margin_above
        }
    }

    fn bottom_margin_px(&self) -> f64 {
        if self.is_inverted() {
            self.options.scale_margins.top * self.height + self.margin_above
        } else {
            self.options.scale_margins.bottom * self.height + self.margin_below
        }
    }
}

fn from_percent(value: f64, base_value: f64) -> f64 {
    let value = if base_value < 0.0 { -value } else { value };
    (value / 100.0) * base_value + base_value
}

fn to_percent(value: f64, base_value: f64) -> f64 {
    let result = 100.0 * (value - base_value) / base_value;
    if base_value < 0.0 { -result } else { result }
}

fn to_percent_range(range: PriceRange, base_value: f64) -> PriceRange {
    PriceRange::new(
        to_percent(range.min(), base_value),
        to_percent(range.max(), base_value),
    )
}

fn from_indexed_to_100(value: f64, base_value: f64) -> f64 {
    let mut value = value - 100.0;
    if base_value < 0.0 {
        value = -value;
    }
    (value / 100.0) * base_value + base_value
}

fn to_indexed_to_100(value: f64, base_value: f64) -> f64 {
    let result = 100.0 * (value - base_value) / base_value + 100.0;
    if base_value < 0.0 { -result } else { result }
}

fn to_indexed_to_100_range(range: PriceRange, base_value: f64) -> PriceRange {
    PriceRange::new(
        to_indexed_to_100(range.min(), base_value),
        to_indexed_to_100(range.max(), base_value),
    )
}

fn to_log(price: f64, log_formula: LogFormula) -> f64 {
    let magnitude = price.abs();
    if magnitude < 1e-15 {
        return 0.0;
    }
    let value = (magnitude + log_formula.coord_offset).log10() + log_formula.logical_offset;
    if price < 0.0 { -value } else { value }
}

fn from_log(logical: f64, log_formula: LogFormula) -> f64 {
    let magnitude = logical.abs();
    if magnitude < 1e-15 {
        return 0.0;
    }
    let value = 10f64.powf(magnitude - log_formula.logical_offset) - log_formula.coord_offset;
    if logical < 0.0 { -value } else { value }
}

fn convert_price_range_to_log(
    range: Option<PriceRange>,
    formula: LogFormula,
) -> Option<PriceRange> {
    range.map(|r| PriceRange::new(to_log(r.min(), formula), to_log(r.max(), formula)))
}

fn convert_price_range_from_log(
    range: Option<PriceRange>,
    formula: LogFormula,
) -> Option<PriceRange> {
    range.map(|r| PriceRange::new(from_log(r.min(), formula), from_log(r.max(), formula)))
}

fn log_formula_for_price_range(range: Option<PriceRange>) -> LogFormula {
    let default = LogFormula::default();
    let Some(range) = range else {
        return default;
    };
    let diff = (range.max() - range.min()).abs();
    if !(1e-15..1.0).contains(&diff) {
        return default;
    }
    let digits = diff.log10().abs().ceil();
    let logical_offset = default.logical_offset + digits;
    let coord_offset = 1.0 / 10f64.powf(logical_offset);
    LogFormula {
        logical_offset,
        coord_offset,
    }
}

fn log_formulas_are_same(left: LogFormula, right: LogFormula) -> bool {
    (left.logical_offset - right.logical_offset).abs() <= f64::EPSILON
        && (left.coord_offset - right.coord_offset).abs() <= f64::EPSILON
}

#[cfg(test)]
mod tests {
    use super::{PriceScale, PriceScaleMode, PriceScaleOptions, StrictRange};

    #[test]
    fn linear_price_coordinate_round_trip_is_stable() {
        let mut price_scale = PriceScale::new("right", PriceScaleOptions::default());
        price_scale.set_height(500.0);
        price_scale.set_price_range(Some(super::PriceRange::new(100.0, 200.0)));
        let y = price_scale
            .price_to_coordinate(150.0, 150.0)
            .expect("price_to_coordinate");
        let p = price_scale
            .coordinate_to_price(y, 150.0)
            .expect("coordinate_to_price");
        assert!((p - 150.0).abs() <= 1e-9);
    }

    #[test]
    fn percentage_price_transform_round_trip() {
        let options = PriceScaleOptions {
            mode: PriceScaleMode::Percentage,
            ..Default::default()
        };
        let mut price_scale = PriceScale::new("right", options);
        price_scale.set_height(400.0);
        price_scale.set_price_range(Some(super::PriceRange::new(-10.0, 10.0)));
        let base = 100.0;
        let y = price_scale
            .price_to_coordinate(105.0, base)
            .expect("price_to_coordinate");
        let p = price_scale
            .coordinate_to_price(y, base)
            .expect("coordinate_to_price");
        assert!((p - 105.0).abs() <= 1e-6);
    }

    #[test]
    fn recalculate_price_range_with_no_sources_falls_back_to_default() {
        let mut price_scale = PriceScale::new("right", PriceScaleOptions::default());
        price_scale.recalculate_price_range(StrictRange::new(0, 10), &[]);
        let range = price_scale.price_range().expect("default range");
        assert!((range.min() + 0.5).abs() <= 1e-9);
        assert!((range.max() - 0.5).abs() <= 1e-9);
    }
}
