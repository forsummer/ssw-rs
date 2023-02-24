#[cfg(target_feature = "sse2")]
pub mod sse2
{
    use std::mem::transmute;
    use std::arch::x86_64::__m128i;
    use std::arch::x86_64::_mm_adds_epu8;
    use std::arch::x86_64::_mm_subs_epu8;
    use std::arch::x86_64::_mm_set1_epi8;
    use std::arch::x86_64::_mm_xor_si128;
    use std::arch::x86_64::_mm_cmpeq_epi8;
    use std::arch::x86_64::_mm_slli_si128;
    use std::arch::x86_64::_mm_movemask_epi8;

    #[derive(Clone, Copy)]
    pub struct M128Epu8(__m128i);

    // #[derive(Clone, Copy)]
    // struct M128Epu16(__m128i);

    impl std::ops::Add for M128Epu8
    {
        type Output = M128Epu8;

        fn add(self, rhs: Self) -> Self::Output
        {
            unsafe { M128Epu8(_mm_adds_epu8(self.0, rhs.0)) }
        }
    }

    impl std::ops::Sub for M128Epu8
    {
        type Output = M128Epu8;

        fn sub(self, rhs: Self) -> Self::Output
        {
            unsafe { M128Epu8(_mm_subs_epu8(self.0, rhs.0)) }
        }
    }

    impl std::ops::Shl<usize> for M128Epu8
    {
        type Output = M128Epu8;

        fn shl(self, rhs: usize) -> Self::Output
        {
            let shift_left_byte = |v: &mut __m128i| unsafe { _mm_slli_si128::<1>(*v) };

            let mut v = self.0;
            let mut step = rhs;
            while step != 0
            {
                shift_left_byte(&mut v);
                step = step - 1;
            }
            M128Epu8(v)
        }
    }

    impl std::cmp::PartialEq for M128Epu8
    {
        fn eq(&self, other: &Self) -> bool
        {
            unsafe { _mm_movemask_epi8(_mm_cmpeq_epi8(self.0, other.0)) == 65535 }
        }
    }

    impl std::fmt::Debug for M128Epu8
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
        {
            f.debug_list().entries(self.to_vec().iter().rev()).finish()
        }
    }

    impl M128Epu8
    {
        #[inline]
        pub fn fill(item: u8) -> M128Epu8
        {
            unsafe { transmute::<[u8; 16], M128Epu8>([item; 16]) }
        }

        #[inline]
        pub fn anyelement_gt(&self, other: &M128Epu8) -> bool
        {
            unsafe
            {
                let tmp = _mm_subs_epu8(self.0, other.0);
                let mask = _mm_cmpeq_epi8(tmp, _mm_set1_epi8(0));
                _mm_movemask_epi8(mask) != 65535
            }
        }

        #[inline]
        pub fn to_vec(self) -> Vec<u8>
        {
            unsafe { transmute::<M128Epu8, [u8; 16]>(self).to_vec() }
        }

        #[inline]
        pub fn get_max(&self) -> u8
        {
            *self.to_vec().iter().max().unwrap()
        }

        #[inline]
        pub fn position(&self, other: u8) -> usize
        {
            self.to_vec().iter().rposition(|item| *item == other).unwrap()
        }

        #[inline]
        pub fn contains(&self, other: u8) -> bool
        {
            let item = unsafe { transmute::<[u8; 16], __m128i>([other; 16]) };
            let mask = unsafe { _mm_movemask_epi8(_mm_cmpeq_epi8(self.0, item)) };
            mask != 0
        }

        #[inline]
        pub fn zero_out(&mut self)
        {
            *self = unsafe { M128Epu8(_mm_xor_si128(self.0, self.0)) };
        }
    }
}

#[cfg(test)]
#[cfg(target_feature = "sse2")]
mod test_m128_epu8
{
    use std::mem::transmute;
    use super::sse2::M128Epu8;

    use std::arch::x86_64::__m128i;
    use std::arch::x86_64::_mm_setzero_si128;

    #[test]
    fn test_fill()
    {
        let v1 = M128Epu8::fill(0);
        let v2 = unsafe { transmute::<[u8; 16], M128Epu8>([0; 16]) };
        assert_eq!(v1, v2);

        let v1 = M128Epu8::fill(1);
        let v2 = unsafe { transmute::<[u8; 16], M128Epu8>([1; 16]) };
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_anyelement_gt()
    {
        let a1 = [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2];
        let a2 = [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
        let v1 = unsafe { transmute::<[u8; 16], M128Epu8>(a1) };
        let v2 = unsafe { transmute::<[u8; 16], M128Epu8>(a2) };
        assert!(v1.anyelement_gt(&v2));

        let a1 = [0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
        let a2 = [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
        let v1 = unsafe { transmute::<[u8; 16], M128Epu8>(a1) };
        let v2 = unsafe { transmute::<[u8; 16], M128Epu8>(a2) };
        assert!(!v1.anyelement_gt(&v2));
    }

    #[test]
    fn test_to_vec()
    {
        let v1 = vec![1, 2, 3, 4 , 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let v2 = unsafe { transmute::<[u8; 16], M128Epu8>([1, 2, 3, 4 , 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]) };
        assert_eq!(v1, v2.to_vec());
    }

    #[test]
    fn test_get_max()
    {
        let v = unsafe { transmute::<[u8; 16], M128Epu8>([1, 2, 3, 4 , 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]) };
        assert_eq!(v.get_max(), 16);

        let v = unsafe { transmute::<[u8; 16], M128Epu8>([255, 2, 3, 4 , 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 255]) };
        assert_eq!(v.get_max(), 255);
    }

    #[test]
    fn test_position()
    {
        let v = unsafe { transmute::<[u8; 16], M128Epu8>([1, 2, 3, 4 , 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]) };
        assert_eq!(v.position(16), 15);

        let v = unsafe { transmute::<[u8; 16], M128Epu8>([1, 2, 3, 4 , 5, 6, 7, 8, 9, 255, 11, 12, 13, 14, 15, 16]) };
        assert_eq!(v.position(255), 9);
    }

    #[test]
    fn test_contains()
    {
        let v = unsafe { transmute::<[u8; 16], M128Epu8>([1, 2, 3, 4 , 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]) };
        assert!(v.contains(1));
        assert!(v.contains(16));
        assert!(!v.contains(20));
        assert!(!v.contains(255));
    }

    #[test]
    fn test_zero_out()
    {
        let mut v = unsafe { transmute::<[u8; 16], M128Epu8>([1, 2, 3, 4 , 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]) };
        let zero = unsafe { transmute::<__m128i, M128Epu8>(_mm_setzero_si128()) };
        v.zero_out();
        assert_eq!(v, zero);
    }
}