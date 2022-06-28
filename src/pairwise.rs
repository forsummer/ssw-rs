// use std::collections::HashMap;
// use std::collections::HashSet;

use crate::max;
// use crate::max_epi32;
use crate::load::Seq;
use crate::utils::Mat;
use crate::utils::AlignResult;
// use crate::utils::M256Epi32;

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

pub fn smith_waterman_serial(d: &Seq, q: &Seq, match_: i32, miss_: i32, go: i32, ge: i32) -> AlignResult
{
    let d_id = d.id.clone();
    let q_id = q.id.clone();
    let d_seq: Vec<char> = d.seq.to_uppercase().chars().collect();
    let q_seq: Vec<char> = q.seq.to_uppercase().chars().collect();
    let d_len = d_seq.len();
    let q_len = q_seq.len();

    let go = go.abs();
    let ge = ge.abs();

    let mut prev_e = vec![0; q_len+1];
    let mut prev_h = vec![0; q_len+1];
    let score = |r1, r2| if r1 == r2 { match_ } else { miss_ };
    
    let mut opt = 0;
    let (mut d_start, mut q_start) = (0, 0);
    let (mut d_end, mut q_end) = (0, 0);
    let mut direction_mat: Mat<char> = Mat::init((d_len+1, q_len+1));
    for i in 1..d_len+1
    {
        let mut left_f = 0;
        let mut left_h = 0;
        let mut current_h = vec![0; q_len+1];
        for j in 1..q_len+1
        {
            let e = max!(prev_h[j]-go, prev_e[j]-ge, 0);
            let f = max!(left_h-go, left_f-ge, 0);

            let ext = prev_h[j-1] + score(d_seq[i-1], q_seq[j-1]);
            let h = max!(ext, e, f, 0);

            let tmp_opt = *[e, f, h].iter().max().unwrap();
            if tmp_opt > opt
            {
                opt = tmp_opt;
                d_end = i;
                q_end = j;
            }

            if h == e { direction_mat[i][j] = 'E' }
            if h == f { direction_mat[i][j] = 'F' }
            if h == ext { direction_mat[i][j] = 'H' }
            if h == 0 { direction_mat[i][j] = char::default() }

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
    while direction_mat[i][j] != char::default()
    {
        if direction_mat[i][j] == 'E'
        {
            d_sub.insert(0, d_seq[i-1]);
            q_sub.insert(0, '-');
            i = i - 1;
            d_start = i;
            continue;
        }

        if direction_mat[i][j] == 'F'
        {
            d_sub.insert(0, '-');
            q_sub.insert(0, q_seq[j-1]);
            j = j - 1;
            q_start = j;
            continue;
        }

        if direction_mat[i][j] == 'H'
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