// `use` は他のモジュール（クレートやファイル）の機能を取り込む宣言です。
use clap::Parser; // コマンドライン引数解析用クレート
use csv::ReaderBuilder; // CSVファイル読み込み用クレート
use rand::seq::SliceRandom; // スライスからランダムに要素を選ぶ機能
use rand::SeedableRng; // 乱数生成器のシード設定用
use std::error::Error; // 標準ライブラリのエラー処理用トレイト
use std::fs::File; // ファイル操作用
use std::path::PathBuf; // ファイルパス操作用
use std::process; // プロセス制御用（終了コードなど）

// `#[derive(...)]` は、指定されたトレイト（振る舞いの定義）を自動実装するマクロです。
// `Debug` はデバッグ出力用、`Parser` は clap クレートがコマンドライン引数を解析するために必要です。
#[derive(Parser, Debug)]
// `#[clap(...)]` は clap クレート固有の属性マクロで、コマンドラインツールの情報を定義します。
#[clap(author, version, about, long_about = None)]
struct Args {
    /// 使用する生徒リストCSVファイル (オプションなしで直接指定)
    #[clap(value_parser)] // 位置引数として設定
    input_file: Option<PathBuf>,

    /// 生徒リストCSVファイルへのパス (オプション)
    #[clap(short, long, value_parser, help = "生徒リストCSVファイルへのパス (オプション)")] // help を追加して明確化
    file: Option<PathBuf>,

    /// 乱数生成器のシード（テスト用）
    #[clap(long, value_parser)]
    seed: Option<u64>,
}

// `serde::Deserialize` は CSV からデータを構造体に変換（デシリアライズ）するために必要です。
// `Clone` はデータを複製可能にするトレイトです。
#[derive(Debug, serde::Deserialize, Clone)] // Cloneトレイトを実装
// `#[serde(deny_unknown_fields)]` は serde クレートの属性で、CSV に定義外の列があるとエラーにします。
#[serde(deny_unknown_fields)] // 不明なフィールドがあった場合に失敗するようにこの行を追加
struct Student {
    id: String,
    name: String,
}

// `main` 関数はプログラムのエントリーポイント（開始地点）です。
// `-> Result<(), Box<dyn Error>>` は、関数の戻り値の型を示します。
// `Result` は成功（`Ok`）か失敗（`Err`）を表す型です。
// `()` は成功時に値がないことを示します（Unit 型）。
// `Box<dyn Error>` は、任意の型のエラーを保持できる型です（トレイトオブジェクト）。
fn main() -> Result<(), Box<dyn Error>> {
    // `run()` 関数の結果を `if let` でパターンマッチングしています。
    // `Err(e)` であれば、エラー `e` を標準エラー出力に出力し、プロセスを終了します。
    if let Err(e) = run() {
        eprintln!("{}", e); // `eprintln!` は標準エラー出力へのマクロ
        process::exit(1); // 終了コード 1 でプロセスを終了
    }
    // エラーがなければ `Ok(())` を返し、正常終了します。
    Ok(())
}

