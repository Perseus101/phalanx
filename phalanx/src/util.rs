pub trait AsyncTryFrom<T>: Sized {
    /// The type returned in the event of a conversion error.
    type Error;

    /// The future type
    type Future: std::future::Future<Output = Result<Self, Self::Error>>;

    /// Performs the conversion.
    fn try_from(value: T) -> Self::Future;
}
