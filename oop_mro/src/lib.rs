extern crate self as oop_mro;

pub use oop_mro_macros::oop_class;

use core::mem::MaybeUninit;

pub trait OopClass {
    const NAME: &'static str;
    const MRO: &'static [&'static str];
    const IS_ABSTRACT: bool = false;
    const METHOD_TABLE: &'static MethodTable = &EMPTY_METHOD_TABLE;
    const ABSTRACT_METHODS: &'static [MethodEntry] = &[];
}

pub trait OopObject {
    type Class: OopClass;
}

/// # Safety
///
/// Implementations must be generated from a valid `oop_class!` graph. The
/// returned self reference, complete class id, source subobject id, and owned
/// complete-object pointer must all describe the same represented subobject.
pub unsafe trait AsClass<Target> {
    #[doc(hidden)]
    fn __oop_as_self(&self) -> &Target;

    #[doc(hidden)]
    fn __oop_as_self_mut(&mut self) -> &mut Target;

    #[doc(hidden)]
    fn __oop_complete_class_id(&self) -> usize;

    #[doc(hidden)]
    fn __oop_source_subobject_id(&self) -> usize;

    #[doc(hidden)]
    fn __oop_into_complete_owned(self: Box<Self>) -> *mut ();
}

pub trait OopBase<Target> {
    #[doc(hidden)]
    fn __oop_as_base(&self) -> &Target;

    #[doc(hidden)]
    fn __oop_as_base_mut(&mut self) -> &mut Target;
}

impl<Target, Source> OopBase<Target> for Box<Source>
where
    Source: OopBase<Target> + ?Sized,
{
    fn __oop_as_base(&self) -> &Target {
        (**self).__oop_as_base()
    }

    fn __oop_as_base_mut(&mut self) -> &mut Target {
        (**self).__oop_as_base_mut()
    }
}

impl<Source, Target> OopBase<Target> for dyn AsClass<Source> + '_
where
    Source: OopBase<Target>,
{
    fn __oop_as_base(&self) -> &Target {
        <Source as OopBase<Target>>::__oop_as_base(
            <dyn AsClass<Source> as AsClass<Source>>::__oop_as_self(self),
        )
    }

    fn __oop_as_base_mut(&mut self) -> &mut Target {
        <Source as OopBase<Target>>::__oop_as_base_mut(
            <dyn AsClass<Source> as AsClass<Source>>::__oop_as_self_mut(self),
        )
    }
}

pub trait OopBaseVia<Via, Target> {
    #[doc(hidden)]
    fn __oop_as_base_via(&self) -> &Target;

    #[doc(hidden)]
    fn __oop_as_base_via_mut(&mut self) -> &mut Target;
}

impl<Source, Via, Target> OopBaseVia<Via, Target> for dyn AsClass<Source> + '_
where
    Source: OopBaseVia<Via, Target>,
{
    fn __oop_as_base_via(&self) -> &Target {
        <Source as OopBaseVia<Via, Target>>::__oop_as_base_via(<dyn AsClass<Source> as AsClass<
            Source,
        >>::__oop_as_self(self))
    }

    fn __oop_as_base_via_mut(&mut self) -> &mut Target {
        <Source as OopBaseVia<Via, Target>>::__oop_as_base_via_mut(
            <dyn AsClass<Source> as AsClass<Source>>::__oop_as_self_mut(self),
        )
    }
}

pub trait OopBaseAccess {
    fn as_base<Target>(&self) -> &Target
    where
        Self: OopBase<Target>,
    {
        <Self as OopBase<Target>>::__oop_as_base(self)
    }

    fn as_base_mut<Target>(&mut self) -> &mut Target
    where
        Self: OopBase<Target>,
    {
        <Self as OopBase<Target>>::__oop_as_base_mut(self)
    }

    fn as_base_via<Via, Target>(&self) -> &Target
    where
        Self: OopBaseVia<Via, Target>,
    {
        <Self as OopBaseVia<Via, Target>>::__oop_as_base_via(self)
    }

    fn as_base_via_mut<Via, Target>(&mut self) -> &mut Target
    where
        Self: OopBaseVia<Via, Target>,
    {
        <Self as OopBaseVia<Via, Target>>::__oop_as_base_via_mut(self)
    }
}

impl<T: ?Sized> OopBaseAccess for T {}

pub trait OopBoxBaseVia<Via, Target: ?Sized> {
    #[doc(hidden)]
    fn __oop_into_base_via(self: Box<Self>) -> Box<Target>;
}

pub trait OopDynBoxBaseVia<Via, Target>: Sized {
    #[doc(hidden)]
    fn __oop_dyn_into_base_via(source: Box<dyn AsClass<Self>>) -> Box<dyn AsClass<Target>>;
}

impl<Source, Via, Target> OopBoxBaseVia<Via, dyn AsClass<Target>> for dyn AsClass<Source>
where
    Source: OopDynBoxBaseVia<Via, Target> + 'static,
    Target: 'static,
{
    fn __oop_into_base_via(self: Box<Self>) -> Box<dyn AsClass<Target>> {
        <Source as OopDynBoxBaseVia<Via, Target>>::__oop_dyn_into_base_via(self)
    }
}

pub trait OopBoxBaseAccess {
    fn into_base_via<Via, Target: ?Sized>(self: Box<Self>) -> Box<Target>
    where
        Self: OopBoxBaseVia<Via, Target>,
    {
        <Self as OopBoxBaseVia<Via, Target>>::__oop_into_base_via(self)
    }
}

