use bevy::{
    prelude::*,
    ecs::*
};

pub fn count_query<Q: WorldQuery>(mut query: Query<Q>){
    let name = std::any::type_name::<Q>();
    println!("{}: {}", name, query.iter_mut().count());
}
