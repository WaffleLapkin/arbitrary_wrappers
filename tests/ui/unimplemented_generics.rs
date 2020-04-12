#![feature(arbitrary_self_types)]
use arbitrary_wrappers::use_ast;

// TODO: make the error cleaner

fn main() {
    assert_eq!(Wrapper(Type, 0).method(), 0);
}

struct Wrapper<T, U>(T, U);

struct Type;

#[use_ast]
impl<U> Wrapper<Type, U> {
    fn method(self) -> U {
        self.1
    }
}

impl<T, U> core::ops::Deref for Wrapper<T, U> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
