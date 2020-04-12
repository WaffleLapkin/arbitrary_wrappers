#![feature(arbitrary_self_types)]

// TODO: make the error cleaner

use arbitrary_wrappers::use_ast;

fn main() {
    assert_eq!(Wrapper(0, Type).method(), 0);
}

struct Wrapper<T>(i32, T);

struct Type;

#[use_ast]
impl Wrapper<Type> {
    const XXX: usize = 0;

    fn method(self) -> i32 {
        self.0
    }
}

impl<T> core::ops::Deref for Wrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}
