use crate::core::{DataPoint, PriceScale, TimeScale, Viewport};
use crate::error::ChartResult;
use serde::{Deserialize, Serialize};

/// Vertex in pixel coordinates used by deterministic area geometry output.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AreaVertex {
    pub x: f64,
    pub y: f64,
}

/// Deterministic geometry for an area series.
///
/// `line_points` follows the mapped data points.
/// `fill_polygon` is an explicitly closed polygon against the baseline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AreaGeometry {
    pub line_points: Vec<AreaVertex>,
    pub fill_polygon: Vec<AreaVertex>,
}

impl AreaGeometry {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            line_points: Vec::new(),
            fill_polygon: Vec::new(),
        }
    }
}

/// Projects points into deterministic area-series geometry.
///
/// Baseline is anchored at the viewport bottom (`viewport.height`) to model the
/// standard area-fill behavior for this baseline parity stage.
pub fn project_area_geometry(
    points: &[DataPoint],
    time_scale: TimeScale,
    price_scale: PriceScale,
    viewport: Viewport,
) -> ChartResult<AreaGeometry> {
    if points.is_empty() {
        return Ok(AreaGeometry::empty());
    }

    let mut line_points = Vec::with_capacity(points.len());
    for point in points {
        let x = time_scale.time_to_pixel(point.x, viewport)?;
        let y = price_scale.price_to_pixel(point.y, viewport)?;
        line_points.push(AreaVertex { x, y });
    }

    let baseline_y = f64::from(viewport.height);
    let first_x = line_points[0].x;
    let last_x = line_points[line_points.len() - 1].x;

    let mut fill_polygon = Vec::with_capacity(line_points.len() + 3);
    fill_polygon.push(AreaVertex {
        x: first_x,
        y: baseline_y,
    });
    fill_polygon.extend(line_points.iter().copied());
    fill_polygon.push(AreaVertex {
        x: last_x,
        y: baseline_y,
    });
    // Explicitly repeat the first baseline vertex so consumers can render this
    // as a closed polygon without adding implicit closure rules.
    fill_polygon.push(AreaVertex {
        x: first_x,
        y: baseline_y,
    });

    Ok(AreaGeometry {
        line_points,
        fill_polygon,
    })
}
