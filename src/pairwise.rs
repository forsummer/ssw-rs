macro_rules! max
{
    ($x:expr) => ( $x );
    ($x:expr, $($xs:expr),+) =>
    {
        std::cmp::max($x, max!( $($xs),+ ))
    };
}

macro_rules! min
{
    ($x:expr) => ( $x );
    ($x:expr, $($xs:expr),+) =>
    {
        std::cmp::min($x, min!( $($xs),+ ))
    };
}

macro_rules! get_unchecked
{
    ($v:expr, $i:expr) =>
    {
        unsafe { $v.get_unchecked($i) }
    };
}

macro_rules! get_mut_unchecked
{
    ($v:expr, $i:expr) =>
    {
        unsafe { $v.get_unchecked_mut($i) }
    };
}

/// Used to control behavior of [`smith_waterman_avx2`](fn@crate::pairwise::smith_waterman_avx2)
/// and [`smith_waterman_scalar`](fn@crate::pairwise::smith_waterman_scalar) function
pub enum AlignFlag { End, Path }

enum AlignEnd
{
    U8  { var: u8,  pos: (usize, usize) },
    U16 { var: u16, pos: (usize, usize) },
    U32 { var: u32, pos: (usize, usize) },
}

/// Defines several types of errors that can occur when running alignment
/// 
/// `AlignErr` already implements [`std::fmt::Display`] trait, so it can be output via [`println!`]
/// 
/// ### Variants
/// * `OverFlow`: Numerical overflow error during calculation
/// * `IllegallChar`: Database or query sequence contain illegal character
/// * `GetScoreErr`: There is no corresponding score for character pairs in the scoring rules
pub enum AlignErr
{
    /// Numerical overflow error during calculation
    OverFlow    { file: String, line: usize, msg: String },

    /// Database or query sequence contain illegal character
    IllegalChar { file: String, line: usize, msg: String },

    /// There is no corresponding score for character pairs in the scoring rules
    GetScoreErr { file: String, line: usize, msg: String },
}

impl std::fmt::Debug for AlignErr
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        let (file, line, msg) = match self
        {
            AlignErr::OverFlow    { file, line, msg } => (file, line, msg),
            AlignErr::IllegalChar { file, line, msg } => (file, line, msg),
            AlignErr::GetScoreErr { file, line, msg } => (file, line, msg),
        };

        f.debug_struct("Error")
            .field("file", file)
            .field("line", line)
            .field("msg", msg)
            .finish()
    }
}

impl std::fmt::Display for AlignErr
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        let (file, line, msg) = match self
        {
            AlignErr::OverFlow    { file, line, msg } => (file, line, msg),
            AlignErr::IllegalChar { file, line, msg } => (file, line, msg),
            AlignErr::GetScoreErr { file, line, msg } => (file, line, msg),
        };

        write!(f, "error: {} in {}, {}", msg, file, line)
    }
}

/// Store alignment result of smith-waterman.
/// If only find the best alignment endpoint with [`smith_waterman_avx2`](fn@crate::pairwise::smith_waterman_avx2),
/// then `AlignResult` only contain the **end position** of **best alignment** and **optimal alignment score**.
/// Otherwise, `AlignResult` will also contain **start** and **end** position of best aignment on **database sequence** and **query sequence** and **best tracing back path** on sequence.
/// 
/// ### Fields
/// * `d_start`: Best alignment start position on **database sequence**
/// * `q_start`: Start position on **query sequence**
/// * `d_end`: Best alignment end position on **database sequence**
/// * `q_end`: End position on **query sequence**
/// * `d_best`: Best tracing back path on **database sequence**
/// * `q_best`: Best tracing back path on **query sequence**
/// * `opt`: **Optimal score** of pairwise alignment
pub struct AlignResult
{
    /// Best alignment start position on **database sequence**
    pub d_start: Option<usize>,

    /// Start position on **query sequence**
    pub q_start: Option<usize>,

    /// Best alignment end position on **database sequence**
    pub d_end: usize,

    /// End position on **query sequence**
    pub q_end: usize,

    /// Best tracing back path on **database sequence**
    pub d_best: Option<Vec<u8>>,

    /// Best tracing back path on **query sequence**
    pub q_best: Option<Vec<u8>>,

    /// **Optimal score** of pairwise alignment
    pub opt: u32,

    // If `flag` equal to `AlignFlag::End`, `AlignResult` will only contain optimal score and end position of alignment.
    // If `flag` equal to `AlignFlag::Path`, `AlignResult` will also contain start position and best trace path of alignment.
    // `std::fmt::Display` will determine how to print the `AlignResult` based on the value of the `flag` variable.
    flag: AlignFlag
}

impl std::fmt::Display for AlignResult
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        if let AlignFlag::End = self.flag
        {
            let align_res = tabular::Table::new("\n{:<} {:<}, {:<} {:<}, {:<} {:<}")
                .with_row(tabular::row!(
                    "optimal_alignment_score:", self.opt,
                    "d_end:", self.d_end,
                    "q_end:", self.q_end));
            return write!(f, "{}", align_res)
        }

        if let AlignFlag::Path = self.flag
        {
            let align_res = tabular::Table::new("\n{:<} {:<}, {:<} {:<}, {:<} {:<}, {:<} {:<}, {:<} {:<}\n")
                .with_row(tabular::row!(
                    "optimal_alignment_score:", self.opt,
                    "d_start", self.d_start.expect("Should contain d_best start position"),
                    "d_end:", self.d_end,
                    "q_start", self.q_start.expect("Should contain q_best start position"),
                    "q_end:", self.q_end));
            
            let d_best = self.d_best.as_ref().expect("Should contain d_best");
            let q_best = self.q_best.as_ref().expect("Should contain q_best");

            let mut sign = Vec::new();
            for (r1, r2) in d_best.iter().zip(q_best.iter())
            {
                let s = if [r1, r2].contains(&&(b'-')) {b' '} else if r1 == r2 {b'|'} else {b'*'};
                sign.push(s);
            }
            
            let seg_len = 60;

            let mut d_seg_start = self.d_start.unwrap();
            let mut q_seg_start = self.q_start.unwrap();
            let mut best = tabular::Table::new("{:<} {:<} {:<} {:<}");
            for (d, (s, q)) in d_best.chunks(seg_len)
                .zip(sign.chunks(seg_len)
                .zip(q_best.chunks(seg_len)))
            {
                let d_gap_num = d.iter().filter(|x| **x == b'-').count();
                let q_gap_num = q.iter().filter(|x| **x == b'-').count();

                let d_seg_end = d_seg_start + d.len() - d_gap_num;
                let q_seg_end = q_seg_start + q.len() - q_gap_num;

                best.add_row(tabular::row!("d_best:", d_seg_start, String::from_utf8(d.to_vec()).unwrap(), d_seg_end-1))
                    .add_row(tabular::row!("", "", String::from_utf8(s.to_vec()).unwrap(), ""))
                    .add_row(tabular::row!("q_best:", q_seg_start, String::from_utf8(q.to_vec()).unwrap(), q_seg_end-1))
                    .add_row(tabular::row!("", "", "", ""));

                d_seg_start = d_seg_end;
                q_seg_start = q_seg_end;
            }

            return write!(f, "{}{}", align_res, best)
        }

        unreachable!()
    }
}

