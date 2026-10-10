# rmk-keyboard-madula

Cornix左右＋MadulaをRMK化する独立した実験用リポジトリ。
既存のZMKリポジトリ・公開ファームは変更しません。

## 現段階

- Madula: nRF52840 Central、直接接続の3キー、USB/BLEホスト接続
- Cornix左・右: nRF52840 BLE Peripheral、各4×7マトリクス
- 50＋3の物理位置を維持。CornixリポジトリのMadula用5レイヤーを基準に移植
- SpaceのFn/Num長押し、EscapeのBT長押し、Ctrl/Escapeのmod-tapを設定（実機タイミングは未検証）
- LayoutShift、BTプロファイル操作は未移植。その位置は誤操作防止のためNoに設定
- TB(PMW3610)、LPPS(ADS1220)、IQS9151、左右エンコーダ、LED、電圧監視は未実装
- DYA StudioはZMK用のため、この版では利用できません。Vial等の編集対応も未実装
- 3台ともUSB CDC ACM Debugログを有効化（実機未確認）。1200 baud bootは未実装

RMK 0.9.0を固定し、依存解決はCargo.lockで固定します。
参考: [RMK split](https://rmk.rs/main/docs/features/split_keyboard)、
[0.9.0の実装](https://github.com/rmk-rs/rmk/tree/986017d901c7609a9a79167686524f014effa68a)。
BLE splitの接続確認には3台ともRMK版が必要です。ZMKの左右とは接続できません。

## USB CDC Debug

RMK 0.9.0標準の`usb_log`で、Madula CentralはUSB HIDとCDCを共存させ、
Cornix左右はそれぞれUSB CDCでローカルのRMKログを出す構成です。
RMKの`defmt`機能とは排他のため、RMKログは`log`経由に切り替えています。
依存ライブラリ・panic用のdefmt/RTTは残しており、それらのログはCDCへ転送されません。

実機起動を確認した後、調査する基板をUSB接続し、そのCOMポートをシリアルモニターで
開いてください（例: 115200 baud / 8N1、フロー制御なし、DTR有効）。
CDCは仮想シリアルで、指定baudはUART通信速度を設定するものではありません。
ログには起動後のミリ秒とレベルが付き、Debugまでを出力します。
左右のログをCentralへ集約する機能ではありません。各基板へのUSB接続が必要です。

- USB初期化前のログ、クラッシュ直前のログの取得は保証しません。
- 1 KiBバッファを超えるログは欠落し得ます。常時トレースの保存用途ではありません。
- 入力イベントがログに含まれ得るため、パスワード入力中の収集やログの無確認公開は避けてください。
- 1200 baudリセット、CDCコマンド、DYA Studio接続は提供しません。
- COM列挙・再接続・HID/BLEとの同時動作は、まだ実機未検証です。

参考: [RMK USB Logging](https://rmk.rs/v0.7.8/guide/features/usb_logging)、
[RMK 0.9.0の機能定義](https://docs.rs/crate/rmk/0.9.0/features)。

## ビルド

### キーマップの参照元

[te9no/zmk-keyboard-cornix の config/madula.keymap](https://github.com/te9no/zmk-keyboard-cornix/blob/1ca91122e8f91f773ecee9fe97af8f6f1aacd329/config/madula.keymap)
を基準に、base / func / num / bt / navi の順序と53キー位置を維持します。
通常のcornix.keymapとは親指のレイヤー番号が異なるため、Madula用を採用しています。
マウスキーの方向は移植していますが、速度・タップ判定はRMKの動作になります。
参照元のnaviは到達用キーがなく、内部のMO(7)も未定義レイヤーを指すため、
勝手に新しい操作を追加せず、naviを保持したうえでMO(7)の位置はNoにしています。
左右エンコーダのsensor-bindingsは、エンコーダ実装時に移植します。

Rust 1.94.1、ARM用GCC、libclangが必要です。Ubuntuでは
gcc-arm-none-eabi / libnewlib-arm-none-eabi / libclang-devが相当します。
Zephyr SDKを流用する場合はCC_thumbv7em_none_eabihfと
AR_thumbv7em_none_eabihfでクロスコンパイラを指定します。
このワークスペースでは、既存zmk-workspace-devイメージを利用する
`bash scripts/build-docker.sh`も使用できます（Rust本体はホスト側）。

```sh
rustup target add thumbv7em-none-eabihf
python3 -m unittest discover -s tests
cargo build --release --locked --bins
```

成果物はtarget/thumbv7em-none-eabihf/release/配下の
madula-central、cornix-left、cornix-right（ELF）。

2026-10-10: Rust 1.94.1、Docker内Zephyr SDK＋libclangで
`cargo build --release --locked --bins`成功（3/3）。
Madula用5レイヤー＋USB CDCログで3台分を再ビルド成功、設定テスト6件成功。
実機起動・BLE接続・入力は未確認です。

## 書き込み禁止の段階

現時点のELFはコンパイル検証用です。UF2生成・配布・書き込みは行いません。
3台それぞれのブートローダー、SoftDevice/MBR、アプリ起動番地と
書き戻し手順の確認が必要です。memory.xは候補であり、起動実証済みではありません。
RMKの保存領域0xE0000–0xEFFFFは既存ZMKの設定領域と重なります。
書き込み試験には設定・ペアリング消失のリスクがあり、事前承認が必要です。
USB VID/PIDもRMK例の開発値であり、公開製品向けの割り当てではありません。

## 配線の根拠と次の作業

ZMKソースのcornix_left_common.dtsi、cornix_right.dts、
madula_central.overlayを参照。左右はcol2row、行入力PullDown、
Madula直接キーはactive-low/PullUp。右側のカラム反転はlayout.mapで保持します。
参照ローカルソース: te9no/zmk-keyboard-cornix、9cfab7e。

次は起動領域の監査→3台のキー・BLE接続試験→ポインティング移植。
PMW3610の3-wire配線は一般的な4-wire SPIドライバをそのまま当てないこと。
LEDの電源制御も有効極性を確認してから追加します。
