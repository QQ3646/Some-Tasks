use std::{
    cmp::min,
    fs::File,
    io::{BufRead, BufReader, Write}, env,
};

// Time: O((matrix_size - pattern_size) * pattern_size)
// Memory: O(matrix_size + pattern_size)

/*
 * Algorithm: an additional submatrix of size matrix_size - pattern_size is created, which shows 
 * from which cell you can start looking at whether a part of the matrix matches the pattern or not. 
 * After the block has passed the check and the change_matrix function has been called, not only the 
 * zone that is correct, but also the adjacent cells become unavailable for check, since part of the 
 * possible zone has already been changed.
*/

struct Matrix {
    size: (usize, usize),
    data: Vec<Vec<char>>,
}

fn read_line(input: &mut BufReader<File>) -> Vec<String> {
    let mut buffer = String::new();
    input.read_line(&mut buffer).expect("Failed to read.");
    buffer
        .trim_end()
        .split_whitespace()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
}

impl Matrix {
    fn collect_matrix(input: &mut BufReader<File>) -> Self {
        let buffer_vec: Vec<usize> = read_line(input)
            .iter()
            .map(|x| x.parse().expect("Not an integer!"))
            .collect();
        let mut matrix = Self {
            size: (buffer_vec[0], buffer_vec[1]),
            data: Vec::new(),
        };

        for _ in 0..matrix.size.0 {
            matrix.data.push(
                read_line(input)
                    .iter()
                    .map(|x| x.chars().next().unwrap())
                    .collect(),
            );
        }

        matrix
    }

    fn create_map(&self, pattern: &Matrix) -> MatrixMap {
        let mut size = (
            self.size.0 as isize - pattern.size.0 as isize + 1,
            self.size.1 as isize - pattern.size.1 as isize + 1,
        );
        if size.0 <= 0 || size.1 <= 0 {
            size = (0, 0)
        }
        MatrixMap {
            size: (size.0 as usize, size.1 as usize),
            map: vec![
                vec![true; size.1 as usize];
                size.0 as usize
            ],
        }
    }

    fn change_matix(&mut self, pattern: &Matrix, coords: (usize, usize), map: &mut MatrixMap) {
        for i in coords.0..coords.0 + pattern.size.0 {
            for j in coords.1..coords.1 + pattern.size.1 {
                if self.data[i][j] == '1' {
                    self.data[i][j] = '2';
                } else {
                    self.data[i][j] = '*';
                }
            }
        }
        map.succses_pattern(coords, pattern);
    }

    #[allow(unused_must_use)]
    fn print(&self, output: &mut File) {
        for i in 0..self.size.0 {
            for j in 0..self.size.1 {
                write!(output, "{} ", self.data[i][j]);
            }
            writeln!(output);
        }
    }
}

struct MatrixMap {
    map: Vec<Vec<bool>>,
    size: (usize, usize),
}

impl MatrixMap {
    fn succses_pattern(&mut self, coords: (usize, usize), pattern: &Matrix) {
        let height = min(pattern.size.0, self.size.0 - coords.0);
        let w_start = coords.1 - {
            let mut j = pattern.size.1 - 1;
            for i in 0..pattern.size.1 {
                if coords.1 < i {
                    j = i - 1;
                    break;
                }
            }
            j
        };
        let width = min(self.size.1 - coords.1, pattern.size.1) + (coords.1 - w_start);

        for i in coords.0..coords.0 + height {
            for j in w_start..w_start + width {
                self.map[i][j] = false;
            }
        }
    }
}

fn main() {
    let input_file = env::args().nth(1).unwrap_or("input.txt".to_string());
    let mut input = BufReader::new(File::open(input_file).expect("File read error."));
    let mut output = File::create("output.txt").expect("File creation error.");

    let (pattern, mut matrix) = (
        Matrix::collect_matrix(&mut input),
        Matrix::collect_matrix(&mut input),
    );

    let mut map = matrix.create_map(&pattern);

    for y in 0..map.size.0 {
        for x in 0..map.size.1 {
            if map.map[y][x] {
                let mut succses_pattern = true;
                for i in 0..pattern.size.0 {
                    for j in 0..pattern.size.1 {
                        if pattern.data[i][j] != matrix.data[y + i][x + j] {
                            succses_pattern = false;
                            break;
                        }
                    }
                    if !succses_pattern {
                        break;
                    }
                }
                if succses_pattern {
                    matrix.change_matix(&pattern, (y, x), &mut map);
                }
            }
        }
    }
    matrix.print(&mut output);
}