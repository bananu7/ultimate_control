use std::io::{Error, ErrorKind};

use miniz_oxide::inflate::decompress_to_vec_zlib_with_limit;
use serde_json::Value;

use crate::types::UcPacket;

/// Upper bound on an inflated ZM payload; the io24 state dump is ~10 kB,
/// Series III mixers send a few hundred kB.
const ZM_INFLATE_LIMIT: usize = 4 * 1024 * 1024;

pub fn print_packet(packet: &UcPacket, outgoing: bool) {
    if outgoing {
        print!("-> ");
    } else {
        print!("<- ");
    }

    match packet {
        UcPacket::ZM {
            ap,
            unknown,
            compressed_payload,
        } => {
            let body = match inflate_zm(compressed_payload) {
                Ok(text) => text,
                Err(e) => format!("<{e}>"),
            };
            println!("ZM ({:02X?}, {}, {})", ap, unknown, body);
        }

        UcPacket::UM { ap, udp_port } => {
            println!("UM ({:02X?}, {})", ap, udp_port);
        }

        UcPacket::Unknown { tag, ap, data } => {
            println!(
                "{} ({:02X?}, {} bytes) {:02X?}",
                String::from_utf8_lossy(tag),
                ap,
                data.len(),
                &data[..data.len().min(32)]
            );
        }

        _ => {
            println!("{:02X?}", packet);
        }
    }
}

/// Inflate the zlib-compressed body of a ZM packet into its JSON text.
pub fn inflate_zm(compressed: &[u8]) -> std::io::Result<String> {
    let payload = decompress_to_vec_zlib_with_limit(compressed, ZM_INFLATE_LIMIT)
        .map_err(|e| Error::new(ErrorKind::InvalidData, format!("ZM inflate failed: {e:?}")))?;

    // The capture in doc/zm.py shows the text may be NUL terminated.
    let end = payload
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(payload.len());
    String::from_utf8(payload[..end].to_vec())
        .map_err(|e| Error::new(ErrorKind::InvalidData, format!("ZM is not UTF-8: {e}")))
}

/// A device parameter as reported in a `Synchronize` state dump.
#[derive(Debug, PartialEq, Clone)]
pub struct Param {
    /// Wire path usable in a PV packet, e.g. `main/ch1/hardwareMute`.
    pub path: String,
    pub value: Value,
}

/// Flatten the `Synchronize` JSON tree into `path = value` pairs.
///
/// The tree alternates `children` (named nodes) and `values` (leaf
/// parameters), so `children.main.children.ch1.values.mute` becomes
/// `main/ch1/mute`, which is exactly the string a PV packet carries.
/// Entries under the top-level `shared` object are flattened the same way.
pub fn flatten_state(state: &Value) -> Vec<Param> {
    let mut out = Vec::new();
    for key in ["children", "shared"] {
        if let Some(node) = state.get(key) {
            walk(node, &mut Vec::new(), &mut out);
        }
    }
    out
}

fn walk(node: &Value, path: &mut Vec<String>, out: &mut Vec<Param>) {
    let Some(map) = node.as_object() else {
        return;
    };
    for (k, v) in map {
        match (k.as_str(), v) {
            ("values", Value::Object(values)) => {
                for (name, value) in values {
                    let mut full = path.clone();
                    full.push(name.clone());
                    out.push(Param {
                        path: full.join("/"),
                        value: value.clone(),
                    });
                }
            }
            ("children", Value::Object(children)) => {
                for (name, child) in children {
                    path.push(name.clone());
                    walk(child, path, out);
                    path.pop();
                }
            }
            // The root of `children`/`shared` is itself a map of named nodes.
            (_, Value::Object(_)) => {
                path.push(k.clone());
                walk(v, path, out);
                path.pop();
            }
            _ => {}
        }
    }
}

/// Parse a ZM payload straight into the flattened parameter list.
pub fn params_from_zm(compressed: &[u8]) -> std::io::Result<Vec<Param>> {
    let text = inflate_zm(compressed)?;
    let state: Value = serde_json::from_str(&text)
        .map_err(|e| Error::new(ErrorKind::InvalidData, format!("ZM is not JSON: {e}")))?;
    Ok(flatten_state(&state))
}

#[cfg(test)]
mod test {
    use super::*;

    // `zmdata2` from doc/zm.py: a small ZM body captured from a Revelator io24.
    const ZM_SMALL: &str = "785eab56ca4c51b252500a2d48492c49f5cd4cc974cd4b29c8cfcc2b2956d2514a85b27d328b4b80aaa2636b0191ac10c1";

    fn unhex(s: &str) -> Vec<u8> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }

    #[test]
    fn inflates_captured_zm() {
        let text = inflate_zm(&unhex(ZM_SMALL)).unwrap();
        let json: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(json["id"], "UpdateMidiEndpoints");
    }

    #[test]
    fn inflate_rejects_garbage() {
        assert!(inflate_zm(b"not zlib at all").is_err());
    }

    #[test]
    fn flattens_state_tree_to_pv_paths() {
        let state: Value = serde_json::from_str(
            r#"{"id":"Synchronize","children":{
                "global":{"values":{"phonesMute":0.0}},
                "main":{"children":{"ch1":{"values":{"mute":0.0,"hardwareMute":1.0}}}}
            },"shared":{"strings":{"values":{"name":"io24"}}}}"#,
        )
        .unwrap();

        let mut paths: Vec<String> = flatten_state(&state).into_iter().map(|p| p.path).collect();
        paths.sort();
        assert_eq!(
            paths,
            vec![
                "global/phonesMute",
                "main/ch1/hardwareMute",
                "main/ch1/mute",
                "strings/name",
            ]
        );
    }
}

#[cfg(test)]
mod fixture_test {
    use super::*;

    /// Full `Synchronize` dump captured from a Revelator io24 (doc/zm.py).
    const IO24_STATE: &[u8] = include_bytes!("../tests/fixtures/io24_synchronize.zlib");

    #[test]
    fn io24_state_dump_lists_main_mutes() {
        let params = params_from_zm(IO24_STATE).unwrap();
        let find = |path: &str| {
            params
                .iter()
                .find(|p| p.path == path)
                .map(|p| p.value.clone())
        };

        assert_eq!(find(crate::params::MAIN_MUTE), Some(Value::from(0.0)));
        assert_eq!(find(crate::params::HARDWARE_MUTE), Some(Value::from(0.0)));
        assert_eq!(find(crate::params::CH1_MUTE), Some(Value::from(0.0)));
        assert!(params.len() > 100, "only {} params", params.len());
    }
}
