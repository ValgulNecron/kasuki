# Command registration translations for ja
# Used by registration_dispatcher.rs for Discord command localization

# ─── Parent commands ─────────────────────────────────────────────────────────

parent-user-name = 一般
parent-user-desc = 一般的な目的のコマンド。

parent-ai-name = ai
parent-ai-desc = AIモジュールからのコマンド

parent-bot-name = bot
parent-bot-desc = ボットに関する情報を取得するコマンド。

parent-music-name = 音楽
parent-music-desc = 音楽モジュールからのコマンド

parent-steam-name = steam
parent-steam-desc = GAMEモジュールからのSteamコマンド。

parent-server-name = サーバー
parent-server-desc = サーバー用の汎用コマンド。

parent-vn-desc = ビジュアルノベルの情報を取得する

parent-random_anime-name = random_anime
parent-random_anime-desc = ANIME モジュールからのコマンド

parent-random_hanime-name = random_hanime
parent-random_hanime-desc = ANIMEモジュールのコマンド

parent-levels-name = レベル
parent-levels-desc = ユーザーのレベル、統計情報を取得するコマンド。

parent-minigame-name = ミニゲーム
parent-minigame-desc = ミニゲームをプレイしてインベントリを管理するためのコマンド。

parent-admin-name = admin
parent-admin-desc = 管理者専用のボット設定。

# ─── Subcommand groups ────────────────────────────────────────────────────────

group-admin-anilist-name = anilist
group-admin-anilist-desc = 管理者権限が必要なAniListモジュールのコマンド。

group-admin-general-name = general
group-admin-general-desc = 管理者権限が必要な一般モジュールのコマンド。

# ─── Commands ─────────────────────────────────────────────────────────────────

# admin/anilist
cmd-add_anime_activity-name = anime_katsudo_wo_tsuika
cmd-add_anime_activity-desc = アニメ活動を追加します。

cmd-delete_anime_activity-name = anime_katsudo_wo_sakujo
cmd-delete_anime_activity-desc = アニメ活動を削除します。

# admin/general
cmd-lang-name = lang
cmd-lang-desc = レスポンスに設定したい言語。

cmd-module-name = モジュール
cmd-module-desc = モジュールをオンまたはオフにします。

# ai
cmd-image-name = 画像
cmd-image-desc = 画像を生成する。

cmd-question-name = question
cmd-question-desc = 質問をすると回答が得られます（これはチャットではなく、コンテキストがありません）。

cmd-transcript-name = トランスクリプト
cmd-transcript-desc = 動画から脚本を生成する。

cmd-translation-name = honyaku
cmd-translation-desc = 翻訳を生成する。

# anilist_server
cmd-list_activity-name = アクティビティリスト
cmd-list_activity-desc = 登録アクティビティのリストを取得する。

cmd-list_user-name = ユーザーリスト
cmd-list_user-desc = 登録ユーザーのリストを取得する。

# anilist_user
cmd-anime-name = anime
cmd-anime-desc = アニメ情報。

cmd-character-name = キャラクター
cmd-character-desc = キャラクター情報。

cmd-compare-name = 比較
cmd-compare-desc = 2人のユーザーを比較する。

cmd-level-name = レベル
cmd-level-desc = ユーザーのレベルを取得する。

cmd-ln-name = ln
cmd-ln-desc = ライトノベル情報。

cmd-manga-name = manga
cmd-manga-desc = マンガ情報。

cmd-random-name = random
cmd-random-desc = ランダムなアニメまたはマンガを取得する。

cmd-register-name = 登録
cmd-register-desc = AniListにユーザー名を登録する。

cmd-seiyuu-name = seiyuu
cmd-seiyuu-desc = 声優情報。

cmd-staff-name = スタッフ
cmd-staff-desc = スタッフ情報。

cmd-studio-name = studio
cmd-studio-desc = スタジオ情報。

cmd-anilist_user-name = anilist_user
cmd-anilist_user-desc = AniListのユーザー情報。

cmd-waifu-name = waifu
cmd-waifu-desc = 最高のワイフを取得する。

# anime
cmd-random_image-name = ランダム画像
cmd-random_image-desc = ランダムなアニメ画像を取得する。

# anime_nsfw
cmd-random_himage-name = ランダムh画像
cmd-random_himage-desc = ランダムなNSFWアニメ画像を取得する。

# bot
cmd-credit-name = クレジット
cmd-credit-desc = アプリのクレジットを取得します。

cmd-info-name = info
cmd-info-desc = ボットの情報を取得します。

cmd-ping-name = ping
cmd-ping-desc = ボットのピング（およびシャードID）を取得します。

# levels
cmd-stats-name = 統計
cmd-stats-desc = レベルの統計を取得する。

# management
cmd-give_premium_sub-name = give_premium_sub
cmd-give_premium_sub-desc = ユーザーにプレミアムサブスクリプションを付与する。

cmd-kill_switch-name = kill_switch
cmd-kill_switch-desc = モジュールをグローバルにオン/オフにする

cmd-remove_test_sub-name = remove_test_sub
cmd-remove_test_sub-desc = ユーザーのプレミアムサブスクリプションを削除する。

