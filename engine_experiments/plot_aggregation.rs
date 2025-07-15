//! Plot data aggregation for efficient rendering of large datasets.
//! Inspired by Rerun's sub-pixel aggregation techniques.

use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Result;

/// Aggregation level for plot data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AggregationLevel {
    /// No aggregation, show all points
    Full,
    /// Aggregate to pixel level
    Pixel,
    /// Aggregate to larger chunks (for overview)
    Coarse,
}

/// Aggregated data point for time series
#[derive(Debug, Clone)]
pub struct AggregatedPoint {
    /// X value (e.g., time)
    pub x: f64,
    /// Minimum Y value in this bucket
    pub y_min: f64,
    /// Maximum Y value in this bucket
    pub y_max: f64,
    /// Average Y value in this bucket
    pub y_avg: f64,
    /// Number of points aggregated
    pub count: usize,
}

/// Viewport information for aggregation
#[derive(Debug, Clone)]
pub struct Viewport {
    /// Minimum X value visible
    pub x_min: f64,
    /// Maximum X value visible
    pub x_max: f64,
    /// Width in pixels
    pub width_pixels: u32,
}

/// Cache for aggregated plot data at different levels
pub struct PlotDataCache {
    /// Original data points
    original_data: Vec<(f64, f64)>,
    /// Cached aggregations at different levels
    aggregations: HashMap<(AggregationLevel, u32), Arc<Vec<AggregatedPoint>>>,
    /// Whether cache needs refresh
    dirty: bool,
}

impl PlotDataCache {
    /// Create a new plot data cache
    pub fn new(data: Vec<(f64, f64)>) -> Self {
        Self {
            original_data: data,
            aggregations: HashMap::new(),
            dirty: false,
        }
    }
    
    /// Update the underlying data
    pub fn update_data(&mut self, data: Vec<(f64, f64)>) {
        self.original_data = data;
        self.aggregations.clear();
        self.dirty = true;
    }
    
    /// Get aggregated data for a given viewport
    pub fn get_aggregated(&mut self, viewport: &Viewport) -> Arc<Vec<AggregatedPoint>> {
        let level = self.determine_aggregation_level(viewport);
        let key = (level, viewport.width_pixels);
        
        if let Some(cached) = self.aggregations.get(&key) {
            return cached.clone();
        }
        
        // Compute aggregation
        let aggregated = match level {
            AggregationLevel::Full => self.no_aggregation(),
            AggregationLevel::Pixel => self.pixel_aggregation(viewport),
            AggregationLevel::Coarse => self.coarse_aggregation(viewport),
        };
        
        let aggregated = Arc::new(aggregated);
        self.aggregations.insert(key, aggregated.clone());
        aggregated
    }
    
    /// Determine appropriate aggregation level based on data density
    fn determine_aggregation_level(&self, viewport: &Viewport) -> AggregationLevel {
        // Filter points in viewport
        let points_in_view = self.original_data.iter()
            .filter(|(x, _)| *x >= viewport.x_min && *x <= viewport.x_max)
            .count();
        
        let points_per_pixel = points_in_view as f64 / viewport.width_pixels as f64;
        
        if points_per_pixel < 2.0 {
            AggregationLevel::Full
        } else if points_per_pixel < 100.0 {
            AggregationLevel::Pixel
        } else {
            AggregationLevel::Coarse
        }
    }
    
    /// No aggregation - convert all points
    fn no_aggregation(&self) -> Vec<AggregatedPoint> {
        self.original_data.iter()
            .map(|(x, y)| AggregatedPoint {
                x: *x,
                y_min: *y,
                y_max: *y,
                y_avg: *y,
                count: 1,
            })
            .collect()
    }
    
    /// Aggregate to pixel level
    fn pixel_aggregation(&self, viewport: &Viewport) -> Vec<AggregatedPoint> {
        if self.original_data.is_empty() {
            return Vec::new();
        }
        
        let x_range = viewport.x_max - viewport.x_min;
        let bucket_size = x_range / viewport.width_pixels as f64;
        
        let mut buckets: HashMap<i32, Vec<f64>> = HashMap::new();
        
        // Group points into buckets
        for (x, y) in &self.original_data {
            if *x >= viewport.x_min && *x <= viewport.x_max {
                let bucket_idx = ((x - viewport.x_min) / bucket_size) as i32;
                buckets.entry(bucket_idx).or_insert_with(Vec::new).push(*y);
            }
        }
        
        // Aggregate each bucket
        let mut result: Vec<AggregatedPoint> = buckets.into_iter()
            .map(|(idx, values)| {
                let x = viewport.x_min + (idx as f64 + 0.5) * bucket_size;
                let y_min = values.iter().cloned().fold(f64::INFINITY, f64::min);
                let y_max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let y_avg = values.iter().sum::<f64>() / values.len() as f64;
                
                AggregatedPoint {
                    x,
                    y_min,
                    y_max,
                    y_avg,
                    count: values.len(),
                }
            })
            .collect();
        
        result.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
        result
    }
    
    /// Coarse aggregation for overview
    fn coarse_aggregation(&self, viewport: &Viewport) -> Vec<AggregatedPoint> {
        // Use larger buckets for coarse view
        let mut viewport_coarse = viewport.clone();
        viewport_coarse.width_pixels = (viewport.width_pixels / 10).max(100);
        self.pixel_aggregation(&viewport_coarse)
    }
    
    /// Get statistics about the cache
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            original_points: self.original_data.len(),
            cached_levels: self.aggregations.len(),
            is_dirty: self.dirty,
        }
    }
}

/// Statistics about the plot cache
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Number of original data points
    pub original_points: usize,
    /// Number of cached aggregation levels
    pub cached_levels: usize,
    /// Whether cache needs refresh
    pub is_dirty: bool,
}

/// Helper function to aggregate data for time series plots
pub fn aggregate_time_series(
    data: &[(f64, f64)],
    viewport: &Viewport,
) -> Vec<AggregatedPoint> {
    let mut cache = PlotDataCache::new(data.to_vec());
    Arc::try_unwrap(cache.get_aggregated(viewport))
        .unwrap_or_else(|arc| (*arc).clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_no_aggregation() {
        let data = vec![(1.0, 10.0), (2.0, 20.0), (3.0, 30.0)];
        let mut cache = PlotDataCache::new(data);
        
        let viewport = Viewport {
            x_min: 0.0,
            x_max: 5.0,
            width_pixels: 1000,
        };
        
        let aggregated = cache.get_aggregated(&viewport);
        assert_eq!(aggregated.len(), 3);
        assert_eq!(aggregated[0].count, 1);
    }
    
    #[test]
    fn test_pixel_aggregation() {
        // Create dense data
        let data: Vec<(f64, f64)> = (0..1000)
            .map(|i| (i as f64 / 100.0, (i as f64).sin()))
            .collect();
        
        let mut cache = PlotDataCache::new(data);
        
        let viewport = Viewport {
            x_min: 0.0,
            x_max: 10.0,
            width_pixels: 100,
        };
        
        let aggregated = cache.get_aggregated(&viewport);
        // Should have roughly 100 points (one per pixel)
        assert!(aggregated.len() <= 100);
        
        // Check that aggregation preserves min/max
        let first = &aggregated[0];
        assert!(first.y_min <= first.y_avg);
        assert!(first.y_avg <= first.y_max);
        assert!(first.count > 1);
    }
} 