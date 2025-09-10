///
/// Holds single value
/// - call add(value) to apply new value
/// - pop current value by calling value()
/// - is_changed() - check if value was changed after las add()
pub trait Filter: std::fmt::Debug {
    type Item;
    ///
    /// - Updates state with value if value != inner
    fn add(&mut self, value: Self::Item) -> Option<Self::Item>;
}
