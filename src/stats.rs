use serde::{Deserialize, Serialize};

pub const FIELD_NAMES: [&str; 7] = [
    "마력(기본 수치, 기본)",
    "마력(기본 수치, 장비 아이템)",
    "마력(몽환의 결정)",
    "크리티컬 확률",
    "크리티컬 데미지",
    "재사용 대기시간 감소",
    "숙련도",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct BaseStats {
    #[serde(rename = "마력(기본 수치, 기본)")]
    pub magic_base: i32,
    #[serde(rename = "마력(기본 수치, 장비 아이템)")]
    pub magic_equip: i32,
    /// 현재 장착 중인 몽환의 결정에 의한 마력 합계 (최적화 시 기저값에서 제외)
    #[serde(rename = "마력(몽환의 결정)")]
    pub magic_crystal: i32,
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
    /// 몽환의 결정 영향을 제외한 기저 마력 수치
    pub fn effective_magic(&self) -> i32 {
        self.magic_base + self.magic_equip - self.magic_crystal
    }

    pub fn get(&self, idx: usize) -> i32 {
        match idx {
            0 => self.magic_base,
            1 => self.magic_equip,
            2 => self.magic_crystal,
            3 => self.crit_rate,
            4 => self.crit_damage,
            5 => self.cooldown_reduction,
            6 => self.mastery,
            _ => 0,
        }
    }

    pub fn set(&mut self, idx: usize, val: i32) {
        match idx {
            0 => self.magic_base = val,
            1 => self.magic_equip = val,
            2 => self.magic_crystal = val,
            3 => self.crit_rate = val,
            4 => self.crit_damage = val,
            5 => self.cooldown_reduction = val,
            6 => self.mastery = val,
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
        let s = BaseStats {
            magic_base: 1000,
            magic_equip: 500,
            magic_crystal: 100,
            crit_rate: 50,
            crit_damage: 150,
            cooldown_reduction: 3,
            mastery: 200,
        };
        let json = serde_json::to_string(&s).unwrap();
        let loaded: BaseStats = serde_json::from_str(&json).unwrap();
        assert_eq!(s, loaded);
    }

    #[test]
    fn effective_magic_excludes_crystal() {
        let s = BaseStats { magic_base: 1000, magic_equip: 500, magic_crystal: 100, ..Default::default() };
        assert_eq!(s.effective_magic(), 1400);
    }

    #[test]
    fn json_uses_korean_keys() {
        let s = BaseStats { magic_base: 1, ..Default::default() };
        let json = serde_json::to_string(&s).unwrap();
        assert!(json.contains("마력(기본 수치, 기본)"));
    }
}
