use super::{
    inner::{AuthReceiverClone, PacketReceiverClone}, AuthReceiver as TransportAuthReceiver, AuthSender as TransportAuthSender, PacketReceiver, PacketSender as TransportSender, RecvError, SendError, Socket as TransportSocket
};

pub struct Socket {
    sockets: Vec<Box<dyn TransportSocket>>,
}


impl Socket {
    pub fn new() -> Self {
        Self {
            sockets: Vec::new(),
        }
    }

    pub fn add_socket(&mut self, socket: Box<dyn TransportSocket>) {
        self.sockets.push(socket);
    }
}
impl Into<Box<dyn TransportSocket>> for Socket {
    fn into(self) -> Box<dyn TransportSocket> {
        Box::new(self)
    }
}

impl TransportSocket for Socket {
    fn listen(
        self: Box<Self>,
    ) -> (
        Box<dyn TransportAuthSender>,
        Box<dyn TransportAuthReceiver>,
        Box<dyn TransportSender>,
        Box<dyn PacketReceiver>,
    ) {
        let mut auth_sender = Vec::new();
        let mut auth_receiver = Vec::new();
        let mut sender = Vec::new();
        let mut packet_receiver = Vec::new();
        
        for socket in self.sockets {
            let (auth_sender_socket, auth_receiver_socket, sender_socket, packet_receiver_socket) =
                socket.listen();
            auth_sender.push(auth_sender_socket);
            auth_receiver.push(auth_receiver_socket);
            sender.push(sender_socket);
            packet_receiver.push(packet_receiver_socket);
        }

        (
            Box::new(MultipleAuthSender::new(auth_sender)),
            Box::new(MultipleAuthReceiver::new(auth_receiver)),
            Box::new(MultipleSender::new(sender)),
            Box::new(MultiplePacketReceiver::new(packet_receiver)),
        )
    }
}

struct MultipleAuthSender {
    targets: Vec<Box<dyn TransportAuthSender>>,
}

impl MultipleAuthSender {
    fn new(targets: Vec<Box<dyn TransportAuthSender>>) -> Self {
        Self { targets }
    }
}

impl TransportAuthSender for MultipleAuthSender {
    fn accept(
        &self,
        address: &crate::user::UserAuthAddr,
        identity_token: &naia_shared::IdentityToken,
    ) -> Result<(), SendError> {
        let mut all_ok = true;
        for target in self.targets.iter() {
            if let Err(_) = target.accept(address, identity_token) {
                all_ok = false;
            }
        }
        if all_ok {
            return Ok(());
        }
        else {
            Err(SendError {})
        }
    }

    fn reject(&self, address: &crate::user::UserAuthAddr) -> Result<(), SendError> {
        let mut all_ok = true;
        for target in self.targets.iter() {
            if let Err(_) = target.reject(address) {
                all_ok = false;
            }
        }
        if all_ok {
            return Ok(());
        } else {
            Err(SendError {})
        }
    }
}

struct MultipleAuthReceiver {
    targets: Vec<Box<dyn TransportAuthReceiver>>,
}

impl MultipleAuthReceiver {
    fn new(targets: Vec<Box<dyn TransportAuthReceiver>>) -> Self {
        Self { targets }
    }
}

impl AuthReceiverClone for MultipleAuthReceiver {
    fn clone_box(&self) -> Box<dyn TransportAuthReceiver> {
        Box::new(MultipleAuthReceiver::new(self.targets.iter().map(|x| x.clone_box()).collect() ))
    }
}

impl TransportAuthReceiver for MultipleAuthReceiver {
    fn receive(&mut self) -> Result<Option<(crate::user::UserAuthAddr, &[u8])>, RecvError> {
        let mut all_ok = true;
        for target in self.targets.iter_mut() {
            match target.receive() {
                Ok(result) => {
                    if result.is_some() {
                        return Ok(result);
                    }
                },
                Err(_) => all_ok = false
            }
        }
        if all_ok {
            return Ok(None);
        } else {
            Err(RecvError {})
        }
    }
}

struct MultipleSender {
    targets: Vec<Box<dyn TransportSender>>,
}

impl MultipleSender {
    fn new(targets: Vec<Box<dyn TransportSender>>) -> Self {
        Self { targets }
    }
}

impl TransportSender for MultipleSender {
    fn send(&self, address: &std::net::SocketAddr, payload: &[u8]) -> Result<(), SendError> {
        let mut all_ok = true;
        for target in self.targets.iter() {
            if let Err(_) = target.send(address, payload) {
                all_ok = false;
            }
        }
        if all_ok {
            return Ok(());
        } else {
            Err(SendError {})
        }
    }
}

struct MultiplePacketReceiver {
    targets: Vec<Box<dyn PacketReceiver>>,
}

impl MultiplePacketReceiver {
    fn new(targets: Vec<Box<dyn PacketReceiver>>) -> Self {
        Self { targets }
    }
}

impl PacketReceiverClone for MultiplePacketReceiver {
    fn clone_box(&self) -> Box<dyn PacketReceiver> {
        Box::new(MultiplePacketReceiver::new(self.targets.iter().map(|x| x.clone_box()).collect() ))
    }
}

impl PacketReceiver for MultiplePacketReceiver {
    fn receive(&mut self) -> Result<Option<(std::net::SocketAddr, &[u8])>, RecvError> {
        for target in self.targets.iter_mut() {
            if let Ok(result) = target.receive() {
                return Ok(result);
            }
        }
        Err(RecvError {})
    }
}
