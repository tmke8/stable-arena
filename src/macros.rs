/// Declare an `Arena` containing one dropless arena and many typed arenas (the
/// types of the typed arenas are specified by the arguments).
///
/// There are three cases of interest.
/// - Types that are `Copy`: these need not be specified in the arguments. They
///   will use the `DroplessArena`.
/// - Types that are `!Copy` and `!Drop`: these must be specified in the
///   arguments. An empty `TypedArena` will be created for each one, but the
///   `DroplessArena` will always be used and the `TypedArena` will stay empty.
///   This is odd but harmless, because an empty arena allocates no memory.
/// - Types that are `!Copy` and `Drop`: these must be specified in the
///   arguments. The `TypedArena` will be used for them.
///
#[macro_export]
macro_rules! declare_arena {
    ([$($a:tt $name:ident: $ty:ty,)*]) => {
        #[derive(Default)]
        pub struct Arena<'tcx> {
            pub dropless: $crate::DroplessArena,
            $($name: $crate::TypedArena<$ty>,)*
        }

        pub trait ArenaAllocatable<'tcx, C = rustc_arena::IsNotCopy>: Sized {
            #[allow(clippy::mut_from_ref)]
            fn allocate_on(self, arena: &'tcx Arena<'tcx>) -> &'tcx mut Self;
            #[allow(clippy::mut_from_ref)]
            fn allocate_from_iter(
                arena: &'tcx Arena<'tcx>,
                iter: impl ::std::iter::IntoIterator<Item = Self>,
            ) -> &'tcx mut [Self];
        }

        // Any type that impls `Copy` can be arena-allocated in the `DroplessArena`.
        impl<'tcx, T: Copy> ArenaAllocatable<'tcx, rustc_arena::IsCopy> for T {
            #[inline]
            #[allow(clippy::mut_from_ref)]
            fn allocate_on(self, arena: &'tcx Arena<'tcx>) -> &'tcx mut Self {
                arena.dropless.alloc(self)
            }
            #[inline]
            #[allow(clippy::mut_from_ref)]
            fn allocate_from_iter(
                arena: &'tcx Arena<'tcx>,
                iter: impl ::std::iter::IntoIterator<Item = Self>,
            ) -> &'tcx mut [Self] {
                arena.dropless.alloc_from_iter(iter)
            }
        }
        $(
            impl<'tcx> ArenaAllocatable<'tcx, rustc_arena::IsNotCopy> for $ty {
                #[inline]
                fn allocate_on(self, arena: &'tcx Arena<'tcx>) -> &'tcx mut Self {
                    if !::std::mem::needs_drop::<Self>() {
                        arena.dropless.alloc(self)
                    } else {
                        arena.$name.alloc(self)
                    }
                }

                #[inline]
                #[allow(clippy::mut_from_ref)]
                fn allocate_from_iter(
                    arena: &'tcx Arena<'tcx>,
                    iter: impl ::std::iter::IntoIterator<Item = Self>,
                ) -> &'tcx mut [Self] {
                    if !::std::mem::needs_drop::<Self>() {
                        arena.dropless.alloc_from_iter(iter)
                    } else {
                        arena.$name.alloc_from_iter(iter)
                    }
                }
            }
        )*

        impl<'tcx> Arena<'tcx> {
            #[inline]
            #[allow(clippy::mut_from_ref)]
            pub fn alloc<T: ArenaAllocatable<'tcx, C>, C>(&'tcx self, value: T) -> &mut T {
                value.allocate_on(self)
            }

            // Any type that impls `Copy` can have slices be arena-allocated in the `DroplessArena`.
            #[inline]
            #[allow(clippy::mut_from_ref)]
            pub fn alloc_slice<T: ::std::marker::Copy>(&self, value: &[T]) -> &mut [T] {
                if value.is_empty() {
                    return &mut [];
                }
                self.dropless.alloc_slice(value)
            }

            #[inline]
            pub fn alloc_str(&self, string: &str) -> &str {
                if string.is_empty() {
                    return "";
                }
                self.dropless.alloc_str(string)
            }

            #[allow(clippy::mut_from_ref)]
            pub fn alloc_from_iter<T: ArenaAllocatable<'tcx, C>, C>(
                &'tcx self,
                iter: impl ::std::iter::IntoIterator<Item = T>,
            ) -> &mut [T] {
                T::allocate_from_iter(self, iter)
            }
        }
    }
}
