export JOBS=${JOBS:=1}
cargo +nightly fuzz run ops -- -limit=256 -jobs=$JOBS
