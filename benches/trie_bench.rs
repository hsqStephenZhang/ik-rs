use criterion::*;
use ik_rs::dict::trie::Trie;

fn trie_build() -> Trie {
    let mut trie = Trie::default();
    trie.insert("Test".chars());
    trie.insert("Tea".chars());
    trie.insert("Background".chars());
    trie.insert("Back".chars());
    trie.insert("Brown".chars());
    trie
}

fn trie_match() {
    let mut trie = trie_build();
    trie.match_word("Back".chars());
    trie.match_word("Tea".chars());
}

fn trie_benchmark(c: &mut Criterion) {
    c.bench_function("trie match", |b| b.iter(trie_match));
}

criterion_group!(benches, trie_benchmark);
criterion_main!(benches);
