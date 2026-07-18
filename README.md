AI 生成です。
AI generated content.

# Features

windows

win-reactor-ui:
cargo run

gpui-ui:
cargo run --no-default-features --features gpui-ui

winio-ui:
cargo run --no-default-features --features winio-ui

# error

config.toml上の以下の記述に起因して、表示エラーが発生する可能性があります。
[profile.dev.package."*"]
codegen-backend = "llvm"

encoding_rsのsimd-accel featureは現バージョンではビルドエラーを引き起こすため、使用していません。

# LICENSE

[MIT License](LICENSE-MIT) or [Apache License 2.0](LICENSE-APACHE), at your option.
