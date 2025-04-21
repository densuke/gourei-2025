# ---- ビルドステージ ----
# マルチアーキテクチャ対応の Rust イメージを使用 (Lockfile v4 対応のため 1.78 以降)
FROM rust:1.78 as builder

# musl ビルドに必要なツールをインストール
RUN apt-get update && apt-get install -y musl-tools build-essential && rm -rf /var/lib/apt/lists/*

# ビルド環境のアーキテクチャを検出し、対応する Rust musl ターゲットを設定
# dpkg --print-architecture は Debian/Ubuntu ベースのイメージで利用可能
RUN ARCH=$(dpkg --print-architecture) && \
    case ${ARCH} in \
        amd64) RUST_TARGET=x86_64-unknown-linux-musl ;; \
        arm64) RUST_TARGET=aarch64-unknown-linux-musl ;; \
        *) echo "Unsupported architecture: ${ARCH}" >&2; exit 1 ;; \
    esac && \
    # 対応する musl ターゲットをインストール
    rustup target add ${RUST_TARGET} && \
    # 後続のステップで使えるようにターゲットトリプルをファイルに保存
    echo "${RUST_TARGET}" > /rust_target.txt

WORKDIR /usr/src/app

# 依存関係のキャッシュ (ターゲットを指定)
COPY Cargo.toml Cargo.lock ./
# ターゲットトリプルをファイルから読み込む
RUN RUST_TARGET=$(cat /rust_target.txt) && \
    mkdir src && echo "fn main() {}" > src/main.rs && \
    cargo build --release --target ${RUST_TARGET} --locked

# ソースコードをコピー
COPY src ./src

# 本番ビルド (ターゲットを指定)
# 古いダミーバイナリを削除
RUN RUST_TARGET=$(cat /rust_target.txt) && \
    rm -f target/${RUST_TARGET}/release/deps/gourei_touban* && \
    rm -f target/${RUST_TARGET}/release/gourei_touban
RUN RUST_TARGET=$(cat /rust_target.txt) && \
    cargo build --release --target ${RUST_TARGET} --locked

# 生成されたバイナリをビルドステージ内の固定パスにコピー
RUN RUST_TARGET=$(cat /rust_target.txt) && \
    mkdir -p /app && \
    cp target/${RUST_TARGET}/release/gourei_touban /app/gourei_touban

# ---- 実行ステージ ----
FROM scratch

# ビルドステージの固定パスからバイナリをコピー
COPY --from=builder /app/gourei_touban /gourei_touban

# エントリーポイントを設定
ENTRYPOINT ["/gourei_touban"]
# CMD ["./students.csv"] # デフォルト引数が必要な場合
