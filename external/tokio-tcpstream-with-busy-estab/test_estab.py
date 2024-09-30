# Adapted from original source:
# https://github.com/cloudflare/cloudflare-blog/tree/master/2019-09-tcp-keepalives

import socket
import time
import utils
import tcptest
import asyncio

async def tokio_main():
    utils.new_ns()

    port = 1

    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM, 0)

    s.bind(('127.0.0.1', port))
    s.listen(16)

    tcpdump = utils.tcpdump_start(port)

    c = await tcptest.connect('127.0.0.1', port)

    # drop packets
    utils.drop_start(dport=port)
    utils.drop_start(sport=port)

    t0 = time.time()
    await c.send(b'hello world')

    time.sleep(1)
    utils.ss(port)

    # utils.drop_stop(dport=port)
    # utils.drop_stop(sport=port)
    # time.sleep(1)
    # utils.ss(port)

    utils.ss(port)

    t1 = time.time()
    print("[ ] took: %f seconds" % (t1-t0,))

asyncio.run(tokio_main())