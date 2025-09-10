use super::filter::Filter;
///
/// Smoothing input  
/// 
/// Calculates the new value according to the following formula:
/// 
/// val = old_calculated_val = (val - old_calculated_val) / k
#[derive(Debug, Clone)]
pub struct FilterSmooth<T> {
    prev: Option<T>,
    factor: f64,
    factor_inv: f64,
}
//
// 
impl<T: Copy> FilterSmooth<T> {
    ///
    /// Creates new FilterSmooth<const N: usize, T>
    /// - `T` - Type of the Filter Item
    pub fn new(initial: Option<T>, factor: f64) -> Self {
        Self {
            prev: initial,
            factor,
            factor_inv: 1.0 / factor,
        }
    }
}
//
//
impl Filter for FilterSmooth<i32> {
    type Item = i32;
    //
    //
    fn add(&mut self, value: Self::Item) -> Option<Self::Item> {
        match self.prev {
            Some(prev) => {
                let value = (prev as f64 + ((value as f64) - (prev as f64)) * self.factor_inv).round() as i32;
                self.prev.replace(value);
                Some(value)
            }
            None => {
                self.prev.replace(value);
                Some(value)
            }
        }
    }
}
impl Filter for FilterSmooth<u16> {
    type Item = u16;
    //
    //
    fn add(&mut self, value: Self::Item) -> Option<Self::Item> {
        match self.prev {
            Some(prev) => {
                let value = (prev as f64 + ((value as f64) - (prev as f64)) * self.factor_inv).round() as u16;
                self.prev.replace(value);
                Some(value)
            }
            None => {
                self.prev.replace(value);
                Some(value)
            }
        }
    }
}
