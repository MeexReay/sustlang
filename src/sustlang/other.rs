pub trait Pohuy<T, E> {
    fn pohuy(&self) {}
}

impl<T, E> Pohuy<T, E> for Result<T, E> {}
