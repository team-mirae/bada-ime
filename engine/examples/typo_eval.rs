//! Evaluates the typo suggestions (src/typo.rs) on synthetic slips of common words.
//! cargo run --release --example typo_eval [WORDS]
use bada::layout::{compose_presses, BADA};
use bada::typo::{bada_keys, scored, slips, JamoLm};

fn main() {
    let dir = format!("{}/data", env!("CARGO_MANIFEST_DIR"));
    let lm = JamoLm::parse(&std::fs::read(format!("{dir}/ko-jamo-4gram.bin")).unwrap()).unwrap();
    let n: usize = std::env::args().nth(1).and_then(|a| a.parse().ok()).unwrap_or(3000);
    let tsv = std::fs::read_to_string(format!("{dir}/ko-words.tsv")).unwrap();
    let mut words: Vec<(&str, u64)> = tsv
        .lines()
        .filter(|l| !l.starts_with('#'))
        .filter_map(|l| l.split_once('\t'))
        .filter_map(|(w, c)| Some((w, c.parse().ok()?)))
        .filter(|(w, _)| w.chars().all(|c| ('가'..='힣').contains(&c)))
        .collect();
    words.sort_by(|a, b| b.1.cmp(&a.1));
    words.truncate(50_000);
    let mut seed: u64 = 0x5eed;
    let mut rand = || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (seed >> 33) as f64 / (1u64 << 31) as f64
    };
    let thresholds = [0.0, -0.5, -1.0, -1.5, -2.0, -3.0];
    let (mut tried, mut hit1, mut hit3) = (0, vec![0; 6], vec![0; 6]);
    let mut clean_any = vec![0; 6];
    let mut clean_n = vec![0usize; 6];
    let mut by_cost: std::collections::BTreeMap<String, (u32, u32)> = Default::default();
    // Words drawn by frequency, from the 50 000 commonest.
    let total: f64 = words.iter().map(|w| (w.1 as f64).sqrt()).sum();
    for _ in 0..n {
        let mut r = rand() * total;
        let word = words.iter().find(|w| { r -= (w.1 as f64).sqrt(); r <= 0.0 }).unwrap_or(&words[0]).0;
        let Some(keys) = bada_keys(word) else { continue };
        if compose_presses(BADA, &keys) != word {
            continue;
        }
        let clean = scored(&lm, &keys);
        for (i, t) in thresholds.iter().enumerate() {
            let k = clean.iter().filter(|s| s.1 >= *t).count().min(3);
            clean_n[i] += k;
            clean_any[i] += (k > 0) as usize;
        }
        // One slip, drawn by its cost.
        let v: Vec<_> = slips(&keys).into_iter().filter(|(p, _)| compose_presses(BADA, p) != word).collect();
        if v.is_empty() {
            continue;
        }
        let wsum: f64 = v.iter().map(|x| 10f64.powf(x.1)).sum();
        let mut r = rand() * wsum;
        let (typo, cost) = v.iter().find(|x| { r -= 10f64.powf(x.1); r <= 0.0 }).unwrap_or(&v[0]);
        tried += 1;
        let got = scored(&lm, typo);
        let rank = got.iter().position(|s| s.0 == word);
        let e = by_cost.entry(format!("{cost}")).or_default();
        e.0 += 1;
        if rank.is_some_and(|r| r < 3) && got[rank.unwrap()].1 >= -1.0 {
            e.1 += 1;
        }
        for (i, t) in thresholds.iter().enumerate() {
            let shown: Vec<_> = got.iter().filter(|s| s.1 >= *t).take(3).collect();
            if shown.first().is_some_and(|s| s.0 == word) {
                hit1[i] += 1;
            }
            if shown.iter().any(|s| s.0 == word) {
                hit3[i] += 1;
            }
        }
    }
    println!("{tried} slipped words, {n} clean words (suggestions capped at 3)");
    println!("threshold  found@1  found@3  clean: any suggestion  avg shown");
    for (i, t) in thresholds.iter().enumerate() {
        println!(
            "{t:>9}  {:>6.1}%  {:>6.1}%  {:>20.1}%  {:>9.2}",
            100.0 * hit1[i] as f64 / tried as f64,
            100.0 * hit3[i] as f64 / tried as f64,
            100.0 * clean_any[i] as f64 / n as f64,
            clean_n[i] as f64 / n as f64
        );
    }
    println!("by slip cost (found@3 at -1.0):");
    for (c, (t, h)) in by_cost {
        println!("  {c:>5}: {h}/{t} = {:.1}%", 100.0 * h as f64 / t as f64);
    }
}
