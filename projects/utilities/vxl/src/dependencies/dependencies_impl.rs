use crate::{
    CreateTempDir, DirectoryEntry, DisplayImage, ListDir, ReadFile, ResolvePrefsPaths, RunProgram,
    TerminalColumns, WriteFile, WriteStdout,
};
use crossterm::{
    event::{poll, read},
    terminal::{disable_raw_mode, enable_raw_mode, is_raw_mode_enabled},
};
use image::{DynamicImage, RgbaImage};
#[cfg(unix)]
use libc::{STDOUT_FILENO, TIOCGWINSZ, ioctl, winsize};
use meshconv::{
    DependenciesImpl as MeshconvDependenciesImpl, DirectoryEntry as MeshDirectoryEntry,
    ForwardDependencies as ForwardMeshDependencies, ListDir as MeshListDir,
    ReadFile as MeshReadFile, WriteFile as MeshWriteFile,
};
#[cfg(unix)]
use std::mem;
use std::{
    ffi::OsString,
    fs,
    io::{self, Error as IOError, IsTerminal, Result as IOResult, Write},
    path::Path,
    process::Command,
    time::Duration,
};
use tempfile::{Builder, TempDir};
use ty_preferences::{
    Dependencies as PreferencesDependencies, DependenciesImpl as PreferencesDependenciesImpl,
    PrefsPaths, resolve_prefs_paths,
};
use viuer::{Config, print};
use voxconv::{DependenciesImpl as VoxconvDependenciesImpl, ForwardDependencies};

/// The dependencies over the real filesystem, processes, and terminal. The
/// codec and config traits forward to their crates' impls.
#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl ForwardDependencies for DependenciesImpl {
    type Target = VoxconvDependenciesImpl;

    fn target(&self) -> &VoxconvDependenciesImpl {
        &VoxconvDependenciesImpl
    }
}

impl ForwardMeshDependencies for DependenciesImpl {
    type Target = MeshconvDependenciesImpl;

    fn target(&self) -> &MeshconvDependenciesImpl {
        &MeshconvDependenciesImpl
    }
}

impl MeshReadFile for DependenciesImpl {
    fn read_file(&self, path: &Path) -> IOResult<Vec<u8>> {
        ReadFile::read_file(self, path)
    }
}

impl MeshListDir for DependenciesImpl {
    fn list_dir(&self, path: &Path) -> IOResult<Vec<MeshDirectoryEntry>> {
        Ok(ListDir::list_dir(self, path)?
            .into_iter()
            .map(|entry| MeshDirectoryEntry {
                path: entry.path,
                is_dir: entry.is_dir,
            })
            .collect())
    }
}

impl MeshWriteFile for DependenciesImpl {
    fn write_file(&self, path: &Path, bytes: &[u8]) -> IOResult<()> {
        WriteFile::write_file(self, path, bytes)
    }
}

impl PreferencesDependencies for DependenciesImpl {
    fn read_file(&self, path: &Path) -> IOResult<Option<Vec<u8>>> {
        PreferencesDependenciesImpl.read_file(path)
    }

    fn write_file(&self, path: &Path, contents: &[u8]) -> IOResult<()> {
        PreferencesDependenciesImpl.write_file(path, contents)
    }
}

impl ResolvePrefsPaths for DependenciesImpl {
    fn resolve_prefs_paths(&self) -> IOResult<PrefsPaths> {
        resolve_prefs_paths()
    }
}

impl CreateTempDir for DependenciesImpl {
    fn create_temp_dir(&self) -> IOResult<TempDir> {
        Builder::new().prefix("vxl-").tempdir()
    }
}

impl RunProgram for DependenciesImpl {
    fn run_program(&self, program: &str, args: &[OsString]) -> IOResult<Option<i32>> {
        Ok(Command::new(program).args(args).status()?.code())
    }
}

impl ReadFile for DependenciesImpl {
    fn read_file(&self, path: &Path) -> IOResult<Vec<u8>> {
        fs::read(path)
    }
}

impl ListDir for DependenciesImpl {
    fn list_dir(&self, path: &Path) -> IOResult<Vec<DirectoryEntry>> {
        fs::read_dir(path)?
            .map(|entry| {
                let entry = entry?;

                Ok(DirectoryEntry {
                    path: entry.path(),
                    is_dir: entry.file_type()?.is_dir(),
                })
            })
            .collect()
    }
}

impl WriteFile for DependenciesImpl {
    fn write_file(&self, path: &Path, bytes: &[u8]) -> IOResult<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, bytes)
    }
}

impl WriteStdout for DependenciesImpl {
    fn write_stdout(&self, contents: &[u8]) -> IOResult<()> {
        io::stdout().write_all(contents)
    }
}

impl TerminalColumns for DependenciesImpl {
    #[cfg(unix)]
    fn terminal_columns(&self) -> Option<usize> {
        // Safety: winsize is plain data; ioctl fills it for the stdout fd, and
        // the result is read only when the call reports success.
        unsafe {
            let mut size: winsize = mem::zeroed();
            if ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut size) == 0 && size.ws_col > 0 {
                Some(size.ws_col as usize)
            } else {
                None
            }
        }
    }

    /// No terminal-width detection off unix; the `text-rows` layout does not wrap.
    #[cfg(not(unix))]
    fn terminal_columns(&self) -> Option<usize> {
        None
    }
}

impl DisplayImage for DependenciesImpl {
    /// Probes for a graphics protocol only when standard input and output
    /// are both a terminal, because the probe reads its reply from the
    /// terminal in raw mode. A piped run prints half blocks unprobed.
    fn display_image(&self, width: u32, height: u32, rgba: &[u8]) -> IOResult<()> {
        let image = DynamicImage::ImageRgba8(
            RgbaImage::from_raw(width, height, rgba.to_vec()).expect("the samples fill the image"),
        );

        let interactive = io::stdout().is_terminal() && io::stdin().is_terminal();

        let config = display_config(interactive);

        let show = || print(&image, &config).map(|_| ()).map_err(IOError::other);

        if interactive {
            with_capability_probe_guard(show)
        } else {
            show()
        }
    }
}

/// The viuer config for an inline image: anchored at the cursor over the
/// terminal's background, probing for the Kitty and iTerm2 protocols only
/// under `probe`.
fn display_config(probe: bool) -> Config {
    // No Windows terminal supports the Kitty graphics or iTerm2 inline-image
    // protocols, but viuer still probes for them by writing escape sequences
    // and blocking on the reply, which Windows Terminal never sends.
    let probe = probe && !cfg!(windows);

    Config {
        absolute_offset: false,
        transparent: true,
        use_kitty: probe,
        use_iterm: probe,
        ..Default::default()
    }
}

/// Runs `print`, which writes terminal graphics escape sequences, in raw
/// mode, so the capability-probe replies are not echoed as visible garbage.
/// Because viuer toggles raw mode itself, raw mode is re-asserted afterwards
/// and straggling reply bytes are drained before the original mode returns.
fn with_capability_probe_guard<T>(print: impl FnOnce() -> IOResult<T>) -> IOResult<T> {
    let was_raw = is_raw_mode_enabled()?;
    enable_raw_mode()?;

    let result = print();

    let drained = enable_raw_mode().and_then(|()| drain_replies());
    let restored = if was_raw { Ok(()) } else { disable_raw_mode() };

    let value = result?;
    drained?;
    restored?;

    Ok(value)
}

fn drain_replies() -> IOResult<()> {
    while poll(Duration::from_millis(50))? {
        read()?;
    }

    Ok(())
}
