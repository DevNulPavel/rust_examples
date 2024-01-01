panic: runtime error: invalid memory address or nil pointer dereference
[signal SIGSEGV: segmentation violation code=0x1 addr=0x0 pc=0x824a1d]

goroutine 21 [running]:
github.com/google/pprof/internal/driver.fetch({0x7ffc1c2288cd, 0x1}, 0xffffffffc4653600, 0x0?, {0x7f35c837f2b8, 0xc000010180}, {0x9d09a0, 0xc00007cae0})
	/home/ershov/go/pkg/mod/github.com/google/pprof@v0.0.0-20231229205709-960ae82b1e42/internal/driver/fetch.go:514 +0x31d
github.com/google/pprof/internal/driver.grabProfile(0xc0001ae8c0, {0x7ffc1c2288cd?, 0x0?}, {0x0?, 0x0?}, {0x9d24b0, 0xc000079a30}, {0x7f35c837f2b8, 0xc000010180}, {0x9d09a0, ...})
	/home/ershov/go/pkg/mod/github.com/google/pprof@v0.0.0-20231229205709-960ae82b1e42/internal/driver/fetch.go:334 +0x145
github.com/google/pprof/internal/driver.concurrentGrab.func1(0xc0001d99c0)
	/home/ershov/go/pkg/mod/github.com/google/pprof@v0.0.0-20231229205709-960ae82b1e42/internal/driver/fetch.go:211 +0x97
created by github.com/google/pprof/internal/driver.concurrentGrab
	/home/ershov/go/pkg/mod/github.com/google/pprof@v0.0.0-20231229205709-960ae82b1e42/internal/driver/fetch.go:209 +0xae
