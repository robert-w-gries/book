#[derive(Copy, Clone)]
pub enum MobType {
    Walk,
    Swim,
    Fly,
}

impl MobType {
    pub fn to_str(self) -> String {
        match self {
            MobType::Walk => "walk".to_string(),
            MobType::Swim => "swim".to_string(),
            MobType::Fly => "fly".to_string(),
        }
    }
}

pub const MOB_TYPES: [MobType; 3] = [MobType::Walk, MobType::Swim, MobType::Fly];