mod sw_avx2
{
    use std::mem::swap;

    use crate::avx::avx2::{ M256Epu8, M256Epu16, max_epu8, max_epu16 };
    use crate::pairwise::{ AlignEnd, AlignErr, AlignFlag, AlignResult };
    
    enum Profile
    {
        Byte { bias: u8,  profile: Vec<Vec<M256Epu8>>  },
        Word { bias: u16, profile: Vec<Vec<M256Epu16>> },
    }
    
    enum ProfileType { Epu8, Epu16 }

    fn query_profile<S>(d: &[u8], q: &[u8], p: ProfileType, f: S) -> Result<Profile, AlignErr>
    where
        S: Fn(u8, u8) -> Option<i8>
    {
        let mut bitmap_d: Vec<u8> = vec![0; 27];
        d.iter().for_each(|r| bitmap_d[(*r - 65) as usize] = 1);
        let mut bitmap_q: Vec<u8> = vec![0; 27];
        q.iter().for_each(|r| bitmap_q[(*r - 65) as usize] = 1);

        let mut alphabet_d = Vec::with_capacity(27);
        bitmap_d.iter().enumerate().for_each(|(i, sign)| if *sign==1 { alphabet_d.push((i + 65) as u8) });

        let mut alphabet_q = Vec::with_capacity(27);
        bitmap_q.iter().enumerate().for_each(|(i, sign)| if *sign==1 { alphabet_q.push((i + 65) as u8) });
        
        let mut bias = 0;
        for r1 in alphabet_d.iter()
        {
            for r2 in alphabet_q.iter()
            {
                let pair_score = match f(*r1, *r2)
                {
                    None => Err(
                        AlignErr::GetScoreErr
                        { 
                            file: file!().to_string(),
                            line: line!() as usize,
                            msg: "Can not get pair score with this scoring function".to_string() 
                        })?,
                    Some(score) => score,
                };
                bias = min!(bias, pair_score);
            }
        }
        bias = bias.abs();

        let n = |r1, r2|
        {
            match (r1, r2)
            {
                (b'*', _)      => 0,
                (_, b'*')      => 0,
                (a, b) => f(a, b).unwrap() + bias,
            }
        };

        let seg_len = match p
        {
            ProfileType::Epu8  => 32,
            ProfileType::Epu16 => 16,
        };

        let seg_num = (q.len() + seg_len - 1) / seg_len;

        let mut seg_set = vec![Vec::new(); seg_num];

        for (i, s) in seg_set.iter_mut().enumerate().take(seg_num)
        {
            let mut seg = Vec::with_capacity(seg_len);
            for j in 0..seg_len
            {
                let residue = q.get(j * seg_num + i)
                    .map_or(b'*', |r| *r);
                seg.push(residue);
            }
            swap::<Vec<u8>>(s, &mut seg);
        }

        let profile_len = alphabet_d.iter().max().map(|item| *item as usize).unwrap() - 64;

        if seg_len == 16
        {
            let mut profile = vec![Vec::new(); profile_len];
            for residue in alphabet_d.iter().copied()
            {
                let mut score_set = Vec::with_capacity(seg_num);
                for seg in seg_set.iter()
                {
                    let score = seg.iter()
                        .copied()
                        .zip(vec![residue; seg_len])
                        .map(|(r1, r2)| n(r1, r2).unsigned_abs() as u16)
                        .collect::<Vec<u16>>();
                    score_set.push(M256Epu16::from(&score[..]));
                }
                profile[(residue - 65) as usize] = score_set;
            }
            return Ok(Profile::Word { bias: bias.unsigned_abs() as u16, profile })
        }
        
        if seg_len == 32
        {
            let mut profile = vec![Vec::new(); profile_len];
            for residue in alphabet_d.iter().copied()
            {
                let mut score_set = Vec::with_capacity(seg_num);
                for seg in seg_set.iter()
                {
                    let score = seg.iter()
                        .copied()
                        .zip(vec![residue; seg_len])
                        .map(|(r1, r2)| n(r1, r2).unsigned_abs())
                        .collect::<Vec<u8>>();
                    score_set.push(M256Epu8::from(&score[..]));
                }
                profile[(residue - 65) as usize] = score_set;
            }
            return Ok(Profile::Byte { bias: bias.unsigned_abs(), profile })
        }

        unreachable!()
    }

