mod ecs;
use ecs::world::World;

use crate::ecs::components::health::Health;
use crate::ecs::components::tag::Tag;

fn main() {
    let mut world = World::new();

    let player = world.spawn();
    world.insert(player, Health::new(100, 100));
    world.insert(player, Tag::new("Valhizen"));

    let goblin = world.spawn();
    world.insert(goblin, Health::new(20, 20));
    world.insert(goblin, Tag::new("Goblin"));

    for (entity, h, t) in world.query2::<Health, Tag>() {
        println!("{} has {} / {} hp", t.tag, h.current, h.max);
    }
}
