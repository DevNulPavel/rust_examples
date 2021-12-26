<p align="center">
  <img src="logo-header.svg" alt="RustDesk - Your remote desktop"><br>
  <a href="#免费公共服务器">服务器</a> •
  <a href="#基本构建步骤">编译</a> •
  <a href="#使用Docker编译">Docker</a> •
  <a href="#文件结构">结构</a> •
  <a href="#截图">截图</a><br>
  [<a href="README.md">English</a>] | [<a href="README-ES.md">Español</a>] | [<a href="README-FR.md">Français</a>] | [<a href="README-DE.md">Deutsch</a>] | [<a href="README-PL.md">Polski</a>] | [<a href="README-JP.md">日本語</a>] | [<a href="README-RU.md">Русский</a>] | [<a href="README-PT.md">Português</a>]<br>
</p>

Chat with us: [知乎](https://www.zhihu.com/people/rustdesk) | [Discord](https://discord.gg/nDceKgxnkV) | [Reddit](https://www.reddit.com/r/rustdesk)

[![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/I2I04VU09)

远程桌面软件，开箱即用，无需任何配置，替代TeamViewer和AnyDesk。您完全掌控数据，不用担心安全问题。您可以使用我们的注册/中继服务器，
或者[自己设置](https://rustdesk.com/blog/id-relay-set/)，
亦或者[开发您的版本](https://github.com/rustdesk/rustdesk-server-demo)。

欢迎大家贡献代码，  请看 [`CONTRIBUTING.md`](CONTRIBUTING.md).

[**可执行程序下载**](https://github.com/rustdesk/rustdesk/releases)

## 免费公共服务器
以下是您免费使用的服务器，它可能会随着时间的推移而变化。如果您不靠近其中之一，您的网络可能会很慢。
- 首尔, AWS lightsail, 1 VCPU/0.5G RAM
- 新加坡, Vultr, 1 VCPU/1G RAM
- 达拉斯, Vultr, 1 VCPU/1G RAM

## 依赖

桌面版本界面使用[sciter](https://sciter.com/), 请自行下载。

[Windows](https://raw.githubusercontent.com/c-smile/sciter-sdk/master/bin.win/x64/sciter.dll) | 
[Linux](https://raw.githubusercontent.com/c-smile/sciter-sdk/master/bin.lnx/x64/libsciter-gtk.so) |
[macOS](https://raw.githubusercontent.com/c-smile/sciter-sdk/master/bin.osx/libsciter.dylib)

## 基本构建步骤
* 请准备好Rust开发环境和C++编译环境

* 安装[vcpkg](https://github.com/microsoft/vcpkg), 正确设置`VCPKG_ROOT`环境变量

   - Windows: vcpkg install libvpx:x64-windows-static libyuv:x64-windows-static opus:x64-windows-static
   - Linux/Osx: vcpkg install libvpx libyuv opus
   
* 运行 `cargo run`

## 在Linux上编译

### Ubuntu 18 (Debian 10)
```sh
sudo apt install -y g++ gcc git curl wget nasm yasm libgtk-3-dev clang libxcb-randr0-dev libxdo-dev libxfixes-dev libxcb-shape0-dev libxcb-xfixes0-dev libasound2-dev libpulse-dev cmake
```

### Fedora 28 (CentOS 8)
```sh
sudo yum -y install gcc-c++ git curl wget nasm yasm gcc gtk3-devel clang libxcb-devel libxdo-devel libXfixes-devel pulseaudio-libs-devel cmake alsa-lib-devel
```

### Arch (Manjaro)
```sh
sudo pacman -Syu --needed unzip git cmake gcc curl wget yasm nasm zip make pkg-config clang gtk3 xdotool libxcb libxfixes alsa-lib pulseaudio
```

### 安装vcpkg
```sh
git clone https://github.com/microsoft/vcpkg 
cd vcpkg
git checkout 134505003bb46e20fbace51ccfb69243fbbc5f82
cd ..
vcpkg/bootstrap-vcpkg.sh
export VCPKG_ROOT=$HOME/vcpkg
vcpkg/vcpkg install libvpx libyuv opus
```

### 修复libvpx (仅仅针对Fedora)
```sh
cd vcpkg/buildtrees/libvpx/src
cd *
./configure
sed -i 's/CFLAGS+=-I/CFLAGS+=-fPIC -I/g' Makefile
sed -i 's/CXXFLAGS+=-I/CXXFLAGS+=-fPIC -I/g' Makefile
make
cp libvpx.a $HOME/vcpkg/installed/x64-linux/lib/
cd
```

### 构建
```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
git clone https://github.com/rustdesk/rustdesk
cd rustdesk
mkdir -p target/debug
wget https://raw.githubusercontent.com/c-smile/sciter-sdk/master/bin.lnx/x64/libsciter-gtk.so
mv libsciter-gtk.so target/debug
cargo run
```

## 使用Docker编译

首先克隆存储库并构建 docker 容器：

```sh
git clone https://github.com/rustdesk/rustdesk
cd rustdesk
docker build -t "rustdesk-builder" .
```

然后，每次需要构建应用程序时，运行以下命令：

```sh
docker run --rm -it -v $PWD:/home/user/rustdesk -v rustdesk-git-cache:/home/user/.cargo/git -v rustdesk-registry-cache:/home/user/.cargo/registry -e PUID="$(id -u)" -e PGID="$(id -g)" rustdesk-builder
```


请注意，第一次构建可能需要比较长的时间，因为需要缓存依赖项，后续构建会更快。此外，如果您需要为构建命令指定不同的参数，
您可以在命令末尾的 `<OPTIONAL-ARGS>` 位置执行此操作。例如，如果你想构建一个优化的发布版本，你可以在命令后跟 `---release`。
将在target下产生可执行程序，请通过以下方式运行调试版本：
```sh
target/debug/rustdesk
```

或者运行发布版本:

```sh
target/release/rustdesk
```

请确保您从 RustDesk 存储库的根目录运行这些命令，否则应用程序可能无法找到所需的资源。另请注意，此方法当前不支持其他`Cargo`子命令，
例如 `install` 或 `run`，因为运行在容器里，而不是宿主机上。

### 把Wayland修改成X11 (Xorg)
RustDesk暂时不支持Wayland，不过正在积极开发中.
请查看[this](https://docs.fedoraproject.org/en-US/quick-docs/configuring-xorg-as-default-gnome-session/)配置X11.

## 文件结构

- **[libs/hbb_common](https://github.com/rustdesk/rustdesk/tree/master/libs/hbb_common)**: 视频编解码, 配置, tcp/udp封装, protobuf, 文件传输相关文件系统操作函数, 以及一些其他实用函数
- **[libs/scrap](https://github.com/rustdesk/rustdesk/tree/master/libs/scrap)**: 截屏
- **[libs/enigo](https://github.com/rustdesk/rustdesk/tree/master/libs/enigo)**: 平台相关的鼠标键盘输入
- **[src/ui](https://github.com/rustdesk/rustdesk/tree/master/src/ui)**: GUI
- **[src/server](https://github.com/rustdesk/rustdesk/tree/master/src/server)**: 被控端服务，audio/clipboard/input/video服务, 已经连接实现
- **[src/client.rs](https://github.com/rustdesk/rustdesk/tree/master/src/client.rs)**: 控制端
- **[src/rendezvous_mediator.rs](https://github.com/rustdesk/rustdesk/tree/master/src/rendezvous_mediator.rs)**: 与[rustdesk-server](https://github.com/rustdesk/rustdesk-server)保持UDP通讯, 等待远程连接（通过打洞直连或者中继）
- **[src/platform](https://github.com/rustdesk/rustdesk/tree/master/src/platform)**: 平台服务相关代码

## 截图
![image](https://user-images.githubusercontent.com/71636191/113112362-ae4deb80-923b-11eb-957d-ff88daad4f06.png)

![image](https://user-images.githubusercontent.com/71636191/113112619-f705a480-923b-11eb-911d-97e984ef52b6.png)

![image](https://user-images.githubusercontent.com/71636191/113112857-3fbd5d80-923c-11eb-9836-768325faf906.png)

![image](https://user-images.githubusercontent.com/71636191/113112990-65e2fd80-923c-11eb-840e-349b4d6e340d.png)
