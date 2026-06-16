use oop_mro::prelude::*;

oop_class! {
    class Slot<T> {
        value: Option<T> = None,
    }
    class Left: Slot<i32> {}
    class Right: Slot<i32> {}
    class Diamond: Left, Right {}
}

fn main() {
    let diamond = Diamond::default();
    let _ = <Diamond as AsSlot<i32>>::as_slot(&diamond);
}
