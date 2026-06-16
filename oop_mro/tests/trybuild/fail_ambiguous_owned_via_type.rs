use oop_mro::prelude::*;

oop_class! {
    class Root {}
    class Left: Root {}
    class Right: Root {}
    class Branch: Left, Right {}
    class Diamond: Branch {}
}

fn main() {
    let diamond: Box<dyn AsDiamond> = Box::new(Diamond::default());
    let _ = diamond.into_base_via::<Branch, dyn AsRoot>();
}
