#![feature(arbitrary_self_types)]
use arbitrary_wrappers::use_ast;

fn main() {
    let mut wrap = Wrapper(0, Type);

    (&mut wrap).mut_();
    (&wrap).ref_();
    assert_eq!(wrap.method(), 0);
}

struct Wrapper<T>(i32, T);

struct Type;

#[use_ast(Type)]
impl Wrapper<Type> {
    fn method(self) -> i32 {
        self.0
    }

    fn mut_(&mut self) {}

    fn ref_(&self) {}
}

impl<T> core::ops::Deref for Wrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}
