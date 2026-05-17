use std::sync::mpsc;

use crate::item::{Item, OptionKind};
use crate::stats::BaseStats;

const COOL_REDUCE_STRENGTH: [f64; 10] =
    [0.0, 1.94, 4.19, 6.83, 9.94, 13.7, 15.78, 18.11, 20.71, 23.64];

#[derive(Debug, Clone)]
pub struct ComboResult {
    pub indices: Vec<usize>,
    pub strength: f64,
}

pub enum OptimizeMsg {
    Status(String),
    Progress(usize, usize),
    Done(Vec<ComboResult>),
    Error(String),
}

pub fn calculate_strength(base: &BaseStats, combo_items: &[&Item]) -> f64 {
    let mut magic = base.effective_magic();
    let mut magic_percent = 0i32;
    let mut crit_damage = base.crit_damage;
    let mut cool_reduce = base.cooldown_reduction;
    let mut mastery = base.mastery;

    for item in combo_items {
        for opt in &item.options {
            match opt.kind {
                OptionKind::Magic => magic += opt.value,
                OptionKind::MagicPercent => magic_percent += opt.value,
                OptionKind::CritDamage => crit_damage += opt.value,
                OptionKind::CooldownReduction => cool_reduce += opt.value,
                OptionKind::Mastery => mastery += opt.value,
                OptionKind::CritRate => {}
            }
        }
    }

    let cool_idx = cool_reduce.clamp(0, 9) as usize;
    (magic as f64)
        * (1.0 + magic_percent as f64 / 100.0)
        * (crit_damage as f64 / 100.0)
        * (1.0 + mastery.min(100) as f64 / 600.0)
        * (1.0 + COOL_REDUCE_STRENGTH[cool_idx] / 100.0)
}

fn total_crit_rate(base: &BaseStats, combo_items: &[&Item]) -> i32 {
    let mut cr = base.crit_rate;
    for item in combo_items {
        for opt in &item.options {
            if opt.kind == OptionKind::CritRate {
                cr += opt.value;
            }
        }
    }
    cr
}

fn combinations(m: usize, n: usize) -> Vec<Vec<usize>> {
    let mut result = Vec::new();
    let mut combo = Vec::with_capacity(n);
    gen_combos(0, m, n, &mut combo, &mut result);
    result
}

fn gen_combos(
    start: usize,
    m: usize,
    n: usize,
    combo: &mut Vec<usize>,
    result: &mut Vec<Vec<usize>>,
) {
    if combo.len() == n {
        result.push(combo.clone());
        return;
    }
    let remaining = n - combo.len();
    if start + remaining > m {
        return;
    }
    for i in start..=(m - remaining) {
        combo.push(i);
        gen_combos(i + 1, m, n, combo, result);
        combo.pop();
    }
}

pub fn run_optimize(
    items: Vec<Item>,
    base: BaseStats,
    n: usize,
    k: usize,
    tx: mpsc::Sender<OptimizeMsg>,
) {
    if n > items.len() {
        let _ = tx.send(OptimizeMsg::Error(format!(
            "사용할 결정 수({n})가 보유한 결정 수({})보다 많습니다",
            items.len()
        )));
        return;
    }

    let _ = tx.send(OptimizeMsg::Status("조합 생성 중...".to_string()));
    let combos = combinations(items.len(), n);
    let total = combos.len();
    let _ = tx.send(OptimizeMsg::Status(format!("조합 {total}개 생성됨, 전투력 계산 중...")));
    let _ = tx.send(OptimizeMsg::Progress(0, total));

    let mut results: Vec<ComboResult> = Vec::new();
    for (i, indices) in combos.iter().enumerate() {
        if i % 500 == 0 || i + 1 == total {
            let _ = tx.send(OptimizeMsg::Progress(i + 1, total));
        }
        let combo_items: Vec<&Item> = indices.iter().map(|&idx| &items[idx]).collect();
        if total_crit_rate(&base, &combo_items) < 100 {
            continue;
        }
        let strength = calculate_strength(&base, &combo_items);
        results.push(ComboResult { indices: indices.clone(), strength });
    }

    let valid_count = results.len();
    let _ = tx.send(OptimizeMsg::Status(format!(
        "유효한 조합 {valid_count}개에서 top-{k} 탐색 중..."
    )));

    results.sort_by(|a, b| {
        b.strength.partial_cmp(&a.strength).unwrap_or(std::cmp::Ordering::Equal)
    });

    let top_k: Vec<ComboResult> = if k == 0 || k >= results.len() {
        results
    } else {
        let cutoff = results[k - 1].strength;
        results.into_iter().take_while(|r| r.strength >= cutoff).collect()
    };

    let found = top_k.len();
    let _ = tx.send(OptimizeMsg::Status(format!("완료! top-{k} 조합 {found}개 발견")));
    let _ = tx.send(OptimizeMsg::Done(top_k));
}
