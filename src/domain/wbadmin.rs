use crate::domain::validation::BackupTarget;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupVersion {
    pub backup_time: String,
    pub version_id: String,
    pub target_location: String,
    pub recoverable_items: String,
}

fn append_network_credentials(args: &mut Vec<String>, target: &BackupTarget) {
    if let BackupTarget::NetworkUnc { user, password, .. } = target {
        if let Some(u) = user
            && !u.is_empty()
        {
            args.push(format!("-user:{u}"));
        }
        if let Some(p) = password
            && !p.is_empty()
        {
            args.push(format!("-password:{p}"));
        }
    }
}

/// `wbadmin start backup` の引数配列を構築する。
pub fn build_start_backup_args(includes: &[char], target: &BackupTarget) -> Vec<String> {
    let mut seen = [false; 26];
    let mut include_arg = String::from("-include:");
    let mut first = true;
    for &c in includes {
        let upper = c.to_ascii_uppercase();
        if upper.is_ascii_uppercase() {
            let idx = (upper as u8 - b'A') as usize;
            if seen[idx] {
                continue;
            }
            seen[idx] = true;
        }
        if !first {
            include_arg.push(',');
        }
        first = false;
        include_arg.push(upper);
        include_arg.push(':');
    }
    let target_arg = format!("-backupTarget:{}", target.as_wbadmin_target());

    let mut args = vec![
        "start".to_string(),
        "backup".to_string(),
        target_arg,
        include_arg,
        "-allCritical".to_string(),
        "-quiet".to_string(),
    ];
    append_network_credentials(&mut args, target);
    args
}

/// 画面表示・コピー用の安全なコマンドプレビューを生成する（パスワードはマスク）。
pub fn build_command_preview(includes: &[char], target: &BackupTarget) -> String {
    let args = build_start_backup_args(includes, target);
    let masked: Vec<String> = args
        .into_iter()
        .map(|arg| {
            if arg.starts_with("-password:") {
                "-password:****".to_string()
            } else if arg.contains(' ') {
                format!("\"{arg}\"")
            } else {
                arg
            }
        })
        .collect();
    format!("wbadmin {}", masked.join(" "))
}

/// `wbadmin get versions` の引数配列を構築する。
pub fn build_get_versions_args(target: &BackupTarget) -> Vec<String> {
    let mut args = vec![
        "get".to_string(),
        "versions".to_string(),
        format!("-backupTarget:{}", target.as_wbadmin_target()),
    ];
    append_network_credentials(&mut args, target);
    args
}

/// `wbadmin stop job -quiet` の引数配列を構築する。
pub fn build_stop_job_args() -> Vec<String> {
    vec!["stop".to_string(), "job".to_string(), "-quiet".to_string()]
}

/// 出力行から進捗パーセンテージ（0..=100）を抽出する。
pub fn parse_progress_percent(line: &str) -> Option<u8> {
    let bytes = line.as_bytes();
    for (idx, &b) in bytes.iter().enumerate() {
        if b == b'%' && idx > 0 {
            let mut end = idx;
            while end > 0 && bytes[end - 1] == b' ' {
                end -= 1;
            }
            let mut start = end;
            while start > 0 && bytes[start - 1].is_ascii_digit() {
                start -= 1;
            }
            if start < end
                && let Ok(val) = line[start..end].parse::<u16>()
                && val <= 100
            {
                return Some(val as u8);
            }
        }
    }
    None
}

/// `MM/DD/YYYY-HH:MM` 形式のバージョン識別子っぽいか判定する。
fn looks_like_version_id(s: &str) -> bool {
    let t = s.trim();
    if t.len() != 16 {
        return false;
    }
    let b = t.as_bytes();
    b[0].is_ascii_digit()
        && b[1].is_ascii_digit()
        && b[2] == b'/'
        && b[3].is_ascii_digit()
        && b[4].is_ascii_digit()
        && b[5] == b'/'
        && b[6..10].iter().all(|c| c.is_ascii_digit())
        && b[10] == b'-'
        && b[11].is_ascii_digit()
        && b[12].is_ascii_digit()
        && b[13] == b':'
        && b[14].is_ascii_digit()
        && b[15].is_ascii_digit()
}

fn contains_ascii_ignore_case(haystack: &str, needle: &str) -> bool {
    let h = haystack.as_bytes();
    let n = needle.as_bytes();
    if n.is_empty() || h.len() < n.len() {
        return false;
    }
    h.windows(n.len()).any(|w| w.eq_ignore_ascii_case(n))
}

