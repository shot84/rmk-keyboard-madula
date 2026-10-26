# Madula Rynk (experimental)

LPPS / PMW3610 / IQS9151の中央機にRynkを選択できます。
VialとRynkは同時利用できません。既存のVial版とは別ファイルにしています。

| モジュール | Rynk版 |
| --- | --- |
| LPPS | madula-lpps-rynk.uf2 |
| PMW3610 / TB | madula-tb-rynk.uf2 |
| IQS9151 | madula-iqs-rynk.uf2 |

左右Peripheralは既存のRMK版を使用します。ZMK版とは接続できません。
中央機のUSB/BLE HostServiceへRynkを接続し、保存領域とキーマップを引き継ぐ構成です。
実機接続・Web編集・再起動後の保存は未検証です。USB/BLEのHID構成が変わるため、
BLEホスト側で再ペアリングが必要になる場合があります。

## ビルド

~~~sh
bash scripts/build-release.sh madula-lpps-rynk
bash scripts/build-release.sh madula-tb-rynk
bash scripts/build-release.sh madula-iqs-rynk
~~~

直接cargoを使う場合は、モジュールfeatureとmadula-rynkを指定します。
madula-vial / cornix-tpと同時指定するとエラーになります。
通常のmadula-lpps / madula-tb / madula-iqsは引き続きVial版です。
配布JSONのhost_protocolでvial / rynkを識別できます。

## Webツール

Rynkは実験段階で、公式Web版はローカルビルドする参考実装です。
ブラウザーはChrome/Edge、接続はBLE経由のWebHIDです。
USBでRynkを使うにはネイティブのrynk-usbクライアントが必要です。
この追加はブラウザーUSB接続を実装するものではありません。

ファームウェアと同じrmk-types 0.4.0 / RMK 0.9.0の公開元リビジョン
a917b60a1f5d4fac50eca474635c774b13a13ebbを使用します。
このリビジョンのrynk-wasmは0.3.0です。最新mainへ無条件に更新しないでください。

~~~sh
cargo install wasm-pack --locked
bash scripts/build-rynk-web.sh
# スクリプトが表示するpython3のserveコマンドを実行
~~~

Chrome/Edgeで http://localhost:8000/ を開きます。
OSでMadulaをBluetoothペアリングし、接続した状態でページの接続操作を行います。
WebHIDの選択画面ではMadulaを選んでください。
上記Webツール自体のビルドと実機操作は今回未検証です。

通常のキー・レイヤー・コンボ・マクロ編集はRynkの既定権限に従います。
ロックされた操作を行う際は、Tab＋Backspaceの物理位置（左右最上段外端）を保持して解除します。
insecureは無効です。Vialとは異なり、両端末で同じプロトコルの世代を使う必要があります。

公式資料: https://rmk.rs/main/docs/features/rynk
参考Web実装: https://github.com/rmk-rs/rmk/tree/a917b60a1f5d4fac50eca474635c774b13a13ebb/rynk/rynk-wasm
