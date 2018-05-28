    macro_rules! check {
        ($dollar:tt $a:lifetime) => {
            macro_rules! double_check {
                ($dollar($c:pat,)* $a) => {}
            }
            double_check!(4,'a);
        };
    }

    fn main() {
        check!($'a);
        // foo!(vis, pub, pub, 'a);
        // foo!(3. 'a 'c a);
    }
    
