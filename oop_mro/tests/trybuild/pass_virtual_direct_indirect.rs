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

    class Branch: virtual Root {
        constructor(): Root(1) {}
    }

    class Diamond: virtual Root, Branch {
        constructor(): Root(5), Branch() {}

        #[override]
        virtual fn dispatched(&self) -> usize {
            self.as_root().value() + 20
        }
    }
}

fn main() {
    let mut diamond = Diamond::new();

    assert!(core::ptr::eq(
        diamond.as_root(),
        diamond.as_branch().as_root(),
    ));
    assert_eq!(diamond.as_root().value(), 5);
    assert_eq!(diamond.as_root().dispatched(), 25);

    diamond.as_branch_mut().as_root_mut().set_value(9);
    assert_eq!(diamond.as_root().value(), 9);
    assert_eq!(diamond.as_root().dispatched(), 29);
}
