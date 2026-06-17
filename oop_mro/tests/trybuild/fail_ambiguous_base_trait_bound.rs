use oop_mro::prelude::*;

oop_class! {
    class Root {}
    class Left: Root {}
    class Right: Root {}
    class Diamond: Left, Right {}
}

fn require_root<T: AsClass<Root>>(_: &T) {}

fn main() {
    let diamond = Diamond::default();
    require_root(&diamond);
}
