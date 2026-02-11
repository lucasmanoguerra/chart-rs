use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::core::{OhlcBar, PriceScale, TimeScale, Viewport};
use crate::error::{ChartError, ChartResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarkerSide {
    Above,
    Below,
    Center,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MarkerPosition {
    AboveBar,
    BelowBar,
    InBar,
    Price(f64),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeriesMarker {
    pub id: String,
    pub time: f64,
    pub position: MarkerPosition,
    pub text: Option<String>,
    pub priority: i32,
}

impl SeriesMarker {
    #[must_use]
    pub fn new(id: impl Into<String>, time: f64, position: MarkerPosition) -> Self {
        Self {
            id: id.into(),
            time,
            position,
            text: None,
            priority: 0,
        }
    }

    #[must_use]
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    #[must_use]
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MarkerPlacementConfig {
    pub marker_size_px: f64,
    pub label_char_width_px: f64,
    pub label_height_px: f64,
    pub label_horizontal_padding_px: f64,
    pub marker_label_gap_px: f64,
    pub lane_gap_px: f64,
    pub min_horizontal_gap_px: f64,
    pub vertical_offset_px: f64,
}

impl Default for MarkerPlacementConfig {
    fn default() -> Self {
        Self {
            marker_size_px: 8.0,
            label_char_width_px: 7.0,
            label_height_px: 14.0,
            label_horizontal_padding_px: 6.0,
            marker_label_gap_px: 4.0,
            lane_gap_px: 4.0,
            min_horizontal_gap_px: 2.0,
            vertical_offset_px: 6.0,
        }
    }
}

impl MarkerPlacementConfig {
    fn validate(self) -> ChartResult<Self> {
        for (value, name) in [
            (self.marker_size_px, "marker_size_px"),
            (self.label_char_width_px, "label_char_width_px"),
            (self.label_height_px, "label_height_px"),
            (
                self.label_horizontal_padding_px,
                "label_horizontal_padding_px",
            ),
            (self.marker_label_gap_px, "marker_label_gap_px"),
            (self.lane_gap_px, "lane_gap_px"),
            (self.min_horizontal_gap_px, "min_horizontal_gap_px"),
            (self.vertical_offset_px, "vertical_offset_px"),
        ] {
            if !value.is_finite() || value <= 0.0 {
                return Err(ChartError::InvalidData(format!(
                    "marker config `{name}` must be finite and > 0"
                )));
            }
        }
        Ok(self)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarkerLabelGeometry {
    pub text: String,
    pub left_px: f64,
    pub top_px: f64,
    pub width_px: f64,
    pub height_px: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlacedMarker {
    pub id: String,
    pub time: f64,
    pub price: f64,
    pub side: MarkerSide,
    pub lane: usize,
    pub x: f64,
    pub y: f64,
    pub label: Option<MarkerLabelGeometry>,
    pub collision_left_px: f64,
    pub collision_right_px: f64,
}

/// Places markers relative to candle anchors with deterministic collision rules.
///
/// Placement order is stable by logical x, priority (desc), then marker id.
pub fn place_markers_on_candles(
    markers: &[SeriesMarker],
    candles: &[OhlcBar],
    time_scale: TimeScale,
    price_scale: PriceScale,
    viewport: Viewport,
    config: MarkerPlacementConfig,
) -> ChartResult<Vec<PlacedMarker>> {
    let config = config.validate()?;
    if markers.is_empty() {
        return Ok(Vec::new());
    }

    let mut prepared = Vec::with_capacity(markers.len());
    for (index, marker) in markers.iter().enumerate() {
        if !marker.time.is_finite() {
            return Err(ChartError::InvalidData(
                "marker time must be finite".to_owned(),
            ));
        }

        let side = side_for_position(marker.position);
        let price = resolve_marker_price(marker, candles)?;
        let x_raw = time_scale.time_to_pixel(marker.time, viewport)?;
        let label_width = marker_label_width(marker.text.as_deref(), config);
        let span_half = 0.5 * config.marker_size_px.max(label_width.unwrap_or(0.0));
        let x = clamp_x(x_raw, span_half, f64::from(viewport.width));
        let left = x - span_half;
        let right = x + span_half;

        prepared.push(PreparedMarker {
            index,
            marker,
            side,
            price,
            x,
            left,
            right,
        });
    }

    prepared.sort_by(|a, b| {
        OrderedFloat(a.x)
            .cmp(&OrderedFloat(b.x))
            .then_with(|| b.marker.priority.cmp(&a.marker.priority))
            .then_with(|| a.marker.id.cmp(&b.marker.id))
            .then_with(|| a.index.cmp(&b.index))
    });

    let lane_step = config.marker_size_px
        + config.marker_label_gap_px
        + config.label_height_px
        + config.lane_gap_px;

    let mut above_lane_last_right = Vec::<f64>::new();
    let mut below_lane_last_right = Vec::<f64>::new();
    let mut center_lane_last_right = Vec::<f64>::new();
    let mut placed = Vec::with_capacity(prepared.len());

    for item in prepared {
        let lane_last_right = match item.side {
            MarkerSide::Above => &mut above_lane_last_right,
            MarkerSide::Below => &mut below_lane_last_right,
            MarkerSide::Center => &mut center_lane_last_right,
        };
        let lane = allocate_lane(
            lane_last_right,
            item.left,
            item.right,
            config.min_horizontal_gap_px,
        );

        let base_y = price_scale.price_to_pixel(item.price, viewport)?;
        let lane_offset = lane as f64 * lane_step;
        let y = match item.side {
            MarkerSide::Above => base_y - config.vertical_offset_px - lane_offset,
            MarkerSide::Below => base_y + config.vertical_offset_px + lane_offset,
            MarkerSide::Center => base_y + lane_offset,
        };

        let label = build_label_geometry(item.marker.text.as_deref(), item.x, y, item.side, config);
        placed.push(PlacedMarker {
            id: item.marker.id.clone(),
            time: item.marker.time,
            price: item.price,
            side: item.side,
            lane,
            x: item.x,
            y,
            label,
            collision_left_px: item.left,
            collision_right_px: item.right,
        });
    }

    Ok(placed)
}

#[derive(Debug)]
struct PreparedMarker<'a> {
    index: usize,
    marker: &'a SeriesMarker,
    side: MarkerSide,
    price: f64,
    x: f64,
    left: f64,
    right: f64,
}

fn side_for_position(position: MarkerPosition) -> MarkerSide {
    match position {
        MarkerPosition::AboveBar => MarkerSide::Above,
        MarkerPosition::BelowBar => MarkerSide::Below,
        MarkerPosition::InBar | MarkerPosition::Price(_) => MarkerSide::Center,
    }
}

fn resolve_marker_price(marker: &SeriesMarker, candles: &[OhlcBar]) -> ChartResult<f64> {
    match marker.position {
        MarkerPosition::Price(price) => {
            if !price.is_finite() {
                return Err(ChartError::InvalidData(
                    "marker price must be finite".to_owned(),
                ));
            }
            Ok(price)
        }
        MarkerPosition::AboveBar => nearest_candle(candles, marker.time)
            .map(|c| c.high)
            .ok_or_else(|| ChartError::InvalidData("marker requires candle anchors".to_owned())),
        MarkerPosition::BelowBar => nearest_candle(candles, marker.time)
            .map(|c| c.low)
            .ok_or_else(|| ChartError::InvalidData("marker requires candle anchors".to_owned())),
        MarkerPosition::InBar => nearest_candle(candles, marker.time)
            .map(|c| c.close)
            .ok_or_else(|| ChartError::InvalidData("marker requires candle anchors".to_owned())),
    }
}

fn nearest_candle(candles: &[OhlcBar], time: f64) -> Option<OhlcBar> {
    candles
        .iter()
        .copied()
        .min_by_key(|bar| OrderedFloat((bar.time - time).abs()))
}

fn marker_label_width(text: Option<&str>, config: MarkerPlacementConfig) -> Option<f64> {
    text.map(|value| {
        value.chars().count() as f64 * config.label_char_width_px
            + 2.0 * config.label_horizontal_padding_px
    })
}

fn clamp_x(x: f64, span_half: f64, viewport_width: f64) -> f64 {
    if viewport_width <= 2.0 * span_half {
        viewport_width * 0.5
    } else {
        x.clamp(span_half, viewport_width - span_half)
    }
}

fn allocate_lane(last_right: &mut Vec<f64>, left: f64, right: f64, min_gap: f64) -> usize {
    for (lane, lane_last_right) in last_right.iter_mut().enumerate() {
        if left >= *lane_last_right + min_gap {
            *lane_last_right = right;
            return lane;
        }
    }
    last_right.push(right);
    last_right.len() - 1
}

fn build_label_geometry(
    text: Option<&str>,
    x: f64,
    y: f64,
    side: MarkerSide,
    config: MarkerPlacementConfig,
) -> Option<MarkerLabelGeometry> {
    let text = text?.to_owned();
    let width = marker_label_width(Some(text.as_str()), config)?;
    let top = match side {
        MarkerSide::Above => {
            y - 0.5 * config.marker_size_px - config.marker_label_gap_px - config.label_height_px
        }
        MarkerSide::Below => y + 0.5 * config.marker_size_px + config.marker_label_gap_px,
        MarkerSide::Center => y + 0.5 * config.marker_size_px + config.marker_label_gap_px,
    };

    Some(MarkerLabelGeometry {
        text,
        left_px: x - 0.5 * width,
        top_px: top,
        width_px: width,
        height_px: config.label_height_px,
    })
}
