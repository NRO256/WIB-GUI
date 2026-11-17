use crate::domain::decode::StreamLineDecoder;
use crate::domain::wbadmin::{build_stop_job_args, parse_progress_percent};
use crate::platform::win32::{
    ES_CONTINUOUS, ES_SYSTEM_REQUIRED, GetSystemDirectoryW, HANDLE, HWND, INFINITE, PostMessageW,
    SetThreadExecutionState, WM_APP, WaitForSingleObject, decode_cp932,
};
use std::collections::VecDeque;
use std::io::Read;
use std::os::windows::io::AsRawHandle;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

pub const WM_APP_WORKER_EVENT: u32 = WM_APP + 1;
const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const MAX_QUEUE_LEN: usize = 5000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerKind {
    Backup,
    GetVersions,
}

#[derive(Debug, Clone)]
pub enum WorkerEvent {
    LogLine {
        kind: WorkerKind,
        text: String,
    },
    Progress {
        percent: u8,
    },
    Finished {
        kind: WorkerKind,
        exit_code: Option<i32>,
        full_output: String,
    },
    Error {
        kind: WorkerKind,
        message: String,
    },
}

pub fn resolve_wbadmin_path() -> Result<PathBuf, String> {
    let mut buf = [0u16; 260];
    let len = unsafe { GetSystemDirectoryW(buf.as_mut_ptr(), buf.len() as u32) };
    if len > 0 && (len as usize) < buf.len() {
        let sys_dir = String::from_utf16_lossy(&buf[..len as usize]);
        Ok(PathBuf::from(sys_dir).join("wbadmin.exe"))
    } else {
        Err("System32 ディレクトリのパス取得に失敗しました。".to_string())
    }
}

pub struct WorkerHandle {
    kind: WorkerKind,
    child: Arc<Mutex<Option<Child>>>,
    join_handle: Option<JoinHandle<()>>,
}

impl WorkerHandle {
    pub fn kind(&self) -> WorkerKind {
        self.kind
    }

    pub fn request_stop(&self) {
        if self.kind == WorkerKind::Backup {
            // wbadmin.exe の強制終了ではなく正規の停止コマンドで wbengine.exe を停止する
            std::thread::spawn(|| {
                if let Ok(wbadmin) = resolve_wbadmin_path() {
                    let args = build_stop_job_args();
                    let _ = Command::new(wbadmin)
                        .args(&args)
                        .creation_flags(CREATE_NO_WINDOW)
                        .stdin(Stdio::null())
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .status();
                }
            });
        } else if let Ok(mut guard) = self.child.lock()
            && let Some(ref mut ch) = *guard
        {
            let _ = ch.kill();
        }
    }
}

impl Drop for WorkerHandle {
    fn drop(&mut self) {
        if self.kind != WorkerKind::Backup
            && let Ok(mut guard) = self.child.lock()
            && let Some(ref mut ch) = *guard
        {
            let _ = ch.kill();
        }
        // UIスレッドでのブロックを避けるため join_handle は待機せず切り離す
        self.join_handle.take();
    }
}

fn push_event(queue: &Arc<Mutex<VecDeque<WorkerEvent>>>, hwnd_raw: usize, ev: WorkerEvent) {
    if let Ok(mut q) = queue.lock() {
        if q.len() >= MAX_QUEUE_LEN {
            q.pop_front();
        }
        q.push_back(ev);
    }
    unsafe {
        PostMessageW(hwnd_raw as HWND, WM_APP_WORKER_EVENT, 0, 0);
    }
}

fn spawn_pipe_reader<R: Read + Send + 'static>(
    mut reader: R,
    kind: WorkerKind,
    queue: Arc<Mutex<VecDeque<WorkerEvent>>>,
    hwnd_raw: usize,
    output_acc: Arc<Mutex<String>>,
) -> JoinHandle<()> {
    std::thread::spawn(move || {
        let mut decoder = StreamLineDecoder::new(decode_cp932);
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    for line in decoder.push_bytes(&buf[..n]) {
                        emit_line(&line, kind, &queue, hwnd_raw, &output_acc);
                    }
                }
                Err(_) => break,
            }
        }
        if let Some(last) = decoder.finish() {
            emit_line(&last, kind, &queue, hwnd_raw, &output_acc);
        }
    })
}

