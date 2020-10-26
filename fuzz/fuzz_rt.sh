export JOBS=${JOBS:=1}
cargo +nightly fuzz run rt -- -jobs=$JOBS
