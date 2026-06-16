use oop_mro::prelude::*;

oop_class! {
    class Slot<T> {
        label: String,

        constructor(label: String) {
            self.label = label;
        }

        fn label(&self) -> &str {
            &self.label
        }

        virtual fn type_name(&self) -> &'static str {
            core::any::type_name::<T>()
        }
    }

    class Left: virtual Slot<i32> {
        constructor(): Slot<i32>("left".into()) {}
    }

    class Right: virtual Slot<String> {
        constructor(): Slot<String>("right".into()) {}
    }

    class Diamond: Left, Right {
        constructor():
            Slot<i32>("int".into()),
            Slot<String>("string".into()),
            Left(),
            Right()
        {}
    }
}

fn main() {
    let diamond = Diamond::new();
    let left_slot: &Slot<i32> = <Diamond as AsSlot<i32>>::as_slot(&diamond);
    let right_slot: &Slot<String> = <Diamond as AsSlot<String>>::as_slot(&diamond);

    assert_eq!(left_slot.label(), "int");
    assert_eq!(right_slot.label(), "string");
    assert_eq!(left_slot.type_name(), "i32");
    assert_eq!(right_slot.type_name(), "alloc::string::String");
}
