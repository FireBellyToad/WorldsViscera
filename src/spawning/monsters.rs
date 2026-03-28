use std::collections::{HashMap, HashSet, LinkedList};

use hecs::{Entity, World};
use macroquad::math::Rect;

use crate::{
    components::{
        actions::WantsToApply,
        combat::{CanHide, CombatStats, GazeAttack, GazeEffectEnum, SufferingDamage},
        common::{
            BlocksTile, Hates, Immobile, Immunity, ImmunityTypeEnum, MyTurn, Named, Position,
            ProduceCorpse, ProduceSound, Renderable, SmellIntensity, Smellable, Species,
            SpeciesEnum, SpellList, Viewshed,
        },
        health::{DiseaseType, Hunger},
        items::{BodyLocation, Deadly, Edible, Equipped, InBackback},
        monster::{
            Aquatic, DiseaseBearer, Grappler, LeaveTrail, Monster, Prey, SingleSnakeCreature,
            Small, Smart, SnakeBody, SnakeHead, Venomous,
        },
    },
    constants::{
        BASE_MONSTER_VIEW_RADIUS, BASE_VIEW_RADIUS, FAST, FILTH_TRAIL_LIFETIME,
        MAX_HUNGER_TICK_COUNTER, NORMAL, SLOW, SLUG_TRAIL_LIFETIME, TILE_SIZE_F32,
    },
    maps::zone::{DecalType, Zone},
    spawning::spawner::Spawn,
    systems::hunger_check::HungerStatus,
    utils::{assets::TextureName, roll::Roll},
};

type MonsterSpawnData = (
    Named,
    Species,
    CombatStats,
    i32,
    Edible,
    Smellable,
    ProduceSound,
    bool,
    Vec<ImmunityTypeEnum>,
    f32,
    f32,
    i32,
    i32,
);

