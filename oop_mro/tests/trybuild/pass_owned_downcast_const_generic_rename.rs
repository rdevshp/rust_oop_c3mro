use oop_mro::prelude::*;

oop_class! {
    class ConstBase<const N: usize> {
        virtual fn value(&self) -> usize {
            N
        }
    }

    class ConstMid<const M: usize>: ConstBase<M> {
        #[override]
        virtual fn value(&self) -> usize {
            M + 1
        }
    }

    class ConstLeaf<const K: usize>: ConstMid<K> {
        #[override]
        virtual fn value(&self) -> usize {
            K + 2
        }
    }
}

fn main() {
    let leaf = ConstLeaf::<4>::default();
    assert_eq!(leaf.as_base::<ConstBase<4>>().value(), 6);

    let base: Box<dyn AsClass<ConstBase<4>>> = Box::new(ConstLeaf::<4>::default());
    let mid = match base.downcast::<dyn AsClass<ConstMid<4>>>() {
        Ok(mid) => mid,
        Err(_) => panic!("ConstLeaf should downcast through renamed const parameter"),
    };
    assert_eq!(mid.as_base::<ConstMid<4>>().value(), 6);
}
