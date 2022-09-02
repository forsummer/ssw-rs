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

#[derive(Debug)]
enum AlignErr { OverFlow }

#[derive(Debug, Clone)]
struct AlignEnd<T: Sized + Copy> { var: T, pos: (usize, usize) }

pub struct AlignResult
{
    d_id: String,
    q_id: String,
    d_start: usize,
    q_start: usize,
    d_end: usize,
    q_end: usize,
    d_best: String,
    q_best: String,
    opt: u32
}

impl std::fmt::Display for AlignResult
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        let result = tabular::Table::new("{:<} {:<} {:<}")
            .with_row(tabular::row!("d_id", ":", &self.d_id))
            .with_row(tabular::row!("q_id", ":", &self.q_id))
            .with_row(tabular::row!("opt", ":", self.opt));

        let mut indication_line = String::with_capacity(self.d_best.len());
        for (r1, r2) in self.d_best.chars().zip(self.q_best.chars())
        {
            let identifier = if [r1, r2].contains(&'-') {' '} else if r1 == r2 {'|'} else {'*'};
            indication_line.push(identifier);
        }

        let sub_seq = tabular::Table::new("{:<} {:<} {:<} {:<} {:<}")
            .with_row(tabular::row!("d_sub", ":", self.d_start, &self.d_best, self.d_end))
            .with_row(tabular::row!("", "", "", indication_line, ""))
            .with_row(tabular::row!("q_sub", ":", self.q_start, &self.q_best, self.q_end));
        
        write!(f, "{}{}", result, sub_seq)
    }
}

mod sw_avx2
{
    use std::mem::swap;

    use crate::load::Seq;
    use crate::avx::avx2::M256Epu8;
    use crate::avx::avx2::max_epu8;
    use crate::avx::avx2::M256Epu16;
    use crate::avx::avx2::max_epu16;
    use crate::pairwise::AlignEnd;
    use crate::pairwise::AlignErr;
    use crate::pairwise::AlignResult;

    enum VecType { Epu8, Epu16 }

    enum Profile
    {
        Byte { bias: u8,  profile: Vec<Vec<M256Epu8>>  },
        Word { bias: u16, profile: Vec<Vec<M256Epu16>> },
    }

