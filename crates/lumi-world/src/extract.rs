use lumi_assets::{Asset, Handle};

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

impl<T, U> Extract<U> for Handle<T>
where
    T: Extract<U> + Asset,
{
    #[inline]
    fn extract(&self, extract: &mut dyn FnMut(&U)) {
        if let Some(asset) = self.get() {
            asset.extract(extract);
        }
    }

    #[inline]
    fn extract_enumerated(&self, extract: &mut dyn FnMut(usize, &U)) {
        if let Some(asset) = self.get() {
            asset.extract_enumerated(extract);
        }
    }
}

impl<T, U> ExtractOne<U> for Handle<T>
where
    T: ExtractOne<U> + Asset,
{
    #[inline]
    fn extract_one(&self) -> Option<&U> {
        self.get()?.extract_one()
    }
}
