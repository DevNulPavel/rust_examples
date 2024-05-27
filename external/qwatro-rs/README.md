# qwatro
qwatro is a simple network tool which can scan tcp ports and provides proxy server functionality.

# Usage
Use `--help` to see all application arguments.
```
Usage: qwatro.exe <COMMAND>
Commands:
  ps     Port scan
  proxy  Proxy
  help   Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

# Port scanning
```
Usage: qwatro.exe ps [OPTIONS] [IP]

Arguments:
  [IP]  Scanning IP-address [default: 127.0.0.1]

Options:
  -p, --port-range <PORT_RANGE>              Port range [default: 1-65535]
      --tcp                                  Enable tcp scanning
      --tcp-resp-timeout <TCP_RESP_TIMEOUT>  TCP response timeout (ms) [default: 300]
      --max-tasks <MAX_TASKS>                Maximum parallel scan tasks [default: 500]
  -h, --help                                 Print help

```

### Example:
```bash
# run tcp port scanning on 192.168.100.1, port range 7000-9000
qwatro ps 192.168.100.1 -p 7000-9000 --tcp 
```

### Output:
```bash
[2023-03-19T10:10:55Z INFO  qwatro_port_scanner::scanner] start port scanning on 192.168.100.1, port range: (7000 - 9000)
[2023-03-19T10:10:55Z INFO  qwatro] 192.168.100.1:8079/TCP
[2023-03-19T10:10:55Z INFO  qwatro] 192.168.100.1:8099/TCP
[2023-03-19T10:10:55Z INFO  qwatro] 192.168.100.1:8089/TCP
[2023-03-19T10:10:55Z INFO  qwatro] 192.168.100.1:8080/TCP
[2023-03-19T10:10:55Z INFO  qwatro] 192.168.100.1:8090/TCP
[2023-03-19T10:10:55Z INFO  qwatro] 192.168.100.1:8191/TCP
[2023-03-19T10:10:55Z INFO  qwatro] 192.168.100.1:8443/TCP
```

Port range can be specified as `-p 7000` (single port) or `7000-9000` (port range).

# Proxy
You can run application as a proxy server using `qwatro proxy`
```
Usage: qwatro.exe proxy <COMMAND>

Commands:
  tcp   TCP
  udp   UDP
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

## TCP
To run program as tcp proxy use `qwatro proxy tcp` subcommand:
```
Usage: qwatro.exe proxy tcp [HOST_TO_SERVER]...

Arguments:
  [HOST_TO_SERVER]...  List of host to server mapping. Example: 127.0.0.1:9998>127.0.0.1:9999

Options:
  -h, --help  Print help
```

### Example:
```
# proxying 127.0.0.1:9998 -> 127.0.0.1:9999
qwatro proxy tcp 127.0.0.1:9998>127.0.0.1:9999`
```

You can specify several proxy tasks:

```
qwatro proxy tcp 127.0.0.1:9998>127.0.0.1:9999 127.0.0.1:10000>127.0.0.1:10001
```

## UDP
not implemented yet ...
