#!/bin/sh -e

# /Users/devnul/.distcc/hosts
# Через / пишем количество потоков, желательно - количество ядер * 2
# Pump
# localhost/13,cpp,lzo
# 192.168.1.14/5,cpp,lzo
#
# Normal
# localhost/13
# 192.168.1.14/5
#
# 127.0.0.1
# 192.168.1.14
# 192.168.1.14,cpp,lzo
# 127.0.0.1,cpp,lzo

# distccd --no-detach --daemon --allow 192.168.1.0/24 --allow 127.0.0.1 --log-stderr --verbose --enable-tcp-insecure
# distccd --jobs 12 -N 12 --no-detach --daemon --allow 192.168.1.0/24 --allow 127.0.0.1 --log-stderr --enable-tcp-insecure --verbose


if type -p /usr/local/bin/ccache >/dev/null 2>&1; then
    export CCACHE_MAXSIZE=50G
    export CCACHE_CPP2=true
    export CCACHE_HARDLINK=true
    export CCACHE_SLOPPINESS=file_macro,time_macros,include_file_mtime,include_file_ctime,file_stat_matches
    # /usr/local/bin/ccache
    # /usr/local/bin/distcc
    # /usr/local/bin/pump
    exec \
        /usr/local/bin/ccache \
        /Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/clang -lstdc++ "$@"
else
    exec /Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/clang "$@"
fi