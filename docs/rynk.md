# Madula Rynk — RMK GUI対応

v0.1.0-rc.6は https://gui.rmk.rs のRynk v0.2通信形式に対応します。
rc.5以前のRynk版はv0.1のため「Firmware and app versions don’t match」で接続に失敗します。

| モジュール | 書き込む中央機ファイル |
| --- | --- |
| LPPS | madula-lpps-rynk.uf2 |
| PMW3610 / TB | madula-tb-rynk.uf2 |
| IQS9151 | madula-iqs-rynk.uf2 |

VialとRynkは同時利用できません。末尾に`-rynk`のないMadula UF2はVial版です。
左右Peripheralは既存のRMK版を使えます。ZMK版とは接続できません。

## 接続

1. 装着モジュールに合う上記UF2をMadula中央機に書き込みます。
2. Chrome/Edgeで https://gui.rmk.rs を開きます。
3. USB接続ならUSB、Bluetooth接続なら対応する接続方式を選び、Madula RMKを接続します。

RMK GUIはブラウザーUSB（WebUSB）にも対応しています。以前の説明の「WebはBLEのみ」は
上流の参考デモの制限であり、このGUIには当てはまりません。

## 対応仕様と制限

GUIが固定しているRMKリビジョン `933b0630a23393961e5b3bfa7a176a68d9bb8b8c`
の通信型を移植しています。機能一覧には`max_macros` / `macro_max_size` / `macros_writable`
が含まれ、マクロはオフセット指定ではなくスロット単位で読み書きします。
開発中のプロトコルなので、単にv0.2と表示される別リビジョンとの互換性は保証しません。
検証したGUIファイルのURL・SHA-256はルートの`rynk-compat.json`に記録しています。

既存の保存形式・保存領域・スキーマIDは変更していません。キーマップ、ペアリング、
LED表示、Dial処理を維持する構成です。
マクロは既存エンジンの形式に変換して保存します（32スロット、1マクロ32操作まで、全体の既存バッファ容量内）。
通常のHIDキーのTap/Press/Release、ASCII文字、0〜65024msのDelayに対応します。
PauseForRelease、レイヤー操作などの拡張Action、上限超過はエラーにし、既存データを変更しません。
既存の拡張Vialマクロや32操作超のマクロがある場合は、Rynkからの読み出しもエラーになります。
その場合はVial版で編集してください。Vial版のマクロ処理は変更していません。

通常のキーマップ編集はロック解除不要です。危険な操作の解除には
Tab＋Backspaceの物理位置（左右最上段外端）を保持します。`insecure`は無効です。

## 検証範囲

- 公開GUIの実WASMとホスト上の通信テスト用サーバーで、ハンドシェイク・機能一覧・機器情報・
  キー読み書き・マクロ読み書き・不正マクロ拒否・BLE状態取得を確認。
- 上流の通信バイト列スナップショットと型のテスト、保存形式変換の境界値テストを実施。
- LPPS/TB/IQS Rynk版、およびVial版をビルド。
- この修正版の実機USB/BLE接続・再起動後の保存確認は未実施。

通信テストの再実行（Linux / Rust / Node.jsが必要）:

~~~sh
python3 scripts/test-rynk-gui.py
~~~

検証時の公開GUIファイルを取得し、SHA-256を確認して実行します。旧v0.1では元のエラー、
修正後のv0.2では成功することを同じクライアントで検証します。実機にはアクセスしません。

## ビルド

~~~sh
bash scripts/build-release.sh madula-lpps-rynk
bash scripts/build-release.sh madula-tb-rynk
bash scripts/build-release.sh madula-iqs-rynk
~~~

上流の参考デモをローカルで使う場合は、同じ通信リビジョンに固定した次のスクリプトを使用します。
このデモ自体のビルドは未検証です。通常は公開RMK GUIを使ってください。

~~~sh
cargo install wasm-pack --locked
bash scripts/build-rynk-web.sh
~~~
