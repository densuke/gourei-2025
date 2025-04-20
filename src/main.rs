use clap::Parser;
use csv::ReaderBuilder;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use std::error::Error;
use std::fs::File;
use std::path::PathBuf;
use std::process;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to the student list CSV file. Defaults to ./students.csv
    #[clap(short, long, value_parser)]
    file: Option<PathBuf>,

    /// Seed for the random number generator (for testing)
    #[clap(long, value_parser)]
    seed: Option<u64>,
}

#[derive(Debug, serde::Deserialize, Clone)] // Added Clone
#[serde(deny_unknown_fields)] // Add this line to fail on unknown fields
struct Student {
    id: String,
    name: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    if let Err(e) = run() {
        eprintln!("{}", e);
        process::exit(1);
    }
    Ok(())
}

fn run() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    // Use canonicalize to handle relative paths more robustly, especially for the default.
    // Fallback to the original path if canonicalization fails (e.g., file doesn't exist yet).
    let file_path = args.file.unwrap_or_else(|| PathBuf::from("./students.csv"));
    let canonical_path = file_path.canonicalize().unwrap_or_else(|_| file_path.clone());

    // --- CSV File Reading ---
    let file = File::open(&canonical_path).map_err(|e| {
         format!("Error: Could not open file '{}': {}", canonical_path.display(), e)
    })?;

    // Configure ReaderBuilder: disable flexible mode to enforce column count
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .flexible(false) // Add this line
        .from_reader(file);
    let students: Vec<Student> = rdr.deserialize().collect::<Result<_, _>>().map_err(|e| {
        format!("Error: Failed to parse CSV file '{}': {}", canonical_path.display(), e)
    })?;

    // --- Validation ---
    if students.is_empty() {
       return Err(format!("Error: The student list in '{}' is empty.", canonical_path.display()).into());
    }
    if students.len() < 2 {
       return Err(format!(
            "Error: Not enough students in '{}' to select two. Found {}.",
            canonical_path.display(),
            students.len()
        ).into());
    }

    // --- Random Selection ---
    let mut rng = match args.seed {
        Some(seed) => rand::rngs::StdRng::seed_from_u64(seed),
        None => rand::rngs::StdRng::from_entropy(),
    };

    // choose_multiple returns a Vec<&Student>, so clone is needed if Student owns data.
    let chosen_students_refs = students
        .choose_multiple(&mut rng, 2)
        .collect::<Vec<_>>();

    // Clone the selected students to own the data before printing
    let chosen_students = chosen_students_refs.iter().map(|&s| s.clone()).collect::<Vec<_>>();

    // --- Output ---
    // spec.md の「6. その他」に基づき、最初に選ばれた学生を正担当とする
    println!("正担当: {} {}", chosen_students[0].id, chosen_students[0].name);
    println!("副担当: {} {}", chosen_students[1].id, chosen_students[1].name);

    Ok(())
}
