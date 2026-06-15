use oop_mro::prelude::*;

oop_class! {
    class ConstBox<T: Default> {
        value: T,

        const fn get(&self) -> &T {
            &self.value
        }

        const fn passthrough<U>(&self, value: U) -> U {
            value
        }

        const unsafe fn unchecked(&self) -> &T {
            &self.value
        }
    }
}

const BOXED: ConstBox<usize> = ConstBox {
    value: 7,
};
const VALUE: &usize = BOXED.get();
const PASSTHROUGH: usize = BOXED.passthrough(11);
const UNSAFE_VALUE: &usize = unsafe { BOXED.unchecked() };

fn main() {
    assert_eq!(*VALUE, 7);
    assert_eq!(PASSTHROUGH, 11);
    assert_eq!(*UNSAFE_VALUE, 7);
}
