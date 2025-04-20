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
    /// 生徒リストCSVファイルへのパス。デフォルトは ./students.csv
    #[clap(short, long, value_parser)]
    file: Option<PathBuf>,

    /// 乱数生成器のシード（テスト用）
    #[clap(long, value_parser)]
    seed: Option<u64>,
}

#[derive(Debug, serde::Deserialize, Clone)] // Cloneトレイトを実装
#[serde(deny_unknown_fields)] // 不明なフィールドがあった場合に失敗するようにこの行を追加
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
    // 相対パス、特にデフォルトパスをより堅牢に扱うために canonicalize を使用します。
    // canonicalize が失敗した場合（例：ファイルがまだ存在しない場合）は、元のパスにフォールバックします。
    let file_path = args.file.unwrap_or_else(|| PathBuf::from("./students.csv"));
    let canonical_path = file_path.canonicalize().unwrap_or_else(|_| file_path.clone());

    // --- CSVファイル読み込み ---
    let file = File::open(&canonical_path).map_err(|e| {
         format!("Error: Could not open file '{}': {}", canonical_path.display(), e)
    })?;

    // ReaderBuilderの設定: flexibleモードを無効にして列数を強制します
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .flexible(false) // この行を追加
        .from_reader(file);
    let students: Vec<Student> = rdr.deserialize().collect::<Result<_, _>>().map_err(|e| {
        format!("Error: Failed to parse CSV file '{}': {}", canonical_path.display(), e)
    })?;

    // --- バリデーション ---
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

    // --- ランダム選択 ---
    let mut rng = match args.seed {
        Some(seed) => rand::rngs::StdRng::seed_from_u64(seed),
        None => rand::rngs::StdRng::from_entropy(),
    };

    // choose_multiple は Vec<&Student> を返すため、Student がデータを所有している場合は clone が必要です。
    let chosen_students_refs = students
        .choose_multiple(&mut rng, 2)
        .collect::<Vec<_>>();

    // 選択された生徒をクローンして、出力前にデータを所有します
    let chosen_students = chosen_students_refs.iter().map(|&s| s.clone()).collect::<Vec<_>>();

    // --- 出力 ---
    // spec.md の「6. その他」に基づき、最初に選ばれた学生を正担当とする
    println!("正担当: {} {}", chosen_students[0].id, chosen_students[0].name);
    println!("副担当: {} {}", chosen_students[1].id, chosen_students[1].name);

    Ok(())
}