    fn banded_sw<S>(d: &[u8], q: &[u8], go: u8, ge: u8, score: &S) -> (Vec<u8>, Vec<u8>)
    where
        S: Fn(u8, u8) -> Option<i8>
    {
        let d_len = d.len();
        let q_len = q.len();

        let go = go as u16;
        let ge = ge as u16;

        let mut left_f: u16 = 0;
        let mut left_h: u16 = 0;
        let mut prev_e: Vec<u16> = vec![0; q_len+1];
        let mut prev_h: Vec<u16> = vec![0; q_len+1];
        let mut current_h = vec![0; q_len+1];

        let mut direction = vec![vec![0_u8; q_len+1]; d_len+1];
        for i in 1..d_len+1
        {
            for j in 1..q_len+1
            {
                assert!(d_len+1 >= i);
                assert!(q_len+1 >= j);

                let e = max!(prev_h[j].saturating_sub(go), prev_e[j].saturating_sub(ge));
                let f = max!(left_h.saturating_sub(go), left_f.saturating_sub(ge));

                let pair = score(d[i-1], q[j-1]).unwrap();
                let ext = match pair > 0
                {
                    true  => prev_h[j-1].saturating_add(pair.unsigned_abs() as u16),
                    false => prev_h[j-1].saturating_sub(pair.unsigned_abs() as u16),
                };
                let h = max!(ext, e, f);

                direction[i][j] = match h
                {
                    var1 if var1 == e   => 1,
                    var2 if var2 == f   => 2,
                    var3 if var3 == ext => 3,
                    _                        => 0,
                };

                left_f = f;
                left_h = h;
                prev_e[j] = e;
                current_h[j] = h;
            }
            swap::<Vec<u16>>(&mut prev_h, &mut current_h);
        }

        let mut d_best = Vec::new();
        let mut q_best = Vec::new();

        let mut i = d_len;
        let mut j = q_len;
        while direction[i][j] != 0
        {
            if direction[i][j] == 1
            {
                d_best.insert(0, d[i-1]);
                q_best.insert(0, b'-');
                i = i - 1;
                continue;
            }

            if direction[i][j] == 2
            {
                d_best.insert(0, b'-');
                q_best.insert(0, q[j-1]);
                j = j - 1;
                continue;
            }

            if direction[i][j] == 3
            {
                d_best.insert(0, d[i-1]);
                q_best.insert(0, q[j-1]);
                i = i - 1;
                j = j - 1;
                continue;
            }
        }
        (d_best, q_best)
    }

    fn ssw_byte(d: &[u8], q: &[u8], go: u8, ge: u8, terminater: u8, profile: &Profile) -> Result<AlignEnd, AlignErr>
    {
        let go = M256Epu8::fill(go);
        let ge = M256Epu8::fill(ge);

        let (bias, profile) = match profile
        {
            Profile::Byte { bias, profile } => (*bias, profile),
            _ => panic!("Unacceptable profile"),
        };

        let overflow_threshold = u8::MAX.saturating_sub(bias);

        let seg_num = (q.len() + 31) / 32;
        let bias = M256Epu8::fill(bias);
        let mut f = M256Epu8::fill(0);
        let mut e_store = vec![M256Epu8::fill(0); seg_num];
        let mut h_store = vec![M256Epu8::fill(0); seg_num];
        let mut h_buffer = vec![M256Epu8::fill(0); seg_num];

        let mut opt = 0;
        let mut pos = (0, 0);
        let mut max = M256Epu8::fill(0);
        let mut is_overflow = 0;

        if terminater == 0
        {
            for (i, r) in d.iter().copied().enumerate()
            {
                f.zero_out();

                let mut prev_h = *h_store.last().unwrap();
                prev_h = prev_h << 1;

                let profile_col = get_unchecked!(profile, (r - 65) as usize);
                for j in 0..seg_num
                {
                    let score = *get_unchecked!(profile_col,  j);
                    let prev_e = *get_unchecked!(e_store, j);

                    let h = max_epu8(max_epu8(prev_h + score - bias, prev_e), f);
                    
                    let h_sub_go = h - go;

                    let e = max_epu8(h_sub_go, prev_e - ge);
                    f = max_epu8(h_sub_go, f - ge);
                    
                    let e_store_mut_ref = get_mut_unchecked!(e_store, j);
                    *e_store_mut_ref = e;

                    let h_buffer_mut_ref = get_mut_unchecked!(h_buffer, j);
                    *h_buffer_mut_ref = h;

                    prev_h = *get_unchecked!(h_store, j);
                }

                f = f << 1;
                let mut j = 0;
                while f.anyelement_gt(&(*get_unchecked!(h_buffer, j) - go))
                {
                    let h_buffer_uncorrect = *get_unchecked!(h_buffer, j);
                    let h_buffer_mut_ref = get_mut_unchecked!(h_buffer, j);
                    *h_buffer_mut_ref = max_epu8(f, h_buffer_uncorrect);

                    let h_buffer_correct = *get_unchecked!(h_buffer, j);
                    let e_store_uncorrect = *get_unchecked!(e_store, j);
                    let e_store_mut_ref = get_mut_unchecked!(e_store, j);
                    *e_store_mut_ref = max_epu8(e_store_uncorrect, h_buffer_correct - go);

                    f = f - ge;

                    j = j + 1;
                    if j >= seg_num
                    {
                        f = f << 1;
                        j = 0;
                    }
                }

                h_buffer.iter().for_each(|h| max = max_epu8(max, *h));
                let tmp = max.get_max();

                if tmp == overflow_threshold
                {
                    is_overflow = is_overflow + 1;
                    if is_overflow > 1
                    {
                        Err(AlignErr::OverFlow
                        {
                            file: file!().to_string(),
                            line: line!() as usize,
                            msg: "Score out of u8 range".to_string()
                        })?
                    }
                }

                if tmp > opt
                {
                    opt = tmp;
                    for (j, h) in h_buffer.iter().enumerate()
                    {
                        if h.contains(opt)
                        {
                            pos.0 = i;
                            pos.1 = h.position(opt) * seg_num + j;
                            break;
                        }
                    }
                }
                swap::<Vec<M256Epu8>>(&mut h_store, &mut h_buffer);
            }
        }
        else
        {
            'outer: for (i, r) in d.iter().copied().enumerate()
            {
                f.zero_out();
                let mut prev_h = *h_store.last().unwrap();
                prev_h = prev_h << 1;

                let profile_col = get_unchecked!(profile, (r- 65) as usize);
                for j in 0..seg_num
                {
                    let score = *get_unchecked!(profile_col,  j);
                    let prev_e = *get_unchecked!(e_store, j);

                    let h = max_epu8(max_epu8(prev_h + score - bias, prev_e), f);
                    
                    let h_sub_go = h - go;

                    let e = max_epu8(h_sub_go, prev_e - ge);
                    f = max_epu8(h_sub_go, f - ge);
                    
                    let e_store_mut_ref = get_mut_unchecked!(e_store, j);
                    *e_store_mut_ref = e;

                    let h_buffer_mut_ref = get_mut_unchecked!(h_buffer, j);
                    *h_buffer_mut_ref = h;

                    prev_h = *get_unchecked!(h_store, j);
                }

                f = f << 1;
                let mut j = 0;
                while f.anyelement_gt(&(*get_unchecked!(h_buffer, j) - go))
                {
                    let h_buffer_uncorrect = *get_unchecked!(h_buffer, j);
                    let h_buffer_mut_ref = get_mut_unchecked!(h_buffer, j);
                    *h_buffer_mut_ref = max_epu8(f, h_buffer_uncorrect);

                    let h_buffer_correct = *get_unchecked!(h_buffer, j);
                    let e_store_uncorrect = *get_unchecked!(e_store, j);
                    let e_store_mut_ref = get_mut_unchecked!(e_store, j);
                    *e_store_mut_ref = max_epu8(e_store_uncorrect, h_buffer_correct - go);
                    
                    f = f - ge;

                    j = j + 1;
                    if j >= seg_num
                    {
                        f = f << 1;
                        j = 0;
                    }
                }

                for (j, h) in h_buffer.iter().enumerate()
                {
                    if h.contains(terminater)
                    {
                        opt = terminater;
                        pos.0 = i;
                        pos.1 = h.position(opt) * seg_num + j;
                        break 'outer;
                    }
                }
                swap::<Vec<M256Epu8>>(&mut h_store, &mut h_buffer);
            }
        }
        Ok(AlignEnd::U8 { var: opt, pos })
    }

