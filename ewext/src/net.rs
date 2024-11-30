use std::{
    env,
    io::ErrorKind,
    net::{SocketAddr, TcpStream},
    time::Duration,
};

use shared::{NoitaInbound, NoitaOutbound};
use tungstenite::{client, WebSocket};

pub(crate) struct NetManager {
    ws: WebSocket<TcpStream>,
}

impl NetManager {
    pub(crate) fn new() -> eyre::Result<Self> {
        let address: SocketAddr = env::var("NP_NOITA_ADDR")
            .ok()
            .and_then(|x| x.parse().ok())
            .unwrap_or_else(|| SocketAddr::new("127.0.0.1".parse().unwrap(), 21251));

        let request = format!("ws://{address}");

        let tcp = TcpStream::connect_timeout(&address, Duration::from_secs(2))?;
        tcp.set_read_timeout(Some(Duration::from_secs(2)))?;
        tcp.set_nodelay(true)?;
        let (ws, _) = client(request, tcp)?;

        Ok(NetManager { ws })
    }

    pub(crate) fn switch_to_non_blocking(&mut self) -> eyre::Result<()> {
        self.ws.get_mut().set_read_timeout(None)?;
        self.ws.get_mut().set_nonblocking(true)?;
        Ok(())
    }

    pub(crate) fn send(&mut self, msg: &NoitaOutbound) -> eyre::Result<()> {
        self.ws
            .send(tungstenite::Message::Binary(bitcode::encode(msg)))?;
        Ok(())
    }

    pub(crate) fn recv(&mut self) -> eyre::Result<Option<NoitaInbound>> {
        loop {
            match self.ws.read() {
                Ok(tungstenite::Message::Binary(msg)) => break Ok(Some(bitcode::decode(&msg)?)),
                Ok(_) => {}
                Err(tungstenite::Error::Io(err))
                    if err.kind() == ErrorKind::WouldBlock || err.kind() == ErrorKind::TimedOut =>
                {
                    break Ok(None)
                }
                Err(err) => break Err(err.into()),
            }
        }
    }
}

impl Drop for NetManager {
    fn drop(&mut self) {
        println!("Closing netmanager");
        self.ws.get_mut().set_nonblocking(false).ok();
        self.ws.close(None).ok();
        self.ws.flush().ok();
    }
}