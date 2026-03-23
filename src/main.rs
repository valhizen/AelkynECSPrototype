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

    world.read_value(player);
    world.read_value(goblin);

    if let Some(h) = world.get::<Health>(player) {
        println!("Health: {} / {}", h.current, h.max);
    }

    if let Some(t) = world.get::<Tag>(player) {
        println!("Name: {}", t.tag);
    }

    world.deal_damage(goblin, 10);

    if let Some(h) = world.get::<Health>(goblin) {
        println!("Health: {} / {}", h.current, h.max);
    }
}
