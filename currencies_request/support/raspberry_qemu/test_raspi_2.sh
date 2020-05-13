#!/bin/sh

# https://gist.github.com/tinjaw/5bc5527ff379e8dd299a0b67e2bc9b62
# https://gist.github.com/hfreire/5846b7aa4ac9209699ba
# https://gist.github.com/tinjaw/5bc5527ff379e8dd299a0b67e2bc9b62
# https://gist.github.com/tinjaw/5bc5527ff379e8dd299a0b67e2bc9b62
# https://habr.com/ru/post/216205/
# https://stackoverflow.com/questions/28880833/how-to-emulate-the-raspberry-pi-2-on-qemu
# https://github.com/raspberrypi/firmware/tree/master/boot
# https://medium.com/@dali.may28/emulate-raspberry-pi-2-on-your-pc-91a4af826cba


## MacOS Catalina

# brew install qemu

export QEMU=$(which qemu-system-arm)

export TMP_DIR=$(pwd)/tmp

export KERNEL_URL=https://github.com/dhruvvyas90/qemu-rpi-kernel/blob/master/kernel-qemu-4.14.79-stretch?raw=true
export RPI_KERNEL=${TMP_DIR}/kernel-qemu-4.14.79-stretch

export RPI_FS=${TMP_DIR}/2019-09-26-raspbian-buster-lite.img

export BOARD_URL=https://github.com/raspberrypi/firmware/blob/master/boot/bcm2710-rpi-2-b.dtb
export PTB_FILE=${TMP_DIR}/bcm2709-rpi-2-b.dtb

export IMAGE_FILE=2019-09-26-raspbian-buster-lite.zip
export IMAGE_URL=http://downloads.raspberrypi.org/raspbian_lite/images/raspbian_lite-2019-09-30/${IMAGE_FILE}

mkdir -p $TMP_DIR; cd $TMP_DIR

# wget -c ${KERNEL_URL} -O ${RPI_KERNEL}

# wget -c ${BOARD_URL} -O ${PTB_FILE}

# wget -c $IMAGE_URL
# unzip $IMAGE_FILE

# -dtb ${PTB_FILE} \
# -append "root=/dev/sda2 panic=1 rootfstype=ext4 rw" \
$QEMU -kernel ${RPI_KERNEL} \
    -cpu arm1176 \
    -m 1G \
    -M raspi2 \
    -no-reboot \
    -serial stdio \
    -append "rw earlyprintk loglevel=8 console=ttyAMA0,115200 dwc_otg.lpm_enable=0 root=/dev/mmcblk0p2" \
    -drive "file=${RPI_FS},index=0,media=disk,format=raw" \
    -net user,hostfwd=tcp::5022-:22 -net nic


# Raspberry Login + Password: pi / raspberry

# Login to Raspberry Pi, then enable ssh: sudo raspi-config -> enable ssh

# ssh -p 5022 pi@localhost