impl Spawn {
    /// Generic monster creation
    pub fn create_monster(ecs_world: &mut World, monster_data: MonsterSpawnData) -> Entity {
        let (
            named,
            species,
            combat_stats,
            view_range,
            edible,
            smells,
            sounds,
            produce_corpse,
            immunities,
            tile_x,
            tile_y,
            x,
            y,
        ) = monster_data;

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
                range: view_range,
                must_recalculate: true,
            },
            named,
            BlocksTile {},
            combat_stats,
            SufferingDamage {
                damage_received: 0,
                toughness_damage_received: 0,
                dexterity_damage_received: 0,
                damager: None,
            },
            Hunger {
                tick_counter: Roll::dice(1, MAX_HUNGER_TICK_COUNTER),
                current_status: HungerStatus::Satiated,
            },
            MyTurn {},
            smells,
            sounds,
            edible,
            Hates {
                list: HashSet::new(),
            },
        );

        let monster_spawned = ecs_world.spawn(monster_entity);
        let mut immunity_comp = Immunity { to: HashMap::new() };
        for immunity in immunities {
            immunity_comp.to.insert(immunity, 1);
        }
        let _ = ecs_world.insert(monster_spawned, (immunity_comp,));

        // Not all monsters produce corpses on death
        if produce_corpse {
            let _ = ecs_world.insert_one(monster_spawned, ProduceCorpse {});
        }

        monster_spawned
    }

    pub fn deep_one(ecs_world: &mut World, x: i32, y: i32) {
        Spawn::create_monster(
            ecs_world,
            (
                Named {
                    name: "Deep One".to_string(),
                    attack_verb: Some("bites".to_string()),
                },
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
                BASE_MONSTER_VIEW_RADIUS,
                Edible {
                    nutrition_dice_number: 5,
                    nutrition_dice_size: 6,
                },
                Smellable {
                    smell_log: Some("dried human sweat".to_string()),
                    intensity: SmellIntensity::Faint,
                },
                ProduceSound {
                    sound_log: "someone weezing".to_string(),
                },
                true,
                vec![],
                1.0,
                0.0,
                x,
                y,
            ),
        );
    }

    pub fn abyssal_one(ecs_world: &mut World, x: i32, y: i32) {
        let abyssal_one = Spawn::create_monster(
            ecs_world,
            (
                Named {
                    name: "Abyssal One".to_string(),
                    attack_verb: Some("bites".to_string()),
                },
                Species {
                    value: SpeciesEnum::DeepSpawn,
                },
                CombatStats {
                    level: 10,
                    current_stamina: 15,
                    max_stamina: 15,
                    base_armor: 0,
                    unarmed_attack_dice: 6,
                    current_toughness: 13,
                    max_toughness: 13,
                    current_dexterity: 14,
                    max_dexterity: 14,
                    speed: NORMAL,
                },
                BASE_MONSTER_VIEW_RADIUS,
                Edible {
                    nutrition_dice_number: 4,
                    nutrition_dice_size: 6,
                },
                Smellable {
                    smell_log: Some("organic waste".to_string()),
                    intensity: SmellIntensity::Faint,
                },
                ProduceSound {
                    sound_log: "someone panting".to_string(),
                },
                true,
                vec![],
                1.0,
                1.0,
                x,
                y,
            ),
        );

        let _ = ecs_world.insert(
            abyssal_one,
            (
                Smart {},
                DiseaseBearer {
                    disease_type: DiseaseType::FleshRot,
                },
            ),
        );
    }

    pub fn calcificator(ecs_world: &mut World, x: i32, y: i32) {
        let calcificator = Spawn::create_monster(
            ecs_world,
            (
                Named {
                    name: "Calcificator".to_string(),
                    attack_verb: Some("scratches".to_string()),
                },
                Species {
                    value: SpeciesEnum::Undead,
                },
                CombatStats {
                    level: 3,
                    current_stamina: 4,
                    max_stamina: 4,
                    base_armor: 2,
                    unarmed_attack_dice: 2,
                    current_toughness: 12,
                    max_toughness: 12,
                    current_dexterity: 6,
                    max_dexterity: 6,
                    speed: SLOW,
                },
                BASE_MONSTER_VIEW_RADIUS,
                Edible {
                    nutrition_dice_number: 1,
                    nutrition_dice_size: 1,
                },
                Smellable {
                    smell_log: Some("chalk".to_string()),
                    intensity: SmellIntensity::Faint,
                },
                ProduceSound {
                    sound_log: "chalk scratching on floor".to_string(),
                },
                true,
                vec![
                    ImmunityTypeEnum::Disease(DiseaseType::Calcification),
                    ImmunityTypeEnum::Disease(DiseaseType::Fever),
                    ImmunityTypeEnum::Disease(DiseaseType::FleshRot),
                ],
                10.0,
                1.0,
                x,
                y,
            ),
        );

        let _ = ecs_world.insert(
            calcificator,
            (DiseaseBearer {
                disease_type: DiseaseType::Calcification,
            },),
        );
    }

    pub fn living_fossil(ecs_world: &mut World, x: i32, y: i32) {
        let living_fossil = Spawn::create_monster(
            ecs_world,
            (
                Named {
                    name: "Living Fossil".to_string(),
                    attack_verb: Some("scratches".to_string()),
                },
                Species {
                    value: SpeciesEnum::Undead,
                },
                CombatStats {
                    level: 7,
                    current_stamina: 8,
                    max_stamina: 8,
                    base_armor: 2,
                    unarmed_attack_dice: 2,
                    current_toughness: 15,
                    max_toughness: 15,
                    current_dexterity: 6,
                    max_dexterity: 6,
                    speed: NORMAL,
                },
                BASE_MONSTER_VIEW_RADIUS,
                Edible {
                    nutrition_dice_number: 1,
                    nutrition_dice_size: 1,
                },
                Smellable {
                    smell_log: Some("bone powder".to_string()),
                    intensity: SmellIntensity::Faint,
                },
                ProduceSound {
                    sound_log: "bone ticking on stone".to_string(),
                },
                true,
                vec![
                    ImmunityTypeEnum::Disease(DiseaseType::Calcification),
                    ImmunityTypeEnum::Disease(DiseaseType::Fever),
                    ImmunityTypeEnum::Disease(DiseaseType::FleshRot),
                ],
                10.0,
                2.0,
                x,
                y,
            ),
        );

        let _ = ecs_world.insert(
            living_fossil,
            (
                DiseaseBearer {
                    disease_type: DiseaseType::Calcification,
                },
                Grappler {},
            ),
        );
    }

    pub fn living_filth(ecs_world: &mut World, x: i32, y: i32) {
        let living_filth = Spawn::create_monster(
            ecs_world,
            (
                Named {
                    name: "Living filth".to_string(),
                    attack_verb: Some("burns".to_string()),
                },
                Species {
                    value: SpeciesEnum::Slime,
                },
                CombatStats {
                    level: 1,
                    current_stamina: 6,
                    max_stamina: 6,
                    base_armor: 1,
                    unarmed_attack_dice: 2,
                    current_toughness: 8,
                    max_toughness: 8,
                    current_dexterity: 5,
                    max_dexterity: 5,
                    speed: SLOW,
                },
                BASE_MONSTER_VIEW_RADIUS,
                Edible {
                    nutrition_dice_number: 1,
                    nutrition_dice_size: 1,
                },
                Smellable {
                    smell_log: Some("foul sewage".to_string()),
                    intensity: SmellIntensity::Strong,
                },
                ProduceSound {
                    sound_log: "slimy flop".to_string(),
                },
                false,
                vec![],
                11.0,
                0.0,
                x,
                y,
            ),
        );

        // TODO add damage resistance from weapon maybe?
        let _ = ecs_world.insert(
            living_filth,
            (
                LeaveTrail {
                    of: DecalType::Filth,
                    trail_lifetime: FILTH_TRAIL_LIFETIME,
                },
                DiseaseBearer {
                    disease_type: DiseaseType::Fever,
                },
            ),
        );
    }

    pub fn freshwater_viperfish(ecs_world: &mut World, x: i32, y: i32) {
        let freshwater_viperfish = Spawn::create_monster(
            ecs_world,
            (
                Named {
                    name: "Freshwater viperfish".to_string(),
                    attack_verb: Some("bites".to_string()),
                },
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
                BASE_MONSTER_VIEW_RADIUS,
                Edible {
                    nutrition_dice_number: 3,
                    nutrition_dice_size: 6,
                },
                Smellable {
                    smell_log: None,
                    intensity: SmellIntensity::None,
                },
                ProduceSound {
                    sound_log: "a splash in the water".to_string(),
                },
                true,
                vec![],
                4.0,
                0.0,
                x,
                y,
            ),
        );

        let _ = ecs_world.insert(freshwater_viperfish, (Aquatic {}, CanHide { cooldown: 0 }));
    }

    pub fn cave_shrimp(ecs_world: &mut World, x: i32, y: i32) {
        let cave_shrimp = Spawn::create_monster(
            ecs_world,
            (
                Named {
                    name: "Cave shrimp".to_string(),
                    attack_verb: Some("nibbles".to_string()),
                },
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
                BASE_MONSTER_VIEW_RADIUS,
                Edible {
                    nutrition_dice_number: 2,
                    nutrition_dice_size: 6,
                },
                Smellable {
                    smell_log: None,
                    intensity: SmellIntensity::None,
                },
                ProduceSound {
                    sound_log: "a drop of water".to_string(),
                },
                true,
                vec![],
                8.0,
                0.0,
                x,
                y,
            ),
        );

        let _ = ecs_world.insert(cave_shrimp, (Prey {}, Aquatic {}, CanHide { cooldown: 0 }));
    }

    pub fn cave_crab(ecs_world: &mut World, x: i32, y: i32) {
        let cave_crab = Spawn::create_monster(
            ecs_world,
            (
                Named {
                    name: "Cave crab".to_string(),
                    attack_verb: Some("pinches".to_string()),
                },
                Species {
                    value: SpeciesEnum::Fish,
                },
                CombatStats {
                    level: 3,
                    current_stamina: 5,
                    max_stamina: 5,
                    base_armor: 2,
                    unarmed_attack_dice: 2,
                    current_toughness: 8,
                    max_toughness: 8,
                    current_dexterity: 5,
                    max_dexterity: 5,
                    speed: SLOW,
                },
                BASE_MONSTER_VIEW_RADIUS,
                Edible {
                    nutrition_dice_number: 3,
                    nutrition_dice_size: 6,
                },
                Smellable {
                    smell_log: None,
                    intensity: SmellIntensity::None,
                },
                ProduceSound {
                    sound_log: "an splashing tickling".to_string(),
                },
                true,
                vec![],
                8.0,
                1.0,
                x,
                y,
            ),
        );

        let _ = ecs_world.insert(
            cave_crab,
            (Grappler {}, Aquatic {}, CanHide { cooldown: 0 }),
        );
    }

    pub fn pseudoscorpion(ecs_world: &mut World, x: i32, y: i32) {
        let pseudoscorpion = Spawn::create_monster(
            ecs_world,
            (
                Named {
                    name: "Pseudoscorpion".to_string(),
                    attack_verb: Some("pinches".to_string()),
                },
                Species {
                    value: SpeciesEnum::Bug,
                },
                CombatStats {
                    level: 2,
                    current_stamina: 5,
                    max_stamina: 5,
                    base_armor: 1,
                    unarmed_attack_dice: 2,
                    current_toughness: 8,
                    max_toughness: 8,
                    current_dexterity: 5,
                    max_dexterity: 5,
                    speed: NORMAL,
                },
                BASE_MONSTER_VIEW_RADIUS,
                Edible {
                    nutrition_dice_number: 3,
                    nutrition_dice_size: 6,
                },
                Smellable {
                    smell_log: Some("Munched bugs".to_string()),
                    intensity: SmellIntensity::Faint,
                },
                ProduceSound {
                    sound_log: "faint clicking".to_string(),
                },
                true,
                vec![],
                15.0,
                0.0,
                x,
                y,
            ),
        );

        let _ = ecs_world.insert(pseudoscorpion, (Grappler {},));
    }

    pub fn scorpion(ecs_world: &mut World, x: i32, y: i32) {
        let scorpion = Spawn::create_monster(
            ecs_world,
            (
                Named {
                    name: "Scorpion".to_string(),
                    attack_verb: Some("sting".to_string()),
                },
                Species {
                    value: SpeciesEnum::Bug,
                },
                CombatStats {
                    level: 5,
                    current_stamina: 7,
                    max_stamina: 7,
                    base_armor: 1,
                    unarmed_attack_dice: 6,
                    current_toughness: 9,
                    max_toughness: 9,
                    current_dexterity: 10,
                    max_dexterity: 10,
                    speed: NORMAL,
                },
                BASE_MONSTER_VIEW_RADIUS,
                Edible {
                    nutrition_dice_number: 3,
                    nutrition_dice_size: 6,
                },
                Smellable {
                    smell_log: Some("Munched bugs".to_string()),
                    intensity: SmellIntensity::Faint,
                },
                ProduceSound {
                    sound_log: "faint clicking".to_string(),
                },
                true,
                vec![],
                15.0,
                1.0,
                x,
                y,
            ),
        );

        let _ = ecs_world.insert(scorpion, (Grappler {}, Venomous {}));
    }

    pub fn gremlin(ecs_world: &mut World, x: i32, y: i32) {
        let gremlin = Spawn::create_monster(
            ecs_world,
            (
                Named {
                    name: "Gremlin".to_string(),
                    attack_verb: Some("scratches".to_string()),
                },
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
                BASE_MONSTER_VIEW_RADIUS,
                Edible {
                    nutrition_dice_number: 3,
                    nutrition_dice_size: 6,
                },
                Smellable {
                    smell_log: Some("cheap leather".to_string()),
                    intensity: SmellIntensity::Faint,
                },
                ProduceSound {
                    sound_log: "someone cackling".to_string(),
                },
                true,
                vec![],
                3.0,
                0.0,
                x,
                y,
            ),
        );

        let _ = ecs_world.insert(gremlin, (Smart {}, Small {}));

        if Roll::d6() > 5 {
            let wand = Spawn::wand(ecs_world, x, y);
            let _ = ecs_world.remove_one::<Position>(wand);
            let _ = ecs_world.insert_one(
                wand,
                InBackback {
                    owner: gremlin,
                    assigned_char: 'b',
                },
            );
        } else if Roll::d6() > 3 {
            Spawn::give_slingshot_and_ammo(ecs_world, gremlin);
        }
    }

    pub fn enthropic_gremlin(ecs_world: &mut World, x: i32, y: i32) {
        let enthropic_gremlin = Spawn::create_monster(
            ecs_world,
            (
                Named {
                    name: "Enthropic gremlin".to_string(),
                    attack_verb: Some("scratches".to_string()),
                },
                Species {
                    value: SpeciesEnum::Gremlin,
                },
                CombatStats {
                    level: 12,
                    current_stamina: 10,
                    max_stamina: 10,
                    base_armor: 0,
                    unarmed_attack_dice: 3,
                    current_toughness: 13,
                    max_toughness: 13,
                    current_dexterity: 16,
                    max_dexterity: 16,
                    speed: FAST,
                },
                BASE_MONSTER_VIEW_RADIUS,
                Edible {
                    nutrition_dice_number: 3,
                    nutrition_dice_size: 6,
                },
                Smellable {
                    smell_log: Some("cheap leather".to_string()),
                    intensity: SmellIntensity::Faint,
                },
                ProduceSound {
                    sound_log: "someone raving madly".to_string(),
                },
                true,
                vec![],
                3.0,
                1.0,
                x,
                y,
            ),
        );

        let _ = ecs_world.insert(
            enthropic_gremlin,
            (Smart {}, Small {}, CanHide { cooldown: 0 }),
        );

        for _ in 0..3 {
            let wand = Spawn::wand(ecs_world, x, y);
            let _ = ecs_world.remove_one::<Position>(wand);
            let _ = ecs_world.insert_one(
                wand,
                InBackback {
                    owner: enthropic_gremlin,
                    assigned_char: 'a',
                },
            );
        }
    }

    pub fn centipede(ecs_world: &mut World, x: i32, y: i32) {
        let centipede = Spawn::create_monster(
            ecs_world,
            (
                Named {
                    name: "Centipede".to_string(),
                    attack_verb: Some("bites".to_string()),
                },
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
                BASE_MONSTER_VIEW_RADIUS,
                Edible {
                    nutrition_dice_number: 2,
                    nutrition_dice_size: 8,
                },
                Smellable {
                    smell_log: None,
                    intensity: SmellIntensity::None,
                },
                ProduceSound {
                    sound_log: "skittering of many legs".to_string(),
                },
                true,
                vec![],
                5.0,
                0.0,
                x,
                y,
            ),
        );

        let _ = ecs_world.insert(centipede, (Venomous {}, Small {}));
    }

    pub fn giant_trogloraptor(ecs_world: &mut World, x: i32, y: i32) {
        let giant_trogloraptor = Spawn::create_monster(
            ecs_world,
            (
                Named {
                    name: "Trogloraptor".to_string(),
                    attack_verb: Some("bites".to_string()),
                },
                Species {
                    value: SpeciesEnum::Bug,
                },
                CombatStats {
                    level: 4,
                    current_stamina: 3,
                    max_stamina: 3,
                    base_armor: 0,
                    unarmed_attack_dice: 4,
                    current_toughness: 5,
                    max_toughness: 5,
                    current_dexterity: 14,
                    max_dexterity: 14,
                    speed: NORMAL,
                },
                BASE_MONSTER_VIEW_RADIUS,
                Edible {
                    nutrition_dice_number: 1,
                    nutrition_dice_size: 10,
                },
                Smellable {
                    smell_log: None,
                    intensity: SmellIntensity::None,
                },
                ProduceSound {
                    sound_log: "skittering from above".to_string(),
                },
                true,
                vec![],
                5.0,
                1.0,
                x,
                y,
            ),
        );

        let _ = ecs_world.insert(giant_trogloraptor, (CanHide { cooldown: 0 }, Grappler {}));
    }

    pub fn moleman(ecs_world: &mut World, x: i32, y: i32) {
        let moleman = Spawn::create_monster(
            ecs_world,
            (
                Named {
                    name: "Mole-man".to_string(),
                    attack_verb: Some("hits".to_string()),
                },
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
                BASE_MONSTER_VIEW_RADIUS,
                Edible {
                    nutrition_dice_number: 4,
                    nutrition_dice_size: 6,
                },
                Smellable {
                    smell_log: Some("coal drenched in vinegar".to_string()),
                    intensity: SmellIntensity::Faint,
                },
                ProduceSound {
                    sound_log: "someone mumbling".to_string(),
                },
                true,
                vec![],
                2.0,
                0.0,
                x,
                y,
            ),
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
                        body_location: BodyLocation::BothHands,
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

    pub fn moleman_farmer(ecs_world: &mut World, x: i32, y: i32) -> Entity {
        let moleman_farmer = Spawn::create_monster(
            ecs_world,
            (
                Named {
                    name: "Mole-man farmer".to_string(),
                    attack_verb: Some("hits".to_string()),
                },
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
                BASE_MONSTER_VIEW_RADIUS,
                Edible {
                    nutrition_dice_number: 4,
                    nutrition_dice_size: 6,
                },
                Smellable {
                    smell_log: Some("mushroom drenched in vinegar".to_string()),
                    intensity: SmellIntensity::Faint,
                },
                ProduceSound {
                    sound_log: "someone mumbling".to_string(),
                },
                true,
                vec![],
                2.0,
                1.0,
                x,
                y,
            ),
        );

        Spawn::moleman_chain(ecs_world, moleman_farmer);
        Spawn::give_crossbow_and_ammo(ecs_world, moleman_farmer);

        // Moleman farmer has a ration
        let ration = Spawn::ration(ecs_world, x, y);
        let _ = ecs_world.remove_one::<Position>(ration);
        let _ = ecs_world.insert_one(
            ration,
            InBackback {
                owner: moleman_farmer,
                assigned_char: 'd',
            },
        );

        let _ = ecs_world.insert(moleman_farmer, (Smart {}, Immobile {}));
        moleman_farmer
    }

    pub fn giant_cockroach(ecs_world: &mut World, x: i32, y: i32) {
        let centipede = Spawn::create_monster(
            ecs_world,
            (
                Named {
                    name: "Cockroach".to_string(),
                    attack_verb: Some("nibbles".to_string()),
                },
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
                BASE_MONSTER_VIEW_RADIUS,
                Edible {
                    nutrition_dice_number: 4,
                    nutrition_dice_size: 6,
                },
                Smellable {
                    smell_log: Some("cupboard dust".to_string()),
                    intensity: SmellIntensity::Faint,
                },
                ProduceSound {
                    sound_log: "nervous skittering".to_string(),
                },
                true,
                vec![],
                6.0,
                0.0,
                x,
                y,
            ),
        );

        let _ = ecs_world.insert(centipede, (Small {}, Prey {}));
    }

    pub fn bombardier_bettle(ecs_world: &mut World, x: i32, y: i32) {
        let bombardier_bettle = Spawn::create_monster(
            ecs_world,
            (
                Named {
                    name: "Bombardier beetle".to_string(),
                    attack_verb: Some("nibbles".to_string()),
                },
                Species {
                    value: SpeciesEnum::Bug,
                },
                CombatStats {
                    level: 3,
                    current_stamina: 5,
                    max_stamina: 5,
                    base_armor: 0,
                    unarmed_attack_dice: 2,
                    current_toughness: 7,
                    max_toughness: 7,
                    current_dexterity: 10,
                    max_dexterity: 10,
                    speed: NORMAL,
                },
                BASE_MONSTER_VIEW_RADIUS,
                Edible {
                    nutrition_dice_number: 4,
                    nutrition_dice_size: 6,
                },
                Smellable {
                    smell_log: Some("burnt cupboard dust".to_string()),
                    intensity: SmellIntensity::Faint,
                },
                ProduceSound {
                    sound_log: "faint pop".to_string(),
                },
                true,
                vec![],
                6.0,
                1.0,
                x,
                y,
            ),
        );

        // Bombardier bettle proyectile
        let burning_spray_spell = Spawn::burning_spray(ecs_world);
        let _ = ecs_world.insert_one(
            bombardier_bettle,
            SpellList {
                spells: vec![burning_spray_spell],
            },
        );

        let _ = ecs_world.insert(bombardier_bettle, (Small {}, Prey {}));
    }

    pub fn giant_slug(ecs_world: &mut World, x: i32, y: i32) {
        let slug = Spawn::create_monster(
            ecs_world,
            (
                Named {
                    name: "Slug".to_string(),
                    attack_verb: Some("nibbles".to_string()),
                },
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
                BASE_MONSTER_VIEW_RADIUS,
                Edible {
                    nutrition_dice_number: 6,
                    nutrition_dice_size: 6,
                },
                Smellable {
                    smell_log: Some("foul saliva".to_string()),
                    intensity: SmellIntensity::Faint,
                },
                ProduceSound {
                    sound_log: "slow slushing".to_string(),
                },
                true,
                vec![],
                7.0,
                0.0,
                x,
                y,
            ),
        );

        // Drip a slime trail
        let _ = ecs_world.insert(
            slug,
            (
                Small {},
                Prey {},
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
            (
                Named {
                    name: "Sulfuric slug".to_string(),
                    attack_verb: Some("nibbles".to_string()),
                },
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
                BASE_MONSTER_VIEW_RADIUS,
                Edible {
                    nutrition_dice_number: 6,
                    nutrition_dice_size: 6,
                },
                Smellable {
                    smell_log: Some("nasty sulphuric fumes".to_string()),
                    intensity: SmellIntensity::Strong,
                },
                ProduceSound {
                    sound_log: "something sizzling".to_string(),
                },
                true,
                vec![],
                7.0,
                1.0,
                x,
                y,
            ),
        );

        // Drip a slime trail
        let _ = ecs_world.insert(
            slug,
            (
                Small {},
                Prey {},
                Deadly {},
                LeaveTrail {
                    of: DecalType::Acid,
                    trail_lifetime: SLUG_TRAIL_LIFETIME,
                },
            ),
        );
    }

    pub fn refugee(ecs_world: &mut World, x: i32, y: i32) {
        let refugee = Spawn::create_monster(
            ecs_world,
            (
                Named {
                    name: "Human refugee".to_string(),
                    attack_verb: Some("hits".to_string()),
                },
                Species {
                    value: SpeciesEnum::Human,
                },
                CombatStats {
                    level: 2,
                    current_stamina: 6,
                    max_stamina: 6,
                    base_armor: 0,
                    unarmed_attack_dice: 3,
                    current_toughness: 10,
                    max_toughness: 10,
                    current_dexterity: 10,
                    max_dexterity: 10,
                    speed: NORMAL,
                },
                BASE_VIEW_RADIUS,
                Edible {
                    nutrition_dice_number: 5,
                    nutrition_dice_size: 4,
                },
                Smellable {
                    smell_log: Some("human sweat".to_string()),
                    intensity: SmellIntensity::Faint,
                },
                ProduceSound {
                    sound_log: "faint breathing".to_string(),
                },
                true,
                vec![],
                0.0,
                1.0,
                x,
                y,
            ),
        );

        let weapon_roll = Roll::d6();
        if weapon_roll > 5 {
            Spawn::give_slingshot_and_ammo(ecs_world, refugee);
        } else if weapon_roll > 2 {
            let lantern = Spawn::lantern(ecs_world, x, y);
            let _ = ecs_world.remove_one::<Position>(lantern);
            let _ = ecs_world.insert_one(
                lantern,
                InBackback {
                    owner: refugee,
                    assigned_char: 'b',
                },
            );
            let _ = ecs_world.insert_one(refugee, WantsToApply { item: lantern });
        }

        // Refugee has 1 to 3 rations
        for _ in 0..Roll::dice(1, 3) {
            let ration = Spawn::ration(ecs_world, x, y);
            let _ = ecs_world.remove_one::<Position>(ration);
            let _ = ecs_world.insert_one(
                ration,
                InBackback {
                    owner: refugee,
                    assigned_char: 'c',
                },
            );
        }
        let flask_of_water = Spawn::flask_of_water(ecs_world, x, y);
        let _ = ecs_world.remove_one::<Position>(flask_of_water);
        let _ = ecs_world.insert_one(
            flask_of_water,
            InBackback {
                owner: refugee,
                assigned_char: 'd',
            },
        );

        let _ = ecs_world.insert(refugee, (Smart {}, Prey {}));
    }

    pub fn stonedust_cultist(ecs_world: &mut World, x: i32, y: i32) -> Entity {
        let stonedust_cultist = Spawn::create_monster(
            ecs_world,
            (
                Named {
                    name: "Stonedust cultist".to_string(),
                    attack_verb: Some("hits".to_string()),
                },
                Species {
                    value: SpeciesEnum::Human,
                },
                CombatStats {
                    level: 6,
                    current_stamina: 8,
                    max_stamina: 8,
                    base_armor: 0,
                    unarmed_attack_dice: 3,
                    current_toughness: 11,
                    max_toughness: 11,
                    current_dexterity: 13,
                    max_dexterity: 13,
                    speed: NORMAL,
                },
                BASE_VIEW_RADIUS,
                Edible {
                    nutrition_dice_number: 5,
                    nutrition_dice_size: 4,
                },
                Smellable {
                    smell_log: Some("stone dust".to_string()),
                    intensity: SmellIntensity::Faint,
                },
                ProduceSound {
                    sound_log: "rythmic chanting".to_string(),
                },
                true,
                vec![],
                14.0,
                0.0,
                x,
                y,
            ),
        );

        // Stonedust cultist has dazing spell
        let daze_spell = Spawn::daze(ecs_world);
        let _ = ecs_world.insert_one(
            stonedust_cultist,
            SpellList {
                spells: vec![daze_spell],
            },
        );

        // Stonedust cultist has lantern
        let lantern = Spawn::lantern(ecs_world, x, y);
        let _ = ecs_world.remove_one::<Position>(lantern);
        let _ = ecs_world.insert_one(
            lantern,
            InBackback {
                owner: stonedust_cultist,
                assigned_char: 'a',
            },
        );

        // turn on lantern
        let _ = ecs_world.insert(
            stonedust_cultist,
            (WantsToApply { item: lantern }, Smart {}),
        );

        for _ in 0..Roll::dice(1, 2) {
            let ration = Spawn::ration(ecs_world, x, y);
            let _ = ecs_world.remove_one::<Position>(ration);
            let _ = ecs_world.insert_one(
                ration,
                InBackback {
                    owner: stonedust_cultist,
                    assigned_char: 'c',
                },
            );
        }
        let flask_of_water = Spawn::flask_of_water(ecs_world, x, y);
        let _ = ecs_world.remove_one::<Position>(flask_of_water);
        let _ = ecs_world.insert_one(
            flask_of_water,
            InBackback {
                owner: stonedust_cultist,
                assigned_char: 'd',
            },
        );

        stonedust_cultist
    }

    pub fn stonedust_acolyte(ecs_world: &mut World, x: i32, y: i32) -> Entity {
        let stonedust_acolyte = Spawn::create_monster(
            ecs_world,
            (
                Named {
                    name: "Stonedust acolyte".to_string(),
                    attack_verb: Some("hits".to_string()),
                },
                Species {
                    value: SpeciesEnum::Human,
                },
                CombatStats {
                    level: 8,
                    current_stamina: 10,
                    max_stamina: 10,
                    base_armor: 0,
                    unarmed_attack_dice: 3,
                    current_toughness: 13,
                    max_toughness: 13,
                    current_dexterity: 13,
                    max_dexterity: 13,
                    speed: NORMAL,
                },
                BASE_VIEW_RADIUS,
                Edible {
                    nutrition_dice_number: 5,
                    nutrition_dice_size: 4,
                },
                Smellable {
                    smell_log: Some("stone dust".to_string()),
                    intensity: SmellIntensity::Faint,
                },
                ProduceSound {
                    sound_log: "rythmic preaching".to_string(),
                },
                true,
                vec![],
                14.0,
                1.0,
                x,
                y,
            ),
        );

        // Stonedust cultist has dazing spell
        let daze_spell = Spawn::daze(ecs_world);
        let stone_fell = Spawn::stone_fell(ecs_world);
        let _ = ecs_world.insert_one(
            stonedust_acolyte,
            SpellList {
                spells: vec![daze_spell, stone_fell],
            },
        );

        // Stonedust cultist has lantern
        let lantern = Spawn::lantern(ecs_world, x, y);
        let _ = ecs_world.remove_one::<Position>(lantern);
        let _ = ecs_world.insert_one(
            lantern,
            InBackback {
                owner: stonedust_acolyte,
                assigned_char: 'a',
            },
        );

        for _ in 0..Roll::dice(1, 2) {
            let ration = Spawn::ration(ecs_world, x, y);
            let _ = ecs_world.remove_one::<Position>(ration);
            let _ = ecs_world.insert_one(
                ration,
                InBackback {
                    owner: stonedust_acolyte,
                    assigned_char: 'c',
                },
            );
        }
        let flask_of_water = Spawn::flask_of_water(ecs_world, x, y);
        let _ = ecs_world.remove_one::<Position>(flask_of_water);
        let _ = ecs_world.insert_one(
            flask_of_water,
            InBackback {
                owner: stonedust_acolyte,
                assigned_char: 'd',
            },
        );
        // turn on lantern
        let _ = ecs_world.insert(
            stonedust_acolyte,
            (WantsToApply { item: lantern }, Smart {}),
        );

        stonedust_acolyte
    }

    pub fn living_dead(ecs_world: &mut World, x: i32, y: i32) {
        let _ = Spawn::create_monster(
            ecs_world,
            (
                Named {
                    name: "Living dead".to_string(),
                    attack_verb: Some("bites".to_string()),
                },
                Species {
                    value: SpeciesEnum::Undead,
                },
                CombatStats {
                    level: 1,
                    current_stamina: 6,
                    max_stamina: 6,
                    base_armor: 0,
                    unarmed_attack_dice: 0,
                    current_toughness: 10,
                    max_toughness: 10,
                    current_dexterity: 3,
                    max_dexterity: 3,
                    speed: SLOW,
                },
                BASE_MONSTER_VIEW_RADIUS,
                Edible {
                    nutrition_dice_number: 1,
                    nutrition_dice_size: 1,
                },
                Smellable {
                    smell_log: Some("decomposition".to_string()),
                    intensity: SmellIntensity::Faint,
                },
                ProduceSound {
                    sound_log: "dragging feet".to_string(),
                },
                true,
                vec![
                    ImmunityTypeEnum::Disease(DiseaseType::Calcification),
                    ImmunityTypeEnum::Disease(DiseaseType::Fever),
                    ImmunityTypeEnum::Disease(DiseaseType::FleshRot),
                ],
                10.0,
                0.0,
                x,
                y,
            ),
        );
    }

    pub fn darkling(ecs_world: &mut World, x: i32, y: i32) {
        let darkling = Spawn::create_monster(
            ecs_world,
            (
                Named {
                    name: "Darkling".to_string(),
                    attack_verb: Some("slashes".to_string()),
                },
                Species {
                    value: SpeciesEnum::DeepSpawn,
                },
                CombatStats {
                    level: 12,
                    current_stamina: 10,
                    max_stamina: 10,
                    base_armor: 1,
                    unarmed_attack_dice: 8,
                    current_toughness: 15,
                    max_toughness: 15,
                    current_dexterity: 15,
                    max_dexterity: 15,
                    speed: NORMAL,
                },
                BASE_MONSTER_VIEW_RADIUS,
                Edible {
                    nutrition_dice_number: 1,
                    nutrition_dice_size: 12,
                },
                Smellable {
                    smell_log: None,
                    intensity: SmellIntensity::None,
                },
                ProduceSound {
                    sound_log: "someone whispering".to_string(),
                },
                true,
                vec![ImmunityTypeEnum::Blindness],
                12.0,
                0.0,
                x,
                y,
            ),
        );

        let _ = ecs_world.insert(
            darkling,
            (GazeAttack {
                effect: GazeEffectEnum::Blindness,
            },),
        );
    }

    pub fn colossal_worm(ecs_world: &mut World, x: i32, y: i32, zone: &Zone) {
        let colossal_worm = Spawn::create_monster(
            ecs_world,
            (
                Named {
                    name: "Colossal Worm".to_string(),
                    attack_verb: Some("munches".to_string()),
                },
                Species {
                    value: SpeciesEnum::Gastropod,
                },
                CombatStats {
                    level: 15,
                    current_stamina: 20,
                    max_stamina: 20,
                    base_armor: 2,
                    unarmed_attack_dice: 12,
                    current_toughness: 18,
                    max_toughness: 18,
                    current_dexterity: 10,
                    max_dexterity: 10,
                    speed: NORMAL,
                },
                BASE_MONSTER_VIEW_RADIUS,
                Edible {
                    nutrition_dice_number: 6,
                    nutrition_dice_size: 20,
                },
                Smellable {
                    smell_log: Some("stomach acid and stone dust".to_string()),
                    intensity: SmellIntensity::Strong,
                },
                ProduceSound {
                    sound_log: "cave rumbling".to_string(),
                },
                true,
                vec![],
                13.0,
                0.0,
                x,
                y,
            ),
        );

        //Generate body
        let mut body = LinkedList::new();
        let worm_size = Roll::dice(1, 3) + 1;
        let mut free_x = x;
        let mut free_y = y;
        for it in 0..worm_size {
            // Search for free space. If worm is too big, it cannot fit and we despawn it
            let adjacent_tiles = zone.get_adjacent_passable_tiles(free_x, free_y, true, false);
            if adjacent_tiles.is_empty() {
                //Cannot place worm body here, despawn and exit
                println!("Cannot fit colossal worm!");
                let _ = ecs_world.despawn(colossal_worm);
                return;
            } else {
                free_x = adjacent_tiles[0].0;
                free_y = adjacent_tiles[0].1;
            }

            let tile_y = if it == worm_size - 1 { 2.0 } else { 1.0 };

            let body_part = ecs_world.spawn((
                Monster {},
                Named {
                    name: "Colossal Worm's body".to_string(),
                    attack_verb: Some("".to_string()),
                },
                Renderable {
                    texture_name: TextureName::Creatures,
                    texture_region: Rect {
                        x: 13.0 * TILE_SIZE_F32,
                        y: tile_y * TILE_SIZE_F32,
                        w: TILE_SIZE_F32,
                        h: TILE_SIZE_F32,
                    },
                    z_index: 1,
                },
                Species {
                    value: SpeciesEnum::Gastropod,
                },
                SnakeBody {
                    head: colossal_worm,
                },
                Position {
                    x: free_x,
                    y: free_y,
                },
                Smellable {
                    smell_log: Some("stomach acid and stone dust".to_string()),
                    intensity: SmellIntensity::Strong,
                },
                SufferingDamage {
                    damage_received: 0,
                    toughness_damage_received: 0,
                    dexterity_damage_received: 0,
                    damager: None,
                },
                BlocksTile {},
                SingleSnakeCreature {},
            ));
            body.push_back(body_part);
        }

        //Join head and body
        let _ = ecs_world.insert(
            colossal_worm,
            (SnakeHead { body }, SingleSnakeCreature {}, Grappler {}),
        );
    }
}
