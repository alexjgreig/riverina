//TODO: Fix buffer issue that causes FIX messages to be cut off. The buffer should be dynamically
//sized at runtime instead so messages aren't cut off. This could be because multiple updates for
//instruments are being placed in the buffer and it is overflowing. If multiple messages are being
//read at once this needs to be fixed in the message parser as this would cause prices to be
//overidden
use crate::message_constructer::MessageConstructer;
use crate::message_parser;
use chrono::Utc;
use rustls;
use std::io;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::str::from_utf8;
use std::sync::Arc;
use std::time::Duration;
use webpki;
use webpki_roots;

pub struct TlsClient {
    pub tls_stream: rustls::StreamOwned<rustls::ClientSession, std::net::TcpStream>,
    pub message_sequence_number: u64,
    pub order_id: u64,
}

impl io::Write for TlsClient {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.message_sequence_number += 1;
        self.tls_stream.write(bytes)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.tls_stream.flush()
    }
}

impl io::Read for TlsClient {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        self.tls_stream.read(bytes)
    }
}
impl TlsClient {
    pub fn new(host: &str, port: u16) -> TlsClient {
        let mut config = rustls::ClientConfig::new();
        config
            .root_store
            .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);
        let dns_name = webpki::DNSNameRef::try_from_ascii_str(host).unwrap();
        let socket = TcpStream::connect((host, port)).unwrap();
        socket
            .set_read_timeout(Some(Duration::from_millis(1000)))
            .unwrap();
        let tls_session = rustls::ClientSession::new(&Arc::new(config), dns_name);

