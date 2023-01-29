pub trait Serialize<T> where Self: Into<T>{
    #[inline]
    fn serialize(self) -> T {
        self.into()
    }
}