    fn ssw_word(d: &[u8], q: &[u8], go: u8, ge: u8, terminater: u16, profile: &Profile) -> Result<AlignEnd, AlignErr>
    {
        let go = M256Epu16::fill(go as u16);
        let ge = M256Epu16::fill(ge as u16);

        let (bias, profile) = match profile
        {
            Profile::Word { bias, profile } => (*bias, profile),
            _ => panic!("Unacceptable profile"),
        };

        let overflow_threshold = u16::MAX.saturating_sub(bias);
        let seg_num = (q.len() + 15) / 16;
        let bias = M256Epu16::fill(bias);

        let mut f = M256Epu16::fill(0);
        let mut h_store = vec![M256Epu16::fill(0); seg_num];
        let mut e_store = vec![M256Epu16::fill(0); seg_num];
        let mut h_buffer = vec![M256Epu16::fill(0); seg_num];

        let mut opt = 0;
        let mut pos = (0, 0);
        let mut max = M256Epu16::fill(0);
        let mut is_overflow = 0;

        if terminater == 0
        {
            for (i, r) in d.iter().copied().enumerate()
            {
                f.zero_out();

                let mut prev_h = *h_store.last().unwrap();
                prev_h = prev_h << 1;

                let profile_col = get_unchecked!(profile, (r - 65) as usize);
                for j in 0..seg_num
                {
                    let score = *get_unchecked!(profile_col, j);
                    let prev_e = *get_unchecked!(e_store, j);

                    let h = max_epu16(max_epu16(prev_h + score - bias, prev_e), f);

                    let h_sub_go = h - go;

                    let e = max_epu16(h_sub_go, prev_e - ge);
                    f = max_epu16(h_sub_go, f - ge);

                    let e_store_mut_ref = get_mut_unchecked!(e_store, j);
                    *e_store_mut_ref = e;

                    let h_buffer_mut_ref = get_mut_unchecked!(h_buffer, j);
                    *h_buffer_mut_ref = h;

                    prev_h = *get_unchecked!(h_store, j);
                }

                f = f << 1;
                let mut j = 0;
                while f.anyelement_gt(&(*get_unchecked!(h_buffer, j) - go))
                {
                    let h_buffer_uncorrect = *get_unchecked!(h_buffer, j);
                    let h_buffer_mut_ref = get_mut_unchecked!(h_buffer, j);
                    *h_buffer_mut_ref = max_epu16(f, h_buffer_uncorrect);

                    let h_buffer_correct = *get_unchecked!(h_buffer, j);
                    let e_store_uncorrect = *get_unchecked!(e_store, j);
                    let e_store_mut_ref = get_mut_unchecked!(e_store, j);
                    *e_store_mut_ref = max_epu16(e_store_uncorrect, h_buffer_correct - go);

                    f = f - ge;

                    j = j + 1;
                    if j >= seg_num
                    {
                        f = f << 1;
                        j = 0;
                    }
                }

                h_buffer.iter().for_each(|h| max = max_epu16(max, *h));

                let tmp = max.get_max();

                if tmp == overflow_threshold
                {
                    is_overflow = is_overflow + 1;
                    if is_overflow > 1
                    {
                        Err (AlignErr::OverFlow
                        {
                            file: file!().to_string(),
                            line: line!() as usize, 
                            msg: "Score out of u16 range".to_string(),
                        })?
                    }
                }

                if tmp > opt
                {
                    opt = tmp;
                    for (j, h) in h_buffer.iter().enumerate()
                    {
                        if h.contains(opt)
                        {
                            pos.0 = i;
                            pos.1 = h.position(opt) * seg_num + j;
                            break;
                        }
                    }
                }
                swap::<Vec<M256Epu16>>(&mut h_buffer, &mut h_store);
            }
        }
        else
        {
            'outer: for (i, r) in d.iter().enumerate()
            {
                f.zero_out();

                let mut prev_h = *h_store.last().unwrap();
                prev_h = prev_h << 1;

                for j in 0..seg_num
                {
                    let score = profile[(*r - 65) as usize][j];
                    let h = max_epu16(max_epu16(prev_h + score - bias, e_store[j]), f);
                    let e = max_epu16(h - go, e_store[j] - ge);
                    f = max_epu16(h - go, f - ge);

                    e_store[j] = e;
                    h_buffer[j] = h;
                    prev_h = h_store[j];
                }

                f = f << 1;
                let mut j = 0;
                while f.anyelement_gt(&(h_buffer[j] - go))
                {
                    h_buffer[j] = max_epu16(f, h_buffer[j]);
                    e_store[j] = max_epu16(e_store[j], h_buffer[j] - go);
                    f = f - ge;

                    j = j + 1;
                    if j >= seg_num
                    {
                        f = f << 1;
                        j = 0;
                    }
                }

                for (j, h) in h_buffer.iter().enumerate()
                {
                    if h.contains(terminater)
                    {
                        opt = terminater;
                        pos.0 = i;
                        pos.1 = h.position(opt) * seg_num + j;
                        break 'outer;
                    }
                }
                swap::<Vec<M256Epu16>>(&mut h_buffer, &mut h_store);
            }
        }
        Ok (AlignEnd::U16 { var: opt, pos })
    }

