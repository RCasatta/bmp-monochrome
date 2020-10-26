# bmp-monochrome fuzz

## Install

```
cargo install cargo-fuzz
```

## Launch fuzzing

`$JOB` is one of the name in `fuzz_targets`.

`$JOBS` is the number of parallel processes.

```
cargo +nightly fuzz run $JOB -- -jobs=$JOBS
```

## Handling shared corpus

Todo before/if pushing corpus in the git repo

```
cargo +nightly fuzz cmin $JOB
```

To add the corpus force add since it is in gitignore
