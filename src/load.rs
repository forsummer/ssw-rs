use std::path::Path;
use needletail::parse_fastx_file;

pub struct Seq { pub id: String, pub seq: String }

pub fn load_fastx<P>(path: P) -> Vec<Seq>
where
    P: AsRef<Path>
{
    let mut reader = parse_fastx_file(path)
        .map_or_else(|err| panic!("{}", err.msg), |reader| reader);

    let mut seq_set = Vec::new();

    while let Some(item) = reader.next()
    {
        let record = item.map_or_else(|err| panic!("{}", err.msg), |r| r);
        let id = String::from_utf8(record.id().to_vec()).map_or_else(|err| panic!("{}", err), |s| s);
        let seq = String::from_utf8(record.seq().to_vec()).map_or_else(|err| panic!("{}", err), |s| s);
        seq_set.push(Seq { id, seq });
    }
    seq_set
}