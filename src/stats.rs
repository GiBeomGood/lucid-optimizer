use serde::{Deserialize, Serialize};

pub const FIELD_NAMES: [&str; 5] = [
    "마력(기본)",
    "크리티컬 확률",
    "크리티컬 데미지",
    "재사용 대기시간 감소",
    "숙련도",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct BaseStats {
    #[serde(rename = "마력(기본)")]
    pub magic: i32,
    #[serde(rename = "크리티컬 확률")]
    pub crit_rate: i32,
    #[serde(rename = "크리티컬 데미지")]
    pub crit_damage: i32,
    #[serde(rename = "재사용 대기시간 감소")]
    pub cooldown_reduction: i32,
    #[serde(rename = "숙련도")]
    pub mastery: i32,
}

impl BaseStats {
    pub fn get(&self, idx: usize) -> i32 {
        match idx {
            0 => self.magic,
            1 => self.crit_rate,
            2 => self.crit_damage,
            3 => self.cooldown_reduction,
            4 => self.mastery,
            _ => 0,
        }
    }

    pub fn set(&mut self, idx: usize, val: i32) {
        match idx {
            0 => self.magic = val,
            1 => self.crit_rate = val,
            2 => self.crit_damage = val,
            3 => self.cooldown_reduction = val,
            4 => self.mastery = val,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_set_roundtrip() {
        let mut s = BaseStats::default();
        for i in 0..5 {
            s.set(i, (i as i32 + 1) * 10);
        }
        for i in 0..5 {
            assert_eq!(s.get(i), (i as i32 + 1) * 10);
        }
    }

    #[test]
    fn serde_roundtrip() {
        let s = BaseStats { magic: 100, crit_rate: 50, crit_damage: 150, cooldown_reduction: 3, mastery: 200 };
        let json = serde_json::to_string(&s).unwrap();
        let loaded: BaseStats = serde_json::from_str(&json).unwrap();
        assert_eq!(s, loaded);
    }

    #[test]
    fn json_uses_korean_keys() {
        let s = BaseStats { magic: 1, ..Default::default() };
        let json = serde_json::to_string(&s).unwrap();
        assert!(json.contains("마력(기본)"));
    }
}