# minigame
cmd-fish_inventory-name = 魚インベントリ
cmd-fish_inventory-desc = 魚のインベントリを確認する。

cmd-fishing-name = 釣り
cmd-fishing-desc = 釣りに行こう！

cmd-inventory-name = インベントリ
cmd-inventory-desc = インベントリを確認する。

# music
cmd-clear-name = クリア
cmd-clear-desc = 現在のキューをクリアする。

cmd-join-name = 参加
cmd-join-desc = ボイスチャンネルに参加する。

cmd-leave-name = 退出
cmd-leave-desc = ボイスチャンネルから退出する。

cmd-pause-name = 一時停止
cmd-pause-desc = 現在の曲を一時停止する。

cmd-play-name = 再生
cmd-play-desc = 曲を再生する。

cmd-queue-name = キュー
cmd-queue-desc = 現在のキューを表示する。

cmd-remove-name = 削除
cmd-remove-desc = キューから曲を削除する。

cmd-resume-name = 再開
cmd-resume-desc = 現在の曲を再開する。

cmd-seek-name = シーク
cmd-seek-desc = 現在の曲の位置にシークする。

cmd-skip-name = スキップ
cmd-skip-desc = 現在の曲をスキップする。

cmd-stop-name = 停止
cmd-stop-desc = 現在の曲を停止する。

cmd-swap-name = スワップ
cmd-swap-desc = キュー内の2つの曲を交換する。

# server
cmd-guild_image-name = ギルド画像
cmd-guild_image-desc = ギルドのプロフィール画像を生成する。

cmd-guild_image_g-name = ギルド画像_グローバル
cmd-guild_image_g-desc = ギルドのグローバルプロフィール画像を生成する。

cmd-guild-name = ギルド
cmd-guild-desc = ギルドの情報を取得する。

# steam
cmd-game-name = ゲーム
cmd-game-desc = Steamゲームの情報を取得する。

# user
cmd-avatar-name = アバター
cmd-avatar-desc = アバターを取得します。

cmd-banner-name = バナー
cmd-banner-desc = バナーを取得します。

cmd-command_usage-name = コマンドの使用状況
cmd-command_usage-desc = 各コマンドの使用状況を表示します。

cmd-profile-name = プロフィール
cmd-profile-desc = ユーザーのプロフィールを表示します。

# vn
cmd-vn_game-desc = ビジュアルノベルの情報を取得する。

cmd-vn_character-desc = VNキャラクターの情報を取得する。

cmd-vn_staff-desc = VNスタッフの情報を取得する。

cmd-vn_producer-desc = VNプロデューサーの情報を取得する。

cmd-vn_user-desc = VNユーザーの情報を取得する。

cmd-vn_stats-desc = VN統計を取得する。

# ─── Arg translations ─────────────────────────────────────────────────────────

# admin/anilist/add_anime_activity
arg-add_anime_activity-anime_name-name = anime_no_namae
arg-add_anime_activity-anime_name-desc = アクティビティとして追加したいアニメの名前。
arg-add_anime_activity-delays-name = keishi
arg-add_anime_activity-delays-desc = 秒単位の遅延。

# admin/anilist/delete_anime_activity
arg-delete_anime_activity-anime_name-name = anime_no_namae
arg-delete_anime_activity-anime_name-desc = アクティビティとして削除したいアニメの名前。

# admin/general/lang
arg-lang-lang_choice-name = 言語選択
arg-lang-lang_choice-desc = レスポンスに設定したい言語。

# admin/general/module
arg-module-name-name = モジュール名
arg-module-name-desc = 状態を変更したいモジュール。
arg-module-state-name = 状態
arg-module-state-desc = 適用したい状態。

# ai/image
arg-image-description-name = 説明
arg-image-description-desc = 生成したい画像の説明を入力してください。
arg-image-n-name = n
arg-image-n-desc = 生成する画像の数。

# ai/question
arg-question-prompt-name = prompt
arg-question-prompt-desc = 質問したい内容。

# ai/transcript
arg-transcript-video-name = video
arg-transcript-video-desc = ビデオファイルをアップロード（最大25MB）。
arg-transcript-prompt-name = プロンプト
arg-transcript-prompt-desc = オーディオスタイルのガイドテキスト。オーディオの言語と一致する必要があります。
arg-transcript-lang-name = lang
arg-transcript-lang-desc = 入力言語を選択（ISO-639-1)

# ai/translation
arg-translation-video-name = video
arg-translation-video-desc = ビデオファイルをアップロード（最大25MB）。
arg-translation-lang-name = lang
arg-translation-lang-desc = 入力言語を選択（ISO-639-1)

# anilist_user/anime
arg-anime-anime_name-name = anime_no_namae
arg-anime-anime_name-desc = チェックしたいアニメの名前。

# anilist_user/character
arg-character-name-name = 名前
arg-character-name-desc = チェックしたいキャラクターの名前。

# anilist_user/compare
arg-compare-username-name = ユーザーネーム
arg-compare-username-desc = 比較したい最初のユーザーのユーザー名。
arg-compare-username2-name = ユーザーネーム2
arg-compare-username2-desc = 比較したい2番目のユーザーのユーザー名。