fn emit_line(
    line: &str,
    kind: WorkerKind,
    queue: &Arc<Mutex<VecDeque<WorkerEvent>>>,
    hwnd_raw: usize,
    output_acc: &Arc<Mutex<String>>,
) {
    if kind == WorkerKind::GetVersions
        && let Ok(mut acc) = output_acc.lock()
    {
        acc.push_str(line);
        acc.push('\n');
    }
    if kind == WorkerKind::Backup
        && let Some(pct) = parse_progress_percent(line)
    {
        push_event(queue, hwnd_raw, WorkerEvent::Progress { percent: pct });
    }
    push_event(
        queue,
        hwnd_raw,
        WorkerEvent::LogLine {
            kind,
            text: line.to_string(),
        },
    );
}

/// wbadmin を非同期起動し、終了まで管理する WorkerHandle を返す。
pub fn spawn_wbadmin(
    hwnd: HWND,
    kind: WorkerKind,
    args: Vec<String>,
    queue: Arc<Mutex<VecDeque<WorkerEvent>>>,
) -> WorkerHandle {
    let hwnd_raw = hwnd as usize;
    let child_arc: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));
    let child_ref = Arc::clone(&child_arc);

    let jh = std::thread::spawn(move || {
        if kind == WorkerKind::Backup {
            unsafe {
                SetThreadExecutionState(ES_CONTINUOUS | ES_SYSTEM_REQUIRED);
            }
        }

        let wbadmin = match resolve_wbadmin_path() {
            Ok(p) => p,
            Err(msg) => {
                if kind == WorkerKind::Backup {
                    unsafe {
                        SetThreadExecutionState(ES_CONTINUOUS);
                    }
                }
                push_event(&queue, hwnd_raw, WorkerEvent::Error { kind, message: msg });
                return;
            }
        };
        let spawn_res = Command::new(&wbadmin)
            .args(&args)
            .creation_flags(CREATE_NO_WINDOW)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        let mut child = match spawn_res {
            Ok(c) => c,
            Err(e) => {
                if kind == WorkerKind::Backup {
                    unsafe {
                        SetThreadExecutionState(ES_CONTINUOUS);
                    }
                }
                push_event(
                    &queue,
                    hwnd_raw,
                    WorkerEvent::Error {
                        kind,
                        message: format!("wbadmin.exe の起動に失敗しました: {e}"),
                    },
                );
                return;
            }
        };

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let raw_handle = child.as_raw_handle() as HANDLE;
        if let Ok(mut guard) = child_ref.lock() {
            *guard = Some(child);
        }

        let output_acc = Arc::new(Mutex::new(String::new()));
        let t_out = stdout.map(|s| {
            spawn_pipe_reader(
                s,
                kind,
                Arc::clone(&queue),
                hwnd_raw,
                Arc::clone(&output_acc),
            )
        });
        let t_err = stderr.map(|s| {
            spawn_pipe_reader(
                s,
                kind,
                Arc::clone(&queue),
                hwnd_raw,
                Arc::clone(&output_acc),
            )
        });

        unsafe {
            WaitForSingleObject(raw_handle, INFINITE);
        }
        let exit_code = child_ref
            .lock()
            .ok()
            .and_then(|mut g| g.as_mut().and_then(|c| c.wait().ok()))
            .and_then(|status| status.code());

        if let Some(t) = t_out {
            let _ = t.join();
        }
        if let Some(t) = t_err {
            let _ = t.join();
        }
        if let Ok(mut guard) = child_ref.lock() {
            *guard = None;
        }
        if kind == WorkerKind::Backup {
            unsafe {
                SetThreadExecutionState(ES_CONTINUOUS);
            }
        }

        let full_output = output_acc
            .lock()
            .map(|mut s| std::mem::take(&mut *s))
            .unwrap_or_default();

        push_event(
            &queue,
            hwnd_raw,
            WorkerEvent::Finished {
                kind,
                exit_code,
                full_output,
            },
        );
    });

    WorkerHandle {
        kind,
        child: child_arc,
        join_handle: Some(jh),
    }
}
