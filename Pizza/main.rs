use std::{ops::{Add, Index, IndexMut, Mul, Sub}, cmp::PartialEq, env};
use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
};

// A couple of macros to make the code a little bit easier.
macro_rules! all_direction {
    () => {
        [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ]
    };
}

macro_rules! ps {
    ($x: ident, $y: ident) => {
        Point::new($x, $y)
    };
}

enum ReadResult {
    Some(usize, usize, i32),
    None,
}

#[derive(Clone, Copy)]
struct SizeP<T> {
    x: T,
    y: T,
}

impl<T> SizeP<T> where T: Add + Add<Output=T> + Copy {
    fn from_tuple(coord: (T, T)) -> SizeP<T> {
        SizeP::new(coord.0, coord.1)
    }

    fn new(x: T, y: T) -> SizeP<T> {
        SizeP { x, y }
    }

    fn to_tuple(self) -> (T, T) {
        (self.x, self.y)
    }

    fn sum(self) -> T {
        self.x + self.y
    }
}

impl<T> PartialEq for SizeP<T> where T: PartialEq {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

// I will define several types for different contexts to make the code more readable.
type Point = SizeP<usize>;
type Size = Point;
type Shift = SizeP<i32>;

impl Mul<i32> for Shift {
    type Output = Shift;

    fn mul(self, rhs: i32) -> Self::Output {
        Shift::from_tuple((self.x * rhs, self.y * rhs))
    }
}

impl Add<Shift> for Point {
    type Output = Point;

    fn add(self, rhs: Shift) -> Self::Output {
        Point::from_tuple((
            (self.x as i32 + rhs.x) as usize,
            (self.y as i32 + rhs.y) as usize,
        ))
    }
}

impl Sub for Point {
    type Output = SizeP<i32>;

    fn sub(self, rhs: Self) -> Self::Output {
        SizeP::new(self.x as i32 - rhs.x as i32, self.y as i32 - rhs.y as i32)
    }
}

/*
Determine the states of the map cells. 
Accordingly, each state has parameters that describe it.
There are three states in total: 
    1. Pizzeria
        1.1. num - serial number of the pizzeria;
        1.2. capacity - the number of cells that the pizzeria can accommodate;
        1.3. movement - "expansion" of the territories of the pizzeria;
        1.4. avalable_blocks - the list of all possible blocks claimed by the current pizzeria;
    2. A cage occupied by a pizzeria
        2.1. The number of the pizzeria that occupied the cage;
    3. Unoccupied cell.
        3.1. List of all possible pizzerias claiming this cell;
These states can describe any field configuration.
*/

#[derive(Clone)]
enum MapBlock {
    Pizzeria {
        num: i32,
        capacity: i32,
        movement: (i32, i32, i32, i32),
        available_blocks: Vec<Point>,
    },
    Occupied(i32),
    Unoccupied(Vec<i32>),
}

// For better debugging
impl std::fmt::Display for MapBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MapBlock::Occupied(x) => write!(f, "{:^6}", x),
            MapBlock::Pizzeria { num, capacity, .. } => {
                write!(f, "{:^6}", format_args!("{}({})", num, capacity))
            }
            MapBlock::Unoccupied(..) => write!(f, "{:^6}", -1),
        }
    }
}

#[derive(Clone)]
struct Map {
    map: Vec<Vec<MapBlock>>,
    size: Size,
    free_block_counter: i32,
    pizzerias_pos: Vec<Point>,
}

// For simple index
impl Index<Point> for Map {
    type Output = MapBlock;
    fn index(&self, index: Point) -> &Self::Output {
        &self.map[index.y as usize][index.x as usize]
    }
}

impl IndexMut<Point> for Map {
    fn index_mut(&mut self, index: Point) -> &mut Self::Output {
        &mut self.map[index.y as usize][index.x as usize]
    }
}