/// `wbadmin get versions` の出力からバックアップ一覧を抽出する（日本語・英語OS対応）。
pub fn parse_get_versions_output(output: &str) -> Vec<BackupVersion> {
    let mut results = Vec::new();
    let mut current: Option<BackupVersion> = None;

    let flush = |cur: &mut Option<BackupVersion>, out: &mut Vec<BackupVersion>| {
        if let Some(v) = cur.take()
            && (!v.version_id.is_empty() || !v.backup_time.is_empty())
        {
            out.push(v);
        }
    };

    for raw_line in output.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            flush(&mut current, &mut results);
            continue;
        }

        if let Some((key, val)) = split_key_value(line) {
            if key.contains("バックアップ時刻") || contains_ascii_ignore_case(key, "backup time")
            {
                if current
                    .as_ref()
                    .is_some_and(|c| !c.backup_time.is_empty() && !c.version_id.is_empty())
                {
                    flush(&mut current, &mut results);
                }
                let entry = current.get_or_insert_with(empty_version);
                entry.backup_time = val.to_string();
            } else if key.contains("バックアップの場所")
                || contains_ascii_ignore_case(key, "backup location")
            {
                let entry = current.get_or_insert_with(empty_version);
                entry.target_location = val.to_string();
            } else if key.contains("バージョン識別子")
                || contains_ascii_ignore_case(key, "version identifier")
                || looks_like_version_id(val)
            {
                let entry = current.get_or_insert_with(empty_version);
                entry.version_id = val.to_string();
            } else if key.contains("回復可能") || contains_ascii_ignore_case(key, "can recover")
            {
                let entry = current.get_or_insert_with(empty_version);
                entry.recoverable_items = val.to_string();
            }
        }
    }

    flush(&mut current, &mut results);
    results
}

fn empty_version() -> BackupVersion {
    BackupVersion {
        backup_time: String::new(),
        version_id: String::new(),
        target_location: String::new(),
        recoverable_items: String::new(),
    }
}

fn split_key_value(line: &str) -> Option<(&str, &str)> {
    // 全角コロンと半角コロンのうち先に出現した方を区切り文字として採用
    let idx = match (line.find(':'), line.find('：')) {
        (Some(a), Some(b)) => a.min(b),
        (Some(a), None) => a,
        (None, Some(b)) => b,
        (None, None) => return None,
    };
    let sep_len = if line[idx..].starts_with('：') {
        '：'.len_utf8()
    } else {
        1
    };
    let key = line[..idx].trim();
    let val = line[idx + sep_len..].trim();
    Some((key, val))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_start_backup_args_local() {
        let args = build_start_backup_args(&['c', 'e', 'C'], &BackupTarget::LocalDrive('d'));
        assert_eq!(
            args,
            vec![
                "start",
                "backup",
                "-backupTarget:D:",
                "-include:C:,E:",
                "-allCritical",
                "-quiet"
            ]
        );
    }

    #[test]
    fn test_build_command_preview_masks_password() {
        let target = BackupTarget::NetworkUnc {
            path: "\\\\NAS\\Share".to_string(),
            user: Some("alice".to_string()),
            password: Some("SuperSecret!".to_string()),
        };
        let preview = build_command_preview(&['C'], &target);
        assert!(preview.contains("-user:alice"));
        assert!(preview.contains("-password:****"));
        assert!(!preview.contains("SuperSecret!"));
    }

    #[test]
    fn test_parse_progress_percent() {
        assert_eq!(
            parse_progress_percent("ボリューム (C:) のコピーを作成しています - 45% 完了"),
            Some(45)
        );
        assert_eq!(parse_progress_percent("100 % completed"), Some(100));
        assert_eq!(parse_progress_percent("0%"), Some(0));
        assert_eq!(parse_progress_percent("エラー 105%"), None);
        assert_eq!(parse_progress_percent("進行状況なし"), None);
    }

    #[test]
    fn test_parse_get_versions_japanese_and_english() {
        let sample = "\
wbadmin 1.0 - バックアップ コマンド ライン ツール
バックアップ時刻: 2026/11/17 00:30
バックアップの場所: 1394/USB ディスク (D:)
バージョン識別子: 11/17/2026-00:30
回復可能な項目: ボリューム, ファイル, アプリケーション, ベア メタル回復, システム状態

Backup time: 11/16/2026 9:15 PM
Backup location: Fixed hard disk (D:)
Version identifier: 11/16/2026-21:15
Can recover: Volume(s), File(s), Application(s), Bare Metal Recovery, System State
";
        let versions = parse_get_versions_output(sample);
        assert_eq!(versions.len(), 2);
        assert_eq!(versions[0].version_id, "11/17/2026-00:30");
        assert_eq!(versions[0].backup_time, "2026/11/17 00:30");
        assert_eq!(versions[1].version_id, "11/16/2026-21:15");
    }

    #[test]
    fn test_parse_get_versions_fullwidth_colon_before_halfwidth() {
        let sample = "\
バックアップ時刻： 2026/11/17 00:30
バックアップの場所： 固定ハード ディスク (D:)
バージョン識別子： 11/17/2026-00:30
回復可能な項目： ボリューム
";
        let versions = parse_get_versions_output(sample);
        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].backup_time, "2026/11/17 00:30");
        assert_eq!(versions[0].version_id, "11/17/2026-00:30");
    }
}
