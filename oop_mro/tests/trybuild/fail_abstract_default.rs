use oop_mro::prelude::*;

oop_class! {
    abstract class AbstractShape {
        abstract virtual fn area(&self) -> usize;
    }

    abstract class StillAbstract: AbstractShape {}
}

#[diagnostic::on_unimplemented(message = "abstract classes must not implement Default")]
trait MustBeDefault {}

impl<T: Default> MustBeDefault for T {}

fn require_default<T: MustBeDefault>() {}

fn main() {
    require_default::<StillAbstract>();
}
