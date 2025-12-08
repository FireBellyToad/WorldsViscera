use std::collections::HashSet;

use hecs::{Entity, World};
use macroquad::math::Rect;

use crate::{
    components::{
        combat::{CanHide, CombatStats, SufferingDamage},
        common::{
            BlocksTile, Hates, Immobile, MyTurn, Named, Position, ProduceCorpse, ProduceSound,
            Renderable, SmellIntensity, Smellable, Species, SpeciesEnum, Viewshed,
        },
        health::Hunger,
        items::{BodyLocation, Deadly, Edible, Equipped, InBackback},
        monster::{Aquatic, IsPrey, LeaveTrail, Monster, Small, Smart, Venomous},
    },
    constants::{
        BASE_MONSTER_VIEW_RADIUS, FAST, MAX_HUNGER_TICK_COUNTER, NORMAL, SLOW, SLUG_TRAIL_LIFETIME,
        TILE_SIZE_F32,
    },
    maps::zone::DecalType,
    spawning::spawner::Spawn,
    systems::hunger_check::HungerStatus,
    utils::{assets::TextureName, roll::Roll},
};

impl Spawn {
    pub fn deep_one(ecs_world: &mut World, x: i32, y: i32) {
        Spawn::create_monster(
            ecs_world,
            "Deep One".to_string(),
            Species {
                value: SpeciesEnum::DeepSpawn,
            },
            CombatStats {
                level: 2,
                current_stamina: 3,
                max_stamina: 3,
                base_armor: 0,
                unarmed_attack_dice: 3,
                current_toughness: 8,
                max_toughness: 8,
                current_dexterity: 10,
                max_dexterity: 10,
                speed: NORMAL,
            },
            Edible {
                nutrition_dice_number: 5,
                nutrition_dice_size: 6,
            },
            Smellable {
                smell_log: "dried human sweat".to_string(),
                intensity: SmellIntensity::Faint,
            },
            ProduceSound {
                sound_log: "someone weezing".to_string(),
            },
            1.0,
            0.0,
            x,
            y,
        );
    }

    pub fn freshwater_viperfish(ecs_world: &mut World, x: i32, y: i32) {
        let freshwater_viperfish = Spawn::create_monster(
            ecs_world,
            "Freshwater viperfish".to_string(),
            Species {
                value: SpeciesEnum::Fish,
            },
            CombatStats {
                level: 4,
                current_stamina: 4,
                max_stamina: 4,
                base_armor: 0,
                unarmed_attack_dice: 4,
                current_toughness: 4,
                max_toughness: 4,
                current_dexterity: 14,
                max_dexterity: 14,
                speed: NORMAL,
            },
            Edible {
                nutrition_dice_number: 3,
                nutrition_dice_size: 6,
            },
            Smellable {
                smell_log: "fish".to_string(),
                intensity: SmellIntensity::None,
            },
            ProduceSound {
                sound_log: "a splash in the water".to_string(),
            },
            4.0,
            0.0,
            x,
            y,
        );

        let _ = ecs_world.insert(freshwater_viperfish, (Aquatic {}, CanHide { cooldown: 0 }));
    }

    pub fn cave_shrimp(ecs_world: &mut World, x: i32, y: i32) {
        let water_worm = Spawn::create_monster(
            ecs_world,
            "Cave shrimp".to_string(),
            Species {
                value: SpeciesEnum::Fish,
            },
            CombatStats {
                level: 1,
                current_stamina: 2,
                max_stamina: 2,
                base_armor: 1,
                unarmed_attack_dice: 1,
                current_toughness: 2,
                max_toughness: 2,
                current_dexterity: 4,
                max_dexterity: 4,
                speed: SLOW,
            },
            Edible {
                nutrition_dice_number: 2,
                nutrition_dice_size: 6,
            },
            Smellable {
                smell_log: "humid algae".to_string(),
                intensity: SmellIntensity::None,
            },
            ProduceSound {
                sound_log: "a drop of water".to_string(),
            },
            8.0,
            0.0,
            x,
            y,
        );

        let _ = ecs_world.insert(water_worm, (IsPrey {}, Aquatic {}, CanHide { cooldown: 0 }));
    }

    pub fn gremlin(ecs_world: &mut World, x: i32, y: i32) {
        let gremlin = Spawn::create_monster(
            ecs_world,
            "Gremlin".to_string(),
            Species {
                value: SpeciesEnum::Gremlin,
            },
            CombatStats {
                level: 5,
                current_stamina: 3,
                max_stamina: 3,
                base_armor: 0,
                unarmed_attack_dice: 2,
                current_toughness: 7,
                max_toughness: 7,
                current_dexterity: 14,
                max_dexterity: 14,
                speed: FAST,
            },
            Edible {
                nutrition_dice_number: 3,
                nutrition_dice_size: 6,
            },
            Smellable {
                smell_log: "cheap leather".to_string(),
                intensity: SmellIntensity::Faint,
            },
            ProduceSound {
                sound_log: "someone cackling".to_string(),
            },
            3.0,
            0.0,
            x,
            y,
        );

        let _ = ecs_world.insert(gremlin, (Smart {}, Small {}));
    }

