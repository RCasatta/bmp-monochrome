export JOBS=${JOBS:=1}
cp ../test_bmp/* corpus/fuzz_target_read/
cargo +nightly fuzz run fuzz_target_read -- -rss_limit_mb=4096 -jobs=$JOBS