// 実際の処理を行う関数。`main` と同じく `Result` を返します。
fn run() -> Result<(), Box<dyn Error>> {
    // `Args::parse()` は clap クレートの機能で、コマンドライン引数を解析して `Args` 構造体を生成します。
    let args = Args::parse();

    // ファイルパスの決定ロジックを修正
    let file_path = args.input_file // まず位置引数を確認
        .or(args.file) // 次に --file オプションを確認
        .unwrap_or_else(|| PathBuf::from("./students.csv")); // どちらもなければデフォルト

    // `canonicalize()` はパスを絶対パスに正規化しようとします。
    // 失敗する可能性があるので `unwrap_or_else` で元のパスを使います。
    let canonical_path = file_path.canonicalize().unwrap_or_else(|_| file_path.clone());

    // --- CSVファイル読み込み ---
    // `File::open` は `Result<File, io::Error>` を返します。
    // `?` 演算子は `Result` が `Ok(value)` なら `value` を、`Err(e)` ならエラー `e` を早期リターンします。
    // `.map_err(|e| ...)` は `Err` の場合にエラーの種類を変換します。ここでは詳細なエラーメッセージを生成しています。
    let file = File::open(&canonical_path).map_err(|e| {
         format!("Error: Could not open file '{}': {}", canonical_path.display(), e)
    })?;

    // `ReaderBuilder` で CSV リーダーの設定を行います。
    let mut rdr = ReaderBuilder::new()
        .has_headers(true) // ヘッダー行があると指定
        .flexible(false) // 列数が固定であることを指定
        .from_reader(file); // ファイルから読み込む
    // `rdr.deserialize()` は CSV の各行を `Student` 構造体にデシリアライズするイテレータを返します。
    // `.collect::<Result<_, _>>()` はイテレータの結果を `Vec<Student>` に集約します。
    // `Result<Vec<Student>, csv::Error>` のような型になります。
    // `_` は型推論に任せることを示します。
    // ここでも `map_err` でエラーメッセージを整形し、`?` でエラー処理をしています。
    let students: Vec<Student> = rdr.deserialize().collect::<Result<_, _>>().map_err(|e| {
        format!("Error: Failed to parse CSV file '{}': {}", canonical_path.display(), e)
    })?;

    // --- バリデーション ---
    // `students.is_empty()` でベクタが空かどうかをチェックします。
    if students.is_empty() {
       // `Err(...)` でエラーを生成し、`.into()` で `Box<dyn Error>` 型に変換して早期リターンします。
       return Err(format!("Error: The student list in '{}' is empty.", canonical_path.display()).into());
    }
    // `students.len()` でベクタの要素数を取得します。
    if students.len() < 2 {
       return Err(format!(
            "Error: Not enough students in '{}' to select two. Found {}.",
            canonical_path.display(),
            students.len()
        ).into());
    }

    // --- ランダム選択 ---
    // `match` 式で `args.seed` の値（`Option<u64>`）に応じて処理を分岐します。
    let mut rng = match args.seed {
        // `Some(seed)` なら、そのシード値で乱数生成器を初期化します。
        Some(seed) => rand::rngs::StdRng::seed_from_u64(seed),
        // `None` なら、OSのエントロピーソースからシードを取得して初期化します。
        None => rand::rngs::StdRng::from_entropy(),
    };

    // `students` ベクタ（実際にはそのスライス）から `rng` を使って重複なく 2 要素をランダムに選択します。
    // `choose_multiple` は要素への参照（`&Student`）のベクタを返すイテレータを生成します。
    // `collect::<Vec<_>>()` でそのイテレータの結果を `Vec<&Student>` に集約します。
    let chosen_students_refs = students
        .choose_multiple(&mut rng, 2) // `&mut rng` は可変の借用
        .collect::<Vec<_>>();

    // 選択された生徒の参照 (`&Student`) から、実際の `Student` データ をクローン（複製）して新しいベクタ `chosen_students` を作成します。
    // `.iter()` で参照のイテレータを取得し、`.map(|&s| s.clone())` で各参照 `&s` をデリファレンス（`*s`相当）して `clone()` し、
    // `.collect::<Vec<_>>()` で `Vec<Student>` に集約します。
    let chosen_students = chosen_students_refs.iter().map(|&s| s.clone()).collect::<Vec<_>>();

    // --- 出力 ---
    // `println!` マクロで標準出力に整形された文字列を出力します。
    // `{}` はプレースホルダーで、後の引数の値が挿入されます。
    println!("正担当: {} {}", chosen_students[0].id, chosen_students[0].name);
    println!("副担当: {} {}", chosen_students[1].id, chosen_students[1].name);

    // すべて成功した場合、`Ok(())` を返します。
    Ok(())
}
