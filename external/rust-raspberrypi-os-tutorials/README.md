# Operating System development tutorials in Rust on the Raspberry Pi

![](https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials/workflows/BSP-RPi3/badge.svg) ![](https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials/workflows/BSP-RPi4/badge.svg) ![](https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials/workflows/Unit-Tests/badge.svg) ![](https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials/workflows/Integration-Tests/badge.svg) ![](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue)

<br/>

<img src="doc/header.jpg" height="379"> <img src="doc/minipush_demo_frontpage.gif" height="379">

## ℹ️ Introduction

This is a tutorial series for hobby OS developers who are new to ARM's 64 bit [ARMv8-A
architecture]. The tutorials will give a guided, step-by-step tour of how to write a [monolithic]
Operating System `kernel` for an `embedded system` from scratch. They cover implementation of common
Operating Systems tasks, like writing to the serial console, setting up virtual memory and handling
HW exceptions. All while leveraging `Rust`'s unique features to provide for safety and speed.

Have fun!

_Best regards,<br>Andre ([@andre-richter])_

P.S.: In the future, Chinese :cn: versions of the tutorials will be maintained as
[`README.CN.md`](README.CN.md) by [@colachg] and [@readlnh].

[ARMv8-A architecture]: https://developer.arm.com/products/architecture/cpu-architecture/a-profile/docs
[monolithic]: https://en.wikipedia.org/wiki/Monolithic_kernel
[@andre-richter]: https://github.com/andre-richter
[@colachg]: https://github.com/colachg
[@readlnh]: https://github.com/readlnh

## 📑 Organization

- Each tutorial contains a stand-alone, bootable `kernel` binary.
- Each new tutorial extends the previous one.
- Each tutorial `README` will have a short `tl;dr` section giving a brief overview of the additions,
  and show the source code `diff` to the previous tutorial, so that you can conveniently inspect the
  changes/additions.
    - Some tutorials have a full-fledged, detailed text in addition to the `tl;dr` section. The
      long-term plan is that all tutorials get a full text, but for now this is exclusive to
      tutorials where I think that `tl;dr` and `diff` are not enough to get the idea.
- The code written in these tutorials supports and runs on the **Raspberry Pi 3** and the
  **Raspberry Pi 4**.
  - Tutorials 1 till 5 are groundwork code which only makes sense to run in `QEMU`.
  - Starting with [tutorial 6](06_drivers_gpio_uart), you can load and run the kernel on the real
    Raspberrys and observe output over `UART`.
- Although the Raspberry Pi 3 and 4 are the main target boards, the code is written in a modular
  fashion which allows for easy porting to other CPU architectures and/or boards.
  - I would really love if someone takes a shot at a **RISC-V** implementation!
- For editing, I recommend [Visual Studio Code] with [Rust Analyzer].
- In addition to the tutorial text, also check out the `make doc` command in each tutorial. It lets
  you browse the extensively documented code in a convenient way.

### Output of `make doc`

![make doc](doc/make_doc.png)

[Visual Studio Code]: https://code.visualstudio.com
[Rust Analyzer]: https://rust-analyzer.github.io

## 🛠 System Requirements

The tutorials are primarily targeted at **Linux**-based distributions. Most stuff will also work on
other Unix flavors such as **macOS**, but this is only _experimental_.

### 🚀 The tl;dr Version

1. [Install Docker][install_docker].
1. Ensure your user account is in the [docker group].
1. Install a suitable `Rust` toolchain:
   1. If you already have a version of Rust installed:
      ```bash
      rustup toolchain add nightly-2020-06-30
      rustup default nightly-2020-06-30
      rustup component add llvm-tools-preview
      rustup target add aarch64-unknown-none-softfloat
      cargo install cargo-binutils
      ```

   2. If you need a fresh install:
      ```bash
      curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- \
          --default-toolchain nightly-2020-06-30                           \
          --component llvm-tools-preview

      source $HOME/.cargo/env
      rustup target add aarch64-unknown-none-softfloat
      cargo install cargo-binutils
      ```

3. In case you use `Visual Studio Code`, I strongly recommend installing the [Rust Analyzer extension].
4. If you are **NOT** running Linux, some `Ruby` gems are needed as well:

   ```bash
   sudo gem install bundler
   bundle config set path '.vendor/bundle'
   bundle install
   ```

[docker group]: https://docs.docker.com/engine/install/linux-postinstall/
[Rust Analyzer extension]: https://marketplace.visualstudio.com/items?itemName=matklad.rust-analyzer

### 🧰 The Long Version: Eliminating Toolchain Hassle

This series tries to put a strong focus on user friendliness. Therefore, efforts were made to
eliminate the biggest painpoint in embedded development as much as possible: Toolchain hassle.

Rust itself is already helping a lot in that regard, because it has built-in support for
cross-compilation. All that we need for cross-compiling from an `x86` host to the Raspberry Pi's
`AArch64` architecture is to install the respective target through `rustup`. However, besides the
Rust compiler, we will use some more tools. Among others:

- `QEMU` to emulate our kernel on the host system.
- A self-made tool called `Minipush` to load a kernel onto the Raspberry Pi on-demand over `UART`.
- `OpenOCD` and `GDB` for debugging on the target.

There is a lot that can go wrong while installing and/or compiling the correct version of each tool
on your host machine. For example, your distribution might not provide the latest version that is
needed. Or you are missing some hard-to-get dependencies for the compilation of one of these tools.

This is why we will make use of [Docker][install_docker] whenever possible. We are providing an
accompanying container that has all the needed tools or dependencies pre-installed, and it gets
pulled in automagically once it is needed. If you want to know more about Docker and peek at the
provided container, please refer to the repository's [docker](docker) folder.

[install_docker]: https://docs.docker.com/get-docker/

## 📟 USB Serial Output

Since the kernel developed in the tutorials runs on the real hardware, it is highly recommended to
get a USB serial debug cable to get the full experience. The cable also powers the Raspberry once
you connect it, so you don't need extra power over the dedicated power-USB.

- You can find USB-to-serial cables that should work right away at [\[1\]] [\[2\]].
- You connect it to the GPIO pins `14/15` as shown below.
- [Tutorial 6](06_drivers_gpio_uart) is the first where you can use it. Check it out for
  instructions on how to prepare the SD card to boot your self-made kernel from it.
- Starting with [tutorial 7](07_uart_chainloader), booting kernels on your Raspberry is getting
  _really_ comfortable. In this tutorial, a so-called `chainloader` is developed, which will be the
  last file you need to manually copy on the SD card for a while. It will enable you to load the
  tutorial kernels during boot on demand over `UART`.

![UART wiring diagram](doc/wiring.png)

[\[1\]]: https://www.amazon.de/dp/B0757FQ5CX/ref=cm_sw_r_tw_dp_U_x_ozGRDbVTJAG4Q
[\[2\]]: https://www.adafruit.com/product/954

## 🙌 Acknowledgements

The original version of the tutorials started out as a fork of [Zoltan
Baldaszti](https://github.com/bztsrc)'s awesome [tutorials on bare metal programming on
RPi3](https://github.com/bztsrc/raspi3-tutorial) in `C`. Thanks for giving me a head start!

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
