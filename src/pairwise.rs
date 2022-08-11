use std::fmt::Display;
use std::collections::HashSet;

use tabular::row;
use tabular::Table;
use ndarray::Array2;

use crate::load::Seq;
use crate::avx::avx::M256Epu16;
use crate::avx::avx::max_epu16;

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

struct AlignEnd { var: u32, pos: (usize, usize) }

pub struct Profile { pub bias: u16, pub profile: Vec<Vec<M256Epu16>> }

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

impl Display for AlignResult
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        let result = Table::new("{:<} {:<} {:<}")
            .with_row(row!("d_id", ":", &self.d_id))
            .with_row(row!("q_id", ":", &self.q_id))
            .with_row(row!("opt", ":", self.opt));

        let mut indication_line = String::with_capacity(self.d_best.len());
        for (r1, r2) in self.d_best.chars().zip(self.q_best.chars())
        {
            let identifier = if [r1, r2].contains(&'-') {' '} else if r1 == r2 {'|'} else {'*'};
            indication_line.push(identifier);
        }

        let sub_seq = Table::new("{:<} {:<} {:<} {:<} {:<}")
            .with_row(row!("d_sub", ":", self.d_start+1, &self.d_best, self.d_end))
            .with_row(row!("", "", "", indication_line, ""))
            .with_row(row!("q_sub", ":", self.q_start+1, &self.q_best, self.q_end));
        
        write!(f, "{}{}", result, sub_seq)
    }
}

pub fn query_profile<S>(d: &Vec<u8>, q: &Vec<u8>, p: usize, f: S) -> Profile
where
    S: Fn(u8, u8) -> i16
{
    let alphabet_d: HashSet<u8> = HashSet::from_iter(d.iter().map(|r| *r));
    let alphabet_q: HashSet<u8> = HashSet::from_iter(q.iter().map(|r| *r));

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

    let seg_len = p;
    let seg_num = (q.len() + p - 1) / p;

    let mut seg_set = vec![Vec::new(); seg_num];
    for i in 0..seg_num
    {
        let mut seg = Vec::with_capacity(seg_len);
        for j in 0..seg_len
        {
            let residue = q.get(j * seg_num + i)
                .map_or(b'*', |item| *item);
            seg.push(residue);
        }
        seg_set[i] = seg;
    }

    let profile_len = alphabet_d.iter().max().map(|item| *item as usize).unwrap() - 64;
    let mut profile = vec![Vec::new(); profile_len];
    for residue in alphabet_d.iter()
    {
        let mut score_vec = Vec::with_capacity(seg_num);
        for seg in seg_set.iter()
        {
            let score = seg.iter()
                .map(|r| *r)
                .zip(vec![*residue; seg_len])
                .map(|(r1, r2)| n(r1, r2).unsigned_abs())
                .collect::<Vec<u16>>();
            score_vec.push(M256Epu16::from(&score[..]));
        }
        profile[(*residue - 65) as usize] = score_vec;
    }

    let bias = bias.unsigned_abs();
    Profile { bias, profile }
}

pub fn smith_waterman_avx2<S>(d: &Seq, q: &Seq, go: u16, ge: u16, f: S) -> u16
where
    S: Fn(u8, u8) -> i16
{
    let d_seq = d.seq.to_ascii_uppercase();
    let q_seq = q.seq.to_ascii_uppercase();
    
    let p = query_profile(&d_seq, &q_seq, 16, f);

    let go = M256Epu16::fill(go);
    let ge = M256Epu16::fill(ge);
    let bias = M256Epu16::fill(p.bias);
    let mut opt = M256Epu16::fill(0);

    let seg_num = (q_seq.len() + 15) / 16;
    let mut h_store = vec![M256Epu16::fill(0); seg_num];
    let mut e_store = vec![M256Epu16::fill(0); seg_num];

    for r in d_seq.iter()
    {
        let mut f = M256Epu16::fill(0);
        let mut prev_h = *h_store.last().unwrap() << 1;
        let mut h_buffer = vec![M256Epu16::fill(0); seg_num];
        for j in 0..seg_num
        {
            let score = p.profile[(*r - 65) as usize][j];
            let h = max_epu16!(prev_h + score - bias, e_store[j]);
            let e = max_epu16!(h-go, e_store[j]-ge);
            f = max_epu16!(h-go, f-ge);

            e_store[j] = e;
            h_buffer[j] = h;
            prev_h = h_store[j];
        }

        f = f << 1;
        let mut j = 0;
        while f > h_buffer[j] - go
        {
            h_buffer[j] = max_epu16!(f, h_buffer[j]);
            f = f - ge;

            if j+1 >= seg_num
            {
                f = f << 1;
                j = 0;
            }
        }
        h_buffer.iter().for_each(|item| opt = max_epu16!(*item, opt));
        h_store = h_buffer;
    }
    opt.get_max()
}

