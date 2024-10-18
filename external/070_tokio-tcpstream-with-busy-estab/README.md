# Непосредственно сама статья

https://xuanwo.io/2024/01-why-sql-hang-for-exactly-940s/

# Tokio TcpStream with busy ESTAB TCP

This project is used to reproduce the cases that the ESTAB TCP is busy. In this case, TcpStream will not emit any events. Thus tokio runtime will not poll our futures anymore which makes the futures hang until TCP reaches it's own retry times.

## How to reproduce

- Setup the environment

```bash
python -m venv venv
source ./venv/bin/activate
pip install maturin
```

- Build rust project into python module

```bash
RUSTFLAGS="--cfg tokio_unstable" maturin develop
```

The `RUSTFLAGS` is required since we want more logs from our patched tokio.

- Run python script

```bash
sudo RUST_LOG=trace python test_estab.py
```

The `RUST_LOG` is required since we want our rust log been printed.

## Whether I reproduced?

```shell
  2024-01-24T03:46:32.851480Z DEBUG tokio::runtime::io::scheduled_io: Readiness poll returns Pending
    at tokio/tokio/src/runtime/io/scheduled_io.rs:554
    in tokio::task::runtime.spawn with kind: task, task.name: , task.id: 36, loc.file: "/home/xuanwo/.cargo/registry/src/index.crates.io-6f17d22bba15001f/pyo3-asyncio-0.20.0/src/tokio.rs", loc.line: 94, loc.col: 23

  2024-01-24T03:46:32.851489Z TRACE tokio::task::waker: op: "waker.clone", task.id: 2252074691592193
    at tokio/tokio/src/runtime/task/waker.rs:69
    in tokio::task::runtime.spawn with kind: task, task.name: , task.id: 36, loc.file: "/home/xuanwo/.cargo/registry/src/index.crates.io-6f17d22bba15001f/pyo3-asyncio-0.20.0/src/tokio.rs", loc.line: 94, loc.col: 23

 00:00:00.000000 IP 127.0.0.1.60382 > 127.0.0.1.1: Flags [S], seq 3101788242, win 33280, options [mss 65495,sackOK,TS val 3407635471 ecr 0,nop,wscale 7], length 0
 00:00:00.000006 IP 127.0.0.1.1 > 127.0.0.1.60382: Flags [S.], seq 2567999131, ack 3101788243, win 33280, options [mss 65495,sackOK,TS val 3407635471 ecr 3407635471,nop,wscale 7], length 0
 00:00:00.000012 IP 127.0.0.1.60382 > 127.0.0.1.1: Flags [.], ack 1, win 260, options [nop,nop,TS val 3407635471 ecr 3407635471], length 0
 00:00:00.016442 IP 127.0.0.1.60382 > 127.0.0.1.1: Flags [P.], seq 1:12, ack 1, win 260, options [nop,nop,TS val 3407635488 ecr 3407635471], length 11
 00:00:00.223164 IP 127.0.0.1.60382 > 127.0.0.1.1: Flags [P.], seq 1:12, ack 1, win 260, options [nop,nop,TS val 3407635695 ecr 3407635471], length 11
 00:00:00.431162 IP 127.0.0.1.60382 > 127.0.0.1.1: Flags [P.], seq 1:12, ack 1, win 260, options [nop,nop,TS val 3407635903 ecr 3407635471], length 11
 00:00:00.839162 IP 127.0.0.1.60382 > 127.0.0.1.1: Flags [P.], seq 1:12, ack 1, win 260, options [nop,nop,TS val 3407636311 ecr 3407635471], length 11
```

`Readiness` poll returns `Pending` and won't be waken up anymore until TCP returns timeout.
