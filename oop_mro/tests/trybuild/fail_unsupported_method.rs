use oop_mro::prelude::*;

oop_class! {
    class Animal {
        const fn speak(&self) -> String {
            "generic".into()
        }
    }
}

fn main() {}
