AI 生成です。

# error

config.toml上の以下の記述に起因して、ビルドエラーが発生する可能性があります。
[profile.dev.package."*"]
codegen-backend = "llvm"

encoder_rsのsimd-accelは現バージョンではビルドエラーを引き起こすため、使用していません。

# LICENSE

[MIT License](LICENSE-MIT) or [Apache License 2.0](LICENSE-APACHE), at your option.
