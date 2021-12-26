<p align="center">
  <img src="logo-header.svg" alt="RustDesk - Your remote desktop"><br>
  <a href="#darmowe-serwery-publiczne">Serwery</a> •
  <a href="#podstawowe-kroki-do-kompilacji">Kompilacja</a> •
  <a href="#jak-kompilować-za-pomocą-dockera">Docker</a> •
  <a href="#struktura-plików">Struktura</a> •
  <a href="#migawkisnapshoty">Snapshot</a><br>
  [<a href="README.md">English</a>] | [<a href="README-ZH.md">中文</a>] | [<a href="README-DE.md">Deutsch</a>] | [<a href="README-ES.md">Española</a>] | [<a href="README-FR.md">Français</a>] | [<a href="README-JP.md">日本語</a>] | [<a href="README-RU.md">Русский</a>] | [<a href="README-PT.md">Português</a>]<br>
  <b>Potrzebujemy twojej pomocy w tłumaczeniu README na twój ojczysty język</b>
</p>

Porozmawiaj z nami na: [Discord](https://discord.gg/nDceKgxnkV) | [Reddit](https://www.reddit.com/r/rustdesk)

[![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/I2I04VU09)

Kolejny program do zdalnego pulpitu, napisany w Rust. Działa od samego początku, nie wymaga konfiguracji. Świetna alternatywa dla TeamViewera i AnyDesk! Masz pełną kontrolę nad swoimi danymi, bez obaw o bezpieczeństwo. Możesz skorzystać z naszego darmowego serwera publicznego , [skonfigurować własny](https://rustdesk.com/blog/id-relay-set/), lub [napisać własny serwer rendezvous/relay server](https://github.com/rustdesk/rustdesk-server-demo). 

RustDesk zaprasza do współpracy każdego.  Zobacz [`CONTRIBUTING.md`](CONTRIBUTING.md) pomoc w uruchomieniu programu.

[**POBIERZ KOMPILACJE**](https://github.com/rustdesk/rustdesk/releases)

## Darmowe Serwery Publiczne
Poniżej znajdują się serwery, z których można korzystać za darmo, może się to zmienić z upływem czasu. Jeśli nie znajdujesz się w pobliżu jednego z nich, Twoja prędkość połączenia może być niska.
| Lokalizacja  | Dostawca        | Specyfikacja      |
| --------- | ------------- | ------------------ |
| Seul     | AWS lightsail | 1 VCPU / 0.5GB RAM |
| Singapur | Vultr         | 1 VCPU / 1GB RAM   |
| Dallas    | Vultr         | 1 VCPU / 1GB RAM   |        |

## Zależności

Wersje desktopowe używają [sciter](https://sciter.com/) dla GUI, proszę pobrać bibliotekę dynamiczną sciter samodzielnie.

[Windows](https://raw.githubusercontent.com/c-smile/sciter-sdk/master/bin.win/x64/sciter.dll) | 
[Linux](https://raw.githubusercontent.com/c-smile/sciter-sdk/master/bin.lnx/x64/libsciter-gtk.so) |
[MacOS](https://raw.githubusercontent.com/c-smile/sciter-sdk/master/bin.osx/libsciter.dylib)

## Podstawowe kroki do kompilacji.
* Przygotuj środowisko programistyczne Rust i środowisko programowania C++

* Zainstaluj [vcpkg](https://github.com/microsoft/vcpkg), i ustaw `VCPKG_ROOT` env zmienną prawidłowo

   - Windows: vcpkg install libvpx:x64-windows-static libyuv:x64-windows-static opus:x64-windows-static
   - Linux/MacOS: vcpkg install libvpx libyuv opus
   
* uruchom `cargo run`

## Jak Kompilować na Linuxie

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

### Zainstaluj vcpkg
```sh
git clone https://github.com/microsoft/vcpkg 
cd vcpkg
git checkout 134505003bb46e20fbace51ccfb69243fbbc5f82
cd ..
vcpkg/bootstrap-vcpkg.sh
export VCPKG_ROOT=$HOME/vcpkg
vcpkg/vcpkg install libvpx libyuv opus
```

### Fix libvpx (For Fedora)
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

### Kompilacja
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

## Jak kompilować za pomocą Dockera

Rozpocznij od sklonowania repozytorium i stworzenia kontenera docker:

```sh
git clone https://github.com/rustdesk/rustdesk
cd rustdesk
docker build -t "rustdesk-builder" .
```

Następnie, za każdym razem, gdy potrzebujesz skompilować aplikację, uruchom następujące polecenie:

```sh
docker run --rm -it -v $PWD:/home/user/rustdesk -v rustdesk-git-cache:/home/user/.cargo/git -v rustdesk-registry-cache:/home/user/.cargo/registry -e PUID="$(id -u)" -e PGID="$(id -g)" rustdesk-builder
```

Zauważ, że pierwsza kompilacja może potrwać dłużej zanim zależności zostaną zbuforowane, kolejne będą szybsze. Dodatkowo, jeśli potrzebujesz określić inne argumenty dla polecenia budowania, możesz to zrobić na końcu komendy w miejscu `<OPTIONAL-ARGS>`. Na przykład, jeśli chciałbyś zbudować zoptymalizowaną wersję wydania, uruchomiłbyś powyższą komendę a następnie `---release`. Powstały plik wykonywalny będzie dostępny w folderze docelowym w twoim systemie, i może być uruchomiony z:


```sh
target/debug/rustdesk
```

Lub, jeśli uruchamiasz plik wykonywalny wersji:

```sh
target/release/rustdesk
```

Upewnij się, że uruchamiasz te polecenia z katalogu głównego repozytorium RustDesk, w przeciwnym razie aplikacja może nie być w stanie znaleźć wymaganych zasobów. Należy również pamiętać, że inne podpolecenia ładowania, takie jak `install` lub `run` nie są obecnie obsługiwane za pomocą tej metody, ponieważ instalowałyby lub uruchamiały program wewnątrz kontenera zamiast na hoście.

### Zmień Wayland na X11 (Xorg)
RustDesk nie obsługuje Waylanda. Sprawdź [this](https://docs.fedoraproject.org/en-US/quick-docs/configuring-xorg-as-default-gnome-session/) by skonfigurować Xorg jako domyślną sesję GNOME.

## Struktura plików

- **[libs/hbb_common](https://github.com/rustdesk/rustdesk/tree/master/libs/hbb_common)**: kodek wideo, config, wrapper tcp/udp, protobuf, funkcje fs do transferu plików i kilka innych funkcji użytkowych
- **[libs/scrap](https://github.com/rustdesk/rustdesk/tree/master/libs/scrap)**: przechwytywanie ekranu
- **[libs/enigo](https://github.com/rustdesk/rustdesk/tree/master/libs/enigo)**: specyficzne dla danej platformy sterowanie klawiaturą/myszą
- **[src/ui](https://github.com/rustdesk/rustdesk/tree/master/src/ui)**: GUI
- **[src/server](https://github.com/rustdesk/rustdesk/tree/master/src/server)**: audio/schowek/wejście(input)/wideo oraz połączenia sieciowe
- **[src/client.rs](https://github.com/rustdesk/rustdesk/tree/master/src/client.rs)**: uruchamia połączenie peer
- **[src/rendezvous_mediator.rs](https://github.com/rustdesk/rustdesk/tree/master/src/rendezvous_mediator.rs)**: Komunikacja z [rustdesk-server](https://github.com/rustdesk/rustdesk-server), wait for remote direct (TCP hole punching) or relayed connection
- **[src/platform](https://github.com/rustdesk/rustdesk/tree/master/src/platform)**: specyficzny dla danej platformy kod

## Migawki(Snapshoty)
![image](https://user-images.githubusercontent.com/71636191/113112362-ae4deb80-923b-11eb-957d-ff88daad4f06.png)

![image](https://user-images.githubusercontent.com/71636191/113112619-f705a480-923b-11eb-911d-97e984ef52b6.png)

![image](https://user-images.githubusercontent.com/71636191/113112857-3fbd5d80-923c-11eb-9836-768325faf906.png)

![image](https://user-images.githubusercontent.com/71636191/113112990-65e2fd80-923c-11eb-840e-349b4d6e340d.png)
