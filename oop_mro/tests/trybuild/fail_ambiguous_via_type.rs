use oop_mro::prelude::*;

oop_class! {
    class Root {}
    class Left: Root {}
    class Right: Root {}
    class Branch: Left, Right {}
    class Diamond: Branch {}
}

fn main() {
    let diamond = Diamond::default();
    let _ = diamond.as_base_via::<Branch, Root>();
}
