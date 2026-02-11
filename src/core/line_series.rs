use crate::core::{DataPoint, PriceScale, TimeScale, Viewport};
use crate::error::ChartResult;
use serde::{Deserialize, Serialize};

/// Projected line segment in pixel coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LineSegment {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
}

/// Projects line-series points into adjacent line segments.
///
/// The function is deterministic and side-effect free so both rendering and
/// tests can consume the exact same geometry output.
pub fn project_line_segments(
    points: &[DataPoint],
    time_scale: TimeScale,
    price_scale: PriceScale,
    viewport: Viewport,
) -> ChartResult<Vec<LineSegment>> {
    if points.len() < 2 {
        return Ok(Vec::new());
    }

    let mut mapped = Vec::with_capacity(points.len());
    for point in points {
        let x = time_scale.time_to_pixel(point.x, viewport)?;
        let y = price_scale.price_to_pixel(point.y, viewport)?;
        mapped.push((x, y));
    }

    let mut segments = Vec::with_capacity(mapped.len() - 1);
    for pair in mapped.windows(2) {
        segments.push(LineSegment {
            x1: pair[0].0,
            y1: pair[0].1,
            x2: pair[1].0,
            y2: pair[1].1,
        });
    }

    Ok(segments)
}
