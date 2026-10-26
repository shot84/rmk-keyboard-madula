<div align="center">

# Cornix × Madula

**左右のキーと、手元のポインティングをひとつに。**

Cornix左右とMadulaをつなぐ、RMKベースのキーボードファームウェア。

[ダウンロード](https://github.com/te9no/rmk-keyboard-madula/releases) · [導入](#はじめに) · [ビルド](#ソースからビルド) · [開発資料](#開発資料)

</div>

## できること

Cornix左右をBLEでMadulaへ接続し、MadulaからPCへ入力を送ります。
本体の3キーと交換式入力モジュールを、同じキーマップで使えます。

- **53キー・5レイヤー** — Cornixの50キーとMadulaの3キー。Fn / Numなどのレイヤー操作に対応。
- **左右エンコーダー** — 上下スクロールを手元で操作。
- **選べるポインティング** — LPPS、PMW3610トラックボール、IQS9151タッチパッド。
- **状態を知らせるLED** — 接続状態、レイヤー番号の点滅、対応電池構成での残量目安。
- **USB CDC** — 各基板のログ取得と1200 baudリセットによるブートローダー移行。

RMK 0.9.0を使用し、依存関係は`Cargo.lock`で固定しています。
USB/BLEホスト接続に対応する構成ですが、BLEホスト接続を含む全条件での検証完了を示すものではありません。

## CornixTP / TPS43

CornixTP用の独立した中央機ターゲットを追加しました。Vialでキー・エンコーダー・ジェスチャーの割り当てを編集できます。
[配線・機能・ビルド方法](docs/cornix-tp.md)を参照してください。実機動作は未確認です。

## Rynk（実験版）

Madula向けにRynk版も追加しました。[ビルドとWebツールの手順](docs/rynk.md)を参照してください。
Vial版とは別のUF2で、ブラウザー接続はBLE経由です。実機検証は未実施です。

## 入力モジュール

| モジュール | 操作 | 配布状況 |
| --- | --- | --- |
| LPPS | 自由なXYカーソル移動。平滑化・デッドゾーン付き | リリースUF2あり |
| PMW3610 / TB | トラックボールによるカーソル移動 | リリースUF2あり |
| IQS9151 | カーソル、1/2/3本指タップ、2本指スクロール | v0.1.0-rc.3から配布対象 |

IQSのタップは順に左・右・中央クリック。2本指移動は縦横スクロールです。
ピンチ、タップドラッグ、慣性スクロールは対応していません。

**モジュール交換は電源を切ってから行い、対応するMadulaファームウェアを使用してください。**
LPPSは起動時に中立位置を校正するため、起動直後は触れずに待ってください。

## はじめに

### 1. ファームウェアを選ぶ

[GitHub Releases](https://github.com/te9no/rmk-keyboard-madula/releases)から、Cornix左右と使用するMadulaモジュールの**計3ファイル**を取得します。

| ファイル | 書き込み先 |
| --- | --- |
| `cornix-left.uf2` | Cornix左 |
| `cornix-right.uf2` | Cornix右 |
| `madula-lpps.uf2` | LPPSを装着したMadula |
| `madula-tb.uf2` | PMW3610を装着したMadula |
| `madula-iqs.uf2` | IQS9151を装着したMadula（rc.3以降） |

3台とも同じリリースのRMKファームウェアに揃えてください。ZMKの左右ファームウェアとは接続できません。
`v0.1.0-rc.*`はプレリリースです。rc.1にはIQS版が含まれません。
rc.2は配布設定の修正により公開を取り止めたビルドです。
ELFはデバッグ用、JSONはビルド情報、`SHA256SUMS`は整合性確認用です。

### 2. 移行の準備

> **通常のZMK更新とは異なります。**
> RMK版は0x1000から配置され、旧SoftDevice領域を上書きします。
> ZMKへ戻すにはSoftDeviceの復旧が必要になる場合があり、アプリUF2のバックアップだけでは十分ではありません。
> 設定・ペアリング情報にも影響するため、対応基板・ブートローダー・復旧手段を確認してから移行してください。

[配布時の注意事項](docs/release-notes.md)を確認し、現在のファームウェアと必要な復旧データを保存してください。
異なる基板向けのファームウェアは流用しないでください。

### 3. 書き込む

移行準備ができたら対象基板をUF2ブートローダーで起動し、対応するUF2をそのドライブへコピーします。
再起動後、MadulaとCornix左右を起動してキー入力を確認してください。
RMK版導入後は各基板のCDCから1200 baudリセットも利用できます。

Windows用の[`flash-xiao.ps1`](scripts/flash-xiao.ps1)はMadulaのXIAO向け補助スクリプトです。
対象COMのPNP ID、UF2、ブートローダーを検査し、`CURRENT.UF2`を退避してからコピーします。
**Cornixには使用しないでください。** 引数はスクリプトを参照し、`-WhatIf`で対象を確認できます。

## キーマップとLED

### Cornixのエンコーダ

左の回転はスクロール、右の回転はWindows Radial Controller（Surface Dial互換）です。
右の押し込みはDialボタンになり、長押しでWindowsのメニュー、短押しで選択を操作します。
全レイヤーで共通です。回転は1ノッチ18度を初期値としています。
USBとPCへのBLE接続の両方で実機動作を確認しています。
BLEのHID定義が変わるため、更新後にPC側で登録を削除して再ペアリングが必要になる場合があります。
Windowsのメニュー表示状態はファームから取得できません。
保存形式の互換IDを固定し、通常の再ビルドによるペアリング情報の初期化を防いでいます。
旧ファームで既に鍵が失われた場合は、更新後に一度再ペアリングしてください。

HID実装は[sekigon-gonnoc/zmk-driver-hires-dial](https://github.com/sekigon-gonnoc/zmk-driver-hires-dial)
を参考にしています。RMK 0.9.0はリポジトリ内の `vendor/rmk` に固定し、
DialのUSB/BLEレポートと送信経路を追加しています。

[`keyboard.toml`](keyboard.toml)でキー割り当てを管理します。
CornixのMadula用キーマップを基準に、base / func / num / bt / naviの5レイヤーを保持しています。
変更後は再ビルドが必要です。

- Madulaは**Vial Webでの編集に対応するビルド**を追加しました。[Web編集の手順](docs/vial-web.md)を参照してください。DYA Studioは非対応です。
- ZMKのLayoutShiftとBTプロファイル操作は未移植で、該当位置は無効にしています。
- naviへの到達キーは現在の標準割り当てにありません。

Madula本体の外付けLEDはUSB接続をシアン、BLEホスト接続を青、未接続を赤で短く表示します。
レイヤー変更時は番号に応じた白点滅を表示します（baseは点滅なし）。

Cornix左右の2連LEDはPWM＋EasyDMAで独立駆動します。
内側（pixel 0）はFn＝薄い青、Num＝紫、BT＝シアン、navi＝橙で、
そのレイヤーが有効な間点灯します。ホストからCaps Lock有効の通知を受けると白を優先します。
外側（pixel 1）はMadulaへの接続成立時に緑を1.5秒表示し、未接続中は3秒ごとに橙を短く表示します。
base・Caps Lock無効・接続正常時は消灯してLED電源を切ります。
こちらはPCとの接続ではなく、CornixとMadula間の接続表示です。
点灯・色の変更は約300ms、消灯は約600msで緩やかにフェードします。
Cornix側のLED対応には左右それぞれのファームウェア更新が必要です。

電池残量表示は、**NiMH 1本・1MΩ/470kΩ分圧の対応Bat基板**が対象です。
緑は50%超、橙は21～50%、赤は20%以下の目安です。
電圧からの推定であり、正確な残容量や充電状態を示すものではありません。
OSへのBLE残量通知は未対応です。

## USB CDCログ

調べたい基板をUSBで接続し、シリアルモニターを**115200 baud / 8N1 / フロー制御なし / DTR有効**で開きます。
各基板のログは個別に取得します。左右のログをMadulaへ集約する機能はありません。

1200 baudでDTRを500 ms以上有効にしてから落とすと、ブートローダーへ移行します。
通常の115200 baudでの開閉ではリセットしません。
入力イベントが含まれる場合があるため、パスワード入力中の収集や未確認のままの公開は避けてください。
起動初期・クラッシュ直前・バッファ容量を超えたログは取得できない場合があります。

## ソースからビルド

Rust 1.94.1、ARM GCC、libclangを用意します。

```sh
rustup toolchain install 1.94.1 --profile minimal --target thumbv7em-none-eabihf
python3 -m unittest discover -s tests

# Cornix左右
cargo +1.94.1 build --release --locked --bin cornix-left --bin cornix-right

# Madula：装着モジュールに合わせて1つ選択
cargo +1.94.1 build --release --locked --bin madula-central --features lpps,battery-audit,madula-vial
cargo +1.94.1 build --release --locked --bin madula-central --features pmw3610,battery-audit,madula-vial
cargo +1.94.1 build --release --locked --bin madula-central --features iqs9151,battery-audit,madula-vial
```

`lpps` / `pmw3610` / `iqs9151`は同時に有効化できません。
`battery-audit`は対応回路の電圧監視・残量推定を有効にする機能名です。
ELFは`target/thumbv7em-none-eabihf/release/`に生成されます。
UF2を含む配布用パッケージは[`build-release.sh`](scripts/build-release.sh)を参照してください（Cornix左右・LPPS・TB・IQS）。

Dockerでのビルドは[`build-docker.sh`](scripts/build-docker.sh)、GitHub Actionsでのビルドは
[`Firmware release`](.github/workflows/release.yml)を利用できます。
タグから作成されるリリースは下書き・プレリリースで、公開操作は別途必要です。

## 開発資料

基本入力・各モジュールの動作には実機確認がありますが、長時間動作や全電池条件など未検証の範囲もあります。
詳細な確認内容は開発資料に分けています。

- [IQS9151の実装・検証記録](docs/iqs9151-bringup.md)
- [配布ファイルと移行上の注意](docs/release-notes.md)
- [初期開発の記録（過去の情報）](docs/development-history.md)

## 謝辞

このファームウェアは、[RMK](https://github.com/rmk-rs/rmk)、[Embassy](https://github.com/embassy-rs/embassy)、
[CornixのZMK実装](https://github.com/te9no/zmk-keyboard-cornix)を基に構築しています。
IQS9151の初期化設定は[ShiniNetのドライバー](https://github.com/ShiniNet/zmk-driver-iqs9151)とAzoteqの設定資料を、
PMW3610の通信はRMKの実装と[3-wire SPIドライバー](https://github.com/te9no/zmk-driver-spi-three-wire)を参照しています。
各プロジェクトの作者・貢献者に感謝します。取り込んだコードのライセンス表記はリポジトリ内に保持しています。

Cornixの外付けLEDのPWM駆動は[numachang/cornix-rmk-custom](https://github.com/numachang/cornix-rmk-custom)を参考にしています。
