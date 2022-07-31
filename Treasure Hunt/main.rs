use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

macro_rules! find_min {
    ($x:expr) => ($x);
    ($x:expr, $($y:expr),+) => (
        std::cmp::min($x, find_min!($($y),+))
    )
}

/*
 * The task solves the problem, literally by brute force.
 * The complexity is O(n), where n is the number of walls, but with a factor of ~400.
 * Together with max n, it gives about 12000 operations, which is quite small.
 *
 *
 * The solution works on the idea that all figures resulting from the walls are
 * convex, since there are no segments whose ends are not on the boundary.
 *
 *
 * The algorithm of work is as follows: we take a point from which not a single wall
 * has left (the ignore_points vector is responsible for this) and connect it to a
 * treasure point and count all intersections. Given that the figures are convex, I think
 * that the point with the correct answer will be.
 */

#[derive(Copy, Clone)]
struct Point {
    x: f32,
    y: f32,
}

#[derive(Copy, Clone)]
struct Line {
    p1: Point,
    p2: Point,
}

struct IgnorePointsList {
    pub top_edge: Vec<i32>,
    pub bottom_edge: Vec<i32>,
    pub left_edge: Vec<i32>,
    pub right_edge: Vec<i32>,
}

impl IgnorePointsList {
    pub fn new() -> Self {
        Self {
            top_edge: Vec::new(),
            bottom_edge: Vec::new(),
            left_edge: Vec::new(),
            right_edge: Vec::new(),
        }
    }

    pub fn dedup_and_sort(&mut self) {
        self.left_edge.dedup();
        self.right_edge.dedup();
        self.bottom_edge.dedup();
        self.top_edge.dedup();
        self.left_edge.sort();
        self.right_edge.sort();
        self.bottom_edge.sort();
        self.top_edge.sort();
    }
}

impl Line {
    pub fn new(p1: Point, p2: Point) -> Self {
        Self { p1, p2 }
    }
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn from_vec(vec: Vec<f32>) -> [Self; 2] {
        let mut res = [Self::new(0f32, 0f32); 2];
        for i in 0..2 {
            res[i] = Self::new(vec[i * 2], vec[i * 2 + 1]);
        }
        res
    }

    pub fn to_tuple(&self) -> (i32, i32) {
        (self.x as i32, self.y as i32)
    }
}


fn inter(a: &Line, b: &Line) -> bool {
    let v1 = (b.p2.x - b.p1.x) * (a.p1.y - b.p1.y) - (b.p2.y - b.p1.y) * (a.p1.x - b.p1.x);
    let v2 = (b.p2.x - b.p1.x) * (a.p2.y - b.p1.y) - (b.p2.y - b.p1.y) * (a.p2.x - b.p1.x);
    let v3 = (a.p2.x - a.p1.x) * (b.p1.y - a.p1.y) - (a.p2.y - a.p1.y) * (b.p1.x - a.p1.x);
    let v4 = (a.p2.x - a.p1.x) * (b.p2.y - a.p1.y) - (a.p2.y - a.p1.y) * (b.p2.x - a.p1.x);

    (v1 * v2 < 0f32) && (v3 * v4 < 0f32)
}

fn read_line(buffer: &mut String, input: &mut BufReader<File>) -> Vec<f32> {
    buffer.clear();
    input
        .read_line(buffer)
        .expect("File reading error.");
    buffer
        .trim()
        .split_whitespace()
        .map(|x| x.parse::<f32>().expect("String parsing eror"))
        .collect()
}

fn count_intersects(p: Point, t_pos: Point, line_v: &Vec<Line>) -> i32 {
    let line = Line::new(p, t_pos);
    let mut counter = 1;
    for k in line_v.iter() {
        if inter(&line, k) {
            counter += 1;
        }
    }
    counter
}

