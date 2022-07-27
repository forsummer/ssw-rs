pub mod avx
{
    use std::ops::Add;
    use std::ops::Sub;
    use std::ops::Shl;
    use std::ops::Index;
    use std::fmt::Debug;
    use std::fmt::Display;
    use std::mem::transmute;
    use std::arch::x86_64::__m256i;
    use std::arch::x86_64::_mm256_adds_epu16;
    use std::arch::x86_64::_mm256_subs_epu16;
    use std::arch::x86_64::_mm256_load_si256;
    use std::arch::x86_64::_mm256_cmpeq_epi16;
    use std::arch::x86_64::_mm256_cmpgt_epi16;
    use std::arch::x86_64::_mm256_movemask_epi8;
    use std::arch::x86_64::_mm256_setzero_si256;

    #[derive(Clone, Copy)]
    pub struct M256Epu16(pub __m256i);

    impl PartialEq for M256Epu16
    {
        fn eq(&self, other: &Self) -> bool
        {
            unsafe { _mm256_movemask_epi8(_mm256_cmpeq_epi16(self.0, other.0)) == -1 }
        }

        fn ne(&self, other: &Self) -> bool
        {
            !unsafe { _mm256_movemask_epi8(_mm256_cmpeq_epi16(self.0, other.0)) == -1 }
        }
    }

    impl PartialOrd for M256Epu16
    {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering>
        {
            if unsafe {_mm256_movemask_epi8(_mm256_cmpeq_epi16(self.0, other.0)) == -1}
            {
                Some(std::cmp::Ordering::Equal)
            }
            else if unsafe {_mm256_movemask_epi8(_mm256_cmpgt_epi16(self.0, other.0)) == -1}
            {
                Some(std::cmp::Ordering::Greater)
            }
            else if unsafe {_mm256_movemask_epi8(_mm256_cmpgt_epi16(other.0, self.0)) == -1}
            {
                Some(std::cmp::Ordering::Less)
            }
            else
            {
                None
            }
        }
    }

    impl Add for M256Epu16
    {
        type Output = M256Epu16;
        fn add(self, rhs: Self) -> Self::Output
        {
            unsafe { M256Epu16(_mm256_adds_epu16(self.0, rhs.0)) }
        }
    }

    impl Sub for M256Epu16
    {
        type Output = M256Epu16;
        fn sub(self, rhs: Self) -> Self::Output
        {
            unsafe { M256Epu16(_mm256_subs_epu16(self.0, rhs.0)) }
        }
    }

    impl Shl<usize> for M256Epu16
    {
        type Output = M256Epu16;
        fn shl(self, rhs: usize) -> Self::Output
        {
            if rhs > 16 { panic!("Out of bound"); }
            unsafe
            {
                let arr = transmute::<__m256i, [u16; 16]>(self.0.clone());
                let ptr_arr = arr.as_ptr().add(rhs);
                let mut out = [0; 16];
                ptr_arr.copy_to(out.as_mut_ptr(), 16 - rhs);
                M256Epu16(transmute::<[u16; 16], __m256i>(out))
            }
        }
    }

    impl Index<usize> for M256Epu16
    {
        type Output = u16;
        fn index(&self, index: usize) -> &Self::Output
        {
            if index > 15
            {
                panic!("Out of bound");
            }
            let M256Epu16(arr) = self;
            unsafe
            {
                let ptr_arr = transmute::<&__m256i, *const __m256i>(arr);
                &*transmute::<*const __m256i, *const u16>(ptr_arr).add(index)
            }
        }
    }

    impl Display for M256Epu16
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
        {
            write!(f, "[{}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}]", 
            self[0], self[1], self[2], self[3], self[4], self[5], self[6], self[7],
            self[8], self[9], self[10], self[11], self[12], self[13], self[14], self[15])
        }
    }

    impl Debug for M256Epu16
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
        {
            f.debug_list()
                .entry(&self[0])
                .entry(&self[1])
                .entry(&self[2])
                .entry(&self[3])
                .entry(&self[4])
                .entry(&self[5])
                .entry(&self[6])
                .entry(&self[7])
                .entry(&self[8])
                .entry(&self[9])
                .entry(&self[10])
                .entry(&self[11])
                .entry(&self[12])
                .entry(&self[13])
                .entry(&self[14])
                .entry(&self[15])
                .finish()
        }
    }

    #[allow(dead_code)]
    impl M256Epu16
    {
        pub fn zero() -> M256Epu16
        {
            unsafe { M256Epu16(_mm256_setzero_si256()) }
        }

        pub fn set(e0: u16, e1: u16, e2: u16, e3: u16,
                e4: u16, e5: u16, e6: u16, e7: u16,
                e8: u16, e9: u16, e10: u16, e11: u16, 
                e12: u16, e13: u16, e14: u16, e15: u16) -> M256Epu16
        {
            let arr_u16 = [e15, e14, e13, e12,
                        e11, e10, e9, e8,
                        e7, e6, e5, e4,
                        e3, e2, e1, e0].as_ptr();
            unsafe
            {
                let ptr_m256 = *transmute::<*const u16, *const __m256i>(arr_u16);
                M256Epu16(_mm256_load_si256(&ptr_m256 as *const __m256i))
            }
        }

        pub fn setr(e0: u16, e1: u16, e2: u16, e3: u16,
                e4: u16, e5: u16, e6: u16, e7: u16,
                e8: u16, e9: u16, e10: u16, e11: u16, 
                e12: u16, e13: u16, e14: u16, e15: u16) -> M256Epu16
        {
            let ptr_arr_u16 = [e0, e1, e2, e3,
                        e4, e5, e6, e7,
                        e8, e9, e10, e11,
                        e12, e13, e14, e15].as_ptr();
            unsafe
            {
                let ptr_m256 = transmute::<*const u16, *const __m256i>(ptr_arr_u16);
                M256Epu16(_mm256_load_si256(ptr_m256))
            }
        }

        pub fn fill(item: u16) -> M256Epu16
        {
            let ptr_arr_u16 = vec![item; 16].as_ptr();
            unsafe
            {
                let mut m256 = _mm256_setzero_si256();
                let ptr_m256 = transmute::<*mut __m256i, *mut u16>(&mut m256);
                ptr_arr_u16.copy_to(ptr_m256, 16);
                M256Epu16(m256)
            }
        }

        pub fn from_arr(arr: &[u16; 16]) -> M256Epu16
        {
            unsafe { M256Epu16(*transmute::<*const u16, *const __m256i>(arr.as_ptr())) }
        }

        pub fn from_vec(vec: &Vec<u16>) -> M256Epu16
        {
            if vec.len() != 16 { panic!("The length of vec should equal to 16") }
            unsafe
            {
                let vec = vec.iter().rev().map(|item| *item).collect::<Vec<u16>>();
                M256Epu16(*transmute::<*const u16, *const __m256i>(vec.as_ptr()))
            }
        }

        pub fn to_arr(&self) -> [u16; 16]
        {
            unsafe
            {
                let mut arr = [0; 16];
                let ptr = transmute::<*const __m256i, *const u16>(&(self.0) as *const __m256i);
                ptr.copy_to(arr.as_mut_ptr(), 16);
                arr
            }
        }

        pub fn to_vec(&self) -> Vec<u16>
        {
            unsafe
            {
                let mut vec = vec![0; 16];
                let ptr = transmute::<*const __m256i, *const u16>(&self.0 as *const __m256i);
                ptr.copy_to(vec.as_mut_ptr(), 16);
                vec
            }
        }

        pub fn get_max_m256_u16(&self) -> u16
        {
            let mut arr = self.to_arr();
            arr.sort();
            *(arr.last().unwrap())
        }
    }

    #[macro_export]
    macro_rules! max_epu16
    {
        ( $ ( $arr: expr ), * ) => 
        { 
            {
                let mut max = M256Epu16::zero();
                use std::arch::x86_64::_mm256_max_epu16;
                let max_arr = |a: M256Epu16, b: M256Epu16| 
                {
                    let (M256Epu16(x), M256Epu16(y)) = (a, b);
                    unsafe { _mm256_max_epu16(x, y) }
                };
                $( max = M256Epu16(max_arr(max, $arr)); )*
                max
            }
        };
    }
}

