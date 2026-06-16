use oop_mro::prelude::*;

oop_class! {
    class Root {}
    class Left: virtual Root {}
    class Right: Root {}
    class Diamond: Left, Right {}
}

fn main() {}
