pub mod avx2
{    
    use std::ops::Add;
    use std::ops::Sub;
    use std::ops::Shl;
    use std::ops::Index;
    use std::fmt::Debug;
    use std::mem::size_of;
    use std::mem::transmute;
    use std::arch::x86_64::__m256i;
    use std::arch::x86_64::_mm256_adds_epu8;
    use std::arch::x86_64::_mm256_adds_epu16;
    use std::arch::x86_64::_mm256_subs_epu8;
    use std::arch::x86_64::_mm256_subs_epu16;
    use std::arch::x86_64::_mm256_load_si256;
    use std::arch::x86_64::_mm256_cmpeq_epi8;
    use std::arch::x86_64::_mm256_cmpeq_epi16;
    use std::arch::x86_64::_mm256_cmpgt_epi8;
    use std::arch::x86_64::_mm256_cmpgt_epi16;
    use std::arch::x86_64::_mm256_movemask_epi8;
    use std::arch::x86_64::_mm256_alignr_epi8;
    use std::arch::x86_64::_mm256_permute2x128_si256;

    #[derive(Clone, Copy)]
    pub struct M256Epu8(pub __m256i);

    #[derive(Clone, Copy)]
    pub struct M256Epu16(pub __m256i);

    impl From<&[u8]> for M256Epu8
    {   
        fn from(s: &[u8]) -> Self
        {
            if s.len() * size_of::<u8>() != 32
            {
                panic!("The capacity of slice should equal to 32 bytes")
            }
            let v = s.iter().rev().map(|item| *item).collect::<Vec<u8>>();
            unsafe { M256Epu8(*transmute::<*const u8, *const __m256i>(v.as_ptr())) }
        }
    }

    impl From<&[u16]> for M256Epu16
    {
        #[inline]
        fn from(s: &[u16]) -> Self
        {
            if s.len() * size_of::<u16>() != 32
            {
                panic!("The capacity of slice should equal to 32 bytes")
            }
            let mut v: [u16; 16] = [0; 16];
            v.copy_from_slice(s);
            unsafe { M256Epu16(transmute::<[u16; 16], __m256i>(v)) }
        }
    }
    
    impl FromIterator<u8> for M256Epu8
    {
        #[inline]
        fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self
        {
            let v = iter.into_iter().collect::<Vec<u8>>();
            if v.len() * size_of::<u8>() != 32
            {
                panic!("The capacity of iter should equal to 32 bytes");
            }
            let ptr_rev_v = v.iter().rev().map(|item| *item).collect::<Vec<u8>>().as_ptr();
            unsafe { M256Epu8(*transmute::<*const u8, *const __m256i>(ptr_rev_v)) }
        }
    }

    impl FromIterator<u16> for M256Epu16
    {
        #[inline]
        fn from_iter<T: IntoIterator<Item = u16>>(iter: T) -> Self
        {
            let v = iter.into_iter().collect::<Vec<u16>>();
            if v.len() * size_of::<u16>() != 32
            {
                panic!("The capacity of iter should equal to 32 bytes")
            }
            let ptr_rev_v = v.iter().rev().map(|item| *item).collect::<Vec<u16>>().as_ptr();
            unsafe { M256Epu16(*transmute::<*const u16, *const __m256i>(ptr_rev_v)) }
        }
    }

    impl PartialEq for M256Epu8
    {
        #[inline]
        fn eq(&self, other: &Self) -> bool
        {
            unsafe { _mm256_movemask_epi8(_mm256_cmpeq_epi8(self.0, other.0)) == -1 }
        }

        #[inline]
        fn ne(&self, other: &Self) -> bool
        {
            !unsafe { _mm256_movemask_epi8(_mm256_cmpeq_epi8(self.0, other.0)) == -1 }
        }
    }

    impl PartialEq for M256Epu16
    {
        #[inline]
        fn eq(&self, other: &Self) -> bool
        {
            unsafe { _mm256_movemask_epi8(_mm256_cmpeq_epi8(self.0, other.0)) == -1 }
        }

        #[inline]
        fn ne(&self, other: &Self) -> bool
        {
            !unsafe { _mm256_movemask_epi8(_mm256_cmpeq_epi16(self.0, other.0)) == -1 }
        }
    }

    impl PartialOrd for M256Epu8
    {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering>
        {
            if unsafe { _mm256_movemask_epi8(_mm256_cmpeq_epi8(self.0, other.0)) == -1 }
            {
                Some(std::cmp::Ordering::Equal)
            }
            else if unsafe { _mm256_movemask_epi8(_mm256_cmpgt_epi8(self.0, other.0)) == -1 }
            {
                Some(std::cmp::Ordering::Greater)
            }
            else if unsafe { _mm256_movemask_epi8(_mm256_cmpgt_epi8(other.0, self.0)) == -1 }
            {
                Some(std::cmp::Ordering::Less)
            }
            else
            {
                None
            }
        }
    }