    pub fn centipede(ecs_world: &mut World, x: i32, y: i32) {
        let centipede = Spawn::create_monster(
            ecs_world,
            "Giant centipede".to_string(),
            Species {
                value: SpeciesEnum::Bug,
            },
            CombatStats {
                level: 4,
                current_stamina: 3,
                max_stamina: 3,
                base_armor: 1,
                unarmed_attack_dice: 3,
                current_toughness: 6,
                max_toughness: 6,
                current_dexterity: 13,
                max_dexterity: 14,
                speed: NORMAL,
            },
            Edible {
                nutrition_dice_number: 2,
                nutrition_dice_size: 8,
            },
            Smellable {
                smell_log: "something off and dusty".to_string(),
                intensity: SmellIntensity::None,
            },
            ProduceSound {
                sound_log: "skittering of many legs".to_string(),
            },
            5.0,
            0.0,
            x,
            y,
        );

        let _ = ecs_world.insert(centipede, (Venomous {}, Small {}));
    }

    pub fn moleman(ecs_world: &mut World, x: i32, y: i32) {
        let moleman = Spawn::create_monster(
            ecs_world,
            "Mole-man".to_string(),
            Species {
                value: SpeciesEnum::Undergrounder,
            },
            CombatStats {
                level: 5,
                current_stamina: 4,
                max_stamina: 4,
                base_armor: 0,
                unarmed_attack_dice: 3,
                current_toughness: 10,
                max_toughness: 10,
                current_dexterity: 8,
                max_dexterity: 8,
                speed: SLOW,
            },
            Edible {
                nutrition_dice_number: 4,
                nutrition_dice_size: 6,
            },
            Smellable {
                smell_log: "coal drenched in vinegar".to_string(),
                intensity: SmellIntensity::Faint,
            },
            ProduceSound {
                sound_log: "someone mumbling".to_string(),
            },
            2.0,
            0.0,
            x,
            y,
        );

        if Roll::d6() > 3 {
            Spawn::moleman_chain(ecs_world, moleman);
        }

        let weapon_roll = Roll::d6();
        if weapon_roll > 5 {
            let pickaxe = Spawn::pickaxe(ecs_world, x, y);
            let _ = ecs_world.remove_one::<Position>(pickaxe);
            let _ = ecs_world.insert(
                pickaxe,
                (
                    InBackback {
                        owner: moleman,
                        assigned_char: 'b',
                    },
                    Equipped {
                        owner: moleman,
                        body_location: BodyLocation::RightHand,
                    },
                ),
            );
        } else if weapon_roll > 2 {
            let rockpick = Spawn::rockpick(ecs_world, x, y);
            let _ = ecs_world.remove_one::<Position>(rockpick);
            let _ = ecs_world.insert(
                rockpick,
                (
                    InBackback {
                        owner: moleman,
                        assigned_char: 'b',
                    },
                    Equipped {
                        owner: moleman,
                        body_location: BodyLocation::RightHand,
                    },
                ),
            );
        }

        let _ = ecs_world.insert(moleman, (Smart {},));
    }

    pub fn moleman_farmer(ecs_world: &mut World, x: i32, y: i32) {
        let moleman_farmer = Spawn::create_monster(
            ecs_world,
            "Mole-man farmer".to_string(),
            Species {
                value: SpeciesEnum::Undergrounder,
            },
            CombatStats {
                level: 6,
                current_stamina: 6,
                max_stamina: 6,
                base_armor: 0,
                unarmed_attack_dice: 3,
                current_toughness: 12,
                max_toughness: 12,
                current_dexterity: 8,
                max_dexterity: 8,
                speed: SLOW,
            },
            Edible {
                nutrition_dice_number: 4,
                nutrition_dice_size: 6,
            },
            Smellable {
                smell_log: "coal drenched in vinegar".to_string(),
                intensity: SmellIntensity::Faint,
            },
            ProduceSound {
                sound_log: "someone mumbling".to_string(),
            },
            2.0,
            0.0,
            x,
            y,
        );

        Spawn::moleman_chain(ecs_world, moleman_farmer);
        let crosswbow = Spawn::crowssbow(ecs_world, x, y);
        let _ = ecs_world.remove_one::<Position>(crosswbow);
        let _ = ecs_world.insert(
            crosswbow,
            (
                InBackback {
                    owner: moleman_farmer,
                    assigned_char: 'b',
                },
                Equipped {
                    owner: moleman_farmer,
                    body_location: BodyLocation::BothHands,
                },
            ),
        );

        // Give the farmer some ammo
        for _ in 0..3 {
            let crosswbow_ammo = Spawn::crossbow_ammo(ecs_world, x, y);
            let _ = ecs_world.remove_one::<Position>(crosswbow_ammo);
            let _ = ecs_world.insert(
                crosswbow_ammo,
                (InBackback {
                    owner: moleman_farmer,
                    assigned_char: 'c',
                },),
            );
        }
        // TODO Farmer cannot be hungry
        // let _ = ecs_world.remove_one::<Hunger>(moleman_farmer);
        let _ = ecs_world.insert(moleman_farmer, (Smart {}, Immobile {}));
    }