fn main() {
    let file_name = env::args().nth(1).unwrap_or("input.txt".to_string());
    let mut input = BufReader::new(File::open(&file_name).expect("File reading error."));
    let mut output = File::create("output.txt").expect("File creation error.");
    let mut buffer = String::new();

    let count = read_line(&mut buffer, &mut input)[0] as i32;

    let mut line_vec: Vec<Line> = Vec::new();
    let mut ignore_points = IgnorePointsList::new();
    for _ in 0..count {
        let p_array = Point::from_vec(read_line(&mut buffer, &mut input));
        for i in 0..1 {
            match p_array[i].to_tuple() {
                (0, 0) => {
                    ignore_points.bottom_edge.push(0);
                    ignore_points.left_edge.push(0);
                },
                (100, 0) => {
                    ignore_points.bottom_edge.push(100);
                    ignore_points.right_edge.push(0);
                },
                (0, 100) => {
                    ignore_points.top_edge.push(0);
                    ignore_points.left_edge.push(100);
                },
                (100, 100) => {
                    ignore_points.top_edge.push(100);
                    ignore_points.right_edge.push(100);
                },
                (x, 100) => ignore_points.top_edge.push(x),
                (x, 0) => ignore_points.bottom_edge.push(x),
                (0, y) => ignore_points.left_edge.push(y),
                (100, y) => ignore_points.right_edge.push(y),
                _ => (),
            }
        }
        line_vec.push(Line::new(p_array[0], p_array[1]));
    }

    ignore_points.dedup_and_sort();

    let treasure = read_line(&mut buffer, &mut input);
    let treasure_pos = Point::new(treasure[0], treasure[1]);
    let mut minimum = i32::MAX;
    for i in 0..=100 {
        minimum = find_min!(
            minimum,
            if !ignore_points.top_edge.contains(&i) {
                count_intersects(Point::new(i as f32, 100f32), treasure_pos, &line_vec)
            } else {
                i32::MAX
            },
            if !ignore_points.bottom_edge.contains(&i) {
                count_intersects(Point::new(i as f32, 0f32), treasure_pos, &line_vec)
            } else {
                i32::MAX
            },
            if !ignore_points.left_edge.contains(&i) {
                count_intersects(Point::new(0f32, i as f32), treasure_pos, &line_vec)
            } else {
                i32::MAX
            },
            if !ignore_points.right_edge.contains(&i) {
                count_intersects(Point::new(100f32, i as f32), treasure_pos, &line_vec)
            } else {
                i32::MAX
            }
        );
    }
    if ignore_points.right_edge.len() > 0 {
        for i in 0..ignore_points.right_edge.len() - 1 {
            minimum = find_min!(minimum, count_intersects(Point::new(100f32, (ignore_points.right_edge[i] + ignore_points.right_edge[i + 1]) as f32 / 2f32), treasure_pos, &line_vec));
        }
    }
    if ignore_points.left_edge.len() > 0 {
        for i in 0..ignore_points.left_edge.len() - 1 {
            minimum = find_min!(minimum, count_intersects(Point::new(0f32, (ignore_points.left_edge[i] + ignore_points.left_edge[i + 1]) as f32 / 2f32), treasure_pos, &line_vec));
        }
    }
    if ignore_points.top_edge.len() > 0 {
        for i in 0..ignore_points.top_edge.len() - 1 {
            minimum = find_min!(minimum, count_intersects(Point::new((ignore_points.top_edge[i] + ignore_points.top_edge[i + 1]) as f32 / 2f32, 100f32), treasure_pos, &line_vec));
        }
    }
    if ignore_points.bottom_edge.len() > 0 {
        for i in 0..ignore_points.bottom_edge.len() - 1 {
            minimum = find_min!(minimum, count_intersects(Point::new((ignore_points.bottom_edge[i] + ignore_points.bottom_edge[i + 1]) as f32 / 2f32, 0f32), treasure_pos, &line_vec));
        }
    }
    writeln!(&mut output, "Number of doors = {minimum}");
}
