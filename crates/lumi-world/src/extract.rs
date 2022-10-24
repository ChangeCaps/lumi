pub trait Extract<T> {
    fn extract(&self, extract: &mut dyn FnMut(&T));
}

impl<T> Extract<T> for T {
    #[inline]
    fn extract(&self, extract: &mut dyn FnMut(&T)) {
        extract(self);
    }
}

impl<T> Extract<T> for [T] {
    #[inline]
    fn extract(&self, extract: &mut dyn FnMut(&T)) {
        for item in self {
            extract(item);
        }
    }
}
