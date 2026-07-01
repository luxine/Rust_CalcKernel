# Native Benchmark Smoke Commands

Use these commands when changing the compiler pipeline or benchmark harness:

```sh
cargo test --test bench_surface_test
cargo bench --bench ckc_perf -- --quick
cargo bench --bench ckc_perf -- --quick --case pricing --task emit-c-o3
```

The benchmark is not a release correctness gate because local timings are
hardware and load dependent. The surface test is a normal correctness gate and
keeps the benchmark entrypoints from disappearing.