    /// `Smith-Waterman` algorithm implemention accelerated by **AVX2**
    ///  
    /// ### Arguments
    /// * `d`: Database sequence 
    /// * `q`: Query sequence
    /// * `go`: Gap open penalty points
    /// * `ge`: Gap extend penalty points
    /// * `flag`: Controls the operating mode of this function
    /// * `f`: Scoring rules
    /// 
    /// ### Function's working mode
    /// * Find alignment **end position** and calculating **optimal score** only.
    /// * Find alignment **start/end position**, calculating **optimal score** and find **best trace path**.
    ///  
    /// ### Select the working mode
    /// function's working mode depend on the values of parameter `flag` (type: [`AlignFlag`](enum@crate::pairwise::AlignFlag))
    /// * `flag` equal to `AlignFlag::End`, function will find alignment **endpoint** and **optimal score** only.
    /// * `flag` equal to `AlignFlag::Path`, the function will find **startpoint/endpoint** of alignment, and **optimal score** and **best trace back path**.
    ///
    /// ### Scoring Rules
    /// As we all know, Smith-waterman algorithm evaluates the similarity of two sequences
    /// based on a score matrix, such as **blosum50**, **blosum62** or a user-defined score rule.
    /// Thus, `ssw` library provides [`blosum50`](fn@crate::score::blosum50), [`blosum62`](fn@crate::score::blosum62),
    /// [`pam120`](fn@crate::score::pam120) scoring matrices wrapped in closures.
    /// Therefore, if you want to use custom scoring rule, just wrap it in closure like `Fn(u8, u8) -> Option<i8>` and pass it to `smith_waterman_avx2` by parameter `f`.
    ///
    /// ### Error Handle
    /// Several errors that can occur during smith-waterman-avx2 execution are wrapping in [`AlignErr`](enum@crate::pairwise::AlignErr).
    /// When an error occurs, the function returns `Err(AignErr)`, so the function should be called with error handling.
    /// 
    /// ### Example1: Find endpoint and optimal score only
    /// ```rust
    /// use ssw::score::blosum50;
    /// use ssw::pairwise::{ AlignFlag, smith_waterman_avx2 };
    /// 
    /// fn main()
    /// {
    ///     // Database sequence
    ///     let d = "CLKQTQMRTDHARCGDFWEESHHHHHHFTLCIA".as_bytes();
    ///     
    ///     // Query sequence
    ///     let q = "CLKQTQMRTDHAMCGDFWEESHHHFTLCIA".as_bytes();
    /// 
    ///     // Gap open penalty
    ///     let go = 3; 
    ///     
    ///     // Gap extend penalty
    ///     let ge = 2; 
    ///    
    ///     let flag = AlignFlag::End;
    ///     
    ///     // Scoring rule: bosum50 scoring matrix
    ///     let pair = blosum50();
    /// 
    ///     // Parameter `flag` equal to `AlignFlag::End`,
    ///     // means find algnment endpoint and optimal score only.
    ///     let res = smith_waterman_avx2(d, q, go, ge, &flag, &pair)
    ///         .map_or_else(|err| panic!("{}", err), |res| res);
    /// 
    ///     println!("{}", res);
    ///     // The output is as follows:
    ///     // 
    ///     // optimal_alignment_score: 216, d_end: 33, q_end: 30
    /// }
    /// ```
    /// 
    /// ### Example2: Find startpoint, endpoint, optimal score and best trace back path
    /// ```rust
    /// use ssw::score::blosum50;
    /// use ssw::pairwise::{ AlignFlag, smith_waterman_avx2 };
    /// 
    /// fn main()
    /// {
    ///     // Database sequence
    ///     let d = "CLKQTQMRTDHARCGDFWEESHHHHHHFTLCIA".as_bytes();
    ///     
    ///     // Query sequence
    ///     let q = "CLKQTQMRTDHAMCGDFWEESHHHFTLCIA".as_bytes();
    /// 
    ///     // Gap open penalty
    ///     let go = 3;
    ///     
    ///     // Gap extend penalty
    ///     let ge = 2; 
    /// 
    ///     let flag = AlignFlag::Path;
    /// 
    ///     let pair = blosum50();
    /// 
    ///     let res = smith_waterman_avx2(d, q, go, ge, &flag, &pair)
    ///         .map_or_else(|err| panic!("{}", err), |res| res);
    /// 
    ///     println!("{}", res);
    ///     // The output is as follows:
    ///     //  
    ///     // optimal_alignment_score: 216, d_start 1, d_end: 33, q_start 1, q_end: 30
    ///     //  
    ///     // d_best: 1 CLKQTQMRTDHARCGDFWEESHHHHHHFTLCIA 33
    ///     //           ||||||||||||*|||||||||||   |||||| 
    ///     // q_best: 1 CLKQTQMRTDHAMCGDFWEESHHH---FTLCIA 30
    /// }
    /// ```
    pub fn smith_waterman_avx2<S>(d: &[u8], q:&[u8], go: u8, ge: u8, flag: &AlignFlag, f: S) -> Result<AlignResult, AlignErr>
    where
        S: Fn(u8, u8) -> Option<i8>
    {
        let (d_seq, q_seq) = match (d.is_ascii(), q.is_ascii())
        {
            (true, true)   => (d.to_ascii_uppercase(), q.to_ascii_uppercase()),
            (true, false)  => Err(AlignErr::IllegalChar
                {
                    file: file!().to_string(),
                    line: line!() as usize,
                    msg: "Non-ascii character contain in database sequence".to_string(),
                })?,
            (false, true)  => Err(AlignErr::IllegalChar
                {
                    file: file!().to_string(),
                    line: line!() as usize,
                    msg: "Non-ascii character contain in query sequence".to_string(),
                })?,
            (false, false) => Err(AlignErr::IllegalChar
                {
                    file: file!().to_string(),
                    line: line!() as usize,
                    msg: "Non-ascii character contain in database/query sequence".to_string(),
                })?,
        };

        let profile_u8 = query_profile(&d_seq, &q_seq, ProfileType::Epu8, &f)?;
        let ext_end = match ssw_byte(&d_seq, &q_seq, go, ge, 0, &profile_u8)
        {
            Ok(res) => res,
            Err(AlignErr::OverFlow {..}) => 
            {
                let profile_u16 = query_profile(&d_seq, &q_seq, ProfileType::Epu16, &f)?;
                ssw_word(&d_seq, &q_seq, go, ge, 0, &profile_u16)?
            },
            _ => unreachable!(),
        };

        if let AlignFlag::End = flag
        {
            if let AlignEnd::U8 { var, pos } = ext_end
            {
                let (opt, (d_end, q_end)) = (var as u32, (pos.0+1, pos.1+1));
                return Ok( AlignResult { d_start: None, q_start: None, d_end, q_end,
                    d_best: None, q_best: None, opt, flag: AlignFlag::End
                } )
            }
            
            if let AlignEnd::U16 { var, pos } = ext_end 
            {
                let (opt, (d_end, q_end)) = (var as u32, (pos.0+1, pos.1+1));
                return Ok( AlignResult { d_start: None, q_start: None, d_end, q_end,
                    d_best: None, q_best: None, opt, flag: AlignFlag::End
                } )
            }

            unreachable!()
        }
        
        if let AlignFlag::Path = flag
        {
            if let AlignEnd::U8 { var, pos } = ext_end
            {
                let (opt, (d_end, q_end)) = (var as u32, (pos.0+1, pos.1+1));

                let mut d_splited_rev = d_seq[0..d_end].to_vec();
                let mut q_splited_rev = q_seq[0..q_end].to_vec();
                d_splited_rev.reverse();
                q_splited_rev.reverse();
                
                let profile_rev = query_profile(&d_splited_rev, &q_splited_rev, ProfileType::Epu8, &f)?;
                let (d_start, q_start) = match ssw_byte(&d_splited_rev, &q_splited_rev, go, ge, var, &profile_rev)?
                {
                    AlignEnd::U8 { pos, .. } => (d_end - pos.0, q_end - pos.1),
                    _ => unreachable!(),
                };
                
                let (d_best_u8, q_best_u8) = banded_sw(&d_seq[d_start-1..d_end], &q_seq[q_start-1..q_end], go, ge, &f);
                let d_best = Some(d_best_u8);
                let q_best = Some(q_best_u8);

                let d_start = Some(d_start);
                let q_start = Some(q_start);

                return Ok( AlignResult { d_start, q_start, d_end, q_end,
                    d_best, q_best, opt, flag: AlignFlag::Path } )
            }
            
            if let AlignEnd::U16 { var, pos } = ext_end
            {
                let (opt, (d_end, q_end)) = (var as u32, (pos.0+1, pos.1+1));

                let mut d_splited_rev = d_seq[0..d_end].to_vec();
                let mut q_splited_rev = q_seq[0..q_end].to_vec();
                d_splited_rev.reverse();
                q_splited_rev.reverse();

                let profile_rev = query_profile(&d_splited_rev, &q_splited_rev, ProfileType::Epu16, &f)?;
                let (d_start, q_start) = match ssw_word(&d_splited_rev, &q_splited_rev, go, ge, var, &profile_rev)?
                {
                    AlignEnd::U16 { pos, .. } => (d_end - pos.0, q_end - pos.1),
                    _ => unreachable!(),
                };

                let (d_best_u8, q_best_u8) = banded_sw(&d_seq[d_start-1..d_end], &q_seq[q_start-1..q_end], go, ge, &f);
                let d_best = Some(d_best_u8);
                let q_best = Some(q_best_u8);

                let d_start = Some(d_start);
                let q_start = Some(q_start);

                return Ok( AlignResult { d_start, q_start, d_end, q_end,
                    d_best, q_best, opt, flag: AlignFlag::Path } )
            }

            unreachable!()
        }

        unreachable!()
    }
}

