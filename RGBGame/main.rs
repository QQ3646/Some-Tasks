use std::{
    collections::LinkedList,
    env,
    fs::File,
    io::{BufRead, BufReader, Write},
};

pub struct Game {
    desk: Vec<Vec<char>>,
    cluster_map: Vec<Vec<i32>>,
    shift_column: Vec<usize>,

    reader: BufReader<File>,
    output: File,
    buffer: String,

    move_counter: usize,
    score: u32,
}

impl Game {
    pub fn new(input_file_name: &str, output_file_name: &str) -> Self {
        Self {
            desk: vec![vec![' '; 15]; 10],
            cluster_map: vec![vec![-1; 15]; 10],
            shift_column: vec![0; 15],

            reader: BufReader::new(File::open(input_file_name).expect("File read error.")),
            output: File::create(output_file_name).expect("File creation error."),
            buffer: String::new(),

            move_counter: 1,
            score: 0,
        }
    }

    fn searching_clasters(&mut self, start_pos: (usize, usize), cluster_num: i32) -> (i32, i32, char) {
        let mut max_cluster: (i32, i32, char) =
            (cluster_num, 0, self.desk[start_pos.0][start_pos.1]); // (number, count of balls, color)
        let mut pos_queue: LinkedList<(usize, usize)> = LinkedList::new();
        pos_queue.push_back(start_pos);

        while !pos_queue.is_empty() {
            let pos = pos_queue.pop_front().unwrap();

            if self.desk[pos.0][pos.1] == ' ' {
                continue;
            }
            // Up
            if pos.0 != 0
                && self.cluster_map[pos.0 - 1][pos.1] == -1
                && self.desk[pos.0 - 1][pos.1] == max_cluster.2
            {
                self.cluster_map[pos.0 - 1][pos.1] = cluster_num;
                pos_queue.push_back((pos.0 - 1, pos.1));
                max_cluster.1 += 1;
            }
            // Down
            if pos.0 != 9
                && self.cluster_map[pos.0 + 1][pos.1] == -1
                && self.desk[pos.0 + 1][pos.1] == max_cluster.2
            {
                self.cluster_map[pos.0 + 1][pos.1] = cluster_num;
                pos_queue.push_back((pos.0 + 1, pos.1));
                max_cluster.1 += 1;
            }
            // Left
            if pos.1 != 0
                && self.cluster_map[pos.0][pos.1 - 1] == -1
                && self.desk[pos.0][pos.1 - 1] == max_cluster.2
            {
                self.cluster_map[pos.0][pos.1 - 1] = cluster_num;
                pos_queue.push_back((pos.0, pos.1 - 1));
                max_cluster.1 += 1;
            }
            // Right
            if pos.1 != 14
                && self.cluster_map[pos.0][pos.1 + 1] == -1
                && self.desk[pos.0][pos.1 + 1] == max_cluster.2
            {
                self.cluster_map[pos.0][pos.1 + 1] = cluster_num;
                pos_queue.push_back((pos.0, pos.1 + 1));
                max_cluster.1 += 1;
            }
        }
        max_cluster
    }

    pub fn start_game(&mut self, game_number: u32) {
        self.reader.read_line(&mut self.buffer); //ignore first line
        for i in 0..10 {
            self.buffer.clear();
            self.reader
                .read_line(&mut self.buffer)
                .expect("Failed to read.");
            self.desk[i] = self.buffer.trim_end().chars().collect();
        }
        self.desk.reverse(); // Reverse the array so that the rest of the cycles do not go from below, but from above.
        writeln!(&mut self.output, "Game: {}", game_number);
    }

    pub fn clustering(&mut self) -> (i32, i32, char) {
        self.cluster_map = vec![vec![-1; 15]; 10];

        let mut max_cluster: (i32, i32, char) = (-1, -1, ' '); // (number, count of balls, color)
        let mut cluster_counter = 0;
        for j in 0..15 {
            for i in 0..10 {
                if self.cluster_map[i][j] == -1 {
                    let t = self.searching_clasters((i, j), cluster_counter);
                    if t.1 > max_cluster.1 && t.1 != 0 {
                        max_cluster = t;
                    }
                    cluster_counter += 1;
                }
            }
        }
        max_cluster
    }

    pub fn delete_cluster(&mut self, cluster_num: i32) -> (usize, usize) {
        let mut pos = (0, 0);

        let mut non_null_columns = vec![false; 15];
        for j in 0..15 {
            for i in 0..10 {
                if self.cluster_map[i][j] == cluster_num {
                    self.desk[i][j] = ' ';
                    if pos == (0, 0) {
                        pos = (i + 1, j + 1);
                    }
                }

                if !non_null_columns[j] && self.desk[i][j] != ' ' {
                    non_null_columns[j] = true;
                }
            }
        }

        for _ in 0..10 {
            for j in 0..15 {
                if !non_null_columns[j] {
                    continue;
                }
                for i in 0..10 - 1 {
                    if self.desk[i][j] == ' ' {
                        self.desk[i][j] = self.desk[i + 1][j];
                        self.desk[i + 1][j] = ' ';
                    }
                }
            }
        }

        for j in 0..15 - 1 {
            if !non_null_columns[j] {
                let mut new_index = j;
                for k in j..15 {
                    if non_null_columns[k] {
                        new_index = k;
                        break;
                    }
                }
                if new_index != j {
                    for i in 0..10 {
                        self.desk[i][j] = self.desk[i][new_index];
                        self.desk[i][new_index] = ' ';
                    }
                    non_null_columns[new_index] = false;
                }
            }
        }

        pos
    }

    pub fn read_count_of_games(&mut self) -> u32 {
        self.reader
            .read_line(&mut self.buffer)
            .expect("Error while read console.");
        self.buffer.trim().parse().expect("this is not an integer")
    }

    #[allow(unused_must_use)]
    pub fn make_move(&mut self, pos: (usize, usize), balls_count: i32, color: char) {
        let gotted_points = (balls_count - 2).pow(2);

        writeln!(&mut self.output, "Move {move_counter} at {pos}: removed {balls_count} balls of color {color}, got {gotted_points} points.",
        move_counter = self.move_counter,
        pos = format_args!("({x}, {y})", x = pos.0, y = pos.1));

        self.score += gotted_points as u32;
        self.move_counter += 1;
    }

    fn clear(&mut self) -> u32 {
        self.shift_column = vec![0; 15];

        let mut counter = 0u32;
        for i in 0..10 {
            for j in 0..15 {
                if self.desk[i][j] != ' ' {
                    counter += 1;
                    self.desk[i][j] = ' ';
                }
            }
        }
        self.cluster_map = vec![vec![-1; 15]; 10];

        self.move_counter = 1;
        self.score = 0;

        counter
    }

    #[allow(unused_must_use)]
    pub fn end_game(&mut self) {
        let mut final_score = self.score;
        let balls_counter = self.clear();

        if balls_counter == 0 {
            final_score += 1000;
        }

        writeln!(
            &mut self.output,
            "Final score: {0}, with {1} balls remaining.\n",
            final_score, balls_counter
        );
    }
}

fn main() {
    let file_name = env::args().nth(1).unwrap_or("input.txt".to_string());
    let mut game = Game::new(&file_name, "output.txt");
    let n = game.read_count_of_games();

    for i in 1..=n {
        game.start_game(i);

        let mut max_cluster = game.clustering();

        while max_cluster != (-1, -1, ' ') {
            let ball_pos = game.delete_cluster(max_cluster.0);
            
            game.make_move(ball_pos, max_cluster.1, max_cluster.2);

            max_cluster = game.clustering();
        }

        game.end_game();
    }
}
