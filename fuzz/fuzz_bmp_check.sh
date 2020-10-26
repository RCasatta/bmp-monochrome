export JOBS=${JOBS:=1}
nohup cargo +nightly fuzz run bmp_check -- -jobs=$JOBS
