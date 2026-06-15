use oop_mro::prelude::*;

oop_class! {
    abstract class Animal {
        abstract virtual const fn speak(&self) -> usize;
    }
}

fn main() {}