impl Map {
    fn new(size: Size) -> Self {
        Self {
            map: vec![vec![MapBlock::Unoccupied(Vec::new()); size.x]; size.y],
            size,
            free_block_counter: (size.x * size.y) as i32,
            pizzerias_pos: Vec::new(),
        }
    }

    fn add_pizzeria(&mut self, pos: Point, capacity: i32, num: i32) {
        let p_pos = Point::new((pos.x - 1) as usize, (self.size.y - pos.y) as usize);
        self[p_pos] = MapBlock::Pizzeria {
            num,
            capacity,
            movement: (0, 0, 0, 0),
            available_blocks: Vec::new(),
        };
        self.free_block_counter -= 1;
        self.pizzerias_pos.push(p_pos);

        self.expansion(p_pos);
    }

    /*  
    The expansion is used to expand the influence of pizzerias.
    The expansion is performed in all possible directions, adding the number 
    of the selected pizzeria to the vector in free blocks. The number of blocks 
    through which there will be a passage depends on the capacity of the pizzeria.
    */
    fn expansion(&mut self, pos: Point) {
        let mut cap = 0;
        let mut n = 0;
        if let MapBlock::Pizzeria { capacity, num, .. } = self[pos] {
            cap = capacity;
            n = num;
        }

        let dirs = all_direction!();
        for d in dirs {
            let shift = d.get_shift();
            for i in 1..=guaranteed_arithmetic(cap, pos, self.size, &d) {
                let mut res = false;
                if let MapBlock::Unoccupied(ref mut v) = self[pos + shift * i] {
                    v.push(n);
                    res = true;
                } else {
                    break;
                }
                if res {
                    if let MapBlock::Pizzeria {
                        ref mut available_blocks,
                        ..
                    } = self[pos]
                    {
                        available_blocks.push(pos + shift * i);
                    }
                }
            }
        }
    }

    fn count_distance_and_shift(&mut self, pos: Point, p_pos: Point) -> (i32, Direction) {
        let dir =
            match (pos - p_pos).to_tuple() {
                (0, 0) => Direction::Down,
                (x, 0) => if x > 0 {
                    Direction::Left
                } else {
                    Direction::Right
                },
                (0, y) => if y > 0 {
                    Direction::Up
                } else {
                    Direction::Down
                },
                _ => Direction::Down,
            };
        let mut count = 1;
        while let MapBlock::Unoccupied(_) = self[pos + dir.get_shift() * count] {
            count += 1;
        }
        (count, dir)
    }

    fn fill_obvious_blocks(&mut self) -> bool {
        let mut global_res = false;
        let mut res = true;
        while res {
            res = false;
            for y in 0..self.size.y {
                for x in 0..self.size.x {
                    let mut num = -1;
                    if let MapBlock::Unoccupied(ref mut vector) = self[ps!(x, y)] {
                        if vector.len() == 1 {
                            num = vector[0];
                        }
                    }
                    if num != -1 {
                        res = true;
                        global_res = true;

                        let p_pos = self.pizzerias_pos[num as usize];
                        let (n, dir) = self.count_distance_and_shift(ps!(x, y), p_pos);
                        let shift = dir.get_shift();
                        for i in 0..n {
                            self[ps!(x, y) + shift * i] = MapBlock::Occupied(num);
                        }

                        self.free_block_counter -= n;

                        self.constriction(p_pos, n, dir);
                    }
                }
            }
        }
        global_res
    }
    /*
     * The constriction function should reduce the influence of pizzerias
     * for which it has already been decided that this pizzeria will occupy the cell.
     */
    fn constriction(&mut self, p_pos: Point, capacity_diff: i32, dir: Direction) {
        let mut n = -1;
        let mut block_pos: Vec<Point> = Vec::new();
        if let MapBlock::Pizzeria {
            ref mut capacity,
            ref mut movement,
            ref mut available_blocks,
            num
        } = self[p_pos]
        {
            n = num;
            *capacity -= capacity_diff;
            match dir {
                Direction::Up => {
                    movement.2 += capacity_diff;
                }
                Direction::Down => {
                    movement.0 += capacity_diff;
                }

                Direction::Left => {
                    movement.1 += capacity_diff;
                }
                Direction::Right => {
                    movement.3 += capacity_diff;
                }
            };

            for block in available_blocks.iter() {
                let mov = match (*block - p_pos).to_tuple() {
                    (0, y) => if y > 0 {
                        movement.2
                    } else {
                        movement.0
                    },
                    (x, 0) => if x > 0 {
                        movement.1
                    } else {
                        movement.3
                    },
                    _ => 0,
                };
                if i32::abs((*block - p_pos).sum()) > *capacity + mov {
                    block_pos.push(*block);
                }
            }
            for block in &block_pos {
                available_blocks.retain(|&x| x != *block);
            }
        }
        for block in block_pos.iter() {
            if let MapBlock::Unoccupied(ref mut v) = self[*block] {
                v.retain(|&x| x != n);
            }
        }
    }

