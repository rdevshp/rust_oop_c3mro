use oop_mro::prelude::*;

oop_class! {
    class Animal {
        virtual const fn speak(&self) -> usize {
            1
        }
    }
}

fn main() {}
