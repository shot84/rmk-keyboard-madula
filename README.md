# rmk-keyboard-madula

Cornix左右＋MadulaをRMK化する独立した実験用リポジトリ。
既存のZMKリポジトリ・公開ファームは変更しません。

## 現段階

- Madula: nRF52840 Central、直接接続の3キー、USB/BLEホスト接続
- Cornix左・右: nRF52840 BLE Peripheral、各4×7マトリクス
- 50＋3の物理位置を維持。初期キー入力確認用の2レイヤー
- ZMKのmod-tap、多層キーマップ、LayoutShiftはまだ移植していません
- TB(PMW3610)、LPPS(ADS1220)、IQS9151、左右エンコーダ、LED、電圧監視は未実装
- DYA StudioはZMK用のため、この版では利用できません。Vial等の編集対応も未実装
- ログはdefmt/RTT。CDC Debugと1200 baud bootは未実装

RMK 0.9.0を固定し、依存解決はCargo.lockで固定します。
参考: [RMK split](https://rmk.rs/main/docs/features/split_keyboard)、
[0.9.0の実装](https://github.com/rmk-rs/rmk/tree/986017d901c7609a9a79167686524f014effa68a)。
BLE splitの接続確認には3台ともRMK版が必要です。ZMKの左右とは接続できません。

## ビルド

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
設定テスト4件・rustfmtチェック成功。実機起動・BLE接続・入力は未確認です。

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
