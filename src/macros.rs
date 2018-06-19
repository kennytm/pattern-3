macro_rules! impl_unboxed_functions {
    ($(
        [$($gen_bounds:tt)*]
        $target:ty = |&$self:ident $(,$name:ident: $args:ty)*| -> $output:ty
        $body:block
    )+) => {$(
        impl $($gen_bounds)* FnOnce<($($args,)*)> for $target {
            type Output = $output;
            #[inline]
            extern "rust-call" fn call_once(self, args: ($($args,)*)) -> Self::Output {
                self.call(args)
            }
        }

        impl $($gen_bounds)* FnMut<($($args,)*)> for $target {
            #[inline]
            extern "rust-call" fn call_mut(&mut self, args: ($($args,)*)) -> Self::Output {
                self.call(args)
            }
        }

        impl $($gen_bounds)* Fn<($($args,)*)> for $target {
            #[inline]
            extern "rust-call" fn call(&$self, ($($name,)*): ($($args,)*)) -> Self::Output
            $body
        }

        impl $($gen_bounds)* Clone for $target {
            #[inline]
            fn clone(&self) -> Self {
                *self
            }
        }

        impl $($gen_bounds)* Copy for $target {}

        impl $($gen_bounds)* ::std::fmt::Debug for $target {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                f.debug_tuple(stringify!($target))
                    .field(&self.0)
                    .finish()
            }
        }
    )+}
}



/*

macro_rules! impl_clone_and_debug_for_wrapper {
    (
        [$($gen_bounds:tt)*] where [$($where_bounds:tt)*]
        $target:ty => ($($cond:ty),+);
        fields ($($field:tt),+)
    ) => {
        impl $($gen_bounds)* Clone for $target
        where
            $($cond: Clone,)+
            $($where_bounds)*
        {
            #[inline]
            fn clone(&self) -> Self {
                Self { $($field: self.$field.clone()),+ }
            }
            #[inline]
            fn clone_from(&mut self, source: &Self) {
                $(self.$field.clone_from(&source.$field);)+
            }
        }

        impl $($gen_bounds)* ::std::fmt::Debug for $target
        where
            $($cond: ::std::fmt::Debug,)+
            $($where_bounds)*
        {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                let mut builder = f.debug_tuple(stringify!($target));
                $(builder.field(&self.$field);)+
                builder.finish()
            }
        }
    }
}

*/