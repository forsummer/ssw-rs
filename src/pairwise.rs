use std::fmt::Display;
use std::vec;

use tabular::row;
use tabular::Table;

use crate::max;
use crate::load::Seq;
use crate::utils::Mat;

pub struct AlignResult
{
    pub d_id: String,
    pub q_id: String,
    pub d_start: usize,
    pub q_start: usize,
    pub d_end: usize,
    pub q_end: usize,
    pub d_sub: String,
    pub q_sub: String,
    pub opt: i32,
}

impl Display for AlignResult
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        let table = Table::new("{:<} {:<} {:<}")
            .set_line_end("\n")
            .with_row(row!("t_id", ":", &self.d_id))
            .with_row(row!("q_id", ":", &self.q_id))
            .with_row(row!("opt", ":", self.opt));

        let mut indication_line = String::new();
        let d_sub = self.d_sub.clone();
        let q_sub = self.q_sub.clone();
        for (r1, r2) in d_sub.chars().zip(q_sub.chars())
        {
            let c = if [r1, r2].contains(&'-') {' '} else if r1 == r2 {'|'} else {'*'};
            indication_line.push(c);
        }

        let seq = Table::new("{:<} {:<} {:<} {:<} {:<}")
                        .with_row(row!("target", ":", self.d_start, &self.d_sub, self.d_end))
                        .with_row(row!("", "", "", indication_line, ""))
                        .with_row(row!("query", ":", self.q_start, &self.q_sub, self.q_end));
        write!(f, "{}{}", &table, &seq)
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