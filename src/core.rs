use bevy_ecs::prelude::*;
use serde::Serialize;

pub trait RpcObject {
    fn rpc_object_type() -> u32;
}

pub trait RpcVariant {
    fn rpc_variant_type() -> u32;
}

pub trait RpcEvent {
    fn rpc_event_type() -> u32;
    fn rpc_event_invoke(bytes: &[u8], world: &mut World);
}

pub trait Rpc {
    fn spawn(&mut self, id: u32, t: u32);
    fn despawn(&mut self, id: u32);
    fn serialize<V: RpcVariant + Serialize>(&mut self, id: u32, variant: &V);
    fn invoke(&mut self, bytes: &[u8], world: &mut World);
    fn reg_event<E: RpcEvent>(&mut self);
    fn snapshot(&mut self) -> &[u8];
    fn clear(&mut self);
    fn data(&self) -> &[u8];
}
