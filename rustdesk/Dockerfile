FROM debian

WORKDIR /
RUN apt update -y && apt install -y g++ gcc git curl wget nasm yasm libgtk-3-dev clang libxcb-randr0-dev libxdo-dev libxfixes-dev libxcb-shape0-dev libxcb-xfixes0-dev libasound2-dev libpulse-dev cmake unzip zip sudo

RUN git clone https://github.com/microsoft/vcpkg && cd vcpkg && git checkout 134505003bb46e20fbace51ccfb69243fbbc5f82
RUN /vcpkg/bootstrap-vcpkg.sh -disableMetrics
RUN /vcpkg/vcpkg --disable-metrics install libvpx libyuv opus

RUN groupadd -r user && useradd -r -g user user --home /home/user && mkdir -p /home/user && chown user /home/user
WORKDIR /home/user
RUN wget https://raw.githubusercontent.com/c-smile/sciter-sdk/master/bin.lnx/x64/libsciter-gtk.so
USER user
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > rustup.sh
RUN chmod +x rustup.sh
RUN ./rustup.sh -y

USER root
COPY ./entrypoint /
ENTRYPOINT ["/entrypoint"]
