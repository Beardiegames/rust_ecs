# simple_ecs
Simple and fast ECS library for Rust.

The name simple comes from it's maximum of a 1000 entities. It has been limited because all entity data is forced onto the stack, leaving some space for actual object data implementation.

Is tested on an older version intel i5, and performs at about 80 million calls/sec. This means it is able to update 80M entities divided by the amount of systems every entity is updated by. The systems where tested by performing an addition of one on all entities every update cycle.

