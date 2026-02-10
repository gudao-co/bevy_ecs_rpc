use crate::core::Rpc;
use crate::core::RpcEvent;
use crate::core::RpcVariant;
use bevy_ecs::prelude::*;
use rmp_serde::encode;
use serde::Serialize;
use std::collections::HashMap;
use std::io::Cursor;

type EventFactory = fn(&[u8], &mut World);

struct EntitySnapshot {
    t: u32,
    var_set: HashMap<u32, Vec<u8>>,
}

#[derive(Resource)]
pub struct RpcMem {
    buf: Vec<u8>,
    map: HashMap<u32, EventFactory>,
    snapshot: HashMap<u32, EntitySnapshot>,
}

impl RpcMem {
    pub fn new() -> Self {
        Self {
            buf: Vec::new(),
            map: HashMap::new(),
            snapshot: HashMap::new(),
        }
    }
}

fn _serialize_dyn(buf: &mut Vec<u8>, id: u32, t: u32, value: &[u8]) {
    buf.extend_from_slice(&[3]);
    buf.extend_from_slice(&id.to_le_bytes());
    buf.extend_from_slice(&t.to_le_bytes());
    let payload_len = value.len();
    buf.extend_from_slice(&(payload_len as u32).to_le_bytes());
    buf.extend_from_slice(value);
}

fn _spawn(buf: &mut Vec<u8>, id: u32, t: u32) {
    buf.extend_from_slice(&[1]);
    buf.extend_from_slice(&id.to_le_bytes());
    buf.extend_from_slice(&t.to_le_bytes());
}

impl Rpc for RpcMem {
    fn spawn(&mut self, id: u32, t: u32) {
        _spawn(&mut self.buf, id, t);
        self.snapshot.insert(
            id,
            EntitySnapshot {
                t,
                var_set: HashMap::new(),
            },
        );
    }

    fn despawn(&mut self, id: u32) {
        self.buf.extend_from_slice(&[2]);
        self.buf.extend_from_slice(&id.to_le_bytes());
        self.snapshot.remove(&id);
    }

    fn serialize<V: RpcVariant + Serialize>(&mut self, id: u32, variant: &V) {
        self.buf.extend_from_slice(&[3]);
        self.buf.extend_from_slice(&id.to_le_bytes());
        let variant_type = V::rpc_variant_type();
        self.buf.extend_from_slice(&variant_type.to_le_bytes());

        // 记住长度占位位置
        let start_len = self.buf.len();
        self.buf.extend_from_slice(&0u32.to_le_bytes()); // 占位长度

        // Cursor 包装 buf，从 start_len + 4 开始写入序列化数据
        {
            let mut cursor = Cursor::new(&mut self.buf); // 这里 reborrow，buf 仍可用
            cursor.set_position((start_len + 4) as u64);
            encode::write(&mut cursor, variant).unwrap();
        }

        // 回填实际序列化长度
        let payload_len = self.buf.len() - start_len - 4;
        self.buf[start_len..start_len + 4].copy_from_slice(&(payload_len as u32).to_le_bytes());
        if let Some(entity) = self.snapshot.get_mut(&id) {
            entity
                .var_set
                .insert(variant_type, self.buf[start_len + 4..].to_vec());
        }
    }

    fn invoke(&mut self, bytes: &[u8], world: &mut World) {
        let mut i: usize = 0;
        while i + 4 <= bytes.len() {
            let id = u32::from_le_bytes(bytes[i..i + 4].try_into().unwrap());
            i += 4;
            if let Some(factory) = self.map.get(&id) {
                if i + 4 <= bytes.len() {
                    let len = u32::from_le_bytes(bytes[i..i + 4].try_into().unwrap());
                    i += 4;
                    factory(&bytes[i..i + len as usize], world);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    fn reg_event<E: RpcEvent>(&mut self) {
        self.map.insert(E::rpc_event_type(), E::rpc_event_invoke);
    }

    fn snapshot(&mut self) -> &[u8] {
        self.buf.clear();
        for (entity_id, entity) in self.snapshot.iter() {
            _spawn(&mut self.buf, *entity_id, entity.t);
            for (t, v) in entity.var_set.iter() {
                _serialize_dyn(&mut self.buf, *entity_id, *t, v);
            }
        }
        &self.buf
    }

    fn clear(&mut self) {
        self.buf.clear();
    }

    fn data(&self) -> &[u8] {
        &self.buf
    }
}
