# ECS

Entities, Components, Systems

Design pattern different from OOP

* **Entity:** A thing which can have many *Components*
* **Component:** Data grouped by meaning (examples: Position, Renderable, Hostile...)
* **System:** Mechanism that *gathers* *and* *modifies* data from *Entities* and *Components* 

## Systems (specs)

**SystemData:** creates a list of types that we want to handle with this system and the Read, Write and Nullable policy. We can then deconstruct it with a touple