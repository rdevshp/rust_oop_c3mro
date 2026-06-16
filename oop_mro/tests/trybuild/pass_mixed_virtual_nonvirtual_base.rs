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
    }

    class Left: virtual Root {
        constructor(): Root(1) {}
    }

    class Right: Root {
        constructor(): Root(2) {}
    }

    class Diamond: Left, Right {
        constructor(): Root(3), Left(), Right() {}
    }
}

fn main() {
    let mut diamond = Diamond::new();

    assert_eq!(diamond.as_root().value(), 3);
    assert_eq!(diamond.as_left().as_root().value(), 3);
    assert_eq!(diamond.as_right().as_root().value(), 2);
    assert_ne!(
        diamond.as_left().as_root() as *const Root,
        diamond.as_right().as_root() as *const Root,
    );

    diamond.as_right_mut().as_root_mut().set_value(4);
    assert_eq!(diamond.as_left().as_root().value(), 3);
    assert_eq!(diamond.as_right().as_root().value(), 4);
}
