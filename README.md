This is an experimental project doing C3 MRO OOP in Rust that is co-developed with Codex.

```rust
use oop_mro::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

oop_class! {
    abstract class Animal {
        abstract virtual fn typ(&self) -> &String;
        abstract virtual fn name(&self) -> &String;
        abstract virtual fn speak(&self);
        abstract virtual fn identity(&self) -> String;
    }
    abstract class Serializable {
        abstract virtual fn serialize(&self) -> String;
    }
    abstract class Mammal: Animal, Serializable {
        typ: String,

        constructor() {
            self.typ = String::from("mammal");
        }
        #[override]
        virtual fn typ(&self) -> &String {
            &self.typ
        }
        #[override]
        virtual fn speak(&self) {
            println!("{} speaking", self.identity());
        }
    }
    class Kangaroo: Mammal {
        name: String,

        constructor(name: String) {
            self.name = name;
        }
        #[override]
        virtual fn name(&self) -> &String {
            &self.name
        }
        #[override]
        virtual fn serialize(&self) -> String {
            String::new()
        }
        #[override]
        virtual fn identity(&self) -> String {
            String::from("Kangaroo")
        }
    }
    class Dog: Mammal {
        name: String,

        constructor(name: String) {
            self.name = name;
        }
        #[override]
        virtual fn name(&self) -> &String {
            &self.name
        }
        #[override]
        virtual fn serialize(&self) -> String {
            String::new()
        }
        #[override]
        virtual fn identity(&self) -> String{
            String::from("Dog")
        }
    }
}

oop_class! {
    class Entity {
        virtual fn describe(&self) -> String {
            "Entity".into()
        }
    }

    class Named: Entity {
        #[override]
        virtual fn describe(&self) -> String {
            format!("Named -> {}", super_call!(Entity::describe, self))
        }
    }

    class Tagged: Entity {
        #[override]
        virtual fn describe(&self) -> String {
            format!("Tagged -> {}", super_call!(Entity::describe, self))
        }
    }

    class Document: Named, Tagged {
    }

    class Document2: Tagged, Named {
    }
}

oop_class! {
    class SharedRoot {
        value: usize,

        constructor(value: usize) {
            self.value = value;
        }

        fn value(&self) -> usize {
            self.value
        }

        fn set_value(&mut self, value: usize) {
            self.value = value;
        }
    }

    class LeftBranch: virtual SharedRoot {
        constructor(): SharedRoot(1) {}
    }

    class RightBranch: virtual SharedRoot {
        constructor(): SharedRoot(2) {}
    }

    class SharedDiamond: LeftBranch, RightBranch {
        constructor(): SharedRoot(3), LeftBranch(), RightBranch() {}
    }
}

oop_class! {
    class Slot<T> {
        virtual fn type_name(&self) -> &'static str {
            core::any::type_name::<T>()
        }
    }

    class IntSlotOwner: virtual Slot<i32> {}
    class StringSlotOwner: virtual Slot<String> {}
    class SpecializedSlots: IntSlotOwner, StringSlotOwner {}
}

oop_class! {
    class Test {
        virtual async unsafe fn f(&self) {}
    }
}
#[derive(Debug)]
#[allow(dead_code)]
struct Job {
    id: u32,
}
oop_class! {
    abstract class Factory<T> {
        abstract virtual fn create(&mut self) -> T;
    }
    class JobFactory: Factory<Job> {
        id: u32 = 50,
        constructor() {}
        #[override]
        virtual fn create(&mut self) -> Job {
            let r = self.id;
            self.id += 1;
            Job { id: r }
        }
    }
}

oop_class! {
    class TicketRegistry {
        pub const PREFIX: &'static str = "ticket";
        const FIRST_ID: usize = 1000;
        static NEXT_ID: AtomicUsize = AtomicUsize::new(1000);

        fn reset_for_example() {
            Self::NEXT_ID.store(Self::FIRST_ID, Ordering::Relaxed);
        }

        fn next_id() -> usize {
            Self::NEXT_ID.fetch_add(1, Ordering::Relaxed)
        }

        fn next_label() -> String {
            format!("{}-{}", Self::PREFIX, Self::next_id())
        }
    }
}

fn main() {
    let dog = Dog::new(String::from("Dog1"));
    let kangaroo = Kangaroo::new(String::from("Kangaroo1"));
    let v: Vec<&Mammal> = vec![dog.as_mammal(), kangaroo.as_mammal()];
    for i in v {
        println!("type: {}, name: {}", i.typ(), i.name());
        i.speak();
        /*
        type: mammal, name: Dog1
        Dog speaking
        type: mammal, name: Kangaroo1
        Kangaroo speaking
        */
    }

    let animals: Vec<Box<dyn AsAnimal>> = vec![
        Box::new(Dog::new(String::from("Dog2"))),
        Box::new(Kangaroo::new(String::from("Kangaroo2"))),
    ];

    for object in &animals {
        let animal: &Animal = object.as_animal();
        println!("{}", animal.name());

        // checked downcast
        match animal.downcast_ref::<Dog>() {
            Some(dog) => {
                println!("{} is a dog!", dog.name());
            }
            None => {
                println!("{} is not a dog!", animal.name());
            }
        }
    }

    assert_eq!(
        <Document as OopClass>::MRO,
        &["Document", "Named", "Tagged", "Entity"]
    );
    println!("{}", Document::default().describe()); // Named -> Entity
    assert_eq!(
        <Document2 as OopClass>::MRO,
        &["Document2", "Tagged", "Named", "Entity"]
    );
    println!("{}", Document2::default().describe()); // Tagged -> Entity

    let mut shared = SharedDiamond::new();
    assert!(core::ptr::eq(
        shared.as_left_branch().as_shared_root(),
        shared.as_right_branch().as_shared_root(),
    ));
    shared
        .as_right_branch_mut()
        .as_shared_root_mut()
        .set_value(4);
    assert_eq!(shared.as_left_branch().as_shared_root().value(), 4);

    let specialized = SpecializedSlots::default();
    assert_eq!(
        <SpecializedSlots as AsSlot<i32>>::as_slot(&specialized).type_name(),
        "i32"
    );
    assert_eq!(
        <SpecializedSlots as AsSlot<String>>::as_slot(&specialized).type_name(),
        "alloc::string::String"
    );

    let mut job_factory = JobFactory::new();
    println!("job id: {:?} ", job_factory.create());
    println!("job id: {:?} ", job_factory.create());

    TicketRegistry::reset_for_example();
    assert_eq!(TicketRegistry::PREFIX, "ticket");
    assert_eq!(TicketRegistry::next_label(), "ticket-1000");
    assert_eq!(TicketRegistry::next_id(), 1001);
    assert_eq!(TicketRegistry::NEXT_ID.load(Ordering::Relaxed), 1002);
    println!("next ticket: {}", TicketRegistry::next_label());

    // checked downcast
    let factory: Box<dyn AsFactory<Job>> = Box::new(job_factory);
    let factory_downcast_result = factory.downcast::<dyn AsJobFactory>();
    match factory_downcast_result {
        Ok(mut job_factory_downcast) => {
            println!("downcasted to job_factory");
            println!(
                "job_factory_downcast job id: {:?}:",
                job_factory_downcast.as_job_factory_mut().create()
            );
        }
        Err(_) => {
            println!("failed to downcast factory");
        }
    }
}
```
