use oop_mro::prelude::*;

oop_class! {
    class Animal {
        virtual fn speak(&self) -> &'static str {
            "animal"
        }
    }

    class Dog: Animal {
        #[override]
        virtual fn speak(&self) -> &'static str {
            "woof"
        }
    }

    class Vehicle {
        virtual fn drive(&self) -> &'static str {
            "drive"
        }
    }
}

fn main() {
    let animal: Box<dyn AsClass<Animal>> = Box::new(Dog::default());
    let _ = animal.downcast::<dyn AsClass<Vehicle>>();
}
