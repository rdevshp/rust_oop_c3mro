use oop_mro::prelude::*;

oop_class! {
    abstract class Animal {
        abstract virtual fn speak(&self) -> &'static str;
    }

    class Mammal: Animal {
        #[override]
        virtual fn speak(&self) -> &'static str {
            "mammal"
        }
    }

    class Dog: Mammal {
        #[override]
        virtual fn speak(&self) -> &'static str {
            "woof"
        }
    }

    class Walker {
        fn legs(&self) -> usize {
            2
        }
    }

    class Kangaroo: Mammal, Walker {
        #[override]
        virtual fn speak(&self) -> &'static str {
            "chuff"
        }
    }
}

fn main() {
    let mammal: Box<dyn AsMammal> = Box::new(Dog::default());
    let animal: Box<dyn AsAnimal> = mammal;
    assert_eq!(animal.as_animal().speak(), "woof");

    let kangaroo: Box<dyn AsKangaroo> = Box::new(Kangaroo::default());
    let walker: Box<dyn AsWalker> = kangaroo;
    assert_eq!(walker.as_walker().legs(), 2);
}