    impl PartialOrd for M256Epu16
    {
        #[inline]
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

    impl Add for M256Epu8
    {
        type Output = M256Epu8;

        #[inline]
        fn add(self, rhs: Self) -> Self::Output
        {
            unsafe { M256Epu8(_mm256_adds_epu8(self.0, rhs.0)) }
        }
    }

    impl Add for M256Epu16
    {
        type Output = M256Epu16;

        #[inline]
        fn add(self, rhs: Self) -> Self::Output
        {
            unsafe { M256Epu16(_mm256_adds_epu16(self.0, rhs.0)) }
        }
    }

    impl Sub for M256Epu8
    {
        type Output = M256Epu8;

        #[inline]
        fn sub(self, rhs: Self) -> Self::Output
        {
            unsafe { M256Epu8(_mm256_subs_epu8(self.0, rhs.0)) }
        }
    }

    impl Sub for M256Epu16
    {
        type Output = M256Epu16;

        #[inline]
        fn sub(self, rhs: Self) -> Self::Output
        {
            unsafe { M256Epu16(_mm256_subs_epu16(self.0, rhs.0)) }
        }
    }

    impl Shl<usize> for M256Epu8
    {
        type Output = M256Epu8;

        #[inline]
        fn shl(self, rhs: usize) -> Self::Output
        {
            if rhs > 32 { panic!("Out of bound") }
            let tail = vec![0; rhs];
            let remainder = unsafe
            {
                transmute::<__m256i, [u8; 32]>(self.0)
                    .to_vec()
                    .drain(rhs..)
                    .collect::<Vec<u8>>() 
            };
            let m256 = unsafe { *transmute::<*const u8, *const __m256i>([&remainder[..], &tail[..]].concat().as_ptr()) };
            M256Epu8(m256)
        }
    }

    impl Index<usize> for M256Epu8
    {
        type Output = u8;

        #[inline]
        fn index(&self, index: usize) -> &Self::Output
        {
            if index > 31
            {
                panic!("Index out of bound");
            }
            unsafe { &*transmute::<*const __m256i, *const u8>(&self.0).add(index) }
        }
    }

    impl Index<usize> for M256Epu16
    {
        type Output = u16;

        #[inline]
        fn index(&self, index: usize) -> &Self::Output
        {
            if index > 15
            {
                panic!("Index out of bound");
            }
            unsafe { &*transmute::<*const __m256i, *const u16>(&self.0).add(index) }
        }
    }

    impl Debug for M256Epu8
    {
        #[inline]
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
        {
            f.debug_list().entries(self.to_vec().iter().rev()).finish()
        }
    }

    impl Debug for M256Epu16
    {
        #[inline]
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
        {
            f.debug_list().entries(self.to_vec().iter().rev()).finish()
        }
    }

    #[allow(dead_code)]
    impl M256Epu8
    {
        #[inline]
        pub fn fill(item: u8) -> M256Epu8
        {
            unsafe { M256Epu8(_mm256_load_si256(&transmute::<[u8; 32], __m256i>([item; 32]))) }
        }

        #[inline]
        pub fn to_vec(&self) -> Vec<u8>
        {
            unsafe { transmute::<__m256i, [u8; 32]>(self.0).iter().rev().map(|item| *item).collect::<Vec<u8>>() }
        }

        #[inline]
        pub fn get_max(&self) -> u8
        {
            *self.to_vec().iter().max().unwrap()
        }

        #[inline]
        pub fn position(&self, item: u8) -> usize
        {
            self.to_vec().iter().rposition(|x| *x == item).unwrap()
        }
    }

    #[allow(dead_code)]
    impl M256Epu16
    {
        #[inline]
        pub fn fill(item: u16) -> M256Epu16
        {
            unsafe { M256Epu16(_mm256_load_si256(&transmute::<[u16; 16], __m256i>([item; 16]))) }
        }

        #[inline]
        pub fn to_vec(&self) -> Vec<u16>
        {
            unsafe { transmute::<__m256i, [u16; 16]>(self.0).to_vec() }
        }

        #[inline]
        pub fn get_max(&self) -> u16
        {
            *self.to_vec().iter().max().unwrap()
        }

