use serde::{Deserialize, Serialize};

use crate::error::{ChartError, ChartResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PaneId(u32);

impl PaneId {
    #[must_use]
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }

    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PaneDescriptor {
    pub id: PaneId,
    pub is_main: bool,
    pub stretch_factor: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PaneLayoutRegion {
    pub pane_id: PaneId,
    pub plot_top: f64,
    pub plot_bottom: f64,
}

impl PaneLayoutRegion {
    #[must_use]
    pub fn height(self) -> f64 {
        (self.plot_bottom - self.plot_top).max(0.0)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaneCollection {
    panes: Vec<PaneDescriptor>,
    next_id: u32,
}

impl Default for PaneCollection {
    fn default() -> Self {
        Self {
            panes: vec![PaneDescriptor {
                id: PaneId::new(0),
                is_main: true,
                stretch_factor: 1.0,
            }],
            next_id: 1,
        }
    }
}

impl PaneCollection {
    #[must_use]
    pub fn panes(&self) -> &[PaneDescriptor] {
        &self.panes
    }

    #[must_use]
    pub fn main_pane_id(&self) -> PaneId {
        // Invariant: `default()` always creates one main pane and removal
        // of main pane is forbidden.
        self.panes
            .iter()
            .find(|pane| pane.is_main)
            .map(|pane| pane.id)
            .unwrap_or(PaneId::new(0))
    }

    #[must_use]
    pub fn contains(&self, pane_id: PaneId) -> bool {
        self.panes.iter().any(|pane| pane.id == pane_id)
    }

    pub fn create_pane(&mut self, stretch_factor: f64) -> ChartResult<PaneId> {
        validate_stretch_factor(stretch_factor)?;
        let pane_id = PaneId::new(self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        self.panes.push(PaneDescriptor {
            id: pane_id,
            is_main: false,
            stretch_factor,
        });
        Ok(pane_id)
    }

    pub fn remove_pane(&mut self, pane_id: PaneId) -> ChartResult<bool> {
        if pane_id == self.main_pane_id() {
            return Err(ChartError::InvalidData(
                "cannot remove main pane".to_owned(),
            ));
        }

        let Some(index) = self.panes.iter().position(|pane| pane.id == pane_id) else {
            return Ok(false);
        };
        self.panes.remove(index);
        if self.panes.is_empty() {
            self.panes.push(PaneDescriptor {
                id: PaneId::new(0),
                is_main: true,
                stretch_factor: 1.0,
            });
            self.next_id = self.next_id.max(1);
        }
        Ok(true)
    }

    pub fn set_stretch_factor(
        &mut self,
        pane_id: PaneId,
        stretch_factor: f64,
    ) -> ChartResult<bool> {
        validate_stretch_factor(stretch_factor)?;
        let Some(pane) = self.panes.iter_mut().find(|pane| pane.id == pane_id) else {
            return Ok(false);
        };
        pane.stretch_factor = stretch_factor;
        Ok(true)
    }

    pub fn normalize_stretch_factors(&mut self) {
        let sum: f64 = self
            .panes
            .iter()
            .filter_map(|pane| {
                if pane.stretch_factor.is_finite() && pane.stretch_factor > 0.0 {
                    Some(pane.stretch_factor)
                } else {
                    None
                }
            })
            .sum();

        if !sum.is_finite() || sum <= 0.0 {
            let equal = 1.0 / (self.panes.len().max(1) as f64);
            for pane in &mut self.panes {
                pane.stretch_factor = equal;
            }
            return;
        }

        for pane in &mut self.panes {
            pane.stretch_factor = if pane.stretch_factor.is_finite() && pane.stretch_factor > 0.0 {
                pane.stretch_factor / sum
            } else {
                0.0
            };
        }
    }

    #[must_use]
    pub fn layout_regions(&self, plot_top: f64, plot_bottom: f64) -> Vec<PaneLayoutRegion> {
        if self.panes.is_empty() {
            return Vec::new();
        }

        let safe_top = if plot_top.is_finite() {
            plot_top.max(0.0)
        } else {
            0.0
        };
        let safe_bottom = if plot_bottom.is_finite() {
            plot_bottom.max(safe_top)
        } else {
            safe_top
        };
        let total_height = (safe_bottom - safe_top).max(0.0);
        if total_height <= 0.0 {
            return self
                .panes
                .iter()
                .map(|pane| PaneLayoutRegion {
                    pane_id: pane.id,
                    plot_top: safe_top,
                    plot_bottom: safe_top,
                })
                .collect();
        }

        let mut weights: Vec<f64> = self
            .panes
            .iter()
            .map(|pane| {
                if pane.stretch_factor.is_finite() && pane.stretch_factor > 0.0 {
                    pane.stretch_factor
                } else {
                    0.0
                }
            })
            .collect();
        let weight_sum: f64 = weights.iter().sum();
        if !weight_sum.is_finite() || weight_sum <= 0.0 {
            let equal = 1.0 / (self.panes.len() as f64);
            weights.fill(equal);
        } else {
            for weight in &mut weights {
                *weight /= weight_sum;
            }
        }

        let mut regions = Vec::with_capacity(self.panes.len());
        let mut cursor = safe_top;
        let last_index = self.panes.len().saturating_sub(1);
        for (index, pane) in self.panes.iter().enumerate() {
            let next_bottom = if index == last_index {
                safe_bottom
            } else {
                (cursor + total_height * weights[index]).clamp(cursor, safe_bottom)
            };
            regions.push(PaneLayoutRegion {
                pane_id: pane.id,
                plot_top: cursor,
                plot_bottom: next_bottom,
            });
            cursor = next_bottom;
        }
        regions
    }
}

fn validate_stretch_factor(stretch_factor: f64) -> ChartResult<()> {
    if !stretch_factor.is_finite() || stretch_factor <= 0.0 {
        return Err(ChartError::InvalidData(
            "pane stretch factor must be finite and > 0".to_owned(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{PaneCollection, PaneId};

    #[test]
    fn pane_collection_default_has_single_main_pane() {
        let panes = PaneCollection::default();
        assert_eq!(panes.panes().len(), 1);
        assert!(panes.panes()[0].is_main);
        assert_eq!(panes.main_pane_id(), PaneId::new(0));
    }

    #[test]
    fn pane_collection_can_create_remove_and_normalize() {
        let mut panes = PaneCollection::default();
        let pane_a = panes.create_pane(2.0).expect("create pane A");
        let pane_b = panes.create_pane(1.0).expect("create pane B");
        assert_eq!(panes.panes().len(), 3);

        panes.normalize_stretch_factors();
        let sum: f64 = panes.panes().iter().map(|pane| pane.stretch_factor).sum();
        assert!((sum - 1.0).abs() <= 1e-12);

        let removed = panes.remove_pane(pane_a).expect("remove pane A");
        assert!(removed);
        assert!(panes.contains(pane_b));
        assert_eq!(panes.panes().len(), 2);
    }

    #[test]
    fn pane_collection_layout_regions_split_plot_area_by_weights() {
        let mut panes = PaneCollection::default();
        let _ = panes.create_pane(1.0).expect("pane A");
        let _ = panes.create_pane(2.0).expect("pane B");
        let regions = panes.layout_regions(0.0, 300.0);
        assert_eq!(regions.len(), 3);
        assert!((regions[0].height() - 75.0).abs() <= 1e-9);
        assert!((regions[1].height() - 75.0).abs() <= 1e-9);
        assert!((regions[2].height() - 150.0).abs() <= 1e-9);
        assert!((regions[2].plot_bottom - 300.0).abs() <= 1e-9);
    }
}
