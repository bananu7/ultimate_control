//! The driver must keep reading after frames it cannot decode: before this
//! the first unrecognised packet type ended the stream and the app exited.

use std::time::Duration;

use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use ultimate_control::async_driver::AsyncUcDriver;
use ultimate_control::packet::ser;
use ultimate_control::types::{AddressPair, UcPacket};

fn frame(tag: &[u8; 2], body: &[u8]) -> Vec<u8> {
    let mut out = b"UC\x00\x01".to_vec();
    out.extend_from_slice(&((body.len() + 2) as u16).to_le_bytes());
    out.extend_from_slice(tag);
    out.extend_from_slice(body);
    out
}

#[tokio::test]
async fn unknown_and_malformed_frames_do_not_end_the_stream() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let ap = AddressPair { a: 0x66, b: 0x6b };
    let good = UcPacket::PV(ap.clone(), "main/ch1/hardwareMute".to_string(), 1.0);
    let good_bytes = ser(&good);

    tokio::spawn(async move {
        let (mut sock, _) = listener.accept().await.unwrap();
        // 1. a type we do not decode (chunk)
        sock.write_all(&frame(b"CK", &[0x66, 0, 0x6b, 0, 9, 9, 9]))
            .await
            .unwrap();
        // 2. a well-framed but unparseable PV (name is not UTF-8, no float)
        sock.write_all(&frame(b"PV", &[0x66, 0, 0x6b, 0, 0xff, 0xfe]))
            .await
            .unwrap();
        // 3. the packet we actually care about
        sock.write_all(&good_bytes).await.unwrap();
        // keep the socket open long enough for the client to read everything
        tokio::time::sleep(Duration::from_secs(2)).await;
    });

    let mut driver = AsyncUcDriver::connect(&addr.to_string()).await.unwrap();
    driver.trace = false;

    let mut seen = Vec::new();
    for _ in 0..3 {
        let p = tokio::time::timeout(Duration::from_secs(2), driver.next_packet())
            .await
            .expect("stream stalled")
            .expect("stream ended early")
            .expect("decode error");
        seen.push(p);
    }

    assert!(matches!(&seen[0], UcPacket::Unknown { tag, .. } if tag == b"CK"));
    assert!(matches!(&seen[1], UcPacket::Unknown { tag, .. } if tag == b"PV"));
    assert_eq!(seen[2], good);
}

#[tokio::test]
async fn set_param_confirmed_waits_for_the_echo() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        let (mut sock, _) = listener.accept().await.unwrap();
        // noise first, then the echo with the address pair swapped
        sock.write_all(&ser(&UcPacket::KA(AddressPair { a: 0, b: 0x66 })))
            .await
            .unwrap();
        sock.write_all(&ser(&UcPacket::PV(
            AddressPair { a: 0x66, b: 0x6b },
            "main/ch1/mute".to_string(),
            1.0,
        )))
        .await
        .unwrap();
        tokio::time::sleep(Duration::from_secs(2)).await;
    });

    let mut driver = AsyncUcDriver::connect(&addr.to_string()).await.unwrap();
    driver.trace = false;
    let v = driver
        .set_param_confirmed("main/ch1/mute", 1.0, Duration::from_secs(2))
        .await
        .unwrap();
    assert_eq!(v, 1.0);
}