#[macro_export]
macro_rules! max
{
    ($x:expr) => ( $x );
    ($x:expr, $($xs:expr),+) =>
    {
        std::cmp::max($x, max!( $($xs),+ ))
    };
}

#[macro_export]
macro_rules! min
{
    ($x:expr) => ( $x );
    ($x:expr, $($xs:expr),+) =>
    {
        std::cmp::min($x, min!( $($xs),+ ))
    };
}

#[cfg(test)]
mod test
{
    use crate::max_epu16;
    use super::avx::M256Epu16;

    #[test]
    fn test_eq()
    {
        let a = M256Epu16::set(9, 21, 2, 0, 3, 1, 9, 21, 9, 21, 2, 0, 3, 1, 9, 21);
        let b = M256Epu16::set(9, 21, 2, 0, 3, 1, 9, 21, 9, 21, 2, 0, 3, 1, 9, 21);
        assert_eq!(a, b);
    }

    #[test]
    fn test_ne()
    {
        let a = M256Epu16::set(9, 21, 2, 0, 3, 1, 9, 21, 9, 21, 2, 0, 3, 1, 9, 21);
        let b = M256Epu16::set(7, 31, 2, 0, 3, 1, 9, 21, 9, 21, 2, 0, 3, 1, 9, 21);
        assert_ne!(a, b);
    }

