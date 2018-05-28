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