    fn find_pair(&self, pizza_list: &Vec<i32>, pos: Point) -> (SizeP<i32>, Vec<i32>) {
        fn vector_intersection(v1: &Vec<i32>, v2: &Vec<i32>) -> Vec<i32> {
            let mut intersection = v1.clone();
            intersection.retain(|x| v2.contains(x));
            intersection
        }

        for y in 0..self.size.y {
            for x in 0..self.size.x {
                if ps!(x, y) != pos {
                    if let MapBlock::Unoccupied(ref v) = self[Point::new(x, y)] {
                        let vi = vector_intersection(pizza_list, v);
                        if vi.len() == 2 {
                            return (SizeP::new(x as i32, y as i32), vi);
                        }
                    }
                }
            }
        }
        (SizeP::new(-1, -1), Vec::new())
    }

    fn place(&mut self, pos: &Vec<Point>, nums: &[i32]) {
        for i in 0..pos.len() {
            let p_pos = self.pizzerias_pos[nums[i] as usize];
            let (n, dir) = self.count_distance_and_shift(pos[i], p_pos);
            let shift = dir.get_shift();
            for k in 0..n {
                self[pos[i] + shift * k] = MapBlock::Occupied(nums[i]);
            }
            self.free_block_counter -= n;

            self.constriction(p_pos, n, dir);
        }
    }

    fn clear_unoccupied_and_pizzerias(&mut self) {
        for y in 0..self.size.y {
            for x in 0..self.size.x {
                if let MapBlock::Unoccupied(ref mut v) = self[ps!(x, y)] {
                    v.clear();
                } else if let MapBlock::Pizzeria { ref mut available_blocks, .. } = self[ps!(x, y)] {
                    available_blocks.clear();
                }
            }
        }
    }

    fn try_to_fill(&mut self, mut pos: Vec<Point>, nums: Vec<i32>) -> bool {
        let mut global_res = false;
        for _ in 0..nums.len() {
            let mut map_c = (*self).clone();
            pos.rotate_right(1);

            // This function checks the configuration for "possibility". 
            // If the possible configuration is incorrect, then this function will tell you about it.
            fn correct_check(mut map_c: Map) -> bool {
                map_c.clear_unoccupied_and_pizzerias();
                let mut possible_free_blocks = map_c.free_block_counter;
                for i in 0..map_c.pizzerias_pos.len() {
                    map_c.expansion(map_c.pizzerias_pos[i]);

                    if let MapBlock::Pizzeria { ref available_blocks, capacity, .. } = map_c[map_c.pizzerias_pos[i]] {
                        possible_free_blocks -= i32::min(capacity, available_blocks.len() as i32);
                        if capacity < 0 {
                            return false;
                        }
                    }
                }

                for y in 0..map_c.size.y {
                    for x in 0..map_c.size.x {
                        if let MapBlock::Unoccupied(ref v) = map_c[ps!(x, y)] {
                            if v.is_empty() {
                                return false;
                            }
                        }
                    }
                }
                possible_free_blocks == 0
            }

            map_c.place(&pos, &nums[..pos.len()]);
            if correct_check(map_c) {
                self.place(&pos, &nums[..pos.len()]);
                global_res = true;
                break;
            }
        }
        global_res
    }

