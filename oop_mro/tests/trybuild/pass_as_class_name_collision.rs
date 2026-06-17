use oop_mro::prelude::*;

oop_class! {
    class Animal {}
    class AsAnimal {}
    class Dog: Animal {}
}

fn main() {
    let animal: Box<dyn AsClass<Animal>> = Box::new(Dog::default());
    let _ = animal.as_base::<Animal>();
    let _ = AsAnimal::default();
}
