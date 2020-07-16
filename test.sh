#!/bin/bash

TESTS_NUMBER=0
TESTS_PASSED=0

function unit () {
  TESTS_NUMBER=$((TESTS_NUMBER+1))
  bash -c "$2"
  if [[ $? -ne 0 ]]; then
    echo $1 " [failed]"
  else
    echo $1 " [ok]"
    TESTS_PASSED=$((TESTS_PASSED+1))
  fi
}

mkdir -p test

if [[ ! -e build/rsramfs.ko ]]; then
    make
fi

unit "Insert Module..." "sudo insmod build/rsramfs.ko"

# unit "Mount..." "sudo mount -t rsramfs none test"
# 
# unit "Create File..." "echo 'Hello World' > test/file"
# 
# TESTS_NUMBER=$((TESTS_NUMBER+1))
# if [[ `cat test/file` = "Hello World" ]]; then
#   echo "File contents... [ok]"
#   TESTS_PASSED=$((TESTS_PASSED+1))
# else
#   echo "File contents... [failed]"
# fi
# 
# unit "Create Directory..." "mkdir test/dir"
# 
# TESTS_NUMBER=$((TESTS_NUMBER+1))
# if [[ -e test/dir ]]; then
#   echo "Directory... [ok]"
#   TESTS_PASSED=$((TESTS_PASSED+1))
# else
#   echo "Directory... [failed]"
# fi
# 
# unit "Unmount..." "sudo umount test"

unit "Remove Module..." "sudo rmmod rsramfs"

rm -rf test

echo "Passed $TESTS_PASSED/$TESTS_NUMBER tests"

exit $((TESTS_NUMBER-TESTS_PASSED))