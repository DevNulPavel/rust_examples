#!/usr/bin/env bash

# brew install gnu-time parallel


clang -O2 test_c.c -o test_c_bin

cd test_rust && \
    cargo build --release -q && \
    cp target/release/test_rust ../test_rust_bin
cd ../

cd executor && \
    cargo build --release -q && \
    cp target/release/executor ../executor_bin
cd ../

PATH=/usr/local/opt/gnu-time/libexec/gnubin:$PATH
TIME_BIN=time #TIME_BIN=/usr/local/opt/gnu-time/libexec/gnubin/time

#--bar
# 11254
# +0
# -pl 
JOBS_COUNT=10 # parallel плохо задействует cpu при большом количестве потоков на мелких и очень быстрых задачах
ITERATIONS=11254
TIME_FORMAT="Elapsed time: %es (%Es)\\nUser time: %Us\\nSystem time: %Ss\\nProcess utilization: %P\\nCotnext switches: %c"

echo "Jobs count = ${JOBS_COUNT}"
echo "Iterations count = ${ITERATIONS}"

echo ""
echo "--- Python test ---"
# seq ${ITERATIONS} | ${TIME_BIN} --format="${TIME_FORMAT}" parallel -j ${JOBS_COUNT} "./test_python.py 2> /dev/null"
${TIME_BIN} --format="${TIME_FORMAT}" ./executor_bin ${JOBS_COUNT} ${ITERATIONS} "./test_python.py"

echo ""
echo "--- Bash test ---"
# seq ${ITERATIONS} | ${TIME_BIN} --format="${TIME_FORMAT}" parallel -j ${JOBS_COUNT} "./test_bash.sh 2> /dev/null"
${TIME_BIN} --format="${TIME_FORMAT}" ./executor_bin ${JOBS_COUNT} ${ITERATIONS} "./test_bash.sh"

echo ""
echo "--- C test ---"
# seq ${ITERATIONS} | ${TIME_BIN} --format="${TIME_FORMAT}" parallel -j ${JOBS_COUNT} "./test_c_bin 2> /dev/null"
${TIME_BIN} --format="${TIME_FORMAT}" ./executor_bin ${JOBS_COUNT} ${ITERATIONS} "./test_c_bin"

echo ""
echo "--- RUST test ---"
# seq ${ITERATIONS} | ${TIME_BIN} --format="${TIME_FORMAT}" parallel -j ${JOBS_COUNT} "./test_rust_bin 2> /dev/null"
${TIME_BIN} --format="${TIME_FORMAT}" ./executor_bin ${JOBS_COUNT} ${ITERATIONS} "./test_rust_bin"
