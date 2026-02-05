#![allow(uncommon_codepoints)]
use {
    criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main},
    e62rs::utils::*,
    serde::{Deserialize, Serialize},
    std::{hint::black_box, time::Duration},
    tempfile::TempDir,
};

#[derive(Debug, Serialize, Deserialize)]
struct TestData {
    id: u64,
    name: String,
    values: Vec<i64>,
    nested: NestedData,
}

#[derive(Debug, Serialize, Deserialize)]
struct NestedData {
    count: usize,
    items: Vec<String>,
}

impl TestData {
    fn new(size: usize) -> Self {
        Self {
            id: 12345,
            name: "test_benchmark".to_string(),
            values: (0..size as i64).collect(),
            nested: NestedData {
                count: size,
                items: (0..size).map(|i| format!("item_{}", i)).collect(),
            },
        }
    }
}

fn bench_deserialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("deserialization");

    group.bench_function("bool_from_str/true", |b| {
        b.iter(|| {
            let mut deserializer = serde_json::Deserializer::from_str("\"t\"");
            deserialize_bool_from_str(&mut deserializer).unwrap()
        })
    });

    group.bench_function("bool_from_str/false", |b| {
        b.iter(|| {
            let mut deserializer = serde_json::Deserializer::from_str("\"f\"");
            deserialize_bool_from_str(&mut deserializer).unwrap()
        })
    });

    group.bench_function("post_ids/empty", |b| {
        b.iter(|| {
            let mut deserializer = serde_json::Deserializer::from_str("\"\"");
            deserialize_post_ids(&mut deserializer).unwrap()
        })
    });

    group.bench_function("post_ids/single", |b| {
        b.iter(|| {
            let mut deserializer = serde_json::Deserializer::from_str("\"{123}\"");
            deserialize_post_ids(&mut deserializer).unwrap()
        })
    });

    group.bench_function("post_ids/multiple", |b| {
        b.iter(|| {
            let mut deserializer = serde_json::Deserializer::from_str("\"{1,2,3,4,5}\"");
            deserialize_post_ids(&mut deserializer).unwrap()
        })
    });

    group.finish();
}

fn bench_string_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_operations");

    let test_paths = vec![
        ("short", "/usr/bin"),
        ("medium", "/home/user/documents/projects/rust/src/main.rs"),
        (
            "long",
            "/very/long/path/with/many/components/that/needs/shortening/for/display/purposes",
        ),
        (
            "deep",
            "/a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q/r/s/t/u/v/w/x/y/z",
        ),
    ];

    for (name, path) in test_paths {
        group.bench_with_input(
            BenchmarkId::new("shorten_path/max_len_10", name),
            path,
            |b, path| b.iter(|| shorten_path(black_box(path), 10)),
        );

        group.bench_with_input(
            BenchmarkId::new("shorten_path/max_len_20", name),
            path,
            |b, path| b.iter(|| shorten_path(black_box(path), 20)),
        );
    }

    let log_levels = vec![
        "debug", "info", "warn", "error", "trace", "d", "i", "w", "e", "t",
    ];
    for level in log_levels {
        group.bench_with_input(
            BenchmarkId::new("string_to_log_level", level),
            level,
            |b, level| b.iter(|| string_to_log_level(black_box(level))),
        );
    }

    group.finish();
}