mod sw_scalar
{
    use std::mem::swap;

    use crate::pairwise::{ AlignEnd, AlignErr, AlignFlag, AlignResult };

    fn sw_scalar<S>(d: &[u8], q: &[u8], go: u32, ge: u32, terminater: u32, score: &S) -> Result<AlignEnd, AlignErr>
    where
        S: Fn(u8, u8) -> Option<i8>
    {
        let d_len = d.len();
        let q_len = q.len();

        let mut left_f: u32 = 0;
        let mut left_h: u32 = 0;
        let mut prev_e: Vec<u32> = vec![0; q_len+1];
        let mut prev_h: Vec<u32> = vec![0; q_len+1];
        let mut current_h = vec![0; q_len+1];

        let mut opt_var = 0;
        let mut opt_pos = (0, 0);
        let mut is_overflow = 0;

        if terminater == 0
        {
            for i in 1..d_len+1
            {
                for j in 1..q_len + 1
                {
                    assert!(d_len+1 >= i);
                    assert!(q_len+1 >= j);

                    let e = max!(prev_e[j].saturating_sub(ge), prev_h[j].saturating_sub(go));
                    let f = max!(left_f.saturating_sub(ge), left_h.saturating_sub(go));

                    let pair = match score(d[i - 1], q[j - 1])
                    {
                        Some(score) => score,
                        None => Err(
                            AlignErr::GetScoreErr
                            { 
                                file: file!().to_string(),
                                line: line!() as usize,
                                msg: "Can not get pair score with this scoring function".to_string() 
                            })?,
                    };

                    let ext = match pair > 0
                    {
                        true  => prev_h[j - 1].saturating_add(pair.unsigned_abs() as u32),
                        false => prev_h[j - 1].saturating_sub(pair.unsigned_abs() as u32),
                    };
                    let h = max!(ext, e, f);

                    left_f = f;
                    left_h = h;
                    prev_e[j] = e;
                    current_h[j] = h;
                }

                let max = *current_h.iter().max().unwrap();

                if max == u32::MAX
                {
                    is_overflow = is_overflow + 1;
                    if is_overflow > 1
                    {
                        Err(AlignErr::OverFlow
                        {
                            file: file!().to_string(),
                            line: line!() as usize,
                            msg: "Score out of u32 range".to_string(),
                        })?
                    }
                }

                if max > opt_var
                {
                    opt_var = max;
                    opt_pos.0 = i;
                    opt_pos.1 = current_h.iter().position(|h| *h == opt_var).unwrap();
                }

                left_f = 0;
                left_h = 0;
                swap::<Vec<u32>>(&mut current_h, &mut prev_h);
            }
        }
        else
        {
            'outer: for i in 1..d_len+1
            {
                for j in 1..q_len+1
                {
                    assert!(d_len+1 >= i);
                    assert!(q_len+1 >= j);
                    let e = max!(prev_e[j].saturating_sub(ge), prev_h[j].saturating_sub(go));
                    let f = max!(left_f.saturating_sub(ge), left_h.saturating_sub(go));

                    let pair = match score(d[i-1], q[j-1])
                    {
                        Some(score) => score,
                        None => Err(
                            AlignErr::GetScoreErr
                            { 
                                file: file!().to_string(),
                                line: line!() as usize,
                                msg: "Can not get pair score with this scoring function".to_string() 
                            })?,
                    };
                    
                    let ext = match pair > 0
                    {
                        true  => prev_h[j-1].saturating_add(pair.unsigned_abs() as u32),
                        false => prev_h[j-1].saturating_sub(pair.unsigned_abs() as u32),
                    };

                    let h = max!(ext, e, f);

                    left_f = f;
                    left_h = h;
                    prev_e[j] = e;
                    current_h[j] = h;
                }

                if current_h.contains(&terminater)
                {
                    opt_var = terminater;
                    opt_pos.0 = i;
                    opt_pos.1 = current_h.iter().position(|h| *h == opt_var).unwrap();
                    break 'outer;
                }

                left_f = 0;
                left_h = 0;
                swap::<Vec<u32>>(&mut current_h, &mut prev_h);
            }
        }
        Ok( AlignEnd::U32 { var: opt_var, pos: opt_pos } )
    }

