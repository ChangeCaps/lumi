pub trait Extract<T> {
    fn extract(&self, extract: &mut dyn FnMut(&T));

    #[inline]
    fn extract_enumerated(&self, extract: &mut dyn FnMut(usize, &T)) {
        let mut i = 0;
        self.extract(&mut |t| {
            extract(i, t);
            i += 1;
        });
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

pub trait ExtractOne<T> {
    fn extract_one(&self) -> Option<&T>;
}
