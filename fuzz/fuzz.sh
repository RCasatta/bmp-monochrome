export JOBS=${JOBS:=1}
nohup cargo +nightly fuzz run bmp_check -- -jobs=$JOBS >bmp_check.log
nohup cargo +nightly fuzz run ops -- -limit=256 -jobs=$JOBS >ops.log
nohup cargo +nightly fuzz run read -- -jobs=$JOBS >read.log
nohup cargo +nightly fuzz run rt -- -jobs=$JOBS >rt.log
