use oop_mro::prelude::*;

oop_class! {
    class Service {
        virtual async fn fetch(&self) -> usize {
            1
        }
    }

    class SyncService: Service {
        #[override]
        virtual fn fetch(&self) -> usize {
            2
        }
    }
}

fn main() {}
