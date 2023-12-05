pub trait Discovery {
    fn discover_neighbors(&self) -> Vec<i32>;
}