impl<T: ?Sized> OopBoxBaseAccess for T {}

pub trait OopDowncastRef {
    fn downcast_ref<Target>(&self) -> Option<&Target>
    where
        Self: OopDowncastRefTarget<Target>,
    {
        <Self as OopDowncastRefTarget<Target>>::downcast_ref_target(self)
    }
}

impl<T: ?Sized> OopDowncastRef for T {}

pub trait OopDowncastRefTarget<Target> {
    fn downcast_ref_target(&self) -> Option<&Target>;
}

pub trait OopDowncastMut {
    fn downcast_mut<Target>(&mut self) -> Option<&mut Target>
    where
        Self: OopDowncastMutTarget<Target>,
    {
        <Self as OopDowncastMutTarget<Target>>::downcast_mut_target(self)
    }
}

impl<T: ?Sized> OopDowncastMut for T {}

pub trait OopDowncastMutTarget<Target> {
    fn downcast_mut_target(&mut self) -> Option<&mut Target>;
}

pub trait OopBoxDowncast {
    fn downcast<Target: ?Sized>(self: Box<Self>) -> Result<Box<Target>, Box<Self>>
    where
        Self: OopBoxDowncastTarget<Target>,
    {
        <Self as OopBoxDowncastTarget<Target>>::downcast_target(self)
    }
}

impl<T: ?Sized> OopBoxDowncast for T {}

pub trait OopBoxDowncastTarget<Target: ?Sized> {
    fn downcast_target(self: Box<Self>) -> Result<Box<Target>, Box<Self>>;
}

pub trait OopDynBoxDowncast<Target>: Sized {
    #[doc(hidden)]
    fn __oop_dyn_downcast(
        source: Box<dyn AsClass<Self>>,
    ) -> Result<Box<dyn AsClass<Target>>, Box<dyn AsClass<Self>>>;
}

impl<Source, Target> OopBoxDowncastTarget<dyn AsClass<Target>> for dyn AsClass<Source>
where
    Source: OopDynBoxDowncast<Target> + 'static,
    Target: 'static,
{
    fn downcast_target(
        self: Box<Self>,
    ) -> Result<Box<dyn AsClass<Target>>, Box<dyn AsClass<Source>>> {
        <Source as OopDynBoxDowncast<Target>>::__oop_dyn_downcast(self)
    }
}

pub type MethodFn = fn();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MethodEntry {
    pub name: &'static str,
    pub owner: &'static str,
    pub signature: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MethodTable {
    pub methods: &'static [MethodEntry],
}

impl MethodTable {
    pub const fn empty() -> Self {
        Self { methods: &[] }
    }

    pub fn find(&self, name: &str) -> Option<&'static MethodEntry> {
        let mut index = 0;
        while index < self.methods.len() {
            let entry = &self.methods[index];
            if str_eq(entry.name, name) {
                return Some(entry);
            }
            index += 1;
        }
        None
    }
}

pub static EMPTY_METHOD_TABLE: MethodTable = MethodTable::empty();

fn str_eq(left: &str, right: &str) -> bool {
    left.as_bytes() == right.as_bytes()
}

#[doc(hidden)]
#[repr(C)]
pub struct VirtualBaseSlot<T> {
    pub __oop_value: MaybeUninit<T>,
    pub __oop_initialized: bool,
}

impl<T> VirtualBaseSlot<T> {
    pub const fn uninit() -> Self {
        Self {
            __oop_value: MaybeUninit::uninit(),
            __oop_initialized: false,
        }
    }

    pub fn init(&mut self, value: T) {
        if self.__oop_initialized {
            unsafe {
                self.__oop_value.assume_init_drop();
            }
        }

        self.__oop_value.write(value);
        self.__oop_initialized = true;
    }

    /// # Safety
    ///
    /// The slot must have been initialized with `init` and not subsequently moved out.
    pub unsafe fn assume_init_ref(&self) -> &T {
        debug_assert!(self.__oop_initialized);
        unsafe { self.__oop_value.assume_init_ref() }
    }

    /// # Safety
    ///
    /// The slot must have been initialized with `init` and not subsequently moved out.
    pub unsafe fn assume_init_mut(&mut self) -> &mut T {
        debug_assert!(self.__oop_initialized);
        unsafe { self.__oop_value.assume_init_mut() }
    }
}

impl<T> Drop for VirtualBaseSlot<T> {
    fn drop(&mut self) {
        if self.__oop_initialized {
            unsafe {
                self.__oop_value.assume_init_drop();
            }
        }
    }
}

pub mod prelude {
    pub use crate::{
        oop_class, super_call, AsClass, MethodEntry, MethodFn, MethodTable, OopBase, OopBaseAccess,
        OopBaseVia, OopBoxBaseAccess, OopBoxBaseVia, OopBoxDowncast, OopBoxDowncastTarget,
        OopClass, OopDowncastMut, OopDowncastMutTarget, OopDowncastRef, OopDowncastRefTarget,
        OopDynBoxBaseVia, OopDynBoxDowncast, OopObject,
    };
}

#[macro_export]
macro_rules! super_call {
    ($($tokens:tt)*) => {
        compile_error!("super_call! can only be used inside methods declared in oop_class!");
    };
}
