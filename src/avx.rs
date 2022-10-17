pub mod avx2
{    
    use std::mem::{ size_of, transmute };
    use std::arch::x86_64::__m256i;
    use std::arch::x86_64::_mm256_set_epi8;
    use std::arch::x86_64::_mm256_max_epu8;
    use std::arch::x86_64::_mm256_adds_epu8;
    use std::arch::x86_64::_mm256_adds_epu16;
    use std::arch::x86_64::_mm256_subs_epu8;
    use std::arch::x86_64::_mm256_subs_epu16;
    use std::arch::x86_64::_mm256_load_si256;
    use std::arch::x86_64::_mm256_cmpeq_epi8;
    use std::arch::x86_64::_mm256_cmpeq_epi16;
    use std::arch::x86_64::_mm256_cmpgt_epi8;
    use std::arch::x86_64::_mm256_cmpgt_epi16;
    use std::arch::x86_64::_mm256_shuffle_epi8;
    use std::arch::x86_64::_mm256_movemask_epi8;
    use std::arch::x86_64::_mm256_alignr_epi8;
    use std::arch::x86_64::_mm256_permute2x128_si256;

    #[derive(Clone, Copy)]
    pub struct M256Epu8(pub __m256i);

    #[derive(Clone, Copy)]
    pub struct M256Epu16(pub __m256i);

    impl std::convert::From<&[u8]> for M256Epu8
    {   
        fn from(s: &[u8]) -> Self
        {
            if s.len() * size_of::<u8>() != 32
            {
                panic!("The capacity of slice should equal to 32 bytes")
            }
            let mut v = [0; 32];
            v.copy_from_slice(s);
            unsafe { M256Epu8(transmute::<[u8; 32], __m256i>(v)) }
        }
    }

    impl std::convert::From<&[u16]> for M256Epu16
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
    
    impl std::iter::FromIterator<u8> for M256Epu8
    {
        #[inline]
        fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self
        {
            let v = iter.into_iter().collect::<Vec<u8>>();
            if v.len() * size_of::<u8>() != 32
            {
                panic!("The capacity of iter should equal to 32 bytes");
            }
            let ptr_rev_v = v.iter().rev().copied().collect::<Vec<u8>>().as_ptr();
            unsafe { M256Epu8(*transmute::<*const u8, *const __m256i>(ptr_rev_v)) }
        }
    }

    impl std::iter::FromIterator<u16> for M256Epu16
    {
        #[inline]
        fn from_iter<T: IntoIterator<Item = u16>>(iter: T) -> Self
        {
            let v = iter.into_iter().collect::<Vec<u16>>();
            if v.len() * size_of::<u16>() != 32
            {
                panic!("The capacity of iter should equal to 32 bytes")
            }
            let ptr_rev_v = v.iter().rev().copied().collect::<Vec<u16>>().as_ptr();
            unsafe { M256Epu16(*transmute::<*const u16, *const __m256i>(ptr_rev_v)) }
        }
    }

    impl std::cmp::PartialEq for M256Epu8
    {
        #[inline]
        fn eq(&self, other: &Self) -> bool
        {
            unsafe { _mm256_movemask_epi8(_mm256_cmpeq_epi8(self.0, other.0)) == -1 }
        }
    }

    impl std::cmp::PartialEq for M256Epu16
    {
        #[inline]
        fn eq(&self, other: &Self) -> bool
        {
            unsafe { _mm256_movemask_epi8(_mm256_cmpeq_epi16(self.0, other.0)) == -1 }
        }
    }

    impl std::cmp::PartialOrd for M256Epu8
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

    impl std::cmp::PartialOrd for M256Epu16
    {
        #[inline]
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering>
        {
            if unsafe { _mm256_movemask_epi8(_mm256_cmpeq_epi16(self.0, other.0)) == -1 }
            {
                Some(std::cmp::Ordering::Equal)
            }
            else if unsafe { _mm256_movemask_epi8(_mm256_cmpgt_epi16(self.0, other.0)) == -1 }
            {
                Some(std::cmp::Ordering::Greater)
            }
            else if unsafe { _mm256_movemask_epi8(_mm256_cmpgt_epi16(other.0, self.0)) == -1 }
            {
                Some(std::cmp::Ordering::Less)
            }
            else
            {
                None
            }
        }
    }

    impl std::ops::Add for M256Epu8
    {
        type Output = M256Epu8;

        #[inline]
        fn add(self, rhs: Self) -> Self::Output
        {
            unsafe { M256Epu8(_mm256_adds_epu8(self.0, rhs.0)) }
        }
    }

    impl std::ops::Add for M256Epu16
    {
        type Output = M256Epu16;

        #[inline]
        fn add(self, rhs: Self) -> Self::Output
        {
            unsafe { M256Epu16(_mm256_adds_epu16(self.0, rhs.0)) }
        }
    }

    impl std::ops::Sub for M256Epu8
    {
        type Output = M256Epu8;

        #[inline]
        fn sub(self, rhs: Self) -> Self::Output
        {
            unsafe { M256Epu8(_mm256_subs_epu8(self.0, rhs.0)) }
        }
    }

    impl std::ops::Sub for M256Epu16
    {
        type Output = M256Epu16;

        #[inline]
        fn sub(self, rhs: Self) -> Self::Output
        {
            unsafe { M256Epu16(_mm256_subs_epu16(self.0, rhs.0)) }
        }
    }

    impl std::ops::Index<usize> for M256Epu8
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

    impl std::ops::Index<usize> for M256Epu16
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

    impl std::fmt::Debug for M256Epu8
    {
        #[inline]
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
        {
            f.debug_list().entries(self.to_vec().iter().rev()).finish()
        }
    }

    impl std::fmt::Debug for M256Epu16
    {
        #[inline]
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
        {
            f.debug_list().entries(self.to_vec().iter().rev()).finish()
        }
    }

    impl std::fmt::Display for M256Epu8
    {
        #[inline]
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
        {
            let v = self.to_vec();
            let output = format!("{:<} {:<} {:<} {:<} {:<} {:<} {:<} {:<} \
                {:<} {:<} {:<} {:<} {:<} {:<} {:<} {:<} \
                {:<} {:<} {:<} {:<} {:<} {:<} {:<} {:<} \
                {:<} {:<} {:<} {:<} {:<} {:<} {:<} {:<}",
                v[0],  v[1],  v[2],  v[3],  v[4],  v[5],  v[6],  v[7],
                v[8],  v[9],  v[10], v[11], v[12], v[13], v[14], v[15],
                v[16], v[17], v[18], v[19], v[20], v[21], v[22], v[23],
                v[24], v[25], v[26], v[27], v[28], v[29], v[30], v[31]);
            write!(f, "{}", output)
        }
    }

    impl std::fmt::Display for M256Epu16
    {
        #[inline]
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
        {
            let v = self.to_vec();
            let output = format!("{:<} {:<} {:<} {:<} {:<} {:<} {:<} {:<} \
                {:<} {:<} {:<} {:<} {:<} {:<} {:<} {:<}",
                v[0], v[1], v[2],  v[3],  v[4],  v[5],  v[6],  v[7],
                v[8], v[9], v[10], v[11], v[12], v[13], v[14], v[15]);
            write!(f, "{}", output)
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
        pub fn to_vec(self) -> Vec<u8>
        {
            unsafe { transmute::<__m256i, [u8; 32]>(self.0).to_vec() }
        }

        #[inline]
        pub fn get_max(&self) -> u8
        {
            const IMM: i32 = 1;
            let mut tmp = unsafe { _mm256_max_epu8(self.0, _mm256_permute2x128_si256::<IMM>(self.0, self.0)) };

            let mask = unsafe 
            { [ _mm256_set_epi8(-127, -127, -127, -127, -127, -127, -127, -127, -127, -127, -127, -127, -127, -127, -127, -127,
                    7, 6, 5, 4, 3, 2, 1, 0, 15, 14, 13, 12, 11, 10, 9, 8),
                _mm256_set_epi8(-127, -127, -127, -127, -127, -127, -127, -127, -127, -127, -127, -127, -127, -127, -127, -127,
                    -127, -127, -127, -127, -127, -127, -127, -127, 3, 2, 1, 0, 7, 6, 5, 4),
                _mm256_set_epi8(-127, -127, -127, -127, -127, -127, -127, -127, -127, -127, -127, -127, -127, -127, -127, -127,
                    -127, -127, -127, -127, -127, -127, -127, -127, -127, -127, -127, -127, 1, 0, 3, 2),
                _mm256_set_epi8(-127, -127, -127, -127, -127, -127, -127, -127, -127, -127, -127, -127, -127, -127, -127, -127,
                    -127, -127, -127, -127, -127, -127, -127, -127, -127, -127, -127, -127, -127, -127, 0, 1) ]
            };

            for m in mask.iter()
            {
                tmp = unsafe { _mm256_max_epu8(tmp, _mm256_shuffle_epi8(tmp, *m)) }
            }
            unsafe { *transmute::<*const __m256i, *const u8>(&tmp as *const __m256i) }
        }

        #[inline]
        pub fn position(&self, item: u8) -> usize
        {
            self.to_vec().iter().rposition(|x| *x == item).unwrap()
        }

        #[inline]
        pub fn contains(&self, other: u8) -> bool
        {
            let item = M256Epu8::fill(other);
            let mask = unsafe { _mm256_movemask_epi8(_mm256_cmpeq_epi8(self.0, item.0)) };
            mask != 0
        }

        #[inline]
        pub fn shift_left_byte(&mut self)
        {
            unsafe 
            {
                let mask = _mm256_permute2x128_si256::<8>(self.0, self.0);
                *self = M256Epu8(_mm256_alignr_epi8::<15>(self.0, mask))
            }
        }

        #[inline]
        pub fn zero_out(&mut self)
        {
            *self = unsafe { M256Epu8(_mm256_permute2x128_si256::<255>(self.0, self.0)) }
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
        pub fn to_vec(self) -> Vec<u16>
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

        #[inline]
        pub fn contains(&self, other: u16) -> bool
        {
            let item = M256Epu16::fill(other);
            let mask = unsafe { _mm256_movemask_epi8(_mm256_cmpeq_epi16(self.0, item.0)) };
            mask != 0
        }

        #[inline]
        pub fn shift_left_bytex2(&mut self)
        {
            unsafe
            {
                let mask = _mm256_permute2x128_si256::<8>(self.0, self.0);
                *self = M256Epu16(_mm256_alignr_epi8::<14>(self.0, mask))
            }
        }

        #[inline]
        pub fn zero_out(&mut self)
        {
            *self = unsafe { M256Epu16(_mm256_permute2x128_si256::<255>(self.0, self.0)) }
        }
    }

    macro_rules! max_epu8
    {
        ($x:expr) => ( $x );
        ($x: expr, $($xs: expr), +)  => 
        { 
            {
                unsafe
                {
                    M256Epu8(std::arch::x86_64::_mm256_max_epu8($x.0, max_epu8!( $($xs.0),+ )))
                }
            }
        };
    }
    pub (crate) use max_epu8;

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
    fn test_shift_left_bytex2()
    {
        let mut a = unsafe { M256Epu16(_mm256_set_epi16(16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1)) };
        let res = unsafe { M256Epu16(_mm256_set_epi16( 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0)) };
        a.shift_left_bytex2();
        assert_eq!(a, res);
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

    #[test]
    fn test_contains()
    {
        let a = M256Epu16(unsafe {_mm256_set_epi16(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16)});
        assert_eq!(a.contains(1), true);
        assert_eq!(a.contains(2), true);
        assert_eq!(a.contains(3), true);
        assert_eq!(a.contains(4), true);
        assert_eq!(a.contains(5), true);
        assert_eq!(a.contains(6), true);
        assert_eq!(a.contains(7), true);
        assert_eq!(a.contains(8), true);
        assert_eq!(a.contains(9), true);
        assert_eq!(a.contains(10), true);
        assert_eq!(a.contains(11), true);
        assert_eq!(a.contains(12), true);
        assert_eq!(a.contains(13), true);
        assert_eq!(a.contains(14), true);
        assert_eq!(a.contains(15), true);
        assert_eq!(a.contains(16), true);
        assert_eq!(a.contains(17), false);
        assert_eq!(a.contains(18), false);
        assert_eq!(a.contains(19), false);
        assert_eq!(a.contains(20), false);
        assert_eq!(a.contains(21), false);
    }
}

#[cfg(test)]
mod test_m256_epu8
{
    use std::mem::transmute;
    use std::arch::x86_64::__m256i;
    use std::arch::x86_64::_mm256_set_epi8;

    use super::avx2::M256Epu8;
    use super::avx2::max_epu8;

    #[test]
    fn test_from_u8()
    {
        let a = unsafe { _mm256_set_epi8(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
            17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32) };
        let tmp = [32, 31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17,
            16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1];
        let b = unsafe { transmute::<[u8; 32], __m256i>(tmp) };

        assert_eq!(M256Epu8(a), M256Epu8(b));
    }

    #[test]
    fn test_eq()
    {
        let a = unsafe { _mm256_set_epi8(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
            17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32) };
        let b = unsafe { _mm256_set_epi8(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
            17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32) };
        assert_eq!(M256Epu8(a), M256Epu8(b))
    }

    #[test]
    fn test_ne()
    {
        let a = unsafe { M256Epu8(_mm256_set_epi8(32, 31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17,
            16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1)) };
        let b = unsafe { M256Epu8(_mm256_set_epi8(32, 31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17,
            16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 0)) };
        assert_ne!(a, b);
    }

    #[test]
    fn test_gt()
    {
        let a = unsafe { M256Epu8(_mm256_set_epi8(32, 31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17,
            16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1)) };
        let b = unsafe { M256Epu8(_mm256_set_epi8(31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17, 16,
            15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0)) };
        assert!(a > b);
    }

    #[test]
    fn test_gt_false()
    {
        let a = unsafe { M256Epu8(_mm256_set_epi8(0, 31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17,
            16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1)) };
        let b = unsafe { M256Epu8(_mm256_set_epi8(31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17, 16,
            15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0)) };
    assert!(!(a > b));
    }

    #[test]
    fn test_le()
    {
        let a = unsafe { M256Epu8(_mm256_set_epi8(32, 31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17,
            16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1)) };
        let b = unsafe { M256Epu8(_mm256_set_epi8(33, 32, 31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17,
            16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2)) };
        assert!(a < b);
    }

    #[test]
    fn test_le_false()
    {
        let a = unsafe { M256Epu8(_mm256_set_epi8(32, 31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17,
            16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1)) };
        let b = unsafe { M256Epu8(_mm256_set_epi8(0, 32, 31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17,
            16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2)) };
        assert!(!(a < b));
    }

    #[test]
    fn test_add()
    {
        let a = unsafe { M256Epu8(transmute::<[u8; 32], __m256i>([1; 32])) };
        let b = unsafe { M256Epu8(transmute::<[u8; 32], __m256i>([2; 32])) };
        let res = unsafe { M256Epu8(transmute::<[u8; 32], __m256i>([3; 32])) };
        assert_eq!(a + b, res);
    }

    #[test]
    fn test_add_saturating()
    {
        let a = unsafe { M256Epu8(transmute::<[u8; 32], __m256i>([250; 32])) };
        let b = unsafe { M256Epu8(transmute::<[u8; 32], __m256i>([10; 32])) };
        let res = unsafe { M256Epu8(transmute::<[u8; 32], __m256i>([255; 32])) };
        assert_eq!(a + b, res);
    }

    #[test]
    fn test_sub()
    {
        let a = unsafe { M256Epu8(transmute::<[u8; 32], __m256i>([250; 32])) };
        let b = unsafe { M256Epu8(transmute::<[u8; 32], __m256i>([10; 32])) };
        let res = unsafe { M256Epu8(transmute::<[u8; 32], __m256i>([240; 32])) };
        assert_eq!(a - b, res);
    }

    #[test]
    fn test_sub_saturating()
    {
        let a = unsafe { M256Epu8(transmute::<[u8; 32], __m256i>([10; 32])) };
        let b = unsafe { M256Epu8(transmute::<[u8; 32], __m256i>([250; 32])) };
        let res = unsafe { M256Epu8(transmute::<[u8; 32], __m256i>([0; 32])) };
        assert_eq!(a - b, res);
    }

    #[test]
    fn test_index()
    {
        let tmp = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
        17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32];
        let a = unsafe { M256Epu8(transmute::<[u8; 32], __m256i>(tmp)) };
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
        assert_eq!(a[16], 17);
        assert_eq!(a[17], 18);
        assert_eq!(a[18], 19);
        assert_eq!(a[19], 20);
        assert_eq!(a[20], 21);
        assert_eq!(a[21], 22);
        assert_eq!(a[22], 23);
        assert_eq!(a[23], 24);
        assert_eq!(a[24], 25);
        assert_eq!(a[25], 26);
        assert_eq!(a[26], 27);
        assert_eq!(a[27], 28);
        assert_eq!(a[28], 29);
        assert_eq!(a[29], 30);
        assert_eq!(a[30], 31);
        assert_eq!(a[31], 32);
    }

    #[test]
    fn test_fill()
    {
        let a = unsafe { M256Epu8(transmute::<[u8; 32], __m256i>([255; 32])) };
        assert_eq!(M256Epu8::fill(255), a);
    }

    #[test]
    fn test_to_vec()
    {
        let a = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
        17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32];
        let b = unsafe { M256Epu8(_mm256_set_epi8(32, 31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17,
            16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1)) };
        assert_eq!(a, b.to_vec());
    }

    #[test]
    fn test_max()
    {
        let v1 = [0, 32, 30, 29, 28, 7, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17,
            16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 25];
        let v2 = [255, 31, 30, 29, 120, 7, 26, 25, 24, 23, 22, 21, 20, 0, 18, 17,
            16, 15, 1, 13, 12, 99, 10, 0, 8, 7, 6, 5, 4, 3, 7, 25];
        let res = [255, 32, 30, 29, 120, 7, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17,
            16, 15, 14, 13, 12, 99, 10, 9, 8, 7, 6, 5, 4, 3, 7, 25];
        let a = M256Epu8::from(&v1[..]);
        let b = M256Epu8::from(&v2[..]);
        let res = M256Epu8::from(&res[..]);
        assert_eq!(max_epu8!(a, b), res);
    }

    #[test]
    fn test_get_max_epu8()
    {
        let v = [0, 31, 30, 29, 28, 7, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17,
            16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 25];
        let a = M256Epu8::from(&v[..]);
        assert_eq!(a.get_max(), 31);
    }

    #[test]
    fn test_position()
    {
        let a = unsafe { M256Epu8(_mm256_set_epi8(32, 31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17,
            16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1)) };
        assert_eq!(a.position(32), 31);
    }

    #[test]
    fn test_contains()
    {
        let a = unsafe { M256Epu8(_mm256_set_epi8(32, 31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17,
            16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1)) };
        assert_eq!(a.contains(32), true);
        assert_eq!(a.contains(33), false);
    }

    #[test]
    fn test_shift_left_byte()
    {
        let v1 = [32, 31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17,
            16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1];
        let v2 = [0, 32, 31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17,
        16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2];
        let mut a = M256Epu8::from(&v1[..]);
        let b = M256Epu8::from(&v2[..]);
        a.shift_left_byte();
        assert_eq!(a, b);
    }
}