        #[inline]
        pub fn position(&self, item: u16) -> usize
        {
            self.to_vec().iter().rposition(|x| *x == item).unwrap()
        }

        pub fn contains(&self, other: u16) -> bool
        {
            let item = M256Epu16::fill(other);
            let mask = unsafe { _mm256_movemask_epi8(_mm256_cmpeq_epi16(self.0, item.0)) };
            mask != 0
        }

        pub fn shift_left_byte(self) -> M256Epu16
        {
            unsafe
            {
                let mask = _mm256_permute2x128_si256::<8>(self.0, self.0);
                M256Epu16(_mm256_alignr_epi8::<14>(self.0, mask))
            }
        }
    }

    #[macro_export]
    macro_rules! max_epu8
    {
        ( $ ( $arr: expr ), * ) => 
        { 
            {
                let mut max = M256Epu8::zero();
                use std::arch::x86_64::_mm256_max_epu8;
                let max_arr = |a: M256Epu8, b: M256Epu8| 
                {
                    unsafe { _mm256_max_epu8(a.0, b.0) }
                };
                $( max = M256Epu8(max_arr(max, $arr)); )*
                max
            }
        };
    }

    macro_rules! max_epu16
    {
        ($x:expr) => ( $x );
        ($x: expr, $($xs: expr), +)  => 
        { 
            {
                unsafe
                {
                    M256Epu16(std::arch::x86_64::_mm256_max_epu16($x.0, max_epu16!( $($xs.0),+ )))
                }
            }
        };
    }

    pub (crate) use max_epu16;
}

#[cfg(test)]
mod test_m256_epu16
{
    use std::arch::x86_64::_mm256_set_epi16;
    use std::arch::x86_64::_mm256_set1_epi16;
    use super::avx2::M256Epu16;

    #[test]
    fn test_eq()
    {
        let a = M256Epu16(unsafe {_mm256_set_epi16(9, 21, 2, 0, 3, 1, 9, 21, 9, 21, 2, 0, 3, 1, 9, 21)});
        let b = M256Epu16(unsafe {_mm256_set_epi16(9, 21, 2, 0, 3, 1, 9, 21, 9, 21, 2, 0, 3, 1, 9, 21)});
        assert_eq!(a, b);
    }

    #[test]
    fn test_ne()
    {
        let a = M256Epu16(unsafe {_mm256_set_epi16(9, 21, 2, 0, 3, 1, 9, 21, 9, 21, 2, 0, 3, 1, 9, 21)});
        let b = M256Epu16(unsafe {_mm256_set_epi16(7, 31, 2, 0, 3, 1, 9, 21, 9, 21, 2, 0, 3, 1, 9, 21)});
        assert_ne!(a, b);
    }

    #[test]
    fn test_gt()
    {
        let a = M256Epu16(unsafe {_mm256_set_epi16(9, 1, 2, 4, 5, 8, 5, 1, 8, 20, 3, 8, 9, 9, 4, 9)});
        let b = M256Epu16(unsafe {_mm256_set_epi16(8, 0, 1, 3, 4, 7, 1, 0, 7, 1, 2, 5, 8, 4, 3, 2)});
        assert!(a > b);
    }

    #[test]
    fn test_gt_false()
    {
        let a = M256Epu16(unsafe {_mm256_set_epi16(9, 1, 2, 4, 5, 8, 5, 1, 8, 20, 3, 8, 9, 9, 4, 9)});
        let b = M256Epu16(unsafe {_mm256_set_epi16(8, 1, 1, 3, 4, 7, 1, 0, 7, 1, 2, 5, 8, 4, 3, 2)});
        assert!(!(a > b));
    }

    #[test]
    fn test_le()
    {
        let a = M256Epu16(unsafe {_mm256_set_epi16(9, 1, 2, 4, 5, 8, 5, 1, 8, 20, 3, 8, 9, 9, 4, 9)});
        let b = M256Epu16(unsafe {_mm256_set_epi16(8, 0, 1, 3, 4, 7, 1, 0, 7, 1, 2, 5, 8, 4, 3, 2)});
        assert!(b < a);
    }

    #[test]
    fn test_le_false()
    {
        let a = M256Epu16(unsafe {_mm256_set_epi16(9, 1, 2, 4, 5, 8, 5, 1, 8, 20, 3, 8, 9, 9, 4, 9)});
        let b = M256Epu16(unsafe {_mm256_set_epi16(8, 1, 1, 3, 4, 7, 1, 0, 7, 1, 2, 5, 8, 4, 3, 2)});
        assert!(!(b < a));
    }

