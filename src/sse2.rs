#[cfg(target_feature = "sse2")]
pub mod sse2
{
    use std::mem::transmute;
    use std::arch::x86_64::__m128i;
    use std::arch::x86_64::_mm_set1_epi8;
    use std::arch::x86_64::_mm_xor_si128;
    use std::arch::x86_64::_mm_subs_epu8;
    use std::arch::x86_64::_mm_cmpeq_epi8;
    use std::arch::x86_64::_mm_movemask_epi8;

    #[derive(Clone, Copy)]
    pub struct M128Epu8(__m128i);

    // #[derive(Clone, Copy)]
    // struct M128Epu16(__m128i);

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
                _mm_movemask_epi8(mask) != -1
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