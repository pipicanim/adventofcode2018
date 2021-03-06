use gif::{Encoder, Frame, Repeat, SetParameter};
use std::borrow::Cow;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::time::{Duration, Instant};

const MAKE_GIF: bool = false;
const GIF_SCALE: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Cell {
    Empty,
    Wall,
    Unit(Unit),
    Distance(Distance),
    Target((usize, usize)),
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Unit {
    y: usize,
    x: usize,
    elf_or_goblin: char,
    hp: i32,
    ap: i32,
    id: usize,
}
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Distance {
    y: usize,
    x: usize,
    distance: usize,
}

fn simulation(is_part2: bool, elf_ap: i32) -> char {
    let mut color_map = Vec::new();
    let mut states: Vec<Vec<u8>> = Vec::new();
    if MAKE_GIF {
        // 0 wall
        color_map.push(0x00);
        color_map.push(0x00);
        color_map.push(0x00);
        // 1 ground
        color_map.push(0xFF);
        color_map.push(0xFF);
        color_map.push(0xFF);
        //elves are from 2-102

        // goblins are from 103-203
    }
    for i in (55u8..=255u8).step_by(2) {
        color_map.push(0);
        color_map.push(i);
        color_map.push(0);
    }
    for i in (55u8..=255u8).step_by(2) {
        color_map.push(i);
        color_map.push(0);
        color_map.push(0);
    }

    let br = BufReader::new(File::open("input.txt").unwrap());
    let mut map: Vec<Cell> = Vec::new();
    let mut y_size = 0;
    let mut last_id = 0;
    let mut winner: char = 'E';
    for (y, line) in br.lines().map(|x| x.unwrap()).enumerate() {
        for (x, cell) in line.chars().enumerate() {
            if cell == '#' {
                map.push(Cell::Wall);
            } else if cell == '.' {
                map.push(Cell::Empty);
            } else if cell == 'E' {
                map.push(Cell::Unit(Unit {
                    elf_or_goblin: 'E',
                    hp: 200,
                    x,
                    y,
                    ap: elf_ap,
                    id: last_id,
                }));
                last_id += 1;
            } else if cell == 'G' {
                map.push(Cell::Unit(Unit {
                    elf_or_goblin: 'G',
                    hp: 200,
                    x,
                    y,
                    ap: 3,
                    id: last_id,
                }));
                last_id += 1;
            }
        }
        y_size += 1;
    }
    let mut round_count = 1;
    'round: loop {
        let mut update = false;
        let mut units: Vec<Unit> = map
            .iter()
            .filter_map(|x| match x {
                Cell::Unit(e) => Some(*e),
                _ => None,
            })
            .collect::<Vec<Unit>>();
        units.sort_by(|u1, u2| (u1.y, u1.x).cmp(&(u2.y, u2.x)));
        for (which_unit, unit) in units.iter_mut().enumerate() {
            match map
                .iter()
                .filter_map(|x| {
                    if let Cell::Unit(u) = x {
                        if unit.id == u.id {
                            Some(u)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .next()
            {
                Some(z) => unit.hp = z.hp,
                None => {
                    continue;
                }
            }
            let mut chosen_enemies = Vec::new();
            let mut rechose_target = false;
            'find_target: loop {
                let enemies = map
                    .iter()
                    .filter_map(|x| match x {
                        Cell::Unit(u) => {
                            if u.elf_or_goblin != unit.elf_or_goblin {
                                Some(*u)
                            } else {
                                None
                            }
                        }
                        _ => None,
                    })
                    .collect::<Vec<Unit>>();
                if enemies.is_empty() {
                    if winner != 'G' {
                        winner = unit.elf_or_goblin;
                    }
                    if which_unit != 0 {
                        round_count -= 1;
                    }
                    break 'round;
                }
                for e in enemies {
                    match map[(e.y + 1) * y_size + e.x] {
                        Cell::Empty => map[(e.y + 1) * y_size + e.x] = Cell::Target((e.y + 1, e.x)),
                        Cell::Unit(u) => {
                            if u == *unit {
                                chosen_enemies.push(e);
                            }
                        }
                        _ => {}
                    }
                    match map[(e.y - 1) * y_size + e.x] {
                        Cell::Empty => map[(e.y - 1) * y_size + e.x] = Cell::Target((e.y - 1, e.x)),
                        Cell::Unit(u) => {
                            if u == *unit {
                                chosen_enemies.push(e);
                            }
                        }
                        _ => {}
                    }
                    match map[e.y * y_size + (e.x + 1)] {
                        Cell::Empty => map[e.y * y_size + (e.x + 1)] = Cell::Target((e.y, e.x + 1)),
                        Cell::Unit(u) => {
                            if u == *unit {
                                chosen_enemies.push(e);
                            }
                        }
                        _ => {}
                    }
                    match map[e.y * y_size + (e.x - 1)] {
                        Cell::Empty => map[e.y * y_size + (e.x - 1)] = Cell::Target((e.y, e.x - 1)),
                        Cell::Unit(u) => {
                            if u == *unit {
                                chosen_enemies.push(e);
                            }
                        }
                        _ => {}
                    }
                }
                if !chosen_enemies.is_empty() {
                    break 'find_target;
                }
                if rechose_target {
                    break 'find_target;
                }
                let mut cur_dist = 0;
                let mut move_target = None;
                'find_place_to_move: loop {
                    let distances = map
                        .iter()
                        .filter_map(|x| match x {
                            Cell::Unit(u) => {
                                if u.id == unit.id && cur_dist == 0 {
                                    Some(Distance {
                                        y: u.y,
                                        x: u.x,
                                        distance: 0,
                                    })
                                } else {
                                    None
                                }
                            }
                            Cell::Distance(i) => {
                                if i.distance == cur_dist {
                                    Some(*i)
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        })
                        .collect::<Vec<Distance>>();
                    let mut break_iter = true;
                    let mut target_distances: Vec<(usize, usize)> = Vec::new();
                    for d in distances {
                        match map[(d.y + 1) * y_size + d.x] {
                            Cell::Empty => {
                                map[(d.y + 1) * y_size + d.x] = Cell::Distance(Distance {
                                    y: d.y + 1,
                                    x: d.x,
                                    distance: cur_dist + 1,
                                });
                                break_iter = false;
                            }
                            Cell::Target(t) => {
                                target_distances.push(t);
                            }
                            _ => {}
                        }
                        match map[(d.y - 1) * y_size + d.x] {
                            Cell::Empty => {
                                map[(d.y - 1) * y_size + d.x] = Cell::Distance(Distance {
                                    y: d.y - 1,
                                    x: d.x,
                                    distance: cur_dist + 1,
                                });
                                break_iter = false;
                            }
                            Cell::Target(t) => {
                                target_distances.push(t);
                            }
                            _ => {}
                        }
                        match map[d.y * y_size + (d.x + 1)] {
                            Cell::Empty => {
                                map[d.y * y_size + (d.x + 1)] = Cell::Distance(Distance {
                                    y: d.y,
                                    x: d.x + 1,
                                    distance: cur_dist + 1,
                                });
                                break_iter = false;
                            }
                            Cell::Target(t) => {
                                target_distances.push(t);
                            }
                            _ => {}
                        }
                        match map[d.y * y_size + (d.x - 1)] {
                            Cell::Empty => {
                                map[d.y * y_size + (d.x - 1)] = Cell::Distance(Distance {
                                    y: d.y,
                                    x: d.x - 1,
                                    distance: cur_dist + 1,
                                });
                                break_iter = false;
                            }
                            Cell::Target(t) => {
                                target_distances.push(t);
                            }
                            _ => {}
                        }
                    }
                    if !target_distances.is_empty() {
                        target_distances.sort();
                        let tdm = *target_distances.iter().next().unwrap();
                        move_target = Some((tdm.0, tdm.1, cur_dist + 1));
                        break 'find_place_to_move;
                    }
                    if break_iter {
                        break 'find_place_to_move;
                    }
                    cur_dist += 1;
                }
                match move_target {
                    Some((y, x, d)) => {
                        let mut possible_paths: Vec<(usize, usize, usize)> = Vec::new();
                        possible_paths.push((y, x, d));
                        let mut targets = Vec::new();
                        'find_square: loop {
                            let mut new_paths: Vec<(usize, usize, usize)> = Vec::new();
                            for p in &possible_paths {
                                match map[(p.0 + 1) * y_size + (p.1)] {
                                    Cell::Distance(d) => {
                                        if d.distance == p.2 - 1 {
                                            new_paths.push((d.y, d.x, d.distance));
                                        }
                                    }
                                    Cell::Unit(t) => {
                                        if t.id == unit.id {
                                            targets.push((p.0, p.1, 0));
                                        }
                                    }
                                    _ => {}
                                }
                                match map[(p.0 - 1) * y_size + (p.1)] {
                                    Cell::Distance(d) => {
                                        if d.distance == p.2 - 1 {
                                            new_paths.push((d.y, d.x, d.distance));
                                        }
                                    }
                                    Cell::Unit(t) => {
                                        if t.id == unit.id {
                                            targets.push((p.0, p.1, 0));
                                        }
                                    }
                                    _ => {}
                                }
                                match map[(p.0) * y_size + (p.1 + 1)] {
                                    Cell::Distance(d) => {
                                        if d.distance == p.2 - 1 {
                                            new_paths.push((d.y, d.x, d.distance));
                                        }
                                    }
                                    Cell::Unit(t) => {
                                        if t.id == unit.id {
                                            targets.push((p.0, p.1, 0));
                                        }
                                    }
                                    _ => {}
                                }
                                match map[(p.0) * y_size + (p.1 - 1)] {
                                    Cell::Distance(d) => {
                                        if d.distance == p.2 - 1 {
                                            new_paths.push((d.y, d.x, d.distance));
                                        }
                                    }
                                    Cell::Unit(t) => {
                                        if t.id == unit.id {
                                            targets.push((p.0, p.1, 0));
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            if !targets.is_empty() {
                                break 'find_square;
                            }
                            possible_paths = new_paths;
                        }
                        targets.sort();
                        let (y, x, _) = targets[0];

                        map[unit.y * y_size + unit.x] = Cell::Empty;
                        unit.y = y;
                        unit.x = x;
                        map[y * y_size + x] = Cell::Unit(*unit);
                        update = true;
                        rechose_target = true;
                    }
                    None => {
                        break 'find_target;
                    }
                }
                clean_map(&mut map);
            }
            clean_map(&mut map);
            if !chosen_enemies.is_empty() {
                chosen_enemies.sort_by(|x, y| (x.hp, x.y, x.x).cmp(&(y.hp, y.y, y.x)));
                let e = chosen_enemies[0];
                if let Cell::Unit(attackee) = &mut map[e.y * y_size + e.x] {
                    attackee.hp -= unit.ap;
                    if attackee.hp <= 0 {
                        if attackee.elf_or_goblin == 'E' && is_part2 {
                            winner = 'G';
                        }
                        update = true;
                        map[e.y * y_size + e.x] = Cell::Empty;
                    }
                }
            }
            if MAKE_GIF && update {
                let mut state: Vec<u8> = vec![1; 32 * 32 * GIF_SCALE * GIF_SCALE];
                for (y, row) in map.chunks(32).enumerate() {
                    for (x, cell) in row.iter().enumerate() {
                        match cell {
                            Cell::Empty => {
                                scaling_fill(&mut state, y, x, 1);
                            }
                            Cell::Wall => {
                                scaling_fill(&mut state, y, x, 0);
                            }
                            Cell::Unit(i) => {
                                if i.elf_or_goblin == 'G' {
                                    scaling_fill(&mut state, y, x, (102 + i.hp / 2) as u8);
                                } else if i.elf_or_goblin == 'E' {
                                    scaling_fill(&mut state, y, x, (2 + i.hp / 2) as u8);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                states.push(state);
            }
        }
        round_count += 1;
    }
    if MAKE_GIF {
        let mut image = File::create(format!("batte-{}.gif", elf_ap)).unwrap();
        let mut encoder = Encoder::new(
            &mut image,
            32u16 * GIF_SCALE as u16,
            32u16 * GIF_SCALE as u16,
            &color_map,
        )
        .unwrap();
        encoder.set(Repeat::Finite(1)).unwrap();
        for state in &states {
            let mut frame = Frame::default();
            frame.width = 32u16 * GIF_SCALE as u16;
            frame.height = 32u16 * GIF_SCALE as u16;
            frame.buffer = Cow::Borrowed(state as &[u8]);
            frame.delay = 1;
            encoder.write_frame(&frame).unwrap();
        }
    }

    let winner_force = map
        .iter()
        .map(|x| if let Cell::Unit(u) = x { u.hp } else { 0 })
        .sum::<i32>();
    if elf_ap == 3 {
        println!("part 1: {}", winner_force * round_count);
    }
    if winner == 'E' {
        println!("part 2: {}", winner_force * round_count);
    }
    winner
}

fn scaling_fill(state: &mut Vec<u8>, y: usize, x: usize, ci: u8) {
    for sy in 0..GIF_SCALE {
        for sx in 0..GIF_SCALE {
            state[(y * GIF_SCALE + sy) * 32 * GIF_SCALE + (x * GIF_SCALE + sx)] = ci;
        }
    }
}

fn main() {
    let now = Instant::now();
    for i in 3.. {
        if simulation(true, i) == 'E' {
            break;
        }
    }

    let d: Duration = now.elapsed();
    println!("> {}.{:03} seconds", d.as_secs(), d.subsec_millis());
}

fn clean_map(map: &mut Vec<Cell>) {
    for m in map {
        match m {
            Cell::Target(_) => *m = Cell::Empty,
            Cell::Distance(_) => *m = Cell::Empty,
            _ => {}
        }
    }
}
