use std::{env};
use std::path::Path;
use std::fs::File;
use std::io::{ErrorKind, stdin, stdout, Write};
use regex::Regex;
use rand::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

const NUMBERS: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

const BUFFER_SIZE: usize = 1000000;
const PROGRESS_BAR_LENGTH: usize = 20;

fn main() {

    // Gathers the program arguments
    let program_args: Vec<String> = env::args().collect();

    // Tests whether the program arguments fit the required scheme
    let conf = check_arguments_for_file_config(&program_args);

    match conf {
        // If the arguments are in the right format ...
        Ok(conf) => {
            println!("File with name \"{}\" and size {}B will be created.", conf.0, conf.1);

            let filename = conf.0;
            let filesize = conf.1;
            let overwrite_all = conf.2;

            let filepath = format!("./{}", filename);
            let path = Path::new(&filepath);

            // If the file already exists and the user didn't specify to always override ...
            if path.is_file() && !overwrite_all {
                println!("The file you are trying to create already exists. Overwrite? [Y/n]");
                let response = read_line();
                let response = response.trim();
                let pattern = Regex::new("^[yYjJ]$").unwrap();

                if !pattern.is_match(&response) && !response.is_empty() {
                    println!("Aborting...");
                    return;
                }
            }

            match File::create(path) {
                Ok(file) => {

                    let start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
                    write_random_bytes(file, filename, filesize);
                    let end = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();

                    let millis = end - start;
                    let secs = millis / 1000;
                    println!("Time taken: {}s {}ms", secs, millis);
                },
                Err(e) => {
                    handle_io_error(e.kind(), filename);
                },
            };
        },
        // If there was an error ...
        Err(e) => {
            if e == 0 {
                println!("The entered arguments do not provide sufficient information about the file.")
            } else {
                println!("The number you entered is too big.");
            }
        },
    }

}

fn check_arguments_for_file_config(args: &Vec<String>) -> Result<(&str, usize, bool), usize> {
    let len = args.len();
    
    if len != 3 && len != 4 {
        return Err(0); // Wrong argument format
    }

    let mut name = "";
    let mut size = "";
    let mut overwrite_always = false;

    if len == 3 {
        name = &args[1];
        size = &args[2];
    } else if args[1] == "-o" {
        overwrite_always = true;
        name = &args[2];
        size = &args[3];
    }

    if !is_numeric_positive(&size) {
        return Err(0); // Wrong argument format
    }
    
    // Tries to parse the size into a number
    return match size.parse() {
        Ok(size) => Ok((name, size, overwrite_always)),
        Err(_) => Err(1), // The number is too big to be parsed into an integer
    };
    
    
    
}


fn is_numeric_positive(string: &str) -> bool {
    let mut contains_only_zero = true;
    for c in string.chars() {
        if !NUMBERS.contains(&c) {
            return false;
        }
        if c != '0' {
            contains_only_zero = false;
        }
    }
    return !contains_only_zero;
}

fn read_line() -> String {
    let mut buf = String::from("");
    stdin().read_line(&mut buf).unwrap();
    return buf;
}

fn write_random_bytes(mut file: File, filename: &str, size: usize) {
    let mut counter: usize = 0;
    while counter < size {
        match file.write(&random_value_array(BUFFER_SIZE.min(size - counter))) {
            Ok(written) => {
                counter += written;
                print_progress(counter, size);
            },
            Err(e) => {
                handle_io_error(e.kind(), filename);
                return;
            }
        };
    }
    println!();
}

fn random_value_array<'a>(size: usize) -> Vec<u8> {
    let mut vec = vec![0 as u8; 0];

    let mut rng = thread_rng();

    for _ in 0..size {
        vec.push(rng.gen_range(0..255) as u8);
    }
    return vec;
}

fn handle_io_error(e: ErrorKind, filename: &str) {
    match e {
        ErrorKind::NotFound => {
            println!("The file \"{}\" could not be created.", filename)
        },
        ErrorKind::PermissionDenied => {
            println!("Missing privileges to write to \"{}\".", filename)
        }
        _ => {
            println!("An unexpected error occurred: {}", e.to_string())
        }
    }
}

fn print_progress(current: usize, max: usize) {
    let fraction = current as f32 / max as f32;

    let percent = (fraction * 100.0).ceil() as usize;


    let filled_space_count: usize =
        (fraction * PROGRESS_BAR_LENGTH as f32).ceil() as usize;
    let empty_space_count = PROGRESS_BAR_LENGTH - filled_space_count;

    let mut filled_spaces = String::from("");
    for _ in 0..filled_space_count {
        filled_spaces.push('*')
    }

    let mut empty_spaces = String::from("");
    for _ in 0..empty_space_count {
        empty_spaces.push(' ')
    }

    print!("\rWriting: |{}{}| - {} %", filled_spaces, empty_spaces, percent);
    stdout().flush().unwrap();
}