fn sw_scalar<S>(d: &Vec<u8>, q: &Vec<u8>, go: u32, ge: u32, terminater: u32, score: &S) -> AlignEnd
where
    S: Fn(u8, u8) -> i16
{
    let d_len = d.len();
    let q_len = q.len();

    let mut left_f: u32 = 0;
    let mut left_h: u32 = 0;
    let mut prev_e: Vec<u32> = vec![0; q_len+1];
    let mut prev_h: Vec<u32> = vec![0; q_len+1];

    let mut opt = AlignEnd { var: 0, pos: (0, 0) };
    if terminater == 0
    {
        for i in 1..d_len+1
        {
            let mut current_h = vec![0; q_len+1];
            for j in 1..q_len+1
            {
                let e = max!(prev_h[j].saturating_sub(go), prev_e[j].saturating_sub(ge));
                let f = max!(left_h.saturating_sub(go), left_f.saturating_sub(ge));

                let pair = score(d[i-1], q[j-1]);         
                let ext = match pair > 0
                {
                    true => prev_h[j-1].saturating_add(pair.unsigned_abs() as u32),
                    false => prev_h[j-1].saturating_sub(pair.unsigned_abs() as u32),
                };
                let h = max!(ext, e, f);

                left_f = f;
                left_h = h;
                prev_e[j] = e;
                current_h[j] = h;
            }

            let h_max = *current_h.iter().max().unwrap();
            if h_max > opt.var
            {
                let j = current_h.iter().position(|item| item==&h_max).unwrap();
                opt.var = h_max;
                opt.pos = (i, j);
            }
            
            prev_h = current_h;
        }
    }
    else
    {
        for i in 1..d_len+1
        {
            let mut current_h = vec![0; q_len+1];
            for j in 1..q_len+1
            {
                let e = max!(prev_h[j].saturating_sub(go), prev_e[j].saturating_sub(ge));
                let f = max!(left_h.saturating_sub(go), left_f.saturating_sub(ge));

                let pair = score(d[i-1], q[j-1]);
                let ext = match pair > 0
                {
                    true => prev_h[j-1].saturating_add(pair.unsigned_abs() as u32),
                    false => prev_h[j-1].saturating_sub(pair.unsigned_abs() as u32),
                };
                let h = max!(ext, e, f);

                if h == terminater
                {
                    opt.var = h;
                    opt.pos = (i, j);
                    break;
                }

                left_f = f;
                left_h = h;
                prev_e[j] = e;
                current_h[j] = h;
            }
            prev_h = current_h;
        }
    }
    opt
}

fn banded_sw_scalar<S>(d: &Vec<u8>, q: &Vec<u8>, go: u32, ge: u32, score: &S) -> (Vec<u8>, Vec<u8>)
where
    S: Fn(u8, u8) -> i16
{
    let d_len = d.len();
    let q_len = q.len();

    let mut left_f: u32 = 0;
    let mut left_h: u32 = 0;
    let mut prev_e: Vec<u32> = vec![0; q_len+1];
    let mut prev_h: Vec<u32> = vec![0; q_len+1];

    let mut direction = Array2::from_shape_vec((d_len+1, q_len+1), vec![0_u8; (d_len+1) * (q_len+1)]).unwrap();
    // let mut direction: Mat<u8> = Mat::init((d_len+1, q_len+1));
    
    for i in 1..d_len+1
    {
        let mut current_h = vec![0; q_len+1];
        for j in 1..q_len+1
        {
            let e = max!(prev_h[j].saturating_sub(go), prev_e[j].saturating_sub(ge));
            let f = max!(left_h.saturating_sub(go), left_f.saturating_sub(ge));

            let pair = score(d[i-1], q[j-1]);
            let ext = match pair > 0
            {
                true => prev_h[j-1].saturating_add(pair.unsigned_abs() as u32),
                false => prev_h[j-1].saturating_sub(pair.unsigned_abs() as u32),
            };
            let h = max!(ext, e, f);

            direction[(i, j)] = match h
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
        prev_h = current_h;
    }

    let mut d_best = Vec::new();
    let mut q_best = Vec::new();

    let mut i = d_len;
    let mut j = q_len;
    while direction[(i, j)] != 0
    {
        if direction[(i, j)] == 1
        {
            d_best.insert(0, d[i-1]);
            q_best.insert(0, b'-');
            i = i - 1;
            continue;
        }

        if direction[(i, j)] == 2
        {
            d_best.insert(0, b'-');
            q_best.insert(0, q[j-1]);
            j = j - 1;
            continue;
        }

        if direction[(i, j)] == 3
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
    S: Fn(u8, u8) -> i16
{
    let d_seq = d.seq.to_ascii_uppercase();
    let q_seq = q.seq.to_ascii_uppercase();
    let d_id = d.id.to_string();
    let q_id = q.id.to_string();
    
    let ext_end = sw_scalar(&d_seq, &q_seq, go, ge, 0, &score);
    let d_end = ext_end.pos.0;
    let q_end = ext_end.pos.1;
    let mut d_splited_rev = d_seq[0..d_end].to_vec();
    let mut q_splited_rev = q_seq[0..q_end].to_vec();

    d_splited_rev.reverse();
    q_splited_rev.reverse();

    let ext_start = sw_scalar(&d_splited_rev, &q_splited_rev, go, ge, ext_end.var, &score);

    let d_start = d_end - ext_start.pos.0;
    let q_start = q_end - ext_start.pos.1;

    let d_sub = d_seq[d_start..d_end].to_vec();
    let q_sub = q_seq[q_start..q_end].to_vec();

    let (d_best_u8, q_best_u8) = banded_sw_scalar(&d_sub, &q_sub, go, ge, &score);
    let d_best = String::from_utf8(d_best_u8).unwrap();
    let q_best = String::from_utf8(q_best_u8).unwrap();
    let opt = ext_end.var;

    AlignResult { d_id, q_id, d_start, q_start, d_end, q_end, d_best, q_best, opt }
}