    fn banded_sw<S>(d: &[u8], q: &[u8], go: u32, ge: u32, score: &S) -> (Vec<u8>, Vec<u8>)
    where
        S: Fn(u8, u8) -> Option<i8>
    {
        let d_len = d.len();
        let q_len = q.len();

        let mut left_f: u32 = 0;
        let mut left_h: u32 = 0;
        let mut prev_e: Vec<u32> = vec![0; q_len+1];
        let mut prev_h: Vec<u32> = vec![0; q_len+1];
        let mut current_h = vec![0; q_len+1];

        let mut direction = vec![vec![0_u8; q_len+1]; d_len+1];

        for i in 1..d_len+1
        {
            for j in 1..q_len+1
            {
                let e = max!(prev_h[j].saturating_sub(go), prev_e[j].saturating_sub(ge));
                let f = max!(left_h.saturating_sub(go), left_f.saturating_sub(ge));

                let pair = score(d[i-1], q[j-1]).unwrap();
                let ext = match pair > 0
                {
                    true  => prev_h[j-1].saturating_add(pair.unsigned_abs() as u32),
                    false => prev_h[j-1].saturating_sub(pair.unsigned_abs() as u32),
                };
                let h = max!(ext, e, f);

                direction[i][j] = match h
                {
                    var1 if var1 == e   => 1,
                    var2 if var2 == f   => 2,
                    var3 if var3 == ext => 3,
                    _                        => unreachable!(),
                };

                left_f = f;
                left_h = h;
                prev_e[j] = e;
                current_h[j] = h;
            }
            swap::<Vec<u32>>(&mut prev_h, &mut current_h);
        }

        let mut d_best = Vec::new();
        let mut q_best = Vec::new();

        let mut i = d_len;
        let mut j = q_len;
        while direction[i][j] != 0
        {
            assert!(d_len+1 >= i);
            assert!(q_len+1 >= j);
            if direction[i][j] == 1
            {
                d_best.insert(0, d[i-1]);
                q_best.insert(0, b'-');
                i = i - 1;
                continue;
            }

            if direction[i][j] == 2
            {
                d_best.insert(0, b'-');
                q_best.insert(0, q[j-1]);
                j = j - 1;
                continue;
            }

            if direction[i][j] == 3
            {
                d_best.insert(0, d[i-1]);
                q_best.insert(0, q[j-1]);
                i = i - 1;
                j = j - 1;
                continue;
            }
        }
        (d_best, q_best)
    }