# anilist_user/level
arg-level-username-name = ユーザーネーム
arg-level-username-desc = レベルを取得したいユーザーのユーザー名。

# anilist_user/ln
arg-ln-ln_name-name = ln_namae
arg-ln-ln_name-desc = チェックしたいライトノベルの名前。

# anilist_user/manga
arg-manga-manga_name-name = manga_no_namae
arg-manga-manga_name-desc = チェックしたいマンガの名前。

# anilist_user/random
arg-random-type-name = type
arg-random-type-desc = ランダムのタイプ（アニメまたはマンガ）。

# anilist_user/register
arg-register-username-name = username
arg-register-username-desc = 登録したいユーザー名。

# anilist_user/seiyuu
arg-seiyuu-staff_name-name = seiyuu_name
arg-seiyuu-staff_name-desc = チェックしたい声優の名前。

# anilist_user/staff
arg-staff-staff_name-name = スタッフ名
arg-staff-staff_name-desc = チェックしたいスタッフの名前。

# anilist_user/studio
arg-studio-studio-name = studio
arg-studio-studio-desc = チェックしたいスタジオの名前。

# anilist_user/user
arg-anilist_user-username-name = ユーザーネーム
arg-anilist_user-username-desc = チェックしたいユーザーのユーザー名。

# anime/random_image
arg-random_image-image_type-name = imeji_taipu
arg-random_image-image_type-desc = 欲しい画像のタイプ。

# anime_nsfw/random_himage
arg-random_himage-image_type-name = imeji_taipu
arg-random_himage-image_type-desc = 欲しい画像のタイプ。

# management/give_premium_sub
arg-give_premium_sub-user-name = ユーザー
arg-give_premium_sub-user-desc = サブスクリプションを付与するユーザー。
arg-give_premium_sub-subscription-name = サブスクリプション
arg-give_premium_sub-subscription-desc = 付与するサブスクリプション。

# management/kill_switch
arg-kill_switch-name-name = モジュール名
arg-kill_switch-name-desc = 状態を変更したいモジュール。
arg-kill_switch-state-name = 状態
arg-kill_switch-state-desc = 適用したい状態。

# management/remove_test_sub
arg-remove_test_sub-user-name = ユーザー
arg-remove_test_sub-user-desc = サブスクリプションを削除するユーザー。

# music/play
arg-play-search-name = 検索
arg-play-search-desc = 曲を検索する。

# music/remove
arg-remove-index-name = インデックス
arg-remove-index-desc = 削除する曲のインデックス。

# music/seek
arg-seek-time-name = 時間
arg-seek-time-desc = シークする時間（秒）。

# music/swap
arg-swap-index1-name = インデックス1
arg-swap-index1-desc = 最初の曲のインデックス。
arg-swap-index2-name = インデックス2
arg-swap-index2-desc = 2番目の曲のインデックス。

# steam/game
arg-game-game_name-name = geemu_no_namae
arg-game-game_name-desc = 情報を取得したいSteamゲームの名前。

# user/avatar
arg-avatar-username-name = ユーザー名
arg-avatar-username-desc = アバターを取得したいユーザーのユーザー名。

# user/banner
arg-banner-username-name = ユーザー名
arg-banner-username-desc = アバターを取得したいユーザーのユーザー名。

# user/command_usage
arg-command_usage-username-name = ユーザー名
arg-command_usage-username-desc = 使用状況を表示したいユーザーのユーザー名。

# user/profile
arg-profile-username-name = ユーザー名
arg-profile-username-desc = アバターを表示したいユーザーのユーザー名。

# vn/game
arg-vn_game-title-name = タイトル
arg-vn_game-title-desc = ビジュアルノベルのタイトル。

# vn/character
arg-vn_character-name-name = 名前
arg-vn_character-name-desc = キャラクターの名前。

# vn/staff
arg-vn_staff-name-name = 名前
arg-vn_staff-name-desc = スタッフの名前。

# vn/producer
arg-vn_producer-name-name = 名前
arg-vn_producer-name-desc = プロデューサーの名前。

# vn/user
arg-vn_user-username-name = ユーザー名
arg-vn_user-username-desc = VNユーザー名。

# ─── Choice translations ──────────────────────────────────────────────────────

# admin/general/lang choices
choice-lang-lang_choice-en-name = 英語
choice-lang-lang_choice-jp-name = 日本語
choice-lang-lang_choice-de-name = ドイツ語
choice-lang-lang_choice-fr-name = フランス語
choice-lang-lang_choice-es-ES-name = スペイン語
choice-lang-lang_choice-zh-CN-name = 中国語 (簡体字)
choice-lang-lang_choice-ru-name = ロシア語

# admin/general/module choices
choice-module-name-AI-name = AI
choice-module-name-ANILIST-name = アニリスト
choice-module-name-GAME-name = ゲーム
choice-module-name-ANIME-name = アニメ
choice-module-name-VN-name = ビジュアルノベル
choice-module-name-LEVEL-name = ビジュアルノベル
choice-module-name-MINIGAME-name = ミニゲーム
