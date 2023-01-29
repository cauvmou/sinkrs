pub trait Serialize<T> : Into<T> {
    fn serialize(self) -> T {
        self.into()
    }
}