#!/bin/sh -e

# defaults write com.apple.Xcode PBXNumberOfParallelBuildSubtasks 64
# defaults write com.apple.Xcode IDEBuildOperationMaxNumberOfConcurrentCompileTasks 1
# defaults write com.apple.dt.Xcode IDEBuildOperationMaxNumberOfConcurrentCompileTasks 1

# Установка количества для IDE
# defaults write com.apple.dt.Xcode IDEBuildOperationMaxNumberOfConcurrentCompileTasks (distcc -j)
#
# Либо вот эти, но скорее всего первый
# defaults write com.apple.Xcode IDEBuildOperationMaxNumberOfConcurrentCompileTasks (distcc -j)
# defaults write com.apple.Xcode IDEBuildOperationMaxNumberOfConcurrentCompileTasks 1

# Установка количества для CLI
# xcodebuild -jobs (distcc -j) -target 'Island2'

# /Users/devnul/.distcc/hosts
# Через / пишем количество потоков, для локального компа - количество ядер, для удаленных - количество ядер * 2
# Normal
# localhost/12
# 192.168.1.14/8

# Pump
# localhost/32,cpp,lzo
# 192.168.1.14/8,cpp,lzo
# 127.0.0.1
# 192.168.1.14
# 192.168.1.14,cpp,lzo
# 127.0.0.1,cpp,lzo

# distccd --no-detach --daemon --allow 192.168.1.0/24 --allow 127.0.0.1 --log-stderr --verbose --enable-tcp-insecure
# distccd --jobs 12 -N 12 --no-detach --daemon --allow 192.168.1.0/24 --allow 127.0.0.1 --log-stderr --enable-tcp-insecure --verbose


if type -p /usr/local/bin/distcc >/dev/null 2>&1; then
    export CCACHE_MAXSIZE=50G
    export CCACHE_CPP2=true
    export CCACHE_HARDLINK=true
    export CCACHE_SLOPPINESS=file_macro,time_macros,include_file_mtime,include_file_ctime,file_stat_matches
    # /usr/local/bin/ccache
    # /usr/local/bin/distcc
    # /usr/local/bin/pump
    # /Users/devnul/Desktop/CC_cache.sh
    export CCACHE_PREFIX="/usr/local/bin/distcc"
    exec \
        /usr/local/bin/ccache \
        /Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/clang "$@"
else
    exec /Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/clang "$@"
fi