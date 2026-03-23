mod ecs;
use ecs::world::World;

use crate::ecs::components::health::Health;
use crate::ecs::components::tag::Tag;

fn main() {
    let mut world = World::new();

    let player = world.spawn();
    world.insert_health(player, Health::new(100, 100));
    world.insert_tag(player, Tag::new("Valhizen"));

    let goblin = world.spawn();
    world.insert_health(goblin, Health::new(20, 20));
    world.insert_tag(goblin, Tag::new("Goblin"));

    world.deal_damage(goblin, 15);

    world.print_value();
}