    #[test]
    fn test_zero()
    {
        let a = M256Epu16::fill(0);
        let b = M256Epu16(unsafe {_mm256_set_epi16(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0)});
        assert_eq!(a, b);
    }

    #[test]
    fn test_add()
    {
        let a = M256Epu16(unsafe {_mm256_set_epi16(1, 2, 3, 9, 0, 8, 4, 19, 1, 2, 3, 9, 0, 8, 4, 19)});
        let b = M256Epu16(unsafe {_mm256_set_epi16(1, 23, 8, 92, 0, 1, 1, 9, 1, 23, 8, 92, 0, 1, 1, 9)});
        let res = M256Epu16(unsafe {_mm256_set_epi16(2, 25, 11, 101, 0, 9, 5, 28, 2, 25, 11, 101, 0, 9, 5, 28)});
        assert_eq!(a+b, res);
    }

    #[test]
    fn test_add_saturating()
    {
        let a = M256Epu16::fill(65535);
        let b = M256Epu16::fill(1);
        let res = M256Epu16::fill(65535);
        assert_eq!(a + b, res);
    }

    #[test]
    fn test_sub()
    {
        let a = M256Epu16(unsafe {_mm256_set_epi16(1, 2, 3, 9, 0, 8, 4, 19, 1, 2, 3, 9, 0, 8, 4, 19)});
        let b = M256Epu16(unsafe {_mm256_set_epi16(1, 2, 1, 7, 0, 1, 1, 9, 0, 1, 2, 7, 0, 5, 3, 12)});
        let res = M256Epu16(unsafe {_mm256_set_epi16(0, 0, 2, 2, 0, 7, 3, 10, 1, 1, 1, 2, 0, 3, 1, 7)});
        assert_eq!(a - b, res);
    }

    #[test]
    fn test_sub_saturating()
    {
        let a = M256Epu16::fill(1);
        let b = M256Epu16::fill(2);
        let res = M256Epu16::fill(0);
        assert_eq!(a - b, res);
    }

    #[test]
    fn test_index()
    {
        let a = M256Epu16(unsafe {_mm256_set_epi16(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16)});
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
    fn test_from()
    {
        let v = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let a = M256Epu16::from(&v[..]);
        assert_eq!(a[0], 1);
        assert_eq!(a[1], 2);
        assert_eq!(a[2], 3);
        assert_eq!(a[3], 4);
        assert_eq!(a[4], 5);
        assert_eq!(a[5], 6);
        assert_eq!(a[6], 7);
        assert_eq!(a[7], 8);
        assert_eq!(a[8], 9);
        assert_eq!(a[9], 10);
        assert_eq!(a[10], 11);
        assert_eq!(a[11], 12);
        assert_eq!(a[12], 13);
        assert_eq!(a[13], 14);
        assert_eq!(a[14], 15);
        assert_eq!(a[15], 16);
    }

    #[test]
    fn test_fill()
    {
        let a = M256Epu16::fill(0);
        assert_eq!(a, M256Epu16::fill(0));

        let b = M256Epu16::fill(10);
        assert_eq!(b, unsafe { M256Epu16(_mm256_set1_epi16(10)) })
    }

    #[test]
    fn test_to_vec()
    {
        let a = M256Epu16(unsafe {_mm256_set_epi16(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16)});
        let res = vec![16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1];
        assert_eq!(a.to_vec(), res);
    }

    #[test]
    fn test_get_max_epu16()
    {
        let a = M256Epu16(unsafe {_mm256_set_epi16(723, 56, 53, 9, 5, 6, 17, 8123, 9899, 27, 53, 9, 5, 6, 17, 8123)});
        assert_eq!(a.get_max(), 9899);
    }

    #[test]
    fn test_position()
    {
        let a = M256Epu16(unsafe {_mm256_set_epi16(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16)});
        assert_eq!(a.position(1), 15);
        assert_eq!(a.position(2), 14);
        assert_eq!(a.position(3), 13);
        assert_eq!(a.position(4), 12);
        assert_eq!(a.position(5), 11);
        assert_eq!(a.position(6), 10);
        assert_eq!(a.position(7), 9);
        assert_eq!(a.position(8), 8);
        assert_eq!(a.position(9), 7);
        assert_eq!(a.position(10), 6);
        assert_eq!(a.position(11), 5);
        assert_eq!(a.position(12), 4);
        assert_eq!(a.position(13), 3);
        assert_eq!(a.position(14), 2);
        assert_eq!(a.position(15), 1);
        assert_eq!(a.position(16), 0);
    }
}

#[cfg(test)]
mod test_m256_epu8
{
    #[test]
    fn test_eq()
    {

    }
}