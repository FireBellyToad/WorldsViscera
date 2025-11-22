use hecs::Entity;

pub struct CombatStats {
    pub level: u32,
    pub current_stamina: i32,
    pub max_stamina: i32,
    pub current_toughness: i32,
    pub max_toughness: i32,
    pub current_dexterity: i32,
    pub max_dexterity: i32,
    pub base_armor: i32,
    pub unarmed_attack_dice: i32,
    pub speed: i32,
}

pub struct SufferingDamage {
    pub damage_received: i32,
    pub toughness_damage_received: i32,
    pub damager: Option<Entity>,
}

pub struct WantsToMelee {
    pub target: Entity,
}

pub struct WantsToZap {
    pub target: (i32, i32),
}

pub struct InflictsDamage {
    pub number_of_dices: i32,
    pub dice_size: i32,
}

pub struct CanHide {
    pub cooldown: i32,
}

pub struct IsHidden {
    pub hidden_counter: i32,
}

pub struct WantsToShoot {
    pub weapon: Entity,
}

pub struct WantsToDig {
    pub target: Entity,
    pub tool: Entity,
}
