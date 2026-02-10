use bevy_ecs::prelude::*;
use serde::Serialize;
use crate::core::Rpc;
use crate::core::RpcVariant;
use crate::core::RpcObject;

pub fn rpc_spawn_system<E: RpcObject + Component, RPC: Rpc + Resource>(
    query: Query<Entity, Added<E>>,
    mut removed: RemovedComponents<E>,
    mut rpc: ResMut<RPC>,
) {
    for entity in query.iter() {
        let id = entity.index_u32();
        let t = E::rpc_object_type();
        rpc.spawn(id, t);
    }
    for entity in removed.read() {
        let id = entity.index_u32();
        rpc.despawn(id);
    }
}

pub fn rpc_change_system<V: RpcVariant + Component + Serialize, RPC: Rpc + Resource>(
    changed: Query<(Entity, &V), Changed<V>>,
    mut rpc: ResMut<RPC>,
) {
    for (entity, rpc_variant) in changed.iter() {
        let id = entity.index_u32();
        rpc.serialize(id, rpc_variant);
    }
}
