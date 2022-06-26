use std::fmt::Display;

use tabular::row;
use tabular::Table;

use crate::max;
use crate::load::Seq;

#[allow(dead_code)]
#[derive(Debug)]
struct Cell
{
    i: usize,
    j: usize,
    score: i32,
}

pub struct AlignResult
{
    pub d_id: String,
    pub q_id: String,
    pub d_start: usize,
    pub q_start: usize,
    pub opt: i32,
    pub sub_opt: i32,
}

impl Display for AlignResult
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        let table = Table::new("{:<} {:<} {:<}")
            .set_line_end("\n")
            .with_row(row!("d_id", ":", self.d_id.clone()))
            .with_row(row!("q_id", ":", self.q_id.clone()))
            .with_row(row!("d_start", ":", self.d_start))
            .with_row(row!("q_start", ":", self.q_start))
            .with_row(row!("opt", ":", self.opt))
            .with_row(row!("sub_opt", ":", self.sub_opt));
        write!(f, "{}", &table)
    }
}

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

    let mut d_start = 0;
    let mut q_start = 0;
    let mut prev_e = vec![0; q_len+1];
    let mut prev_h = vec![0; q_len+1];
    let score = |r1, r2| if r1 == r2 { match_ } else { miss_ };

    let mut opt = 0;
    let mut sub_opt = 0;
    let mut path = Vec::new();
    for i in 1..d_len+1
    {
        let mut left_f = 0;
        let mut left_h = 0;
        let mut current_h = vec![0; q_len+1];
        for j in 1..q_len+1
        {
            let e = max!(prev_h[j]-go, prev_e[j]-ge, 0);
            let f = max!(left_h-go, left_f-ge, 0);
            let h = max!(prev_h[j-1]+score(d_seq[i-1], q_seq[j-1]), e, f, 0);

            let mut cell_score = vec![e, f, h];
            cell_score.sort();

            let tmp_opt = cell_score.pop().unwrap();
            let tmp_sub = cell_score.pop().unwrap();
            if tmp_opt > opt
            {
                opt = tmp_opt;
                d_start = i;
                q_start = j;
                path.push(Cell { i, j, score: opt });
            }

            if tmp_sub > sub_opt
            {
                sub_opt = tmp_sub;
            }

            left_f = f;
            left_h = h;
            prev_e[j] = e; 
            current_h[j] = h;
        }
        prev_h = current_h;
    }
    AlignResult { d_id, q_id, d_start, q_start, opt, sub_opt }
}