    fn query_profile<S>(d: &[u8], q: &[u8], p: VecType, f: S) -> Profile
    where
        S: Fn(u8, u8) -> i8
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
                bias = min!(bias, f(*r1, *r2));
            }
        }
        bias = bias.abs();

        let n = |r1, r2|
        {
            match (r1, r2)
            {
                (b'*', _)      => 0,
                (_, b'*')      => 0,
                (a, b) => f(a, b) + bias,
            }
        };

        let seg_len = if matches!(p, VecType::Epu8) { 32 } else { 16 };
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
            for residue in alphabet_d.iter()
            {
                let mut score_vec = Vec::with_capacity(seg_num);
                for seg in seg_set.iter()
                {
                    let score = seg.iter()
                        .copied()
                        .zip(vec![*residue; seg_len])
                        .map(|(r1, r2)| n(r1, r2).unsigned_abs() as u16)
                        .collect::<Vec<u16>>();
                    score_vec.push(M256Epu16::from(&score[..]));
                }
                profile[(*residue - 65) as usize] = score_vec;
            }
            Profile::Word { bias: bias.unsigned_abs() as u16, profile }
        }
        else
        {
            let mut profile = vec![Vec::new(); profile_len];
            for residue in alphabet_d.iter()
            {
                let mut score_vec = Vec::with_capacity(seg_num);
                for seg in seg_set.iter()
                {
                    let score = seg.iter()
                        .copied()
                        .zip(vec![*residue; seg_len])
                        .map(|(r1, r2)| n(r1, r2).unsigned_abs())
                        .collect::<Vec<u8>>();
                    score_vec.push(M256Epu8::from(&score[..]));
                }
                profile[(*residue - 65) as usize] = score_vec;
            }
            Profile::Byte { bias: bias.unsigned_abs(), profile }
        }
    }

    fn banded_sw<S>(d: &[u8], q: &[u8], go: u8, ge: u8, score: &S) -> (Vec<u8>, Vec<u8>)
    where
        S: Fn(u8, u8) -> i8
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

        let mut direction = vec![vec![0; q_len+1]; d_len+1];
        for i in 1..d_len+1
        {
            for j in 1..q_len+1
            {
                assert!(d_len+1 >= i);
                assert!(q_len+1 >= j);

                let e = max!(prev_h[j].saturating_sub(go), prev_e[j].saturating_sub(ge));
                let f = max!(left_h.saturating_sub(go), left_f.saturating_sub(ge));

                let pair = score(d[i-1], q[j-1]);
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

    fn ssw_byte(d: &[u8], q: &[u8], go: u8, ge: u8, terminater: u8, profile: &Profile) -> Result<AlignEnd<u8>, AlignErr>
    {
        let go = M256Epu8::fill(go);
        let ge = M256Epu8::fill(ge);

        let (bias, profile) = match profile
        {
            Profile::Byte { bias, profile } => (*bias, profile),
            _ => panic!("Unacceptable profile"),
        };

        let overflow_sign = u8::MAX - bias;

        let seg_num = (q.len() + 31) / 32;
        
        let bias = M256Epu8::fill(bias);
        let mut f = M256Epu8::fill(0);
        let mut h_store = vec![M256Epu8::fill(0); seg_num];
        let mut e_store = vec![M256Epu8::fill(0); seg_num];
        let mut h_buffer = vec![M256Epu8::fill(0); seg_num];

        let mut opt_var = 0;
        let mut opt_pos = (0, 0);
        let mut max = M256Epu8::fill(0);
        let mut is_overflow = 0;

        if terminater == 0
        {
            for (i, r) in d.iter().enumerate()
            {
                f.zero_out();

                let mut prev_h = *h_store.last().unwrap();
                prev_h.shift_left_byte();

                for (e_s, (h_buf, (h_s, score))) in e_store
                    .iter_mut()
                    .zip(h_buffer.iter_mut()
                    .zip(h_store.iter_mut()
                    .zip(profile[(*r - 65) as usize].iter())))
                {
                    let h = max_epu8!(prev_h + (*score) - bias, *e_s);
                    let e = max_epu8!(h - go, *e_s - ge);
                    f = max_epu8!(h - go, f - ge);

                    *e_s = e;
                    *h_buf = h;
                    swap::<M256Epu8>(&mut prev_h, h_s);
                }

                f.shift_left_byte();

                let mut j = 0;
                while f > h_buffer[j] - go
                {
                    h_buffer[j] = max_epu8!(f, h_buffer[j]);
                    f = f - ge;

                    if j+1 >= seg_num
                    {
                        f.shift_left_byte();
                        j = 0;
                    }
                }

                h_buffer.iter().for_each(|h| max = max_epu8!(max, *h));

                let tmp = max.get_max();

                if tmp == overflow_sign
                {
                    is_overflow = is_overflow + 1;
                    if is_overflow > 1
                    {
                        Err(AlignErr::OverFlow)?
                    }
                }

                if tmp > opt_var
                {
                    opt_var = tmp;
                    opt_pos.0 = i;
                    
                    for (j, h) in h_buffer.iter().enumerate()
                    {
                        if h.contains(opt_var)
                        {
                            opt_pos.1 = h.position(opt_var) * seg_num + j;
                            break;
                        }
                    }
                }
                swap::<Vec<M256Epu8>>(&mut h_store, &mut h_buffer);
            }
        }
        else
        {
            'outer: for (i, r) in d.iter().enumerate()
            {
                f.zero_out();

                let mut prev_h = *h_store.last().unwrap();
                prev_h.shift_left_byte();
                
                for (e_s, (h_buf, (h_s, score))) in e_store.iter_mut()
                    .zip(h_buffer.iter_mut()
                    .zip(h_store.iter_mut()
                    .zip(profile[(*r - 65) as usize].iter())))
                {
                    let h = max_epu8!(prev_h + (*score) - bias, *e_s);
                    let e = max_epu8!(h - go, *e_s - ge);
                    f = max_epu8!(h - go, f - ge);

                    *e_s = e;
                    *h_buf = h;
                    swap::<M256Epu8>(&mut prev_h, h_s);
                }

                f.shift_left_byte();

                let mut j = 0;
                while f > h_buffer[j] - go
                {
                    h_buffer[j] = max_epu8!(f, h_buffer[j]);
                    f = f - ge;

                    if j+1 >= seg_num
                    {
                        f.shift_left_byte();
                        j = 0;
                    }
                }

                for (j, h) in h_buffer.iter().enumerate()
                {
                    if h.contains(terminater)
                    {
                        opt_pos.0 = i;
                        opt_pos.1 = h.position(terminater) * seg_num + j;
                        opt_var = terminater;
                        break 'outer;
                    }
                }
                swap::<Vec<M256Epu8>>(&mut h_store, &mut h_buffer);
            }
        }
        Ok(AlignEnd { var: opt_var, pos: opt_pos })
    }

    fn ssw_word(d: &Vec<u8>, q: &Vec<u8>, go: u8, ge: u8, terminater: u16, profile: &Profile) -> Result<AlignEnd<u16>, AlignErr>
    {
        let go = M256Epu16::fill(go as u16);
        let ge = M256Epu16::fill(ge as u16);

        let (bias, profile) = match profile
        {
            Profile::Word { bias, profile } => (*bias, profile),
            _ => panic!("Unacceptable profile"),
        };
        
        let overflow_sign = u16::MAX - bias;

        let seg_num = (q.len() + 15) / 16;
        let bias = M256Epu16::fill(bias);

        let mut f = M256Epu16::fill(0);
        let mut h_store = vec![M256Epu16::fill(0); seg_num];
        let mut e_store = vec![M256Epu16::fill(0); seg_num];
        let mut h_buffer = vec![M256Epu16::fill(0); seg_num];

        let mut opt_var = 0;
        let mut opt_pos = (0, 0);
        let mut max = M256Epu16::fill(0);
        let mut is_overflow = 0;

        if terminater == 0
        {
            for (i, r) in d.iter().enumerate().take(d.len())
            {
                f.zero_out();

                let mut prev_h = *h_store.last().unwrap();
                prev_h.shift_left_bytex2();

                for (e_s, (h_buf, (h_s, score))) in e_store.iter_mut()
                    .zip(h_buffer.iter_mut()
                    .zip(h_store.iter_mut()
                    .zip(profile[(r - 65) as usize].iter())))
                {
                    let h = max_epu16!(prev_h + (*score) - bias, e_s);
                    let e = max_epu16!(h - go, *e_s - ge);
                    f = max_epu16!(h - go, f - ge);

                    *e_s = e;
                    *h_buf = h;
                    swap::<M256Epu16>(&mut prev_h, h_s);
                }

                f.shift_left_bytex2();

                let mut j = 0;
                while f > h_buffer[j] - go
                {
                    h_buffer[j] = max_epu16!(f, h_buffer[j]);
                    f = f - ge;

                    if j+1 >= seg_num
                    {
                        f.shift_left_bytex2();
                        j = 0;
                    }
                }

                h_buffer.iter().for_each(|h| max = max_epu16!(*h, max));
                let tmp = max.get_max();

                if tmp == overflow_sign
                {
                    is_overflow = is_overflow + 1;
                    if is_overflow > 1
                    {
                        Err(AlignErr::OverFlow)?
                    }
                }

                if tmp > opt_var
                {
                    opt_var = tmp;
                    opt_pos.0 = i;

                    for (j, h) in h_buffer.iter().enumerate().take(seg_num)
                    {
                        if h.contains(opt_var)
                        {
                            opt_pos.1 = h.position(opt_var) * seg_num + j;
                            break;
                        }
                    }
                }
                swap::<Vec<M256Epu16>>(&mut h_store, &mut h_buffer);
            }
        }
        else
        {
            'outer: for (i, r) in d.iter().enumerate().take(d.len())
            {
                f.zero_out();

                let mut prev_h = *h_store.last().unwrap();
                prev_h.shift_left_bytex2();

                for (e_s, (h_buf, (h_s, score))) in e_store.iter_mut()
                    .zip(h_buffer.iter_mut()
                    .zip(h_store.iter_mut()
                    .zip(profile.get((r - 65) as usize).unwrap().iter())))
                {
                    let h = max_epu16!(prev_h + (*score) - bias, *e_s);
                    let e = max_epu16!(h - go, *e_s - ge);
                    f = max_epu16!(h - go, f - ge);

                    *e_s = e;
                    *h_buf = h;
                    swap::<M256Epu16>(&mut prev_h, h_s);
                }

                f.shift_left_bytex2();

                let mut j = 0;
                while f > h_buffer[j] - go
                {
                    h_buffer[j] = max_epu16!(f, h_buffer[j]);
                    f = f - ge;

                    if j+1 >= seg_num
                    {
                        f.shift_left_bytex2();
                        j = 0;
                    }
                }

                for (j, h) in h_buffer.iter().enumerate()
                {
                    if h.contains(terminater)
                    {
                        opt_pos.0 = i;
                        opt_pos.1 = h.position(terminater) * seg_num + j;
                        opt_var = terminater;
                        break 'outer;
                    }
                }
                swap::<Vec<M256Epu16>>(&mut h_store, &mut h_buffer);
            }
        }
        Ok(AlignEnd { var: opt_var, pos: opt_pos })
    }

    pub fn smith_waterman_avx2<S>(d: &Seq, q:&Seq, go: u8, ge: u8, f: S) -> AlignResult
    where
        S: Fn(u8, u8) -> i8
    {
        let d_id = d.id.clone();
        let q_id = q.id.clone();

        let d_seq = d.seq.to_ascii_uppercase();
        let q_seq = q.seq.to_ascii_uppercase();

        let profile = query_profile(&d_seq, &q_seq, VecType::Epu8, &f);
        let ext_end = match ssw_byte(&d_seq, &q_seq, go, ge, 0, &profile)
        {
            Err(_err)    => None,
            Ok(res)  => Some(res),
        };

        if let Some(res) = ext_end
        {
            let d_end = res.pos.0 + 1;
            let q_end = res.pos.1 + 1;

            let mut d_splited_rev = d_seq[0..d_end].to_vec();
            let mut q_splited_rev = q_seq[0..q_end].to_vec();
            d_splited_rev.reverse();
            q_splited_rev.reverse();

            let profile_rev = query_profile(&d_splited_rev, &q_splited_rev, VecType::Epu8, &f);
            let ext_start = ssw_byte(&d_splited_rev, &q_splited_rev, go, ge, res.var, &profile_rev).unwrap();

            let d_start = d_end - ext_start.pos.0;
            let q_start = q_end - ext_start.pos.1;
    
            let d_sub = &d_seq[d_start-1..d_end];
            let q_sub = &q_seq[q_start-1..q_end];

            let (d_best_u8, q_best_u8) = banded_sw(d_sub, q_sub, go, ge, &f);

            let d_best = String::from_utf8(d_best_u8).unwrap();
            let q_best = String::from_utf8(q_best_u8).unwrap();
            
            let opt = res.var as u32;

            AlignResult { d_id, q_id, d_start, q_start, d_end, q_end, d_best, q_best, opt }
        }
        else
        {
            let profile = query_profile(&d_seq, &q_seq, VecType::Epu16, &f);
            let ext_end = match ssw_word(&d_seq, &q_seq, go, ge, 0, &profile)
            {
                Err(_err)    => panic!("Overflow, input sequence is too long"),
                Ok(res) => res,
            };

            let d_end = ext_end.pos.0 + 1;
            let q_end = ext_end.pos.1 + 1;

            let mut d_splited_rev = d_seq[0..d_end].to_vec();
            let mut q_splited_rev = q_seq[0..q_end].to_vec();
            d_splited_rev.reverse();
            q_splited_rev.reverse();

            let profile_rev = query_profile(&d_splited_rev, &q_splited_rev, VecType::Epu16, &f);
            let ext_start = ssw_word(&d_splited_rev, &q_splited_rev, go, ge, ext_end.var, &profile_rev).unwrap();

            let d_start = d_end - ext_start.pos.0;
            let q_start = q_end - ext_start.pos.1;

            let d_sub = &d_seq[d_start-1..d_end];
            let q_sub = &q_seq[q_start-1..q_end];

            let (d_best_u8, q_best_u8) = banded_sw(d_sub, q_sub, go, ge, &f);

            let d_best = String::from_utf8(d_best_u8).unwrap();
            let q_best = String::from_utf8(q_best_u8).unwrap();

            let opt = ext_end.var as u32;

            AlignResult { d_id, q_id, d_start, q_start, d_end, q_end, d_best, q_best, opt }
        }

    }
}

#[allow(dead_code)]
mod sw_scalar
{
    use std::mem::swap;

    use crate::load::Seq;
    use crate::pairwise::AlignEnd;
    use crate::pairwise::AlignResult;

    fn sw_scalar<S>(d: &[u8], q: &[u8], go: u32, ge: u32, terminater: u32, score: &S) -> AlignEnd<u32>
    where
        S: Fn(u8, u8) -> i8
    {
        // let d_len = d.len();
        let q_len = q.len();

        let mut left_f: u32 = 0;
        let mut left_h: u32 = 0;
        let mut prev_e: Vec<u32> = vec![0; q_len+1];
        let mut prev_h: Vec<u32> = vec![0; q_len+1];
        let mut current_h = vec![0; q_len+1];

        let mut opt = AlignEnd { var: 0, pos: (0, 0) };
        if terminater == 0
        {
            for (i, dr) in d.iter().enumerate()
            {
                let mut prev_h_iter_forward = *prev_h.first().unwrap();

                for (qr, (p_e, (p_h, c_h))) in q.iter()
                    .zip(prev_e.iter_mut().skip(1)
                    .zip(prev_h.iter_mut().skip(1)
                    .zip(current_h.iter_mut().skip(1))))
                {
                    let e = max!(p_e.saturating_sub(ge), p_h.saturating_sub(go));
                    let f = max!(left_f.saturating_sub(ge), left_h.saturating_sub(go));

                    let pair = score(*dr, *qr);
                    let ext = match pair > 0
                    {
                        true  => prev_h_iter_forward.saturating_add(pair.unsigned_abs() as u32),
                        false => prev_h_iter_forward.saturating_sub(pair.unsigned_abs() as u32),
                    };
                    let h = max!(ext, e, f);

                    *c_h = h;
                    *p_e = e;
                    left_f = f;
                    left_h = h;
                    prev_h_iter_forward = *p_h;
                }

                let max = *current_h.iter().max().unwrap();
                if max > opt.var
                {
                    let j = current_h.iter().position(|item| *item==max).unwrap();
                    opt.var = max;
                    opt.pos = (i, j-1);
                }

                swap::<Vec<u32>>(&mut prev_h, &mut current_h);
            }

            // for i in 1..d_len+1
            // {
            //     for j in 1..q_len+1
            //     {
            //         assert!(d_len+1 >= i);
            //         assert!(q_len+1 >= j);
            //         let e = max!(prev_h[j].saturating_sub(go), prev_e[j].saturating_sub(ge));
            //         let f = max!(left_h.saturating_sub(go), left_f.saturating_sub(ge));

            //         let pair = score(d[i-1], q[j-1]);
                    
            //         let ext = match pair > 0
            //         {
            //             true => prev_h[j-1].saturating_add(pair.unsigned_abs() as u32),
            //             false => prev_h[j-1].saturating_sub(pair.unsigned_abs() as u32),
            //         };

            //         let h = max!(ext, e, f);

            //         left_f = f;
            //         left_h = h;
            //         prev_e[j] = e;
            //         current_h[j] = h;
            //     }

            //     let h_max = *current_h.iter().max().unwrap();
            //     if h_max > opt.var
            //     {
            //         let j = current_h.iter().position(|item| item==&h_max).unwrap();
            //         opt.var = h_max;
            //         opt.pos = (i, j);
            //     }
            //     swap::<Vec<u32>>(&mut prev_h, &mut current_h);
            // }
        }
        else
        {
            for (i, dr) in d.iter().enumerate()
            {
                let mut prev_h_iter_forward = *prev_h.first().unwrap();

                for (qr, (p_e, (p_h, c_h))) in q.iter()
                    .zip(prev_e.iter_mut().skip(1)
                    .zip(prev_h.iter_mut().skip(1)
                    .zip(current_h.iter_mut().skip(1))))
                {
                    let e = max!(p_e.saturating_sub(ge), p_h.saturating_sub(go));
                    let f = max!(left_f.saturating_sub(ge), left_h.saturating_sub(go));

                    let pair = score(*dr, *qr);
                    let ext = match pair > 0
                    {
                        true  => prev_h_iter_forward.saturating_add(pair.unsigned_abs() as u32),
                        false => prev_h_iter_forward.saturating_sub(pair.unsigned_abs() as u32),
                    };
                    let h = max!(ext, e, f);

                    *c_h = h;
                    *p_e = e;
                    left_f = f;
                    left_h = h;
                    prev_h_iter_forward = *p_h;
                }

                if current_h.contains(&terminater)
                {
                    let j = current_h.iter().position(|item| *item==terminater).unwrap();
                    opt.var = terminater;
                    opt.pos = (i, j-1);
                    break;
                }

                swap::<Vec<u32>>(&mut prev_h, &mut current_h);
            }

            // for i in 1..d_len+1
            // {
            //     for j in 1..q_len+1
            //     {
            //         assert!(d_len+1 >= i);
            //         assert!(q_len+1 >= j);
            //         let e = max!(prev_h[j].saturating_sub(go), prev_e[j].saturating_sub(ge));
            //         let f = max!(left_h.saturating_sub(go), left_f.saturating_sub(ge));

            //         let pair = score(d[i-1], q[j-1]);
            //         let ext = match pair > 0
            //         {
            //             true => prev_h[j-1].saturating_add(pair.unsigned_abs() as u32),
            //             false => prev_h[j-1].saturating_sub(pair.unsigned_abs() as u32),
            //         };
            //         let h = max!(ext, e, f);

            //         if h == terminater
            //         {
            //             opt.var = h;
            //             opt.pos = (i, j);
            //             break;
            //         }

            //         left_f = f;
            //         left_h = h;
            //         prev_e[j] = e;
            //         current_h[j] = h;
            //     }

            //     for h in current_h.iter().copied()
            //     {
            //         if h == terminater
            //         {
            //             let j = current_h.iter().position(|item| *item==terminater).unwrap();
            //             opt.var = h;
            //             opt.pos = (i, j);
            //             break;
            //         }
            //     }
            //     swap::<Vec<u32>>(&mut prev_h, &mut current_h);
            // }
        }
        opt
    }

    fn banded_sw<S>(d: &[u8], q: &[u8], go: u32, ge: u32, score: &S) -> (Vec<u8>, Vec<u8>)
    where
        S: Fn(u8, u8) -> i8
    {
        let d_len = d.len();
        let q_len = q.len();

        let mut left_f: u32 = 0;
        let mut left_h: u32 = 0;
        let mut prev_e: Vec<u32> = vec![0; q_len+1];
        let mut prev_h: Vec<u32> = vec![0; q_len+1];
        let mut current_h = vec![0; q_len+1];

        let mut direction = vec![vec![0; q_len+1]; d_len+1];

        for (dr, dx) in d.iter()
            .zip(direction.iter_mut().skip(1))
        {

            let mut prev_h_iter_forward = *prev_h.first().unwrap();

            for (qr, (dxy, (p_e, (p_h, c_h)))) in q.iter()
                .zip(dx.iter_mut().skip(1)
                .zip(prev_e.iter_mut().skip(1)
                .zip(prev_h.iter_mut().skip(1)
                .zip(current_h.iter_mut().skip(1)))))
            {
                let e = max!(p_e.saturating_sub(ge), p_h.saturating_sub(go));
                let f = max!(left_f.saturating_sub(ge), left_h.saturating_sub(go));

                let pair = score(*dr, *qr);
                let ext = match pair > 0
                {
                    true  => prev_h_iter_forward.saturating_add(pair.unsigned_abs() as u32),
                    false => prev_h_iter_forward.saturating_sub(pair.unsigned_abs() as u32),
                };
                let h = max!(ext, e, f);

                *dxy = match h
                {
                    var1 if var1 == e   => 1,
                    var2 if var2 == f   => 2,
                    var3 if var3 == ext => 3,
                    _                        => 0,
                };

                *p_e = e;
                *c_h = h;
                left_f = f;
                left_h = h;
                prev_h_iter_forward = *p_h;
            }
            swap::<Vec<u32>>(&mut prev_h, &mut current_h);
        }

        // for i in 1..d_len+1
        // {
        //     for j in 1..q_len+1
        //     {
        //         assert!(d_len+1 >= i);
        //         assert!(q_len+1 >= j);
        //         let e = max!(prev_h[j].saturating_sub(go), prev_e[j].saturating_sub(ge));
        //         let f = max!(left_h.saturating_sub(go), left_f.saturating_sub(ge));

        //         let pair = score(d[i-1], q[j-1]);
        //         let ext = match pair > 0
        //         {
        //             true => prev_h[j-1].saturating_add(pair.unsigned_abs() as u32),
        //             false => prev_h[j-1].saturating_sub(pair.unsigned_abs() as u32),
        //         };
        //         let h = max!(ext, e, f);

        //         direction[i][j] = match h
        //         {
        //             var1 if var1 == e   => 1,
        //             var2 if var2 == f   => 2,
        //             var3 if var3 == ext => 3,
        //             _                        => 0,
        //         };

        //         left_f = f;
        //         left_h = h;
        //         prev_e[j] = e;
        //         current_h[j] = h;
        //     }
        //     swap::<Vec<u32>>(&mut prev_h, &mut current_h);
        // }

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

    pub fn smith_waterman_scalar<S>(d: &Seq, q: &Seq, go: u32, ge: u32, score: S) -> AlignResult
    where
        S: Fn(u8, u8) -> i8
    {
        let d_id = d.id.clone();
        let q_id = q.id.clone();
        let d_seq = d.seq.to_ascii_uppercase();
        let q_seq = q.seq.to_ascii_uppercase();
        
        let ext_end = sw_scalar(&d_seq, &q_seq, go, ge, 0, &score);
        let d_end = ext_end.pos.0 + 1;
        let q_end = ext_end.pos.1 + 1;

        let mut d_splited_rev = d_seq[0..d_end].to_vec();
        let mut q_splited_rev = q_seq[0..q_end].to_vec();
        d_splited_rev.reverse();
        q_splited_rev.reverse();

        let ext_start = sw_scalar(&d_splited_rev, &q_splited_rev, go, ge, ext_end.var, &score);
        let d_start = d_end - ext_start.pos.0;
        let q_start = q_end - ext_start.pos.1;

        let d_sub = &d_seq[d_start-1..d_end];
        let q_sub = &q_seq[q_start-1..q_end];

        let (d_best_u8, q_best_u8) = banded_sw(d_sub, q_sub, go, ge, &score);

        let d_best = String::from_utf8(d_best_u8).unwrap();
        let q_best = String::from_utf8(q_best_u8).unwrap();

        let opt = ext_end.var;

        AlignResult { d_id, q_id, d_start, q_start, d_end, q_end, d_best, q_best, opt }
    }
}

pub use self::sw_avx2::smith_waterman_avx2;
pub use self::sw_scalar::smith_waterman_scalar;