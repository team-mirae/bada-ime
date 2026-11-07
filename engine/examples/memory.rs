//! Loads the shipped data as the shell does, then waits so `footprint` can measure the process.
fn main() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/data/");
    let read = |f: &str| std::fs::read_to_string(format!("{dir}{f}")).unwrap();
    let started = std::time::Instant::now();
    let data = bada::Data::parse(&read("ko-words.tsv"), &read("en-words.txt"), &read("hanja.tsv"), &read("oecd.tsv"))
        .with_japanese(&read("ja-words.txt"))
        .with_kanji(&std::fs::read(format!("{dir}kanji.bin")).unwrap());
    println!("loaded in {:?}, pid {}", started.elapsed(), std::process::id());
    std::thread::sleep(std::time::Duration::from_secs(8));
    std::hint::black_box(data);
}
