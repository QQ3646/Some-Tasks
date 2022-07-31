use std::{
    env,
    fs::File,
    io::{BufRead, BufReader, Write},
};

/*
* Time: O(n),
* Memory:
*/

// For convenience, all io-protection operations in the structure
struct Data {
    buffer: String,
    input: BufReader<File>,
    output: File,
}

impl Data {
    fn new(input: &str, output: &str) -> Data {
        Data {
            buffer: String::new(),
            input: BufReader::new(File::open(input).expect("File read error.")),
            output: File::create(output).expect("File creation error."),
        }
    }

    fn collect_info(&mut self) -> Option<Vec<u32>> {
        self.buffer.clear();
        self.input
            .read_line(&mut self.buffer)
            .expect("Failed to read.");
        let v: Vec<u32> = self
            .buffer
            .trim_end()
            .split_whitespace()
            .map(|x| x.parse().expect("Parsing string error.")) 
            .collect();
        if v[0] == 0 {
            None
        } else {
            Some(v)
        }
    }

    fn read_from_console(&mut self) -> Option<u32> {
        self.buffer.clear();
        self.input
            .read_line(&mut self.buffer)
            .expect("File reading error.");
        let n = self
            .buffer
            .trim()
            .parse()
            .expect("File reading error.");
        if n != 0 {
            Some(n)
        } else {
            None
        }
    }
}

fn main() {
    let file_name = env::args().nth(1).unwrap_or("input.txt".to_string());
    let mut data = Data::new(&file_name, "output.txt");

    while let Some(n) = data.read_from_console() {
        while let Some(v) = data.collect_info() {
            let mut station: Vec<u32> = vec![];

            /* The role of station A is int, since the trains at this station are ordered
             * And the current state of the station can be determined by one variable. */
            let mut a = 1;

            for i in v.iter() {
                if a <= *i {
                    // Is there a right train at station A (2.1.)
                    station.append(&mut (a..*i).collect());
                    a = *i + 1;
                } else if i == station.last().unwrap_or(&0u32) {
                    // Is the correct number at the top of the stack?
                    station.pop();
                } else {
                    // None of the conditions matched -> this case is wrong, end the cycle
                    break;
                }
            }

            /*
             * Successful completion conditions:
             * 1. Trains ran out at station A
             *             AND
             * 2. There is nothing on the stack
             */
            #[allow(unused_must_use)]
            if a - 1 == n && station.is_empty() {
                writeln!(data.output, "Yes");
            } else {
                writeln!(data.output, "No");
            }
        }
    }
}