    /// Generic monster creation
    pub fn create_monster(
        ecs_world: &mut World,
        name: String,
        species: Species,
        combat_stats: CombatStats,
        edible: Edible,
        smells: Smellable,
        sounds: ProduceSound,
        tile_x: f32,
        tile_y: f32,
        x: i32,
        y: i32,
    ) -> Entity {
        let monster_entity = (
            Monster {},
            species,
            Position { x, y },
            Renderable {
                texture_name: TextureName::Creatures,
                texture_region: Rect {
                    x: tile_x * TILE_SIZE_F32,
                    y: tile_y * TILE_SIZE_F32,
                    w: TILE_SIZE_F32,
                    h: TILE_SIZE_F32,
                },
                z_index: 1,
            },
            Viewshed {
                visible_tiles: Vec::new(),
                range: BASE_MONSTER_VIEW_RADIUS,
                must_recalculate: true,
            },
            Named { name },
            BlocksTile {},
            combat_stats,
            SufferingDamage {
                damage_received: 0,
                toughness_damage_received: 0,
                damager: None,
            },
            Hunger {
                tick_counter: MAX_HUNGER_TICK_COUNTER,
                current_status: HungerStatus::Normal,
            },
            MyTurn {},
            smells,
            sounds,
            edible,
        );

        let monster_spawned = ecs_world.spawn(monster_entity);

        let _ = ecs_world.insert(
            monster_spawned,
            (
                ProduceCorpse {},
                Hates {
                    list: HashSet::new(),
                },
            ),
        );

        monster_spawned
    }

    pub fn giant_cockroach(ecs_world: &mut World, x: i32, y: i32) {
        let centipede = Spawn::create_monster(
            ecs_world,
            "Giant cockroach".to_string(),
            Species {
                value: SpeciesEnum::Bug,
            },
            CombatStats {
                level: 1,
                current_stamina: 2,
                max_stamina: 2,
                base_armor: 0,
                unarmed_attack_dice: 1,
                current_toughness: 5,
                max_toughness: 5,
                current_dexterity: 10,
                max_dexterity: 10,
                speed: NORMAL,
            },
            Edible {
                nutrition_dice_number: 4,
                nutrition_dice_size: 6,
            },
            Smellable {
                smell_log: "cupboard dust".to_string(),
                intensity: SmellIntensity::Faint,
            },
            ProduceSound {
                sound_log: "nervous skittering".to_string(),
            },
            6.0,
            0.0,
            x,
            y,
        );

        let _ = ecs_world.insert(centipede, (Small {}, IsPrey {}));
    }

    pub fn giant_slug(ecs_world: &mut World, x: i32, y: i32) {
        let slug = Spawn::create_monster(
            ecs_world,
            "Giant slug".to_string(),
            Species {
                value: SpeciesEnum::Gastropod,
            },
            CombatStats {
                level: 1,
                current_stamina: 2,
                max_stamina: 2,
                base_armor: 0,
                unarmed_attack_dice: 0,
                current_toughness: 3,
                max_toughness: 3,
                current_dexterity: 3,
                max_dexterity: 3,
                speed: SLOW,
            },
            Edible {
                nutrition_dice_number: 6,
                nutrition_dice_size: 6,
            },
            Smellable {
                smell_log: "foul saliva".to_string(),
                intensity: SmellIntensity::Faint,
            },
            ProduceSound {
                sound_log: "slow slushing".to_string(),
            },
            7.0,
            0.0,
            x,
            y,
        );

        // Drip a slime trail
        let _ = ecs_world.insert(
            slug,
            (
                Small {},
                IsPrey {},
                LeaveTrail {
                    of: DecalType::Slime,
                    trail_lifetime: SLUG_TRAIL_LIFETIME,
                },
            ),
        );
    }

    pub fn sulfuric_slug(ecs_world: &mut World, x: i32, y: i32) {
        let slug = Spawn::create_monster(
            ecs_world,
            "Sulfuric slug".to_string(),
            Species {
                value: SpeciesEnum::Gastropod,
            },
            CombatStats {
                level: 7,
                current_stamina: 5,
                max_stamina: 5,
                base_armor: 0,
                unarmed_attack_dice: 0,
                current_toughness: 3,
                max_toughness: 3,
                current_dexterity: 3,
                max_dexterity: 3,
                speed: SLOW,
            },
            Edible {
                nutrition_dice_number: 6,
                nutrition_dice_size: 6,
            },
            Smellable {
                smell_log: "nasty sulphuric fumes".to_string(),
                intensity: SmellIntensity::Strong,
            },
            ProduceSound {
                sound_log: "something sizzling".to_string(),
            },
            7.0,
            1.0,
            x,
            y,
        );

        // Drip a slime trail
        let _ = ecs_world.insert(
            slug,
            (
                Small {},
                IsPrey {},
                Deadly {},
                LeaveTrail {
                    of: DecalType::Acid,
                    trail_lifetime: SLUG_TRAIL_LIFETIME,
                },
            ),
        );
    }
}
