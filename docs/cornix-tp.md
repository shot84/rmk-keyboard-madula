# CornixTP / TPS43 RMK

Cornix左右をBLE Peripheral、TPS43を接続したXIAO nRF52840をCentralとして使用する実験版。
RMK 0.9.0と既存のCornix左右実装を共有し、中央機は `cornix-tp` 専用ターゲットです。

## 配線

既存ZMKの `boards/shields/cornix_tps43_central/cornix_tps43_central.overlay`
およびREADME（commit `0ac2e02f3f6599f57ab49db175dd74b582f662a7`）の現行配線を参照。GPIO番号を優先します。

| TPS43 | nRF52840 |
| --- | --- |
| SDA | P0.28 |
| SCL | P0.05 |
| RDY (active high) | P0.29 |
| NRST (active low) | P0.04 |
| VDDHI / GND | 3V3 / GND |

I2Cは0x74・400kHz。Madulaの3キー、外付けLED、電池分圧回路は初期化しません。
内蔵RGB状態表示と電池残量通知は本版では未実装です。

## 入力

- Cornixの50キー・5レイヤー、左スクロール、右USB/BLE Dialは既存RMK設定を継承。
- TPS43の1本指移動、1本指左タップ、2本指右タップ、250msホールドドラッグ、2本指縦横スクロール。
- カーソルは既存ZMKと同じX=-sensor Y、Y=-sensor X。スクロールも既存のnatural-scroll＋回転設定に対応。
- スクロールは32カウント/ステップ。低速の端数を保持。
- センサー通信失敗時はドラッグを解放してハードウェアリセット・再初期化。
- ストリーミングで指離しを検知。低消費電力調整、ピンチ、慣性スクロール、DYA Studioは未対応。

`cornix-tp.toml`は独立した設定です。`cornix-tp` featureを指定するとbuild.rsが選択します。
中央機に物理キーはなく、row 4の3..9はジェスチャー用仮想キーです。
既存Madula用のrow 4の0..2は互換性維持のため予約し、GPIOには接続しません。

## ビルド

通常のRust 1.94.1、ARM GCC、libclang環境:

```sh
bash scripts/build-release.sh cornix-tp
bash scripts/build-release.sh cornix-left
bash scripts/build-release.sh cornix-right
```

Dockerで中央機のコンパイルを確認する場合:

```sh
bash scripts/build-docker.sh --bin cornix-tp --features cornix-tp
```

配布ファイルはdist/cornix-tp.uf2、cornix-left.uf2、cornix-right.uf2の3台分です。
左右は通常のRMK Cornix Peripheralと共通です。ZMK Peripheralとは接続できません。
別のCentralから切り替える場合の再ペアリングは実機確認が必要です。

## 検証と移行

2026-10-18: Rust 1.94.1のDockerビルドで中央機・左右のUF2生成に成功。
既存Madula IQS版もビルド成功。Python 17件＋TPS43 Rust 6件の計23件が成功。
Vial配列定義のXZ展開一致・UF2元バイナリへの埋め込み・固定UIDも確認済み。

ホスト単体テストは符号・回転、スクロール端数、タップ/ホールド、リセット/ATI/パーム、
GPIO競合と左右の設定互換性を確認します。実機での入力・方向・ドラッグ解除・
USB/BLE切り替え・長時間動作はまだ確認していません。

UF2のアプリ開始位置は0x1000で、旧SoftDevice領域を書き換えます。
既存の移行手順と復旧準備は[release-notes.md](release-notes.md)を参照してください。
2026-10-18に中央機・左右へ書き込み済み。旧中央機のCURRENT.UF2とSoftDevice復旧UF2、左右のCURRENT.UF2をtarget/backupsへ保存しました。


## Vial

CornixTPはVialを有効化しています。中央機のUSB/BLE通信経路にHostServiceを接続し、
編集内容はRMKのフラッシュストレージへ保存します。左右PeripheralのVial対応は不要です。
まずUSBでXIAO中央機をPCへ接続し、Vialアプリで `CornixTP RMK` を選択してください。
USB実機でVial protocol 6、固定UID、配列定義688 bytesの読み出しとソース一致を確認しました。編集・再起動後の保存確認、およびBLEでのVial接続は未検証です。

- 50キー、5レイヤーと左右2個のエンコーダーの回転を編集可能な定義を埋め込み。
- 最下段の6キーは左から「左タップ/ドラッグ、右タップ、上/下/左/右スクロール」。
  物理スイッチではなくセンサー操作への割り当てです。
- ロック解除を求められたら、左端・右端の最上段キー（初期設定Tab + Backspace）を同時に保持。
- 右Dialの既定割り当てはUser20（反時計回り）、User21（時計回り）、User22（押下）。
- Vialの変更は保存済みキーマップに反映されます。TOMLを変更して再書き込みしても、
  保存済み割り当てが優先される場合があります。
- 感度・スクロール倍率・タップ判定時間はVialから変更できません。

`vial.json`は実際の5×14マトリクスに対応し、XZ圧縮してビルド時に埋め込みます。
VialのモデルUIDは固定です。
CornixTPだけをVial有効にするため、RMKマクロの親feature転送判定を
`vendor/rmk-macro`に局所修正しています。Madula側のVial設定は変更していません。

参考: [RMK Vial](https://rmk.rs/docs/features/vial_support)、
[Vial encoder definition](https://get.vial.today/docs/encoders.html)。

## 2026-10-18 書き込み確認

- 中央機XIAO: BD22A5FE5A56CF37、RMK CDC COM127。
- Cornix左: E5082D6AF5183F1B、右: 24A229C615D2D780。
- 両PeripheralへのBLE接続成立を中央機ログで確認。
- 初回のI2C AddressNackを修正。Embassy nRF 0.11 Twim::newの引数はSDA、SCLの順。
- 修正版を書き込み後、取得したログにTPS43初期化エラーなし。カーソル・タップ・スクロールの操作確認はユーザー確認待ち。
- USB Vial定義を実機から全ページ取得し、XZ展開後のvial.json一致を確認。
