# Cornix + Madula — RMK firmware, review build by shot84 (not official)

Based on te9no/rmk-keyboard-madula v0.1.0-rc.7 (`b00a275`), branch `review-rc7`.
The BLE, Dial and touchpad improvements are in rc.7 itself; this build only adds:

- My own 8-layer keymap (tap-hold profiles, left wheel reversed, three-finger swipes for macOS).
- Touchpad vertical scroll in the ZMK driver's direction (natural on macOS, the opposite way on Windows).

Tested on hardware with the touchpad build (`madula-iqs`) only; the LPPS and trackball builds are built but untested.
CI-built UF2 checksums are listed in SHA256SUMS and are not the same files as the ones tested locally.

Below: te9no's original rc.7 release notes, unchanged.

---

# v0.1.0-rc.7 — Dial再接続・BLE入力・IQSジェスチャー改善

shot84さんの修正を機能単位で取り込みました。

- [PR #1](https://github.com/te9no/rmk-keyboard-madula/pull/1): DialのBLE通知購読設定を保存し、再接続後に動かなくなる問題を修正。
- BLEマウス報告を集約し、接続間隔（最低15ms）で送信。8ms前の移動を補間し、報告の滞留を抑制。
- IQS9151の指本数判定・タップ誤検出対策、タップドラッグ、3本指スワイプ/ドラッグ、速度・フィルタ等の調整キーを追加。
- 指離れ対策の切替時に古い移動が再出力される問題、フィルタ書き込み失敗時に再試行されない問題、保持バッファの最大深さでの範囲外アクセスを修正。

## ファイルの選択

装着モジュールに合わせて中央機用UF2を選んでください。

| モジュール | Vial版 | Rynk版（gui.rmk.rs） |
| --- | --- | --- |
| LPPS | madula-lpps.uf2 | madula-lpps-rynk.uf2 |
| PMW3610 / TB | madula-tb.uf2 | madula-tb-rynk.uf2 |
| IQS9151 | madula-iqs.uf2 | madula-iqs-rynk.uf2 |

既存のRMK版Cornix左右はそのまま使用できます。CornixTPには専用のcornix-tp.uf2を使います。
IQSのジェスチャー改善はMadula IQS版が対象です。

## 設定・注意点

- 元の5レイヤー・既存キー割り当て・スクロール方向・保存領域/スキーマID・ホストロック設定を維持。個人用配列やVial常時アンロックは取り込んでいません。
- 3本指スワイプの仮想キー4個（上/下/左/右、行4列10〜13）は既定で未割り当てです。Vial/Rynkで割り当ててください。
- IQSの既定値はカーソル等倍、フィルタbeta 40、指離れ対策3フレーム（約33ms）、タップドラッグ有効です。指離れ対策には遅延とのトレードオフがあります。
- 調整値はRAMのみで、再起動すると既定値へ戻ります。
- rc.6のRynk v0.2互換対応を維持。拡張Action/PauseForRelease等のマクロ制限も継続します。

[操作・調整キー一覧](https://github.com/te9no/rmk-keyboard-madula/blob/v0.1.0-rc.7/docs/input-enhancements.md) · [Rynk接続手順](https://github.com/te9no/rmk-keyboard-madula/blob/v0.1.0-rc.7/docs/rynk.md)

## 検証範囲

設定テスト19件、IQSジェスチャー/回帰テスト24件、通信形式・マクロ互換性テスト、全9構成のCIビルドを実施。
この統合版の実機USB/BLE操作、再接続、再起動後の保存確認は未実施です。

元の機能改善: [4685a71](https://github.com/shot84/rmk-keyboard-madula/commit/4685a7121abcb4505efdd14697dd593d976b6570) / [c179589](https://github.com/shot84/rmk-keyboard-madula/commit/c179589c0780eed907638c606017f08afd70a9c5)

UF2は0x1000から配置するRMK SDCアプリで、旧SoftDeviceを上書きします。ZMKアプリの更新ファイルではありません。
ELFはデバッグ用、JSONはソースコミットとハッシュ、SHA256SUMSは配布物の検証用です。デバイスのバックアップは含みません。
