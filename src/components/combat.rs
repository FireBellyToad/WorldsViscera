
pub struct CombatStats {
    pub current_stamina: i32,
    pub max_stamina: i32,
    pub current_toughness: i32,
    pub max_toughness: i32,
    pub base_armor: i32,
    pub unarmed_attack_dice: i32,
}

pub struct Damageable {
    pub damage_received:i32,
}
