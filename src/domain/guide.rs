/// 保存先やバージョン識別子を反映した復元手順ガイドを生成する。
pub fn generate_restore_guide(target_input: &str, version_input: &str) -> String {
    let target_trimmed = target_input.trim();
    let target = if target_trimmed.is_empty() {
        "D:"
    } else {
        target_trimmed
    };

    let version_trimmed = version_input.trim();
    let version = if version_trimmed.is_empty() {
        "<バージョン識別子（例: 11/17/2026-00:30）>"
    } else {
        version_trimmed
    };

    format!(
        "============================================================\r\n\
         Windows システムイメージ 復元手順ガイド\r\n\
         作成元: WindowsImageBackup GUI (WIB-GUI)\r\n\
         ============================================================\r\n\
         \r\n\
         【重要】事前にお読みください\r\n\
         - Windows（OS本体）を含むシステムイメージの復元は、現在起動しているWindows上からは実行できません。\r\n\
         - 必ず「Windows回復環境（WinRE）」またはWindowsインストールメディア（USB等）から起動して復元を行ってください。\r\n\
         - システムイメージを復元すると、復元先ドライブの現在のデータはバックアップ作成時の状態へ完全に上書きされます。\r\n\
         - バックアップを保存した外付けHDD/SSDは、PC再起動前に接続した状態にしてください。\r\n\
         - BitLocker（ドライブ暗号化）を有効にしている場合は、復元時に「BitLocker回復キー」の入力を求められることがあります。事前に回復キーを手元に控えてください。\r\n\
         \r\n\
         ■ 今回の参照情報\r\n\
         - バックアップ保存先（Windows起動時の表記）: {target}\r\n\
         - バージョン識別子: {version}\r\n\
         \r\n\
         ------------------------------------------------------------\r\n\
         パターンA：画面操作（WinRE GUIウィザード）で復元する手順（推奨）\r\n\
         ------------------------------------------------------------\r\n\
         1. スタートメニューの電源ボタンから、Shiftキーを押したまま「再起動」をクリックします。\r\n\
            （Windowsが起動しない場合は、WindowsインストールUSB等から起動し、左下の「コンピューターを修復する」を選択します）\r\n\
         2. 青い回復メニュー画面で「トラブルシューティング」＞「詳細オプション」＞「イメージでシステムを回復」を選択します。\r\n\
         3. 復元したいシステムイメージ（通常は最新のものが自動選択されます）を確認し、「次へ」進みます。\r\n\
         4. 必要に応じて「ディスクをフォーマットしてパーティションを再分割する」などのオプションを確認し、「完了」をクリックすると復元が始まります。\r\n\
         \r\n\
         ------------------------------------------------------------\r\n\
         パターンB：コマンドプロンプト（WinRE）で復元する手順\r\n\
         ------------------------------------------------------------\r\n\
         【注意】回復環境（WinRE）では、Windows起動時とドライブレター（D: や E: など）がずれることがあります。\r\n\
         コマンドを実行する前に、必ず diskpart 等で WindowsImageBackup フォルダがあるドライブ文字を確認してください。\r\n\
         \r\n\
         1. パターンAと同様に回復環境を起動し、「トラブルシューティング」＞「詳細オプション」＞「コマンド プロンプト」を開きます。\r\n\
         2. 以下のコマンドを順に入力し、WindowsImageBackup フォルダのあるドライブ文字を確認します。\r\n\
            diskpart\r\n\
            list volume\r\n\
            exit\r\n\
            dir D:\\WindowsImageBackup\r\n\
            （D: の部分は list volume で確認した保存先ドライブ文字に読み替えてください）\r\n\
         3. 保存先にあるバックアップのバージョン識別子（日時）を確認します。\r\n\
            wbadmin get versions -backupTarget:{target}\r\n\
         4. 以下のコマンドを実行して復元を開始します（確認メッセージが出たら Y を入力して Enter）。\r\n\
            wbadmin start sysrecovery -version:{version} -backupTarget:{target} -restoreAllVolumes\r\n\
            ※ 上記の {target} は、手順2で確認した回復環境上のドライブ文字に書き換えて実行してください。\r\n\
            ※ 確認を省略して即時実行したい場合は末尾に -quiet を付けますが、誤操作防止のため通常は付けずに確認することを推奨します。\r\n\
         \r\n\
         ------------------------------------------------------------\r\n\
         特定のファイルやフォルダだけを復元したい場合（Windows起動中）\r\n\
         ------------------------------------------------------------\r\n\
         OS全体ではなく一部のファイルだけを取り出したい場合は、Windows起動中に次の方法で復元できます。\r\n\
         1. 保存先ドライブ内の WindowsImageBackup\\<PC名>\\Backup ... フォルダにある拡張子 .vhdx のファイルをダブルクリックします。\r\n\
         2. 仮想ドライブとしてエクスプローラーにマウントされるため、必要なファイルをコピーして取り出します。\r\n\
         3. 取り出し後は、マウントされたドライブを右クリックして「取り出し」を選択してください。\r\n\
         \r\n\
         ------------------------------------------------------------\r\n\
         ネットワーク共有フォルダ（\\\\サーバー名\\共有名）を利用する場合の補足\r\n\
         ------------------------------------------------------------\r\n\
         - 回復環境（WinRE）からネットワーク経由で復元するには、有線LAN接続とネットワーク共有のアクセス資格情報（ユーザー名・パスワード）が必要になります。\r\n\
         - 環境によってはWinREでネットワークドライバが認識されない場合があるため、可能であれば外付けUSB HDD/SSD（NTFS）へのバックアップも併用することをおすすめします。\r\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_restore_guide_contains_all_required_sections() {
        let text = generate_restore_guide("E:", "11/17/2026-00:30");
        assert!(text.contains("現在起動しているWindows上からは実行できません"));
        assert!(text.contains("イメージでシステムを回復"));
        assert!(text.contains("diskpart"));
        assert!(text.contains("list volume"));
        assert!(text.contains("wbadmin get versions -backupTarget:E:"));
        assert!(text.contains("wbadmin start sysrecovery -version:11/17/2026-00:30 -backupTarget:E: -restoreAllVolumes"));
        assert!(text.contains(".vhdx"));
        assert!(text.contains("BitLocker"));
    }

    #[test]
    fn test_generate_restore_guide_defaults_when_empty() {
        let text = generate_restore_guide("", "  ");
        assert!(text.contains("-backupTarget:D:"));
        assert!(text.contains("<バージョン識別子"));
    }
}
