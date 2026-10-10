# rmk-keyboard-madula

Cornix左右＋MadulaをRMK化する独立した実験用リポジトリ。
既存のZMKリポジトリ・公開ファームは変更しません。

## 現段階

2026-10-10更新: XIAO上でMadula Central通常版（設定保存有効）のUSB CDC列挙を確認。
これまでの起動失敗はBLE名`Madula RMK experimental`が22バイト上限を超え、
RMKのGATTサーバー生成でpanicしたことが原因でした。`Madula RMK`へ短縮済みです。
panicをRAMへ保存して読み出した実機ログで特定しました。
以下の調査履歴にある保存処理・起動順序の疑いは原因確定前の仮説です。
Cornix左右へRMKを書き込み、各CDCとCentralへのBLE接続を確認しました。
左のキーイベントがCentralへ届くこともログで確認済みです。
左右の基本入力はユーザー確認済みです。全キー・ポインティング・長時間動作は未確認です。
修正後の通常版から1200 baudリセット→UF2ドライブ→同じ通常版の再書き込み→
COM復帰も実機確認済みです。現在XIAOにはこの通常版が入っています。
CDCで`Start scanning peripherals`と10秒・20秒の稼働ログ
（`storage=enabled`, `boot_audit=false`）も確認済みです。

左右同時接続時には別の再起動問題が発生しました。
RMK 0.9のスリープ通知はBLE接続管理とPeripheralManagerが各1枠を使うため、
2台で4枠必要ですが、display無効時の生成値は3枠でした。
`[event.sleep_state] subs = 2`を指定し、機能別追加分と合わせて4枠を確保しています。
回帰テストを追加し、通常版をCentralへ再書き込みしました。
左右の接続ログとCentralの連続稼働1510秒（約25分）を確認しました（長時間試験は未完了）。
クラッシュ保存の再試行では本文が空だったため、panic位置そのものは未採取です。

- Madula: nRF52840 Central、直接接続の3キー、USB/BLEホスト接続
- Cornix左・右: nRF52840 BLE Peripheral、各4×7マトリクス
- 50＋3の物理位置を維持。CornixリポジトリのMadula用5レイヤーを基準に移植
- SpaceのFn/Num長押し、EscapeのBT長押し、Ctrl/Escapeのmod-tapを設定（実機タイミングは未検証）
- LayoutShift、BTプロファイル操作は未移植。その位置は誤操作防止のためNoに設定
- 左右エンコーダ: 各A=P1.04/B=P1.06、内部プルアップ、4カウント/段。全レイヤーで上下スクロール。右の実機方向の報告を受け、両方ともRMKのCW/CCW割り当ては上/下。両側対応版の動作はユーザー確認済み。
- TB(PMW3610)、IQS9151、電圧監視は未実装。LPPSは開発ブランチでユーザーから操作感良好の報告あり。
- Madula LEDを実装: 内蔵RGBは接続状態とレイヤー色を短く表示。外付けWS2812はUSB=シアン/BLE=青/未接続=赤を1秒、レイヤー変更時は番号回数の白点滅（120ms間隔、base=0回）。外付け電源は消灯15秒後にOFF。2026-10-10にユーザーから実機の点灯確認あり。個別の色・レイヤー点滅回数は未確認。

追加実機確認（2026-10-10）: 新しいMadulaで左右のキー入力と、Fn長押し時の白1回・
Num長押し時の白2回の点滅についてユーザーからOKの報告あり。
上記のレイヤー点滅回数の未確認状態は解消。BLEホスト接続時の色と長時間動作は未確認です。

LEDはZMKのMadula配線を参照し、内蔵RGB=P0.26/P0.30/P0.06（active low）、
SPI3 MOSI=P0.28、LED電源=P0.03（active low）です。
WS2812は4MHz、GRB順、輝度32、0/1符号0x40/0x70、280usのリセット期間を使用します。
LPPSのTWISPI0とは独立し、転送・点滅待機はいずれも非同期です。
バッテリー測定が未実装のため、残量を表す色は表示しません。
- DYA StudioはZMK用のため、この版では利用できません。Vial等の編集対応も未実装
- 3台ともUSB CDC ACM Debugログ・1200 baud bootを実装し、実機でリセット・再書き込み・COM復帰を確認済み

