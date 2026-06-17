use oop_mro::prelude::*;

oop_class! {
    class PtrBase<T> {
        virtual fn value(&self) -> usize {
            1
        }
    }

    class PtrMid<U>: PtrBase<*const U> {
        #[override]
        virtual fn value(&self) -> usize {
            2
        }
    }

    class PtrLeaf<V>: PtrMid<V> {
        #[override]
        virtual fn value(&self) -> usize {
            3
        }
    }

    class SliceBase<T: ?Sized> {
        virtual fn value(&self) -> usize {
            4
        }
    }

    class SliceMid<U>: SliceBase<[U]> {
        #[override]
        virtual fn value(&self) -> usize {
            5
        }
    }

    class SliceLeaf<V>: SliceMid<V> {
        #[override]
        virtual fn value(&self) -> usize {
            6
        }
    }

    class FnBase<T> {
        virtual fn value(&self) -> usize {
            7
        }
    }

    class FnMid<U>: FnBase<fn(U) -> U> {
        #[override]
        virtual fn value(&self) -> usize {
            8
        }
    }

    class FnLeaf<V>: FnMid<V> {
        #[override]
        virtual fn value(&self) -> usize {
            9
        }
    }

    class AssocBase<T> {
        virtual fn value(&self) -> usize {
            10
        }
    }

    class AssocMid<U>: AssocBase<::std::boxed::Box<dyn Iterator<Item = U>>> {
        #[override]
        virtual fn value(&self) -> usize {
            11
        }
    }

    class AssocLeaf<V>: AssocMid<V> {
        #[override]
        virtual fn value(&self) -> usize {
            12
        }
    }
}

fn main() {
    let ptr_base: Box<dyn AsClass<PtrBase<*const String>>> =
        Box::new(PtrLeaf::<String>::default());
    let ptr_mid = match ptr_base.downcast::<dyn AsClass<PtrMid<String>>>() {
        Ok(mid) => mid,
        Err(_) => panic!("PtrLeaf should downcast through raw pointer specialization"),
    };
    assert_eq!(ptr_mid.as_base::<PtrMid<String>>().value(), 3);

    let slice_base: Box<dyn AsClass<SliceBase<[String]>>> =
        Box::new(SliceLeaf::<String>::default());
    let slice_mid = match slice_base.downcast::<dyn AsClass<SliceMid<String>>>() {
        Ok(mid) => mid,
        Err(_) => panic!("SliceLeaf should downcast through slice specialization"),
    };
    assert_eq!(slice_mid.as_base::<SliceMid<String>>().value(), 6);

    let fn_base: Box<dyn AsClass<FnBase<fn(String) -> String>>> =
        Box::new(FnLeaf::<String>::default());
    let fn_mid = match fn_base.downcast::<dyn AsClass<FnMid<String>>>() {
        Ok(mid) => mid,
        Err(_) => panic!("FnLeaf should downcast through function pointer specialization"),
    };
    assert_eq!(fn_mid.as_base::<FnMid<String>>().value(), 9);

    let assoc_base: Box<
        dyn AsClass<AssocBase<::std::boxed::Box<dyn Iterator<Item = String>>>>,
    > = Box::new(AssocLeaf::<String>::default());
    let assoc_mid = match assoc_base.downcast::<dyn AsClass<AssocMid<String>>>() {
        Ok(mid) => mid,
        Err(_) => panic!("AssocLeaf should downcast through associated type binding"),
    };
    assert_eq!(assoc_mid.as_base::<AssocMid<String>>().value(), 12);
}