fn bench_file_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_operations");
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    let data_sizes = vec![10, 100, 1000];

    for size in data_sizes {
        let data = TestData::new(size);

        group.bench_with_input(
            BenchmarkId::new("write_json/pretty", size),
            &data,
            |b, data| {
                b.iter_batched(
                    || temp_path.join(format!("bench_pretty_{}.json", size)),
                    |path| {
                        let mut writer = FileWriter::json(&path, true).unwrap();
                        writer.write(data).unwrap();
                        writer.flush().unwrap();
                        path
                    },
                    BatchSize::SmallInput,
                )
            },
        );

        group.bench_with_input(
            BenchmarkId::new("write_json/compact", size),
            &data,
            |b, data| {
                b.iter_batched(
                    || temp_path.join(format!("bench_compact_{}.json", size)),
                    |path| {
                        let mut writer = FileWriter::json(&path, false).unwrap();
                        writer.write(data).unwrap();
                        writer.flush().unwrap();
                        path
                    },
                    BatchSize::SmallInput,
                )
            },
        );
    }

    let text_sizes = vec![100, 1000, 10000];
    for size in text_sizes {
        let text = "Hello, World! ".repeat(size / 14);

        group.bench_with_input(BenchmarkId::new("write_text", size), &text, |b, text| {
            b.iter_batched(
                || temp_path.join(format!("bench_text_{}.txt", size)),
                |path| {
                    let mut writer = FileWriter::text(&path).unwrap();
                    writer.write_text(text).unwrap();
                    writer.flush().unwrap();
                    path
                },
                BatchSize::SmallInput,
            )
        });
    }

    group.finish();
}

fn bench_repeat_traits(c: &mut Criterion) {
    let mut group = c.benchmark_group("repeat_traits");

    let counter = std::cell::Cell::new(0);
    group.bench_function("repeat/simple", |b| {
        b.iter(|| {
            let counter = &counter;
            let f = || counter.set(counter.get() + 1);
            f.repeat(100);
        })
    });

    group.bench_function("repeat_collect/vec", |b| {
        b.iter(|| {
            let f = || vec![1, 2, 3];
            f.repeat_collect(50)
        })
    });

    group.bench_function("repeat_with/add", |b| {
        b.iter(|| {
            let add = |x: i32| x + 1;
            add.repeat_with(100, 42)
        })
    });

    group.finish();
}

fn bench_repeatable(c: &mut Criterion) {
    let mut group = c.benchmark_group("repeatable_operations");

    let counter = std::cell::Cell::new(0);
    group.bench_function("repeatable/counter", |b| {
        b.iter(|| {
            let counter = &counter;
            repeatable(|| counter.set(counter.get() + 1)).repeat_collect(100)
        })
    });

    group.bench_function("repeatable/string_builder", |b| {
        b.iter(|| {
            let s = std::cell::RefCell::new(String::new());
            repeatable(|| {
                let mut s = s.borrow_mut();
                s.push('a');
                s.len()
            })
            .repeat_collect(50)
        })
    });

    group.finish();
}

fn bench_iterator_extensions(c: &mut Criterion) {
    let mut group = c.benchmark_group("iterator_extensions");

    group.bench_function("repeat_next/range", |b| {
        b.iter(|| {
            let mut iter = 0..1000;
            iter.repeat_next(50)
        })
    });

    group.bench_function("repeat_next/vec_iter", |b| {
        b.iter(|| {
            let vec: Vec<i32> = (0..1000).collect();
            let mut iter = vec.into_iter();
            iter.repeat_next(50)
        })
    });

    group.bench_function("skip_n/range", |b| {
        b.iter(|| {
            let mut iter = 0..1000;
            iter.skip_n(50);
            iter.next()
        })
    });

    group.finish();
}

fn bench_all(c: &mut Criterion) {
    bench_deserialization(c);
    bench_string_operations(c);
    bench_string_operations(c);
    bench_file_operations(c);
    bench_repeat_traits(c);
    bench_repeatable(c);
    bench_iterator_extensions(c);
}

criterion_group!(benches, bench_all);
criterion_main!(benches);

criterion_group! {
    name = deserialization_bench;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(5))
        .sample_size(500);
    targets = bench_deserialization
}

criterion_group! {
    name = file_io_bench;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .sample_size(300);
    targets = bench_file_operations
}

criterion_group! {
    name = string_ops_bench;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(3))
        .sample_size(400);
    targets = bench_string_operations
}
