use oop_mro::prelude::*;

oop_class! {
    class Service {
        const async fn load(&self) -> usize {
            1
        }
    }
}

fn main() {}
