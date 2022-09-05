use needletail::parse_fastx_file;

pub struct Seq { pub id: String, pub seq: Vec<u8> }

pub fn fastx_parser<P>(path: P) -> Vec<Seq>
where
    P: AsRef<std::path::Path>
{
    let mut reader = parse_fastx_file(path)
        .map_or_else(|err| panic!("{}", err.msg), |reader| reader);

    let mut seq_set = Vec::new();

    while let Some(item) = reader.next()
    {
        let record = item.map_or_else(|err| panic!("{}", err.msg), |r| r);
        let id = String::from_utf8(record.id()
                        .to_vec())
                        .map_or_else(|err| panic!("{}", err), |s| s);
        let seq = record.seq().to_vec();
        seq_set.push(Seq { id, seq });
    }
    seq_set
}