    /// Serial implementation of smith-waterman algorithm, **much slower** than [`smith_waterman_avx2`](fn@crate::pairwise::smith_waterman_avx2)
    /// 
    /// Why do we need a serial implementation?
    /// Because the maximum computational accuracy of `smith_waterman_avx2` is u16[0-65535],
    /// while `smith_waterman_scalar` can reach u32[0-4294967295].
    /// Therefore, if a numerical overflow error occurs in `smith_waterman_avx2`,
    /// `smith_waterman_scalar` can be used instead.
    /// 
    /// ### Arguments
    /// * `d`: Database sequence 
    /// * `q`: Query sequence
    /// * `go`: Gap open penalty points
    /// * `ge`: Gap extend penalty points
    /// * `flag`: Controls the operating mode of this function
    /// * `f`: Scoring rules
    /// 
    /// ### Example: Find alignment endpoint and optimal score only
    /// ```rust
    /// use ssw::score::blosum50;
    /// use ssw::pairwise::{ AlignFlag, smith_waterman_scalar };
    /// 
    /// fn main()
    /// {
    ///     let d = "CLKQTQMRTDHARCGDFWEESHHHHHHFTLCIA".as_bytes();
    ///     let q = "CLKQTQMRTDHAMCGDFWEESHHHFTLCIA".as_bytes();
    ///       
    ///     let go = 3;
    ///     let ge = 2;
    ///     let flag = AlignFlag::End;
    ///     let pair_score = blosum50();
    ///       
    ///     // `smith_waterman_avx2` and `smith_waterman_scalar` accept the same parameters,
    ///     // the difference being that the calculation speed of `smith_waterman_scalar`
    ///     // is much slower than `smith_waterman_avx2`
    ///     let align_res = smith_waterman_scalar(d, q, go, ge, &flag, &pair_score)
    ///         .map_or_else(|err| panic!("{}", err), |res| res);
    ///       
    ///     println!("{}", align_res);
    ///     // `AlignResult` has implemented `std::fmt::Display` trait.
    ///     // Therefore, the alignment results can be printed directly,
    ///     // the printed results are as follows:
    ///     //
    ///     // optimal_alignment_score: 216, d_start 1, d_end: 33, q_start 1, q_end: 30
    /// }
    /// ```
    /// 
    /// ### Example2: Find startpoint, endpoint, optimal score and best trace back path
    /// ```rust
    /// use ssw::score::blosum50;
    /// use ssw::pairwise::{ AlignFlag, smith_waterman_scalar };
    /// fn main()
    /// {
    ///     let d = "CLKQTQMRTDHARCGDFWEESHHHHHHFTLCIA".as_bytes();
    ///     let q = "CLKQTQMRTDHAMCGDFWEESHHHFTLCIA".as_bytes();
    ///       
    ///     let go = 3;
    ///     let ge = 2;
    ///     let flag = AlignFlag::Path;
    ///     let pair_score = blosum50();
    ///       
    ///     // `smith_waterman_avx2` and `smith_waterman_scalar` accept the same parameters,
    ///     // the difference being that the calculation speed of `smith_waterman_scalar`
    ///     // is much slower than `smith_waterman_avx2`
    ///     let align_res = smith_waterman_scalar(d, q, go, ge, &flag, &pair_score)
    ///         .map_or_else(|err| panic!("{}", err), |res| res);
    ///       
    ///     println!("{}", align_res);
    ///     // `AlignResult` has implemented `std::fmt::Display` trait.
    ///     // Therefore, the alignment results can be printed directly,
    ///     // the printed results are as follows:
    ///     //
    ///     // optimal_alignment_score: 216, d_start 1, d_end: 33, q_start 1, q_end: 30
    /// 	//
    /// 	// d_best: 1 CLKQTQMRTDHARCGDFWEESHHHHHHFTLCIA 33
    ///   	//           ||||||||||||*|||||||||||   |||||| 
    /// 	// q_best: 1 CLKQTQMRTDHAMCGDFWEESHHH---FTLCIA 30
    /// }
    /// ```
    pub fn smith_waterman_scalar<S>(d: &[u8], q: &[u8], go: u8, ge: u8, flag: &AlignFlag, score: S) -> Result<AlignResult, AlignErr>
    where
        S: Fn(u8, u8) -> Option<i8>
    {
        let (d_seq, q_seq) = match (d.is_ascii(), q.is_ascii())
        {
            (true, true)  => (d.to_ascii_uppercase(), q.to_ascii_uppercase()),
            (true, false) => Err(AlignErr::IllegalChar
                {
                    file: file!().to_string(),
                    line: line!() as usize,
                    msg: "Non-ascii character contain in database sequence".to_string(),
                })?,
            (false, true) => Err(AlignErr::IllegalChar
                {
                    file: file!().to_string(),
                    line: line!() as usize,
                    msg: "Non-ascii character contain in query sequence".to_string(),
                })?,
            _ => unreachable!(),
        };

        let go = go as u32;
        let ge = ge as u32;

        let (opt, (d_end, q_end)) = match sw_scalar(&d_seq, &q_seq, go, ge, 0, &score)?
        {
            AlignEnd::U32 { var, pos } => (var, (pos.0, pos.1)),
            _ => unreachable!(),
        };

        if let AlignFlag::End = flag
        {
            return Ok ( AlignResult { d_start: None, q_start: None, d_end, q_end,
                d_best: None, q_best: None, opt, flag: AlignFlag::End
            } )
        }

        if let AlignFlag::Path = flag
        {
            let mut d_splited_rev = d_seq[0..d_end].to_vec();
            let mut q_splited_rev = q_seq[0..q_end].to_vec();
            d_splited_rev.reverse();
            q_splited_rev.reverse();

            let (d_start, q_start) = match sw_scalar(&d_splited_rev, &q_splited_rev, go, ge, opt, &score)?
            {
                AlignEnd::U32 { pos, .. } => (d_end - pos.0, q_end - pos.1),
                _ => unreachable!(),
            };

            let d_sub = &d_seq[d_start..d_end];
            let q_sub = &q_seq[q_start..q_end];

            let (d_best_u8, q_best_u8) = banded_sw(d_sub, q_sub, go, ge, &score);

            let d_best = Some(d_best_u8);
            let q_best = Some(q_best_u8);

            let d_start = Some(d_start+1);
            let q_start = Some(q_start+1);

            return Ok ( AlignResult { d_start, q_start, d_end, q_end,
                d_best, q_best, opt, flag: AlignFlag::Path } )
        }

        unreachable!()
    }
}

pub use self::sw_avx2::smith_waterman_avx2;
pub use self::sw_scalar::smith_waterman_scalar;