RMK 0.9.0を固定し、依存解決はCargo.lockで固定します。
参考: [RMK split](https://rmk.rs/main/docs/features/split_keyboard)、
[0.9.0の実装](https://github.com/rmk-rs/rmk/tree/986017d901c7609a9a79167686524f014effa68a)。
BLE splitの接続確認には3台ともRMK版が必要です。ZMKの左右とは接続できません。

## USB CDC Debug

### 電圧監視の診断（開発ブランチ）

最新仕様（2026-10-10）: EasyEDAのGeaconPolaris → Bat → P1
（page UUID `88e31726566844998c07abbfa788eb85`）のネットリストを読み取り確認。
RL2=1MΩが昇圧前・QL8後の電池側ノードから`INPUT_VOLTAGE_B`へ、RL3=470kΩが
同信号から`GND_B`へ接続。H5の3番が測定出力です。回路は変更していません。
電池側電圧はADCピン電圧×1470/470で換算します。raw=483なら約1328mV。
ユーザーの実機はNiMH 1本。CDCで`pin_mv`と`battery_mv`を10秒ごとに表示します。
換算版を書き込み、CDCでraw=484 / pin_mv=425 / battery_mv=1330とLPPS測定継続を確認済み。
残量目安はPolaris_hardware.dtsiのNiMH表を流用:
`1100 1150 1200 1220 1240 1260 1280 1300 1320 1350 1400` mVを0～100%の10%刻みに対応させ、
区間を線形補間します。負荷・温度で変わる電圧ベースの推定で、正確な残容量ではありません。
外付けLEDは起動5秒付近と以後60秒周期で1秒表示: 50%超=緑、21～50%=橙、20%以下=赤。
接続表示とレイヤー点滅を優先。測定値が30秒以上古い場合や範囲外なら残量色を表示しません。
2026-10-10: 実機CDCで1330mV・推定83%とLPPS測定継続を確認し、ユーザーから
残量LEDの緑点灯についてOKの報告あり。橙・赤の実機表示と長時間評価は未確認です。
充電判定とOSへのBLE残量通知は未実装。テスターとの精度照合も未実施です。
`rustc +stable --test src/battery_voltage.rs -o target/battery-voltage-tests && target/battery-voltage-tests`
で換算・範囲外入力・NiMH推定境界の3テストを実行できます。以下は換算確定前の調査履歴です。

`bash scripts/build-docker.sh --bin madula-central --features lpps,battery-audit`
でLPPSと独立したSAADC診断を有効化します。Madulaの既存ZMK設定に合わせて
A0/P0.02を読み、12bit・内部0.6V基準・gain 1/6・16倍oversampling・40us取得時間を使用。
10秒ごとにADC生値とピン電圧をCDCへ出します。校正・測定は非同期で1秒のタイムアウト付き。
電池の種類と換算倍率が未確認のため、残量%・充電判定・電池用LED表示は実装しません。
2026-10-10: 診断版のビルドと既存設定テスト12件成功。接続中のMadulaへCDC経由で
バックアップ後に書き込み、COM79復帰とLPPSの測定継続を確認。
A0はraw=2～3（約2～3mV）。電池未接続・計測経路などは未確認であり、電池残量0%とは扱いません。
続いてユーザーが電池を接続した後、同じCOM79でraw=486/483（ピン電圧427/425mV）を確認。
接続前からの変化は確認できましたが、電池の種類・実電圧・換算倍率は未確認です。
この値を電池端子電圧や残量として表示しません。
電池接続後にLPPSドリフトの報告あり。電池接続状態で再起動しても改善しないとの報告。
監視有効版の静止ログではXの中立差が約-1000～-2700で、デッドゾーン1500を超過。
切り分けとして同じソースから`--features lpps`のみで再ビルドし、電圧監視無効版を
CDC経由で書き込み。再校正も伴うため、これだけでSAADC干渉を原因とは断定しません。
比較版は初期約25秒のログでXの持続的な負方向偏りが軽減しましたが、単発の
デッドゾーン超過は残っています。カーソルの実機再確認と長時間評価は保留です。
診断機能は既定ビルドでは無効です。

### LPPS診断（開発ブランチ）

`bash scripts/build-docker.sh --bin madula-central --features lpps`で
ADS1220入力タスクを有効化できます。通常ビルドでは無効です。
Madula/ZMKのSCK=P1.15、MISO=P1.13、MOSI=P1.12、SPI mode 1・1MHzを使用。
設定照合後に100組を平均し、約1秒ごとにX/Y生値をCDCへ表示します。
現行LPPS版はカーソルを動かします。起動約3秒後から100組の校正が終わるまでは触らないでください。
両軸デッドゾーン2500、範囲±12000、最大20/周期、X/Y反転を設定。
各軸を1/4の指数平滑化（約8ms周期で時定数約28ms）してから速度へ変換します。
電池接続時の微動報告を受けた対策で、低速操作・斜め移動・静止ノイズ・手放し後の停止を
ホストテストで検証。2026-10-10に対策版を書き込み、ユーザーから静止時に
「止まってる」と確認あり。続いて操作感も問題なしとの報告あり。
長時間のドリフト再発は未確認です。電圧監視無効での確認結果です。
電池はユーザー申告でNiMH。本数・実電圧・ADC換算倍率が未確認のため残量%は保留し、
ドリフト対策と`battery-audit`を併用する比較ビルドを作成しました。
同日、併用版をMadulaへ書き込み、CDC復帰・A0 raw=483（425mV）・LPPSの測定継続を確認。
電圧監視有効状態について、ユーザーからNiMH 1本構成・「大丈夫そう」と報告あり。
2026-10-10に既存ZMKのbattery_voltage_divider.cも照合し、内部基準0.6V・gain 1/6・
12bit・40us・16倍oversamplingが一致することを確認。既存Madulaの抵抗比設定も1:1。
このためADC換算の実装差だけで425mVを説明できません。残量%の実装は電池端子電圧と
A0への計測経路の確認待ちです。推測で倍率3倍などを適用しません。
各軸の端数を独立に保持して弱い操作と斜め移動を扱います。
標準PointingProcessorを使い、キー割り当て由来のマウスボタン状態を維持します。
ADC飽和時は出力を止めて再初期化します。設定照合とCDCログも継続します。
2026-10-10: Centralへ書き込み、レジスター応答`ff ff ff ff`を確認。
期待値`05 80 43 a0`と一致しないため読み取りを停止・5秒後に再試行します。
この時点では通信未確認でした。
その後、ユーザーから装着済みと確認。ZMKと同じ単一レジスター単位の全二重通信へ
変更して再書き込みしましたが、リセット直後・設定後とも全レジスターがFFでした。
配線・SPIモードは生成DTSとも一致していました。
その後ユーザーから、前の個体にはLPPSが接続されていなかったと訂正がありました。
別のMadulaへ接続変更後、現行ZMK/SoftDeviceをバックアップして診断版へ移行。
新しいCOM79で設定照合が通り、X約49000～51000、Y約-79000～-77000の
生値が連続して読めることを確認しました。全FFは未接続個体の結果であり、
通信方式の不具合と確定したものではありません。その後、ユーザー操作中に両軸が
正負約16000カウントまで変化することを確認。その後、カーソル操作についてユーザーから
「ZMKより動きいい」と報告あり。長時間動作と全条件でのドリフト評価は未確認です。
`rustc +stable --test src/lpps_motion.rs -o target/lpps-motion-tests && target/lpps-motion-tests`
でデッドゾーン・上限・微小移動・独立XY・静止ノイズの3テストを実行できます。
設定の参照元は`te9no/ads1220-zephyr-module`とCornixの`madula-trackpoint`です。

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
- 1200 baudでDTRを500 ms以上有効にしてから落とすと、Adafruitブートローダーへリセットします。
- CDCコマンド、DYA Studio接続は提供しません。
- XIAO CentralのCOM/HID列挙と再書き込み後の復帰、左右PeripheralのCDCとBLE接続は確認済みです。
- 2026-10-10: 左COM77、右COM78を順に1200 baudリセットし、CornixのUF2ブートローダーを確認。各CURRENT.UF2をローカルにバックアップして再書き込みし、同じCOMへの復帰とCentralへのBLE再接続を確認しました。Centralはリセットせず稼働を継続しました。COM番号は環境依存です。
- Centralは10秒ごとに稼働秒数を出力します。これは実行タスクの生存確認であり、左右接続成功を意味しません。

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
エンコーダはRMK標準encoder ID 左0・右1として有効化しています。
割り当てはCentralが処理します。左追加で右のIDも変わるため、3台を同じ構成に更新します。
2026-10-10: 対応版を両方へCDC経由で書き込み、右のBLE再接続とCentralの40秒稼働を確認。
右の方向修正後、左にも同じスクロール割り当てを追加しました。
2026-10-10: 両側対応版の書き込み後にユーザーから動作OKの報告を受けました。
両エンコーダ版を3台へCDC経由で書き込み、左右のBLE再接続とCentralの80秒稼働を確認済みです。

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
Madula用5レイヤー＋USB CDCログで3台分を再ビルド成功、設定・UF2テスト9件成功。
XIAO Centralの起動確認済み。BLE接続・キー入力は未確認です。

## XIAOの自動リセット・診断書き込み

2026-10-10: XIAO nRF52840 Sense / UF2 0.6.1 / S140 7.3.0で、
専用の`diagnostics/cdc-smoke`を0x27000へ書き込み、CDCログ受信、
1200 baud自動リセット、UF2ドライブ復帰、自動再書き込み、ログ復帰を確認しました。
この診断ファームはキー・BLE・センサー・設定保存を使用しません。
RMK一式の起動やCornix左右のブートローダー互換性を実証するものではありません。

Windowsの` scripts/flash-xiao.ps1 `は、明示したCOMのPNP識別情報、
UF2のアドレス・family ID・ベクタ、復帰したドライブのBoard-IDとSoftDeviceを検査し、
CURRENT.UF2をtarget/backupsへ退避してからコピーします。Cornixには使用しないでください。
実行時は`-Firmware`、`-Port`、`-ExpectedPnpDeviceId`を指定します。`-WhatIf`で書き込みを抑止できます。
CDC未対応のファームから最初に移行するときは、手動でブートローダーを起動する必要があります。
ローカルでシリアルポートを開けるアプリはリセットも実行できる点に注意してください。

`vendor/embassy-usb-logger`は0.6.0に制御線監視を追加したローカル改変です。
通常の115200 baudでの開閉ではリセットしません。状態遷移のホストテスト2件も成功。

## RMK一式の起動調査

2026-10-10: 同じXIAOへMadula Central本体を書き込みましたが、
USB CDC/HIDの列挙を確認できませんでした。起動成功とは扱いません。
専用CDC診断ファームの成功とは区別し、RMK本体は原因調査中です。
CDCが出ない状態からの復帰には手動のブートローダー起動が必要です。
Cornix左右の書き込みは未実施、RMK一式のUF2配布は行いません。
その後、同じXIAOをCDC専用診断版へ戻し、COM列挙とログ受信の復旧を確認しました。

重要な未解決点: 現在のmemory.xは旧SoftDeviceを保護する0x27000配置ですが、
[RMKのSDC移行手順](https://rmk.rs/main/docs/migration/v06_v07)は
Adafruitブートローダー使用時に0x1000配置へ変更し、旧SoftDeviceを上書きする構成です。
現在の配置は公式移行構成と一致していません。これだけで停止原因を断定はしませんが、
次の検証にはSoftDevice領域を書き換える承認と、XIAOのブートローダー／SoftDevice復旧手順が必要です。
通常のアプリUF2の退避だけでZMKへ完全復旧できるとは扱いません。

続いてユーザー承認のもと、memory.xを0x1000開始・0xD4000終端へ変更し、
同じXIAOへUF2を転送しましたが、USB列挙は依然として確認できませんでした。
配置だけを原因と断定しません。次回の手動ブートローダー復帰後にCURRENT.UF2を
読み戻して転送内容を照合し、その後BLE/ストレージなどの初期化を段階的に調査します。
初回CURRENT.UF2には0x1000–0x26FFFの全608ブロックが含まれており、
`scripts/extract-softdevice.py`でMBR・ブートローダーを除いた復旧用UF2を作成済みです。
復旧用データはtarget/backupsにローカル保存し、公開しません。復旧書き込み自体は未検証です。

追加切り分け（2026-10-10）:
- CURRENT.UF2の読み戻しで本体479,404バイトがビルド出力と完全一致。転送不良ではありません。
- `xiao-ble-init`診断版は0x1000配置でCDC起動、MPSL初期化成功、
  RMKと同じ2 peripheral/1 central側host接続枠のSDC初期化成功をログで確認。
- 本体でDCDC reg0/reg1を無効化した比較版もUSB列挙なし。効果がないためソース設定は戻しました。
- 次はUSB起動前に行われる設定保存・キーマップ初期化を調査します。原因は未確定です。
  現在実機にはDCDC無効化比較版が入り、CDC復帰には再び手動ブートローダー起動が必要です。

設定保存の切り分け（同日、上記の実機状態を更新）:
- storage機能とstorage.enabledを無効化したMadula CentralはUSB CDCとして列挙しました。
  左右Peripheralはstorage無効時にRMKマクロのコンパイルエラーとなるため、この比較はCentralのみです。
- USBを先に起動する`xiao-ble-init`へ設定保存初期化を追加し、
  `storage completed = true`を実機CDCログで確認しました。
- 設定保存を再び有効化した通常Centralは、初期化済み領域でもUSB列挙しませんでした。
  保存領域の破損だけでは説明できず、通常版の初期化順序・処理経路を調査する必要があります。
- ソースはstorage有効へ戻しています。現在実機は通常Centralの比較版で、
  次回書き込みには手動でUF2ブートローダーへ戻す必要があります。

起動監視の比較:
- 先行CDC診断版ではCentralと同じ5行14列・5レイヤー（無効キー）の
  `initialize_keymap_and_storage`も完了し、独立監視タスクのstage=4を継続受信。
- 通常Centralの`boot-audit`機能はUSBドライバ作成前に100ms待機し、
  20秒後にブートローダー復帰するタスクを起動します。実機ではUSB列挙も
  20秒後の復帰も確認できませんでした。単なる非同期の保存待機とは断定できません。
- 現在のXIAOはこのboot-audit比較版。再度手動復帰が必要です。
  boot-auditは通常ビルドでは無効で、製品機能ではありません。

原因確定:
- `boot-audit`に診断用panic/HardFault処理を追加。上位RAMを予約して記録を残し、
  GPREGRET=0x57でブートローダーへ戻ります。読み取り版も同じRAM予約でビルドします。
- 実機記録: `rmk-0.9.0/src/ble/mod.rs:229`、
  `Device name is too long. Max length is 22 bytes`でunwrapがpanic。
- product_nameを23バイトから10バイトへ短縮した通常版でUSB CDC復帰。
  保存無効化やDCDC変更は採用せず、名前のUTF-8バイト数をテストに追加しました。
- RAM記録は診断用で、電源断後の保持は保証しません。通常ビルドのpanic処理は
  元のpanic-probeを使用し、20秒リセット・診断用RAM予約は無効です。
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
