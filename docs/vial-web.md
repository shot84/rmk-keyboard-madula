# Madula — Vial Web

MadulaのLPPS / PMW3610 / IQS9151構成にVial Web対応ビルドを追加しました。
実機への書き込み、Web上の編集・保存・再起動後の保持はまだ確認していません。

## 接続

1. 使用モジュールに対応する新しいMadula中央機UF2を書き込みます。
2. 中央機をUSBでPCへ接続します。
3. ChromeまたはEdgeで https://vial.rocks/ を開き、接続操作からMadula RMKを選びます。
4. ロック解除を求められたら、Cornix左右の最上段外端（初期設定Tab＋Backspace）を同時に保持します。
5. 割り当てを変更し、実入力と再起動後の保持を確認します。

左右Peripheralは既存RMK版を使用できます。ZMK版Peripheralとは接続できません。
USB HID構成が変わるため、COM番号が変わる場合があります。BLE経由のWeb編集は未検証です。

## 表示

Cornix 50キーと左右エンコーダーに加え、Madulaの物理3ボタンを表示します。
その下の7キーはIQS用の仮想キーで、左/右/中央タップ、上/下/左/右スクロールの順です。
LPPS/TBではIQS用の仮想キーを使いません。センサー感度やLED設定の編集には非対応です。

右DialのUser20/21/22も選択可能です。
キーマップ編集内容はストレージへ保存する構成ですが、実機での保存確認は未実施です。
TOMLの初期値より既存の保存済み割り当てが優先される場合があります。

## ビルド

scripts/build-release.shのmadula-lpps / madula-tb / madula-iqsで自動的に
madula-vial featureを追加します。直接cargoを使う場合もmadula-vialを指定します。

keyboard.tomlを元にビルド時にホスト設定だけをVial有効へ変更するため、
キーマップ・ピン・保存領域の二重管理を避けています。
配列定義はmadula-vial.jsonです。CornixTPとは別の固定モデルUIDを使用します。
旧構成をビルドする場合はcargoのmadula-vial featureを省略します。

参考:
- https://get.vial.today/
- https://rmk.rs/docs/features/vial_support
