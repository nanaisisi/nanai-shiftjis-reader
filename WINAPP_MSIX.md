# winapp CLI / MSIX パッケージ手順

このリポジトリでは実行ファイル名が `nanai-shiftjis-reader.exe` です。
`winapp` のサンプルにある `rust-app.exe` / `rust-app.msix` の名前ではなく、以下の手順を使ってください。

## 1. デバッグ ID を使用した実行

1. 実行可能ファイルをビルドします。

```powershell
cargo build
```

2. デバッグ ID を付与します。

```powershell
winapp create-debug-identity .\target\debug\nanai-shiftjis-reader.exe
```

3. 実行します。

```powershell
.\target\debug\nanai-shiftjis-reader.exe
```

4. 正常なら `Package Family Name:` が表示されます。

## 2. MSIX を使ったパッケージ化

1. リリースビルドを行います。

```powershell
cargo build --release
```

2. パッケージ用ディレクトリを準備します。

```powershell
mkdir dist
copy .\target\release\nanai-shiftjis-reader.exe .\dist\
```

3. 開発用証明書を生成します。

```powershell
winapp cert generate --if-exists skip
```

4. パッケージ化して署名します。

```powershell
winapp pack .\dist --cert .\devcert.pfx
```

5. 証明書をインストールします（管理者として実行）。

```powershell
winapp cert install .\devcert.pfx
```

6. MSIX をインストールします。

```powershell
Add-AppxPackage .\nanai-shiftjis-reader.msix
```

7. 必要に応じてアプリを起動します。

```powershell
nanai-shiftjis-reader
```

## 3. 注意事項

- 実稼働配布には自己署名証明書ではなく、正式なコード署名証明書を使用してください。
- Microsoft Store に提出する場合は、Store が署名するため提出前に署名する必要はありません。
- x64 / Arm64 など複数アーキテクチャをサポートする場合、各アーキテクチャごとに個別の MSIX パッケージが必要になる場合があります。
- `appxmanifest.xml` の `Executable` や `Identity` を変更した場合は、パッケージ化前に正しく反映されていることを確認してください。
