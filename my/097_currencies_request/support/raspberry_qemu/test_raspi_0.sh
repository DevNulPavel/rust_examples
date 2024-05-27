#!/bin/sh

# https://gist.github.com/tinjaw/5bc5527ff379e8dd299a0b67e2bc9b62
# https://gist.github.com/hfreire/5846b7aa4ac9209699ba
# https://gist.github.com/tinjaw/5bc5527ff379e8dd299a0b67e2bc9b62
# https://gist.github.com/tinjaw/5bc5527ff379e8dd299a0b67e2bc9b62
# https://habr.com/ru/post/216205/


## MacOS Catalina

# brew install qemu

export QEMU=$(which qemu-system-arm)
export TMP_DIR=$(pwd)/tmp

export KERNEL_URL=https://github.com/dhruvvyas90/qemu-rpi-kernel/blob/master/kernel-qemu-4.14.79-stretch?raw=true
export RPI_KERNEL=${TMP_DIR}/kernel-qemu-4.14.79-stretch

export RPI_FS=${TMP_DIR}/2019-09-26-raspbian-buster-lite.img

export BOARD_URL=https://github.com/dhruvvyas90/qemu-rpi-kernel/raw/master/versatile-pb.dtb
export PTB_FILE=${TMP_DIR}/versatile-pb.dtb

export IMAGE_FILE=2019-09-26-raspbian-buster-lite.zip
export IMAGE_URL=http://downloads.raspberrypi.org/raspbian_lite/images/raspbian_lite-2019-09-30/${IMAGE_FILE}


mkdir -p $TMP_DIR; cd $TMP_DIR

# wget -c ${KERNEL_URL} -O ${RPI_KERNEL}

# wget -c ${BOARD_URL} -O ${PTB_FILE}

# wget $IMAGE_URL
# unzip $IMAGE_FILE

$QEMU -kernel ${RPI_KERNEL} \
    -cpu arm1176 \
    -m 256 \
    -M versatilepb \
    -dtb ${PTB_FILE} \
    -no-reboot \
    -serial stdio \
    -append "root=/dev/sda2 panic=1 rootfstype=ext4 rw" \
    -drive "file=${RPI_FS},index=0,media=disk,format=raw" \
    -net user,hostfwd=tcp::5022-:22 -net nic


# Raspberry Login + Password: pi / raspberry

# Login to Raspberry Pi, then enable ssh: sudo raspi-config -> enable ssh

# ssh -p 5022 pi@localhost

