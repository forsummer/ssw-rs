use std::fmt::Display;
// use std::collections::HashMap;
// use std::c&ollections::HashSet;

use tabular::row;
use tabular::Table;

use crate::max;
// use crate::max_epi32;
use crate::load::Seq;
use crate::utils::matrix::Mat;
// use crate::utils::M256Epi32;

pub struct AlignResult
{
    d_id: String,
    q_id: String,
    d_start: usize,
    q_start: usize,
    d_end: usize,
    q_end: usize,
    d_sub: String,
    q_sub: String,
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

        let mut indication_line = String::with_capacity(self.d_sub.len());
        for (r1, r2) in self.d_sub.chars().zip(self.q_sub.chars())
        {
            let identifier = if [r1, r2].contains(&'-') {' '} else if r1 == r2 {'|'} else {'*'};
            indication_line.push(identifier);
        }

        let sub_seq = Table::new("{:<} {:<} {:<} {:<} {:<}")
            .with_row(row!("d_sub", ":", self.d_start, &self.d_sub, self.d_end))
            .with_row(row!("", "", "", indication_line, ""))
            .with_row(row!("q_sub", ":", self.q_start, &self.q_sub, self.q_end));
        
        write!(f, "{}{}", result, sub_seq)
    }
}

// fn diving(q: &str, p: usize) -> Vec<String> 
// {
//     let seg_len = (q.len() + p - 1) / p;
//     let residue_arr_num = seg_len;

//     let mut remainder = q;
//     let mut seg_set = Vec::new();
//     while remainder.len() >= seg_len
//     {
//         let (seg, right_remainder) = remainder.split_at(seg_len);
//         remainder = right_remainder;
//         seg_set.push(seg.to_string());
//     }
//     if remainder.len() > 0
//     {
//         let tail = "*".repeat( seg_len - remainder.len() );
//         let seg = format!("{}{}", remainder, tail);
//         seg_set.push(seg);
//     }

//     let mut residue_arr_set = Vec::new();
//     for i in 0..residue_arr_num
//     {
//         let mut residue_arr = String::new();
//         for seg in seg_set.iter()
//         {
//             let residue = seg.chars().collect::<Vec<char>>()[i];
//             residue_arr.push(residue);
//         }
//         residue_arr_set.push(residue_arr);
//     }
//     residue_arr_set
// }

// fn query_profile(d: &str, q: &str, p: usize, match_: i32, miss_: i32) -> HashMap<String, HashMap<char, M256Epi32>>
// {
//     let seg_set = diving(q, p);
//     let alphabet: HashSet<char> = HashSet::from_iter(d.chars());
//     let pair_score = |r1, r2| 
//     {
//         if r1 == '*' || r2 == '*' { 0 }  else if r1 == r2 { match_ } else { miss_ }
//     };

//     let mut profile = HashMap::new();
//     for seg in seg_set.iter()
//     {
//         profile.insert(seg.clone(), HashMap::new());
//         for nucleotide in alphabet.iter()
//         {
//             let mut score = Vec::with_capacity(p);
//             for residue in seg.chars()
//             {
//                 score.push(pair_score(*nucleotide, residue));
//             }
//             profile.get_mut(seg).unwrap()
//                 .insert(*nucleotide, M256Epi32::from_arr(score.as_slice().try_into().unwrap()));
//         }
//     }
//     profile
// }

// #[allow(dead_code)]
// pub fn smith_waterman_avx2(d: &Seq, q: &Seq, match_: i32, miss_: i32, go: i32, ge: i32) -> AlignResult
// {
//     let d_seq: Vec<char> = d.seq.clone().to_uppercase().chars().collect();

//     let go = go.abs();
//     let ge = ge.abs();
    
//     let profile = query_profile(&d.seq, &q.seq, 8, match_, miss_);
//     let residue_arr_set = diving(&q.seq, 8);
//     let residue_arr_num = residue_arr_set.len();

//     let gap_open = M256Epi32::fill(go);
//     let gap_extend = M256Epi32::fill(ge);
//     let zero = M256Epi32::zero();
//     let mut opt = M256Epi32::zero();
//     let mut h_store = vec![M256Epi32::zero(); residue_arr_num+1];
//     let mut e_store = vec![M256Epi32::zero(); residue_arr_num+1];
//     let mut opt_pos = Node { i: 0, j: 0, opt: M256Epi32::zero() };
//     for i in 1..d_seq.len()+1
//     {
//         let mut f = M256Epi32::zero();
//         let mut prev_h = h_store[residue_arr_num-1] << 1;
//         let mut h_buffer = vec![M256Epi32::zero(); residue_arr_num+1];
//         for j in 1..residue_arr_num+1
//         {
//             let score = profile[&residue_arr_set[j-1]][&d_seq[i-1]];
//             let h = max_epi32!(prev_h+score, e_store[j], f, zero);
//             let e = max_epi32!(h-gap_open, e_store[j]-gap_extend, zero);
//             f = max_epi32!(h-gap_open, f-gap_extend, zero);

