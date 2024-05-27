
pub struct TcpListener(MioTcpListener);

impl TcpListener {
    pub fn bind(addr: SocketAddr) -> Result<Self> {
        Ok(Self(MioTcpListener::bind(addr)?))
    }

    pub fn accept(self) -> Accept {
        self.0.into()
    }
}