    /*
    The entire filling process roughly consists of two parts. 
    In the first "obvious" cells are filled, in which there can be only one pizzeria, 
    in the second cells are filled in pairs. 
    */
    fn fill_map(&mut self) {
        let mut global_res = true;
        while self.free_block_counter != 0 && global_res {
            global_res = self.fill_obvious_blocks();

            self.try_to_find_pairs();
            // self.fill_not_obvious_blocks();
            for i in 0..self.size.y {
                for j in 0..self.size.x {
                    let mut p_list = Vec::new();
                    if let MapBlock::Unoccupied(ref v) = self[ps!(j, i)] {
                        p_list = v.clone();
                    }

                    if !p_list.is_empty() {
                        global_res = global_res | self.try_to_fill(vec![ps!(j, i)], p_list);
                    }
                }
            }
        }
    }

    fn try_to_find_pairs(&mut self) {
        for y in 0..self.size.y {
            for x in 0..self.size.x {
                let mut pair_pos: (SizeP<i32>, Vec<i32>) = (SizeP::new(-1, -1), Vec::new());
                if let MapBlock::Unoccupied(ref v) = self[ps!(x, y)] {
                    pair_pos = self.find_pair(v, ps!(x, y));
                }

                if pair_pos.0.to_tuple() != (-1, -1) {
                    let point = Point::new(pair_pos.0.x as usize, pair_pos.0.y as usize);
                    self.try_to_fill(vec![ps!(x, y), point], pair_pos.1);
                }
            }
        }
    }
}

fn read_line(input: &mut BufReader<File>) -> ReadResult {
    let mut buffer = String::new();
    input
        .read_line(&mut buffer)
        .expect("Gotten error while reading file.");
    let res: Vec<u32> = buffer
        .trim()
        .split_whitespace()
        .map(|x| x.parse().expect("This is not an integer!"))
        .collect();
    if res.len() == 1 {
        ReadResult::None
    } else {
        ReadResult::Some(res[0] as usize, res[1] as usize, res[2] as i32)
    }
}

// Denote an auxiliary enumeration in order to better determine the directions of movements
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn get_shift(&self) -> Shift {
        Shift::from_tuple(match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        })
    }
}

// Guaranteed arithmetic is needed so that when moving we do not go beyond the boundaries of the field
fn guaranteed_arithmetic(cap: i32, pos: Point, size: Size, flag: &Direction) -> i32 {
    std::cmp::min::<i32>(
        match flag {
            Direction::Up => pos.y as i32,
            Direction::Down => (size.y - pos.y - 1) as i32,
            Direction::Left => pos.x as i32,
            Direction::Right => (size.x - pos.x - 1) as i32,
        },
        cap,
    )
}

fn main() {
    let file_name = env::args().nth(1).unwrap_or("input.txt".to_string());
    let mut input = BufReader::new(File::open(&file_name).expect("File read error."));
    let mut output = File::create("output.txt").expect("File creation error.");

    let mut case_counter = 1u32;
    while let ReadResult::Some(n, m, k) = read_line(&mut input) {
        let mut map = Map::new(ps!(n, m));
        for num in 0..k {
            if let ReadResult::Some(x, y, cap) = read_line(&mut input) {
                map.add_pizzeria(ps!(x, y), cap, num);
            }
        }

        map.fill_map();

        #[allow(unused_must_use)]
        {
            writeln!(&mut output, "Case {}:", case_counter);
            case_counter += 1;

            for i in map.pizzerias_pos {
                if let MapBlock::Pizzeria { movement, .. } = map.map[i.y][i.x] {
                    writeln!(
                        &mut output,
                        "{} {} {} {}",
                        movement.0, movement.1, movement.2, movement.3
                    );
                }
            }
            writeln!(&mut output);
        }
    }
}
