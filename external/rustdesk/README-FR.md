<p align="center">
  <img src="logo-header.svg" alt="RustDesk - Your remote desktop"><br>
  <a href="#serveurs-publics-libres">Serveurs</a> -
  <a href="#étapes-brutes-de-la-compilationbuild">Build</a> -
  <a href="#comment-construire-avec-docker">Docker</a> -
  <a href="#structure-du-projet">Structure</a> -
  <a href="#images">Images</a><br>
  [<a href="README.md">English</a>] | [<a href="README-ZH.md">中文</a>] | [<a href="README-DE.md">Deutsch</a>] | [<a href="README-ES.md">Española</a>] | [<a href="README-PL.md">Polski</a>] | [<a href="README-JP.md">日本語</a>] | [<a href="README-RU.md">Русский</a>] | [<a href="README-PT.md">Português</a>]<br>
  <b>Nous avons besoin de votre aide pour traduire ce README dans votre langue maternelle</b>.
</p>

Chattez avec nous : [Discord](https://discord.gg/nDceKgxnkV) | [Reddit](https://www.reddit.com/r/rustdesk)

[![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/I2I04VU09)

Encore un autre logiciel de bureau à distance, écrit en Rust. Fonctionne directement, aucune configuration n'est nécessaire. Une excellente alternative à TeamViewer et AnyDesk ! Vous avez le contrôle total de vos données, sans aucun souci de sécurité. Vous pouvez utiliser notre serveur de rendez-vous/relais, [configurer le vôtre](https://rustdesk.com/blog/id-relay-set/), ou [écrire votre propre serveur de rendez-vous/relais](https://github.com/rustdesk/rustdesk-server-demo). 

RustDesk accueille les contributions de tout le monde.  Voir [`CONTRIBUTING.md`](CONTRIBUTING.md) pour plus d'informations.

[**TÉLÉCHARGEMENT BINAIRE**](https://github.com/rustdesk/rustdesk/releases)

## Serveurs publics libres
Ci-dessous se trouvent les serveurs que vous utilisez gratuitement, cela peut changer au fil du temps. Si vous n'êtes pas proche de l'un d'entre eux, votre réseau peut être lent.
- Séoul, AWS lightsail, 1 VCPU/0.5G RAM
- Singapour, Vultr, 1 VCPU/1G RAM
- Dallas, Vultr, 1 VCPU/1G RAM

## Dépendances

Les versions de bureau utilisent [sciter](https://sciter.com/) pour l'interface graphique, veuillez télécharger la bibliothèque dynamique sciter vous-même.

[Windows](https://raw.githubusercontent.com/c-smile/sciter-sdk/master/bin.win/x64/sciter.dll) | 
[Linux](https://raw.githubusercontent.com/c-smile/sciter-sdk/master/bin.lnx/x64/libsciter-gtk.so)
[macOS](https://raw.githubusercontent.com/c-smile/sciter-sdk/master/bin.osx/libsciter.dylib)

## Étapes brutes de la compilation/build
* Préparez votre environnement de développement Rust et votre environnement de compilation C++.

* Installez [vcpkg](https://github.com/microsoft/vcpkg), et définissez correctement la variable d'environnement `VCPKG_ROOT`.

   - Windows : vcpkg install libvpx:x64-windows-static libyuv:x64-windows-static opus:x64-windows-static
   - Linux/Osx : vcpkg install libvpx libyuv opus
   
* Exécuter `cargo run`

## Comment compiler/build sous Linux

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

### Installer vcpkg
```sh
git clone https://github.com/microsoft/vcpkg 
cd vcpkg
git checkout 134505003bb46e20fbace51ccfb69243fbbc5f82
cd ..
vcpkg/bootstrap-vcpkg.sh
export VCPKG_ROOT=$HOME/vcpkg
vcpkg/vcpkg install libvpx libyuv opus
```

### Corriger libvpx (Pour Fedora)
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

### Construire
```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
git clone https://github.com/rustdesk/rustdesk
cd rustdesk
mkdir -p cible/debug
wget https://raw.githubusercontent.com/c-smile/sciter-sdk/master/bin.lnx/x64/libsciter-gtk.so
mv libsciter-gtk.so target/debug
Exécution du cargo
```

## Comment construire avec Docker

Commencez par cloner le dépôt et construire le conteneur Docker :

```sh
git clone https://github.com/rustdesk/rustdesk
cd rustdesk
docker build -t "rustdesk-builder" .
```

Ensuite, chaque fois que vous devez build le logiciel, exécutez la commande suivante :

```sh
docker run --rm -it -v $PWD:/home/user/rustdesk -v rustdesk-git-cache:/home/user/.cargo/git -v rustdesk-registry-cache:/home/user/.cargo/registry -e PUID="$(id -u)" -e PGID="$(id -g)" rustdesk-builder
```

Notez que le premier build peut prendre plus de temps avant que les dépendances ne soient mises en cache, les constructions suivantes seront plus rapides. De plus, si vous devez spécifier différents arguments à la commande de compilation, vous pouvez le faire à la fin de la commande dans la position `<OPTIONAL-ARGS>`. Par exemple, si vous voulez construire une version optimisée de la version release, vous devez exécuter la commande ci-dessus suivie de `---release`. L'exécutable résultant sera disponible dans le dossier cible sur votre système, et peut être lancé avec :

```sh
target/debug/rustdesk
```
Ou, si vous exécutez un exécutable provenant d'une release :

```sh
target/release/rustdesk
```

Veuillez vous assurer que vous exécutez ces commandes à partir de la racine du référentiel RustDesk, sinon l'application ne pourra pas trouver les ressources requises. Notez également que les autres sous-commandes de cargo telles que `install` ou `run` ne sont pas actuellement supportées par cette méthode car elles installeraient ou exécuteraient le programme à l'intérieur du conteneur au lieu de l'hôte.

### Changer Wayland en X11 (Xorg)
RustDesk ne supporte pas Wayland. Lisez [cela](https://docs.fedoraproject.org/en-US/quick-docs/configuring-xorg-as-default-gnome-session/) pour configurer Xorg comme la session GNOME par défaut.

## Structure du projet

- **[libs/hbb_common](https://github.com/rustdesk/rustdesk/tree/master/libs/hbb_common)** : codec vidéo, config, wrapper tcp/udp, protobuf, fonctions fs pour le transfert de fichiers, et quelques autres fonctions utilitaires.
- **[libs/scrap](https://github.com/rustdesk/rustdesk/tree/master/libs/scrap)** : capture d'écran
- **[libs/enigo](https://github.com/rustdesk/rustdesk/tree/master/libs/enigo)** : contrôle clavier/souris spécifique à la plate-forme
- **[src/ui](https://github.com/rustdesk/rustdesk/tree/master/src/ui)** : INTERFACE GRAPHIQUE
- **[src/server](https://github.com/rustdesk/rustdesk/tree/master/src/server)** : services audio/clipboard/input/vidéo, et connexions réseau
- **[src/client.rs](https://github.com/rustdesk/rustdesk/tree/master/src/client.rs)** : démarrer une connexion entre pairs
- **[src/rendezvous_mediator.rs](https://github.com/rustdesk/rustdesk/tree/master/src/rendezvous_mediator.rs)** : Communiquer avec [rustdesk-server](https://github.com/rustdesk/rustdesk-server), attendre une connexion distante directe (TCP hole punching) ou relayée.
- **[src/platform](https://github.com/rustdesk/rustdesk/tree/master/src/platform)** : code spécifique à la plateforme

## Images
![image](https://user-images.githubusercontent.com/71636191/113112362-ae4deb80-923b-11eb-957d-ff88daad4f06.png)

![image](https://user-images.githubusercontent.com/71636191/113112619-f705a480-923b-11eb-911d-97e984ef52b6.png)

![image](https://user-images.githubusercontent.com/71636191/113112857-3fbd5d80-923c-11eb-9836-768325faf906.png)

![image](https://user-images.githubusercontent.com/71636191/113112990-65e2fd80-923c-11eb-840e-349b4d6e340d.png)