//             prev_h = h;
//             h_buffer[j] = h;
//             e_store[j] = e;

//             let tmp = max_epi32!(e, f, h);
//             if tmp.get_max_m256_i32() > opt.get_max_m256_i32()
//             {
//                 opt = tmp;
//                 opt_pos.i = i;
//                 opt_pos.j = j;
//                 opt_pos.opt = opt;
//             }
//         }

//         f = f << 1;
//         let mut j = 0;
//         while f > h_buffer[j] - gap_open
//         {
//             h_buffer[j] = max_epi32!(f, h_buffer[j]);

//             f = f - gap_extend;

//             if j+1 >= residue_arr_num
//             {
//                 f = f << 1;
//                 j = 0;
//             }
//         }
//         h_store = h_buffer;
//     }
//     AlignResult
//     {
//         d_id: d.id.clone(),
//         q_id: q.id.clone(),
//         d_start: opt_pos.i,
//         q_start: opt_pos.j,
//         opt: opt.get_max_m256_i32(),
//     }
// }

pub fn smith_waterman_serial(d: &Seq, q: &Seq, match_: u32, miss_: u32, go: u32, ge: u32) -> AlignResult
{
    let d_id = d.id.clone();
    let q_id = q.id.clone();
    let d_seq: Vec<char> = d.seq.to_uppercase().chars().collect();
    let q_seq: Vec<char> = q.seq.to_uppercase().chars().collect();
    let d_len = d_seq.len();
    let q_len = q_seq.len();

    let mut prev_e: Vec<u32> = vec![0; q_len+1];
    let mut prev_h: Vec<u32> = vec![0; q_len+1];
    
    let mut opt = 0;
    let (mut d_start, mut q_start) = (0, 0);
    let (mut d_end, mut q_end) = (0, 0);
    let mut direction_mat: Mat<u8> = Mat::init((d_len+1, q_len+1));
    for i in 1..d_len+1
    {
        let mut left_f: u32 = 0;
        let mut left_h: u32 = 0;
        let mut current_h: Vec<u32> = vec![0; q_len+1];
        for j in 1..q_len+1
        {
            let e = max!(prev_h[j].saturating_sub(go), prev_e[j].saturating_sub(ge));
            let f = max!(left_h.saturating_sub(go), left_f.saturating_sub(ge));

            let ext = match d_seq[i-1] == q_seq[j-1]
            {
                true => prev_h[j-1].saturating_add(match_),
                false => prev_h[j-1].saturating_sub(miss_),
            };

            let h = max!(ext, e, f);

            let tmp_opt = *[e, f, h].iter().max().unwrap();
            if tmp_opt > opt
            {
                opt = tmp_opt;
                d_end = i;
                q_end = j;
            }

            direction_mat[i][j] = match h
            {
                var1 if var1 == e   => 1,
                var2 if var2 == f   => 2,
                var3 if var3 == ext => 3,
                var4 if var4 == 0   => 0,
                _                        => 0,
            };

            left_f = f;
            left_h = h;
            prev_e[j] = e; 
            current_h[j] = h;
        }
        prev_h = current_h;
    }

    let (mut i, mut j) = (d_end, q_end);
    let mut d_sub  = String::new();
    let mut q_sub  = String::new();
    while direction_mat[i][j] != 0
    {
        if direction_mat[i][j] == 1
        {
            d_sub.insert(0, d_seq[i-1]);
            q_sub.insert(0, '-');
            i = i - 1;
            d_start = i;
            continue;
        }

        if direction_mat[i][j] == 2
        {
            d_sub.insert(0, '-');
            q_sub.insert(0, q_seq[j-1]);
            j = j - 1;
            q_start = j;
            continue;
        }

        if direction_mat[i][j] == 3
        {
            d_sub.insert(0, d_seq[i-1]);
            q_sub.insert(0, q_seq[j-1]);
            (i, j) = (i - 1, j - 1);
            (d_start, q_start) = (i, j);
            continue;
        }
    }
    (d_start, q_start) = (d_start + 1, q_start + 1);
    AlignResult { d_id, q_id, d_start, q_start, d_end, q_end, d_sub, q_sub, opt }
}