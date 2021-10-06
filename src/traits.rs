/*
todo!()
//trait to give a more dsl feel
pub trait KotlinUtils: Sized {
    #[inline]
    fn also<F: Fn(&Self)>(self, f: F) -> Self {
        f(&self);
        self
    }
    #[inline]
    fn run<F: Fn(&Self) -> R, R>(&self, block: F) -> R {
        block(self)
    }

    #[inline]
    fn run_owned<F: Fn(Self) -> R, R>(self, block: F) -> R {
        block(self)
    }

    #[inline]
    fn also_mut<F: Fn(&mut Self)>(mut self, f: F) -> Self {
        f(&mut self);
        self
    }
}

impl<T> KotlinUtils for T {}*/