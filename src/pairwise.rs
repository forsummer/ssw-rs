use std::fmt::Display;
use std::collections::HashMap;
use std::collections::HashSet;

use crate::max_epi32;
use crate::load::Seq;
use crate::utils::M256Epi32;

struct Node
{
    i: usize,
    j: usize,
    opt: M256Epi32
}

pub struct AlignResult
{
    pub d_id: String,
    pub q_id: String,
    pub d_start: usize,
    pub q_start: usize,
    pub opt: i32
}

impl Display for AlignResult
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "target_name: {:>}\nquery_name: {:>}\ntarget_start: {:>}\nquery_start: {:>}\nopt: {:>}", self.d_id, self.q_id, self.d_start, self.q_start, self.opt)
    }
}

fn diving(q: &str, p: usize) -> Vec<String> 
{
    let seg_len = (q.len() + p - 1) / p;
    let residue_arr_num = seg_len;

    let mut remainder = q;
    let mut seg_set = Vec::new();
    while remainder.len() >= seg_len
    {
        let (seg, right_remainder) = remainder.split_at(seg_len);
        remainder = right_remainder;
        seg_set.push(seg.to_string());
    }
    if remainder.len() > 0
    {
        let tail = "*".repeat( seg_len - remainder.len() );
        let seg = format!("{}{}", remainder, tail);
        seg_set.push(seg);
    }

    let mut residue_arr_set = Vec::new();
    for i in 0..residue_arr_num
    {
        let mut residue_arr = String::new();
        for seg in seg_set.iter()
        {
            let residue = seg.chars().collect::<Vec<char>>()[i];
            residue_arr.push(residue);
        }
        residue_arr_set.push(residue_arr);
    }
    residue_arr_set
}

fn query_profile(d: &str, q: &str, p: usize, match_: i32, miss_: i32) -> HashMap<String, HashMap<char, M256Epi32>>
{
    let seg_set = diving(q, p);
    let alphabet: HashSet<char> = HashSet::from_iter(d.chars());
    let pair_score = |r1, r2| 
    {
        if r1 == '*' || r2 == '*' { 0 }  else if r1 == r2 { match_ } else { miss_ }
    };

    let mut profile = HashMap::new();
    for seg in seg_set.iter()
    {
        profile.insert(seg.clone(), HashMap::new());
        for nucleotide in alphabet.iter()
        {
            let mut score = Vec::with_capacity(p);
            for residue in seg.chars()
            {
                score.push(pair_score(*nucleotide, residue));
            }
            profile.get_mut(seg).unwrap()
                .insert(*nucleotide, M256Epi32::from_arr(score.as_slice().try_into().unwrap()));
        }
    }
    profile
}

pub fn smith_waterman_avx2(d: &Seq, q: &Seq, match_: i32, miss_: i32, go: i32, ge: i32) -> AlignResult
{
    let d_seq: Vec<char> = d.seq.clone().to_uppercase().chars().collect();

    let go = go.abs();
    let ge = ge.abs();
    
    let profile = query_profile(&d.seq, &q.seq, 8, match_, miss_);
    let residue_arr_set = diving(&q.seq, 8);
    let residue_arr_num = residue_arr_set.len();

    let gap_open = M256Epi32::fill(go);
    let gap_extend = M256Epi32::fill(ge);
    let zero = M256Epi32::zero();
    let mut opt = M256Epi32::zero();
    let mut h_store = vec![M256Epi32::zero(); residue_arr_num+1];
    let mut e_store = vec![M256Epi32::zero(); residue_arr_num+1];
    let mut opt_pos = Node { i: 0, j: 0, opt: M256Epi32::zero() };
    for i in 1..d_seq.len()+1
    {
        let mut f = M256Epi32::zero();
        let mut prev_h = h_store[residue_arr_num-1] << 1;
        let mut h_buffer = vec![M256Epi32::zero(); residue_arr_num+1];
        for j in 1..residue_arr_num+1
        {
            let score = profile[&residue_arr_set[j-1]][&d_seq[i-1]];
            let h = max_epi32!(prev_h+score, e_store[j], f, zero);
            let e = max_epi32!(h-gap_open, e_store[j]-gap_extend, zero);
            f = max_epi32!(h-gap_open, f-gap_extend, zero);

            prev_h = h;
            h_buffer[j] = h;
            e_store[j] = e;

            let tmp = max_epi32!(e, f, h);
            if tmp.get_max_m256_i32() > opt.get_max_m256_i32()
            {
                opt = tmp;
                opt_pos.i = i;
                opt_pos.j = j;
                opt_pos.opt = opt;
            }
        }

        f = f << 1;
        let mut j = 0;
        while f > h_buffer[j] - gap_open
        {
            h_buffer[j] = max_epi32!(f, h_buffer[j]);

            f = f - gap_extend;

            if j+1 >= residue_arr_num
            {
                f = f << 1;
                j = 0;
            }
        }
        h_store = h_buffer;
    }
    AlignResult
    {
        d_id: d.id.clone(),
        q_id: q.id.clone(),
        d_start: opt_pos.i,
        q_start: opt_pos.j,
        opt: opt.get_max_m256_i32(),
    }
}