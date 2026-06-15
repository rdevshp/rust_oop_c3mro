use oop_mro::prelude::*;

oop_class! {
    class Device {
        virtual unsafe fn read(&self) -> usize {
            0
        }
    }

    class Sensor: Device {
        #[override]
        virtual fn read(&self) -> usize {
            42
        }
    }
}

fn main() {}