    #[test]
    fn test_gt()
    {
        let a = M256Epu16::set(9, 1, 2, 4, 5, 8, 5, 1, 8, 20, 3, 8, 9, 9, 4, 9);
        let b = M256Epu16::set(8, 0, 1, 3, 4, 7, 1, 0, 7, 1, 2, 5, 8, 4, 3, 2);
        assert!(a > b);
    }

    #[test]
    fn test_gt_false()
    {
        let a = M256Epu16::set(9, 1, 2, 4, 5, 8, 5, 1, 8, 20, 3, 8, 9, 9, 4, 9);
        let b = M256Epu16::set(8, 1, 1, 3, 4, 7, 1, 0, 7, 1, 2, 5, 8, 4, 3, 2);
        assert!(!(a > b));
    }

    #[test]
    fn test_le()
    {
        let a = M256Epu16::set(9, 1, 2, 4, 5, 8, 5, 1, 8, 20, 3, 8, 9, 9, 4, 9);
        let b = M256Epu16::set(8, 0, 1, 3, 4, 7, 1, 0, 7, 1, 2, 5, 8, 4, 3, 2);
        assert!(b < a);
    }

    #[test]
    fn test_le_false()
    {
        let a = M256Epu16::set(9, 1, 2, 4, 5, 8, 5, 1, 8, 20, 3, 8, 9, 9, 4, 9);
        let b = M256Epu16::set(8, 1, 1, 3, 4, 7, 1, 0, 7, 1, 2, 5, 8, 4, 3, 2);
        assert!(!(b < a));
    }

    #[test]
    fn test_add()
    {
        let a = M256Epu16::set(1, 2, 3, 9, 0, 8, 4, 19, 1, 2, 3, 9, 0, 8, 4, 19);
        let b = M256Epu16::set(1, 23, 8, 92, 0, 1, 1, 9, 1, 23, 8, 92, 0, 1, 1, 9);
        let res = M256Epu16::set(2, 25, 11, 101, 0, 9, 5, 28, 2, 25, 11, 101, 0, 9, 5, 28);
        assert_eq!(a+b, res);
    }

    #[test]
    fn test_add_saturating()
    {
        let a = M256Epu16::set(65534, 65534, 65534, 65534, 65534, 65534, 65534, 65534, 65534, 65534, 65534, 65534, 65534, 65534, 65534, 65534);
        let b = M256Epu16::set(2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2);
        let res = M256Epu16::set(65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535, 65535);
        assert_eq!(a + b, res);
    }

