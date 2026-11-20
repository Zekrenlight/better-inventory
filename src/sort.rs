//! Sorting as plain left clicks.
//!
//! The game half describes a run of slots as cells: which kind of stack each holds (`kind`, the
//! same number for stacks that can join) and how many. [`plan`] works out where everything should
//! end up and the left clicks that put it there, checking the result on its own copy of the slots
//! first: if the copy doesn't come out sorted, no click is planned at all.

/// One slot's stack.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub kind: u32,
    pub count: u32,
}

/// A copy of the slots and the stack on the cursor, changed by left clicks as the game's menus
/// change them.
struct Slots<'a> {
    cells: Vec<Option<Cell>>,
    cursor: Option<Cell>,
    max: &'a dyn Fn(u32) -> u32,
    clicks: Vec<usize>,
}

impl Slots<'_> {
    fn click(&mut self, i: usize) {
        self.clicks.push(i);
        let slot = self.cells[i];
        match (self.cursor, slot) {
            (None, None) => {}
            (None, Some(s)) => {
                self.cursor = Some(s);
                self.cells[i] = None;
            }
            (Some(c), None) => {
                self.cells[i] = Some(c);
                self.cursor = None;
            }
            (Some(c), Some(s)) if c.kind == s.kind => {
                let room = (self.max)(s.kind).saturating_sub(s.count);
                let moved = room.min(c.count);
                self.cells[i] = Some(Cell {
                    kind: s.kind,
                    count: s.count + moved,
                });
                let left = c.count - moved;
                self.cursor = (left > 0).then_some(Cell {
                    kind: c.kind,
                    count: left,
                });
            }
            (Some(c), Some(s)) => {
                self.cells[i] = Some(c);
                self.cursor = Some(s);
            }
        }
    }
}

/// The left clicks (indices into `cells`) that join part stacks and put the stacks in the order
/// `rank` gives each kind (lowest first), full stacks before a kind's last part stack, empty
/// slots at the end. `None` when it can't be done with plain clicks (a stack over its kind's
/// limit) or the check fails; an empty list when the slots are already sorted.
pub fn plan(cells: &[Option<Cell>], max: &dyn Fn(u32) -> u32, rank: &dyn Fn(u32) -> u64) -> Option<Vec<usize>> {
    if cells
        .iter()
        .flatten()
        .any(|c| c.count == 0 || c.count > max(c.kind).max(1))
    {
        return None;
    }
    let mut s = Slots {
        cells: cells.to_vec(),
        cursor: None,
        max,
        clicks: Vec::new(),
    };
    // 1. Part stacks of a kind are joined, the later ones poured into the earlier ones.
    loop {
        let mut joined = false;
        let n = s.cells.len();
        'outer: for i in 0..n {
            let Some(a) = s.cells[i] else { continue };
            if a.count >= max(a.kind) {
                continue;
            }
            for j in (i + 1..n).rev() {
                let Some(b) = s.cells[j] else { continue };
                if b.kind == a.kind && b.count < max(b.kind) {
                    s.click(j);
                    s.click(i);
                    if s.cursor.is_some() {
                        s.click(j);
                    }
                    joined = true;
                    break 'outer;
                }
            }
        }
        if !joined {
            break;
        }
    }
    // 2. Where each stack goes.
    let mut stacks: Vec<Cell> = s.cells.iter().flatten().copied().collect();
    stacks.sort_by(|a, b| {
        rank(a.kind)
            .cmp(&rank(b.kind))
            .then(a.kind.cmp(&b.kind))
            .then(b.count.cmp(&a.count))
    });
    let mut target: Vec<Option<Cell>> = stacks.into_iter().map(Some).collect();
    target.resize(s.cells.len(), None);
    // 3. Each place in turn gets its stack from a later slot.
    for t in 0..target.len() {
        if s.cells[t] == target[t] {
            continue;
        }
        let Some(want) = target[t] else {
            // Only empty places are left, and everything before is in place.
            break;
        };
        let from = (t + 1..s.cells.len()).find(|&j| s.cells[j] == Some(want) && target[j] != Some(want))
            .or_else(|| (t + 1..s.cells.len()).find(|&j| s.cells[j] == Some(want)))?;
        match s.cells[t] {
            None => {
                s.click(from);
                s.click(t);
            }
            Some(here) if here.kind == want.kind && here.count > want.count => {
                // The bigger stack is picked up first, so the two trade places.
                s.click(t);
                s.click(from);
                s.click(t);
            }
            Some(_) => {
                s.click(from);
                s.click(t);
                s.click(from);
            }
        }
        if s.cursor.is_some() {
            return None;
        }
    }
    (s.cursor.is_none() && s.cells == target).then_some(s.clicks)
}

