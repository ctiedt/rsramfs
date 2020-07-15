#!/bin/bash

function unit () {
  bash -c "$2"
  if [[ $? -ne 0 ]]; then
    echo $1 " [failed]"
  else
    echo $1 " [ok]"
  fi
}

mkdir -p test

if [[ ! -e build/rsramfs.ko ]]; then
    make
fi

unit "Insert Module..." "sudo insmod build/rsramfs.ko"

unit "Mount..." "sudo mount -t rsramfs none test"

unit "Create File..." "echo 'Hello World' > test/file"

if [[ `cat test/file` = "Hello World" ]]; then
  echo "File contents... [ok]"
else
  echo "File contents... [failed]"
fi

unit "Create Directory..." "mkdir test/dir"

if [[ -e test/dir ]]; then
  echo "Directory... [ok]"
else
  echo "Directory... [failed]"
fi

unit "Unmount..." "sudo umount test"

unit "Remove Module..." "sudo rmmod rsramfs"

rm -rf test