    #[test]
    fn test_sub()
    {
        let a = M256Epu16::set(1, 2, 3, 9, 0, 8, 4, 19, 1, 2, 3, 9, 0, 8, 4, 19);
        let b = M256Epu16::set(1, 2, 1, 7, 0, 1, 1, 9, 0, 1, 2, 7, 0, 5, 3, 12);
        let res = M256Epu16::set(0, 0, 2, 2, 0, 7, 3, 10, 1, 1, 1, 2, 0, 3, 1, 7);
        assert_eq!(a - b, res);
    }

    #[test]
    fn test_sub_saturating()
    {
        let a = M256Epu16::set(1, 2, 3, 9, 0, 8, 4, 19, 1, 2, 3, 9, 0, 8, 4, 19);
        let b = M256Epu16::set(2, 3, 4, 10, 10, 9, 5, 20, 2, 3, 4, 10, 10, 9, 5, 20);
        let res = M256Epu16::set(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
        assert_eq!(a - b, res);
    }

    #[test]
    fn test_shl()
    {
        let a = M256Epu16::set(16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1);   
        let res = M256Epu16::set(0, 16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2);
        assert_eq!(a << 1, res);
    }

    #[test]
    #[should_panic]
    fn test_shl_panic()
    {
        let mut _a = M256Epu16::set(16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1);   
        _a = _a << 17;
    }

    #[test]
    fn test_index()
    {
        let a = M256Epu16::set(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);
        assert_eq!(a[0], 16);
        assert_eq!(a[1], 15);
        assert_eq!(a[2], 14);
        assert_eq!(a[3], 13);
        assert_eq!(a[4], 12);
        assert_eq!(a[5], 11);
        assert_eq!(a[6], 10);
        assert_eq!(a[7], 9);
        assert_eq!(a[8], 8);
        assert_eq!(a[9], 7);
        assert_eq!(a[10], 6);
        assert_eq!(a[11], 5);
        assert_eq!(a[12], 4);
        assert_eq!(a[13], 3);
        assert_eq!(a[14], 2);
        assert_eq!(a[15], 1);
    }

    #[test]
    fn test_from_vec()
    {
        let vec = vec![16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1];
        // let res = M256Epu16::set(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);
        assert_eq!(M256Epu16::from_vec(&vec)[0], 1);
        assert_eq!(M256Epu16::from_vec(&vec)[1], 2);
        assert_eq!(M256Epu16::from_vec(&vec)[2], 3);
        assert_eq!(M256Epu16::from_vec(&vec)[3], 4);
        assert_eq!(M256Epu16::from_vec(&vec)[4], 5);
        assert_eq!(M256Epu16::from_vec(&vec)[5], 6);
    }

    #[test]
    fn test_fill()
    {
        let zero = M256Epu16::fill(0);
        assert_eq!(zero, M256Epu16::zero());
    }

    #[test]
    fn test_to_vec()
    {
        let a = M256Epu16::set(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);
        let res = vec![16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1];
        assert_eq!(a.to_vec(), res);
    }

    #[test]
    fn test_to_arr()
    {
        let a = M256Epu16::set(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);
        assert_eq!(a.to_arr(), [16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
    }

    #[test]
    fn test_get_max_epu16()
    {
        let a = M256Epu16::set(65535, 65535, 53, 9, 5, 6, 17, 8123, 9899, 27, 53, 9, 5, 6, 17, 8123);
        assert_eq!(a.get_max_m256_u16(), 65535);
    }

    #[test]
    fn test_max_epu16()
    {
        let a = M256Epu16::set(0, 12, 2, 9, 1, 2, 3, 45, 12, 22, 14, 1231, 54, 11, 87, 98);
        let b = M256Epu16::set(8, 1, 9, 879, 0, 12, 4, 78, 12, 10, 90, 56, 53, 21, 46, 65);
        let c = M256Epu16::set(7, 12, 3, 8, 1, 21, 4, 7, 3, 7, 1, 9, 912, 13223, 12, 43);

        let res = M256Epu16::set(8, 12, 9, 879, 1, 21, 4, 78, 12, 22, 90, 1231, 912, 13223, 87, 98);
        assert_eq!(max_epu16!(a, b, c), res);
    }
}
