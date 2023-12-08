/// This trait represents a discovery strategy for the platform
pub trait Discovery {
    /// Discovers the neighbours of the device
    ///
    /// # Returns
    /// A vector containing the ids of the neighbours
    fn discover_neighbors(&self) -> Vec<i32>;
}