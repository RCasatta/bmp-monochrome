cp ../test_bmp/* corpus/fuzz_target_write/
cargo +nightly fuzz run fuzz_target_write -- -rss_limit_mb=4096