        TlsClient {
            tls_stream: rustls::StreamOwned::new(tls_session, socket),
            message_sequence_number: 1,
            order_id: 1,
        }
    }

    pub fn logon(&mut self, constructer: &MessageConstructer, qualifier: &str) -> String {
        // self.flush().unwrap();
        self.message_sequence_number = 1;
        let mut buffer = [0u8; 4096];
        self.write(constructer.logon(qualifier, 1, 60, true).as_bytes())
            .unwrap();
        match self.read(&mut buffer) {
            Err(e) => return e.to_string(),
            Ok(_) => {
                return format!(
                    "{}",
                    message_parser::parse_fix_message(from_utf8(&mut buffer).unwrap().to_string())
                        .unwrap()
                )
            }
        };
    }
    pub fn heartbeat(&mut self, constructer: &MessageConstructer, qualifier: &str) -> String {
        match self.write(
            constructer
                .heartbeat(qualifier, self.message_sequence_number)
                .as_bytes(),
        ) {
            Err(_) => return "connection_aborted".to_owned(),
            Ok(_) => return "success".to_owned(),
        };
    }

    pub fn market_data_request_establishment(
        &mut self,
        constructer: &MessageConstructer,
        mdr_id: &str,
        symbol: u32,
    ) -> Result<(u32, f64, f64), String> {
        let mut buffer = [0u8; 4096];
        self.write(
            constructer
                .market_data_request(
                    "QUOTE",
                    self.message_sequence_number,
                    mdr_id,
                    1,
                    1,
                    1,
                    symbol,
                )
                .as_bytes(),
        )
        .unwrap();
        match self.read(&mut buffer) {
            Err(e) => {
                if e.kind() == io::ErrorKind::ConnectionReset
                    || e.kind() == io::ErrorKind::ConnectionAborted
                    || e.kind() == io::ErrorKind::BrokenPipe
                {
                    return Err("connection_aborted".to_owned());
                } else if e.kind() == io::ErrorKind::TimedOut
                    || e.kind() == io::ErrorKind::WouldBlock
                {
                    return Err("timed_out".to_owned());
                } else {
                    return Err(e.to_string());
                }
            }
            Ok(x) => {
                if x == 0 {
                    return Err("0_bytes_read".to_owned());
                } else {
                    let parsed_message = message_parser::parse_fix_message(
                        from_utf8(&mut buffer).unwrap().to_string(),
                    )
                    .unwrap();
                    if parsed_message == "test_request".to_owned() {
                        return Err(parsed_message);
                    } else if parsed_message == "heartbeat".to_owned() {
                        return Err(parsed_message);
                    } else {
                        let res = parsed_message.split(',').collect::<Vec<_>>();
                        return Ok((
                            res[0].parse::<u32>().unwrap(),
                            res[1].parse::<f64>().unwrap(),
                            res[2].parse::<f64>().unwrap(),
                        ));
                    }
                }
            }
        }
    }
    pub fn market_data_update(&mut self) -> Result<(u32, f64, f64), String> {
        let mut buffer = [0u8; 4096];
        match self.read(&mut buffer) {
            Err(e) => {
                if e.kind() == io::ErrorKind::ConnectionReset
                    || e.kind() == io::ErrorKind::ConnectionAborted
                    || e.kind() == io::ErrorKind::BrokenPipe
                {
                    return Err("connection_aborted".to_owned());
                } else if e.kind() == io::ErrorKind::TimedOut
                    || e.kind() == io::ErrorKind::WouldBlock
                {
                    return Err("timed_out".to_owned());
                } else {
                    return Err(e.to_string());
                }
            }
            Ok(x) => {
                if x == 0 {
                    return Err("0_bytes_read".to_owned());
                } else {
                    let parsed_message = message_parser::parse_fix_message(
                        from_utf8(&mut buffer).unwrap().to_string(),
                    )
                    .unwrap();
                    if parsed_message == "test_request".to_owned() {
                        return Err(parsed_message);
                    } else if parsed_message == "heartbeat".to_owned() {
                        return Err(parsed_message);
                    } else {
                        let res = parsed_message.split(',').collect::<Vec<_>>();
                        return Ok((
                            res[0].parse::<u32>().unwrap(),
                            res[1].parse::<f64>().unwrap(),
                            res[2].parse::<f64>().unwrap(),
                        ));
                    }
                }
            }
        }
    }

    pub fn single_order(
        &mut self,
        constructer: &MessageConstructer,
        symbol: u32,
        side: u32,
        order_quantity: u64,
        position_id: Option<String>,
    ) -> Result<String, String> {
        let mut buffer = [0u8; 4096];
        let utc_time = Utc::now().format("%Y%m%d-%H:%M:%S").to_string();
        match position_id {
            None => {
                self.write(
                    constructer
                        .single_order_request(
                            "TRADE",
                            self.message_sequence_number,
                            1,
                            symbol,
                            side,
                            &utc_time,
                            order_quantity,
                            1,
                            None,
                        )
                        .as_bytes(),
                )
                .unwrap();
            }
            Some(_) => {
                self.write(
                    constructer
                        .single_order_request(
                            "TRADE",
                            self.message_sequence_number,
                            1,
                            symbol,
                            side,
                            &utc_time,
                            order_quantity,
                            1,
                            position_id,
                        )
                        .as_bytes(),
                )
                .unwrap();
            }
        }
        match self.read(&mut buffer) {
            Err(e) => {
                if e.kind() == io::ErrorKind::ConnectionAborted {
                    return Err("connection_aborted".to_owned());
                } else {
                    panic!("Error when reading connection, Error: {}", e);
                }
            }
            Ok(x) => {
                if x == 0 {
                    return Err("0_bytes_read".to_owned());
                } else {
                    let parsed_message = message_parser::parse_fix_message(
                        from_utf8(&mut buffer).unwrap().to_string(),
                    );
                    match parsed_message {
                        Err(e) if e == "order_cancelled".to_string() => {
                            return Err("order_cancelled".to_string())
                        }
                        Err(e) => return Err(e),
                        Ok(id) => return Ok(id),
                    }
                }
            }
        }
    }

    pub fn logout(&mut self, constructer: &MessageConstructer, qualifier: &str) -> String {
        let mut buffer = [0u8; 4096];
        self.write(
            constructer
                .logout(qualifier, self.message_sequence_number)
                .as_bytes(),
        )
        .unwrap();
        match self.read(&mut buffer) {
            Err(e) => return e.to_string(),
            Ok(_) => {
                return format!(
                    "{}",
                    message_parser::parse_fix_message(from_utf8(&mut buffer).unwrap().to_string())
                        .unwrap()
                );
            }
        }
    }
}
