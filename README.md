# sw_logger_NITS

## 概要

[natsuki-i/sw_logger](https://github.com/natsuki-i/sw_logger) のフォークリポジトリです。[NITS](https://nona-takahara.github.io/sw-train-docs/communicate/NITS.html) でのデジタル伝送を監視するための機能を主軸として、viewer に対していくつかの機能追加・改変をしています。server には変更を加えていません。

- 既存のCSV保存機能に加え、CSVを読み込む機能を追加
- 次回起動時に値を復元する設定を追加
- デジタル信号のやりとりを監視できるウィンドウを追加
- NITS 信号の時系列表示ウィンドウを追加

## 使い方

基本的な使い方は [コンポジット信号をグラフ化するやつ | すとーむすきー](https://stormskey.works/@natsuki_i/pages/1702560276774) を参照してください。以下には追加機能の使い方を示します。

- CSVを読み込む機能は、ウィンドウ上部の File -> Open CSV
- 次回起動時に値を復元する設定は、ウィンドウ上部の Settings -> Keep values on quit
- デジタル信号のやり取りを監視できるウィンドウは、ウィンドウ上部の Digital Table
    - Stormworks 内の Number 信号の32ビット浮動小数点数を32ビットバイナリとして解釈する表示が可能
    - Number 信号での整数を24ビットバイナリとして解釈する表示が可能
    - 表示様式は16進・10進・8進・2進から選択可能
- NITS 信号の時系列表示ウィンドウは、ウィンドウ上部の NITS Timeline
    - Stormworks 側のマイコンには [Steamワークショップ::sw_logger_NITS](https://steamcommunity.com/sharedfiles/filedetails/?id=3409755527) の使用を推奨します
    - 送信車やコマンド種別でフィルターをかけての表示が可能

## 詳細

- CSVを読み込むときの形式は、1行目がラベル名とし、2行目以降に実数値
- NITS Timeline は
    - `NITS N01` から `NITS N32` のラベルを、NITS基本マイコンの出力コンポジットとして解釈して表示します
    - 表示は `NITS N32` の両数カウントに依存しています
