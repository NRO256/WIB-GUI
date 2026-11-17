use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveInfo {
    pub letter: char,
    pub label: String,
    pub fs_name: String,
    pub free_bytes: u64,
    pub total_bytes: u64,
    pub is_system: bool,
    pub disk_number: Option<u32>,
}

impl DriveInfo {
    pub fn is_ntfs(&self) -> bool {
        self.fs_name.eq_ignore_ascii_case("NTFS")
    }

    pub fn format_display(&self) -> String {
        let label = if self.label.is_empty() {
            "ローカル ディスク"
        } else {
            &self.label
        };
        let free_gb = self.free_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        let total_gb = self.total_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        let fs = if self.fs_name.is_empty() {
            "不明"
        } else {
            &self.fs_name
        };
        format!(
            "{}:\\ [{}] ({} / 空き {:.1} GB / 合計 {:.1} GB)",
            self.letter, label, fs, free_gb, total_gb
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackupTarget {
    LocalDrive(char),
    NetworkUnc {
        path: String,
        user: Option<String>,
        password: Option<String>,
    },
}

impl BackupTarget {
    /// wbadmin の -backupTarget に渡す形式（例: "D:" または "\\\\Server\\Share"）
    pub fn as_wbadmin_target(&self) -> String {
        match self {
            BackupTarget::LocalDrive(c) => format!("{}:", c.to_ascii_uppercase()),
            BackupTarget::NetworkUnc { path, .. } => path.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    NoIncludeDrives,
    InvalidDriveLetter(String),
    EmptyTarget,
    InvalidUncPath(String),
    SameDrive(char),
    SamePhysicalDisk {
        include_drive: char,
        target_drive: char,
        disk_number: u32,
    },
    NotNtfs {
        drive: char,
        fs_name: String,
    },
    InvalidCredentials(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::NoIncludeDrives => write!(
                f,
                "バックアップ対象のドライブが選択されていません。少なくともシステムドライブ（C: など）にチェックを入れてください。"
            ),
            ValidationError::InvalidDriveLetter(s) => {
                write!(f, "ドライブ指定が正しくありません: {s}")
            }
            ValidationError::EmptyTarget => write!(
                f,
                "バックアップの保存先が指定されていません。ローカルドライブを選ぶか、ネットワーク共有パスを入力してください。"
            ),
            ValidationError::InvalidUncPath(msg) => write!(
                f,
                "ネットワーク共有パスの形式が正しくありません。\\\\サーバー名\\共有フォルダ名 の形式で入力してください。（詳細: {msg}）"
            ),
            ValidationError::SameDrive(d) => write!(
                f,
                "バックアップ対象（{d}:）と同じドライブを保存先に指定することはできません。別の外付けドライブや共有フォルダを選んでください。"
            ),
            ValidationError::SamePhysicalDisk {
                include_drive,
                target_drive,
                disk_number,
            } => write!(
                f,
                "保存先ドライブ（{target_drive}:）はバックアップ対象（{include_drive}:）と同じ物理ディスク（ディスク {disk_number}）上にあります。故障時にバックアップも失われるため、別の物理ドライブを保存先に指定してください。"
            ),
            ValidationError::NotNtfs { drive, fs_name } => {
                let fs_disp = if fs_name.is_empty() {
                    "不明"
                } else {
                    fs_name.as_str()
                };
                write!(
                    f,
                    "選択された保存先（{drive}:）のファイルシステムは {fs_disp} です。Windowsのシステムイメージバックアップには NTFS 形式のドライブが必要です。"
                )
            }
            ValidationError::InvalidCredentials(msg) => {
                write!(f, "ネットワーク資格情報の入力が正しくありません: {msg}")
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// UNCパスを検証・正規化する（末尾のバックスラッシュは除去）。
pub fn validate_unc_path(input: &str) -> Result<String, ValidationError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(ValidationError::EmptyTarget);
    }
    if trimmed.starts_with('-') {
        return Err(ValidationError::InvalidUncPath(
            "ハイフンで始まるパスは指定できません".to_string(),
        ));
    }
    if !trimmed.starts_with("\\\\") {
        return Err(ValidationError::InvalidUncPath(
            "先頭は \\\\ である必要があります".to_string(),
        ));
    }
    if trimmed.chars().any(|c| c.is_control()) {
        return Err(ValidationError::InvalidUncPath(
            "制御文字が含まれています".to_string(),
        ));
    }
    const FORBIDDEN: [char; 7] = ['"', '<', '>', '|', '*', '?', ':'];
    if trimmed.chars().any(|c| FORBIDDEN.contains(&c)) {
        return Err(ValidationError::InvalidUncPath(
            "パスに使用できない記号が含まれています".to_string(),
        ));
    }

    let normalized = trimmed.trim_end_matches('\\');
    let rest = &normalized[2..];
    let mut seg_count = 0usize;
    let mut has_empty = false;
    for seg in rest.split('\\') {
        seg_count += 1;
        if seg.trim().is_empty() {
            has_empty = true;
            break;
        }
    }
    if seg_count < 2 || has_empty {
        return Err(ValidationError::InvalidUncPath(
            "サーバー名と共有フォルダ名の両方が必要です".to_string(),
        ));
    }
    Ok(normalized.to_string())
}

/// ネットワーク資格情報を検証・正規化する。
pub fn validate_credentials(
    user: Option<&str>,
    password: Option<&str>,
) -> Result<(Option<String>, Option<String>), ValidationError> {
    let u = user.map(str::trim).filter(|s| !s.is_empty());
    let p = password.filter(|s| !s.is_empty());

    if p.is_some() && u.is_none() {
        return Err(ValidationError::InvalidCredentials(
            "パスワードを指定する場合はユーザー名も入力してください".to_string(),
        ));
    }
    if let Some(user_str) = u
        && (user_str.chars().any(|c| c.is_control()) || user_str.contains('"'))
    {
        return Err(ValidationError::InvalidCredentials(
            "ユーザー名に使用できない文字が含まれています".to_string(),
        ));
    }
    if let Some(pass_str) = p
        && pass_str
            .chars()
            .any(|c| c == '\r' || c == '\n' || c == '\0')
    {
        return Err(ValidationError::InvalidCredentials(
            "パスワードに改行や制御文字は使用できません".to_string(),
        ));
    }

    Ok((u.map(str::to_string), p.map(str::to_string)))
}

/// バックアップ設定全体の妥当性を検証する。
pub fn validate_backup_plan(
    includes: &[char],
    target: &BackupTarget,
    drives: &[DriveInfo],
) -> Result<(), ValidationError> {
    if includes.is_empty() {
        return Err(ValidationError::NoIncludeDrives);
    }

    let mut normalized_includes = Vec::with_capacity(includes.len());
    for &c in includes {
        if !c.is_ascii_alphabetic() {
            return Err(ValidationError::InvalidDriveLetter(c.to_string()));
        }
        let upper = c.to_ascii_uppercase();
        if !normalized_includes.contains(&upper) {
            normalized_includes.push(upper);
        }
    }

    match target {
        BackupTarget::LocalDrive(target_char) => {
            if !target_char.is_ascii_alphabetic() {
                return Err(ValidationError::InvalidDriveLetter(target_char.to_string()));
            }
            let target_upper = target_char.to_ascii_uppercase();
            if normalized_includes.contains(&target_upper) {
                return Err(ValidationError::SameDrive(target_upper));
            }
            if let Some(target_info) = drives
                .iter()
                .find(|d| d.letter.eq_ignore_ascii_case(&target_upper))
            {
                if !target_info.is_ntfs() {
                    return Err(ValidationError::NotNtfs {
                        drive: target_upper,
                        fs_name: target_info.fs_name.clone(),
                    });
                }
                if let Some(target_disk) = target_info.disk_number {
                    for &inc in &normalized_includes {
                        if let Some(inc_info) =
                            drives.iter().find(|d| d.letter.eq_ignore_ascii_case(&inc))
                            && inc_info.disk_number == Some(target_disk)
                        {
                            return Err(ValidationError::SamePhysicalDisk {
                                include_drive: inc,
                                target_drive: target_upper,
                                disk_number: target_disk,
                            });
                        }
                    }
                }
            }
        }
        BackupTarget::NetworkUnc {
            path,
            user,
            password,
        } => {
            validate_unc_path(path)?;
            validate_credentials(user.as_deref(), password.as_deref())?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_drives() -> Vec<DriveInfo> {
        vec![
            DriveInfo {
                letter: 'C',
                label: "Windows".to_string(),
                fs_name: "NTFS".to_string(),
                free_bytes: 100 * 1024 * 1024 * 1024,
                total_bytes: 256 * 1024 * 1024 * 1024,
                is_system: true,
                disk_number: Some(0),
            },
            DriveInfo {
                letter: 'D',
                label: "Data".to_string(),
                fs_name: "NTFS".to_string(),
                free_bytes: 500 * 1024 * 1024 * 1024,
                total_bytes: 1000 * 1024 * 1024 * 1024,
                is_system: false,
                disk_number: Some(1),
            },
            DriveInfo {
                letter: 'E',
                label: "SameDiskPart".to_string(),
                fs_name: "NTFS".to_string(),
                free_bytes: 100 * 1024 * 1024 * 1024,
                total_bytes: 200 * 1024 * 1024 * 1024,
                is_system: false,
                disk_number: Some(0),
            },
            DriveInfo {
                letter: 'F',
                label: "ExFatDrive".to_string(),
                fs_name: "exFAT".to_string(),
                free_bytes: 100 * 1024 * 1024 * 1024,
                total_bytes: 128 * 1024 * 1024 * 1024,
                is_system: false,
                disk_number: Some(2),
            },
        ]
    }

    #[test]
    fn test_validate_unc_path() {
        assert_eq!(
            validate_unc_path("\\\\NAS01\\Backup\\"),
            Ok("\\\\NAS01\\Backup".to_string())
        );
        assert_eq!(
            validate_unc_path("\\\\192.168.1.10\\Share\\PC1"),
            Ok("\\\\192.168.1.10\\Share\\PC1".to_string())
        );
        assert!(validate_unc_path("\\\\NAS01").is_err());
        assert!(validate_unc_path("D:\\Backup").is_err());
        assert!(validate_unc_path("\\\\NAS01\\Share & calc").is_ok()); // 引数配列渡しのためスペース等は許容、禁止文字のみ拒否
        assert!(validate_unc_path("\\\\NAS01\\Share|calc").is_err());
        assert!(validate_unc_path("-backupTarget:C:").is_err());
    }

    #[test]
    fn test_validate_backup_plan_ok() {
        let drives = sample_drives();
        assert_eq!(
            validate_backup_plan(&['C'], &BackupTarget::LocalDrive('D'), &drives),
            Ok(())
        );
    }

    #[test]
    fn test_validate_backup_plan_errors() {
        let drives = sample_drives();
        assert_eq!(
            validate_backup_plan(&[], &BackupTarget::LocalDrive('D'), &drives),
            Err(ValidationError::NoIncludeDrives)
        );
        assert_eq!(
            validate_backup_plan(&['C'], &BackupTarget::LocalDrive('C'), &drives),
            Err(ValidationError::SameDrive('C'))
        );
        assert_eq!(
            validate_backup_plan(&['C'], &BackupTarget::LocalDrive('E'), &drives),
            Err(ValidationError::SamePhysicalDisk {
                include_drive: 'C',
                target_drive: 'E',
                disk_number: 0,
            })
        );
        assert_eq!(
            validate_backup_plan(&['C'], &BackupTarget::LocalDrive('F'), &drives),
            Err(ValidationError::NotNtfs {
                drive: 'F',
                fs_name: "exFAT".to_string(),
            })
        );
    }

    #[test]
    fn test_validate_credentials() {
        assert_eq!(validate_credentials(None, None), Ok((None, None)));
        assert_eq!(
            validate_credentials(Some(" admin "), Some("pass")),
            Ok((Some("admin".to_string()), Some("pass".to_string())))
        );
        assert!(validate_credentials(None, Some("pass")).is_err());
        assert!(validate_credentials(Some("user\"bad"), Some("pass")).is_err());
    }
}
