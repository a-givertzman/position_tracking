use std::marker::PhantomData;

use super::filter::Filter;
///
/// Just passes the each value without filtering
#[derive(Debug, Clone)]
pub struct FilterEmpty<T> {
    phantom: PhantomData<T>, 
}
//
// 
impl<T: Clone + std::fmt::Debug> FilterEmpty<T> {
    ///
    /// Creates new [FilterEmpty]
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}
//
//
impl Filter for FilterEmpty<i32> {
    type Item = i32;
    fn add(&mut self, value: Self::Item) -> Option<Self::Item> {
        Some(value)
    }
}
impl Filter for FilterEmpty<u16> {
    type Item = u16;
    fn add(&mut self, value: Self::Item) -> Option<Self::Item> {
        Some(value)
    }
}
