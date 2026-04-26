use serde::de::{self, MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionKind {
    Magic,
    MagicPercent,
    Mastery,
    CritRate,
    CritDamage,
    CooldownReduction,
}

impl OptionKind {
    pub const ALL: [OptionKind; 6] = [
        OptionKind::Magic,
        OptionKind::MagicPercent,
        OptionKind::CritRate,
        OptionKind::CritDamage,
        OptionKind::CooldownReduction,
        OptionKind::Mastery,
    ];

    pub fn display_name(self) -> &'static str {
        match self {
            OptionKind::Magic => "마력",
            OptionKind::MagicPercent => "마력%",
            OptionKind::Mastery => "숙련도",
            OptionKind::CritRate => "크리티컬 확률",
            OptionKind::CritDamage => "크리티컬 데미지",
            OptionKind::CooldownReduction => "재사용 대기시간 감소",
        }
    }

    pub fn index_in_all(self) -> usize {
        Self::ALL.iter().position(|&k| k == self).unwrap_or(0)
    }

    pub fn from_korean(s: &str) -> Option<Self> {
        match s {
            "마력" => Some(OptionKind::Magic),
            "마력%" => Some(OptionKind::MagicPercent),
            "숙련도" => Some(OptionKind::Mastery),
            "크리티컬 확률" => Some(OptionKind::CritRate),
            "크리티컬 데미지" => Some(OptionKind::CritDamage),
            "재사용 대기시간 감소" => Some(OptionKind::CooldownReduction),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemOption {
    pub kind: OptionKind,
    pub value: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub options: [ItemOption; 2],
}

impl Serialize for Item {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry(self.options[0].kind.display_name(), &self.options[0].value)?;
        map.serialize_entry(self.options[1].kind.display_name(), &self.options[1].value)?;
        map.end()
    }
}

impl<'de> Deserialize<'de> for Item {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ItemVisitor;

        impl<'de> Visitor<'de> for ItemVisitor {
            type Value = Item;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "a map with exactly 2 Korean option keys")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Item, A::Error> {
                let mut opts: Vec<ItemOption> = Vec::new();
                while let Some(key) = map.next_key::<String>()? {
                    let value: i32 = map.next_value()?;
                    let kind = OptionKind::from_korean(&key).ok_or_else(|| {
                        de::Error::custom(format!("unknown option kind: {key}"))
                    })?;
                    opts.push(ItemOption { kind, value });
                }
                if opts.len() != 2 {
                    return Err(de::Error::custom(format!(
                        "expected 2 options, got {}",
                        opts.len()
                    )));
                }
                let mut it = opts.into_iter();
                Ok(Item {
                    options: [it.next().unwrap(), it.next().unwrap()],
                })
            }
        }

        deserializer.deserialize_map(ItemVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_option_kinds_roundtrip() {
        for kind in OptionKind::ALL {
            let name = kind.display_name();
            assert_eq!(
                OptionKind::from_korean(name),
                Some(kind),
                "roundtrip failed for {name}"
            );
        }
    }

    #[test]
    fn unknown_korean_returns_none() {
        assert_eq!(OptionKind::from_korean("없는옵션"), None);
    }

    #[test]
    fn item_serde_roundtrip() {
        let item = Item {
            options: [
                ItemOption { kind: OptionKind::Magic, value: 10 },
                ItemOption { kind: OptionKind::CritRate, value: 7 },
            ],
        };
        let json = serde_json::to_string(&item).unwrap();
        let loaded: Item = serde_json::from_str(&json).unwrap();
        assert_eq!(item, loaded);
    }

    #[test]
    fn item_json_uses_korean_keys() {
        let item = Item {
            options: [
                ItemOption { kind: OptionKind::CritDamage, value: 8 },
                ItemOption { kind: OptionKind::CritRate, value: 13 },
            ],
        };
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("크리티컬 데미지"));
        assert!(json.contains("크리티컬 확률"));
    }
}