#[cfg(test)]
mod check {
    use super::*;

    fn run(cells: &[Option<Cell>], clicks: &[usize], max: &dyn Fn(u32) -> u32) -> Vec<Option<Cell>> {
        let mut s = Slots {
            cells: cells.to_vec(),
            cursor: None,
            max,
            clicks: Vec::new(),
        };
        for &c in clicks {
            s.click(c);
        }
        assert!(s.cursor.is_none());
        s.cells
    }
    fn c(kind: u32, count: u32) -> Option<Cell> {
        Some(Cell { kind, count })
    }

    #[test]
    fn joins_and_orders() {
        let max = |k: u32| if k == 9 { 1 } else { 64 };
        let rank = |k: u32| u64::from(k);
        let cells = vec![c(3, 10), None, c(1, 64), c(3, 60), c(9, 1), c(1, 5), None, c(2, 1), c(9, 1)];
        let clicks = plan(&cells, &max, &rank).unwrap();
        let out = run(&cells, &clicks, &max);
        assert_eq!(out, vec![c(1, 64), c(1, 5), c(2, 1), c(3, 64), c(3, 6), c(9, 1), c(9, 1), None, None]);
    }

    #[test]
    fn sorted_needs_nothing() {
        let max = |_: u32| 64;
        let rank = |k: u32| u64::from(k);
        let cells = vec![c(1, 64), c(2, 3), None];
        assert_eq!(plan(&cells, &max, &rank), Some(Vec::new()));
    }

    #[test]
    fn full_and_part_trade_places() {
        let max = |_: u32| 64;
        let rank = |k: u32| u64::from(k);
        let cells = vec![c(1, 20), c(2, 1), c(1, 64)];
        let clicks = plan(&cells, &max, &rank).unwrap();
        assert_eq!(run(&cells, &clicks, &max), vec![c(1, 64), c(1, 20), c(2, 1)]);
    }

    #[test]
    fn many_random_layouts() {
        let mut seed = 0x1234_5678_u64;
        let mut next = move || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        };
        let max = |k: u32| match k % 3 {
            0 => 1,
            1 => 16,
            _ => 64,
        };
        let rank = |k: u32| u64::from(k * 7 % 11);
        for _ in 0..500 {
            let cells: Vec<Option<Cell>> = (0..27)
                .map(|_| {
                    let r = next();
                    if r % 4 == 0 {
                        None
                    } else {
                        let kind = (r >> 8) as u32 % 6;
                        let count = 1 + (r >> 20) as u32 % max(kind);
                        Some(Cell { kind, count })
                    }
                })
                .collect();
            let clicks = plan(&cells, &max, &rank).expect("a plan");
            let out = run(&cells, &clicks, &max);
            let total = |v: &[Option<Cell>], k: u32| v.iter().flatten().filter(|c| c.kind == k).map(|c| c.count).sum::<u32>();
            for k in 0..6 {
                assert_eq!(total(&cells, k), total(&out, k));
            }
            // Sorted: ranks never go down, no gaps before the last stack.
            let ranks: Vec<u64> = out.iter().flatten().map(|c| rank(c.kind)).collect();
            assert!(ranks.windows(2).all(|w| w[0] <= w[1]));
            let filled = out.iter().take_while(|c| c.is_some()).count();
            assert!(out[filled..].iter().all(Option::is_none));
            // A second sort has nothing left to do.
            assert_eq!(plan(&out, &max, &rank), Some(Vec::new()));
        }
    }
}
