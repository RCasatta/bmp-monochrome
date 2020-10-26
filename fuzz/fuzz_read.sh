export JOBS=${JOBS:=1}
cargo +nightly fuzz run read -- -jobs=$JOBS
