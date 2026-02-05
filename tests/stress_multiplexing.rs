use muxis::proto::codec::{Decoder, Encoder};
use muxis::proto::frame::Frame;
use muxis::ClientBuilder;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::test]
async fn test_multiplexing_stress() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let addr_str = format!("redis://{}", addr);

    // Spawn Mock Server
    tokio::spawn(async move {
        loop {
            let (mut socket, _) = match listener.accept().await {
                Ok(s) => s,
                Err(_) => break,
            };

            tokio::spawn(async move {
                let mut decoder = Decoder::new();
                let mut encoder = Encoder::new();
                let mut buf = [0u8; 4096];

                loop {
                    let n = match socket.read(&mut buf).await {
                        Ok(0) => return, // EOF
                        Ok(n) => n,
                        Err(_) => return,
                    };

                    decoder.append(&buf[..n]);

                    while let Ok(Some(frame)) = decoder.decode() {
                        // For any command, respond with PONG or OK
                        let response = match frame {
                            Frame::Array(ref args) => {
                                if let Some(Frame::BulkString(Some(cmd))) = args.first() {
                                    if cmd.eq_ignore_ascii_case(b"PING") {
                                        Frame::SimpleString(b"PONG".to_vec())
                                    } else {
                                        Frame::SimpleString(b"OK".to_vec())
                                    }
                                } else {
                                    Frame::Error("ERR unknown command".to_string().into_bytes())
                                }
                            }
                            _ => Frame::Error("ERR format".to_string().into_bytes()),
                        };

                        encoder.encode(&response);
                        let data = encoder.take();
                        if socket.write_all(&data).await.is_err() {
                            return;
                        }
                    }
                }
            });
        }
    });

    // Client Connect
    // Use a large queue to accommodate burst
    let client = ClientBuilder::new()
        .address(addr_str)
        .queue_size(10000)
        .build()
        .await
        .expect("Failed to connect");

    let mut handles = Vec::new();

    // Spawn 1000 concurrent requests
    for _ in 0..1000 {
        let mut client = client.clone();
        handles.push(tokio::spawn(async move {
            let res = client.ping().await;
            assert_eq!(res.unwrap(), b"PONG".as_slice());
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }
}
