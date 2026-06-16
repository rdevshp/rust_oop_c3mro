use oop_mro::prelude::*;

oop_class! {
    class Root {
        value: usize,

        constructor(value: usize) {
            self.value = value;
        }

        fn value(&self) -> usize {
            self.value
        }

        fn set_value(&mut self, value: usize) {
            self.value = value;
        }

        virtual fn dispatched(&self) -> usize {
            self.value
        }
    }

    class Left: virtual Root {
        constructor(): Root(1) {}
    }

    class Right: virtual Root {
        constructor(): Root(2) {}
    }

    class Diamond: Left, Right {
        constructor(): Root(3), Left(), Right() {}

        #[override]
        virtual fn dispatched(&self) -> usize {
            self.as_root().value() + 10
        }
    }
}

fn main() {
    let mut diamond = Diamond::new();

    assert!(core::ptr::eq(
        diamond.as_left().as_root(),
        diamond.as_right().as_root(),
    ));
    assert_eq!(diamond.as_root().value(), 3);
    diamond.as_right_mut().as_root_mut().set_value(4);
    assert_eq!(diamond.as_left().as_root().value(), 4);
    assert_eq!(diamond.as_root().dispatched(), 14);
}
