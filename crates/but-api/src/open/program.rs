use std::{
    ffi::OsString,
    path::Path,
    process::{Command, Stdio},
    sync::LazyLock,
};

#[cfg(target_os = "macos")]
use std::path::PathBuf;

use crate::open::spawn::spawn_and_reap;

use serde::Serialize;

const FILEPATH_PLACEHOLDER: &str = "{{filepath}}";
const LINE_NUMBER_PLACEHOLDER: &str = "{{line_number}}";

/// Program type to classify an openable program.
#[derive(Clone, PartialEq)]
pub enum ProgramType {
    /// A text editor/IDE.
    Editor,
    /// Anything that doesn't fit within other types and is not worthwhile to add a new type for.
    Other,
    /// Purely for testing, should never be exposed outside of this module. This is only here until
    /// we have the capability to define custom programs through config.
    Test,
}

/// Supported program to open a file or directory in.
#[derive(Clone)]
pub struct ProgramSpec {
    /// Identifier used to refer to the program.
    pub id: String,
    /// Name of the program.
    pub name: String,
    /// The CLI argument formatter for e.g. opening a specific line in a file.
    cli_arg_supplier: CliArgumentSupplier,
    /// The exuctable to invoke to start the program.
    pub executable: ExecutableProgram,
    /// The kind of the program.
    pub kind: ProgramType,
}

impl ProgramSpec {
    /// True if this is a GUI editor.
    pub fn is_gui_editor(&self) -> bool {
        let requires_terminal = match &self.executable {
            ExecutableProgram::ShellExecutable(ShellExecutable { requires_tty, .. }) => {
                *requires_tty
            }
            #[cfg(target_os = "macos")]
            ExecutableProgram::MacosApplication(_) => false,
        };

        !requires_terminal && self.kind == ProgramType::Editor
    }
}

/// A named executable that can be invoked from a shell.
#[derive(Clone)]
pub struct ShellExecutable {
    /// Name of the executable on the PATH, or a qualified path to it.
    pub name_or_path: String,
    /// Whether the program requires an attached TTY or not.
    ///
    /// If this is true, it means that this program cannot be launched reliably from a GUI client,
    /// and also needs the TUI to suspend in order for the editor to take over the terminal.
    pub requires_tty: bool,
}

/// The executable to invoke for a program.
#[derive(Clone)]
pub enum ExecutableProgram {
    /// A program that can be executed from a shell.
    ShellExecutable(ShellExecutable),
    /// A macOS application installed s.t. it has a bundle identifier.
    #[cfg(target_os = "macos")]
    MacosApplication(MacosApplication),
}

/// Supported editor configuration for API clients.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "export-schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "napi", napi_derive::napi(object))]
pub struct Editor {
    /// Identifier used to refer to the editor.
    pub id: String,
    /// Name of the editor.
    pub name: String,
}

impl From<&ProgramSpec> for Editor {
    fn from(editor: &ProgramSpec) -> Self {
        Self {
            id: editor.id.to_string(),
            name: editor.name.to_string(),
        }
    }
}

#[derive(Clone)]
enum CliArgumentSupplier {
    VSCodeLike,
    Zed,
    Neovim,
    Sublime,
    Custom(CustomCliArgumentSupplier),
    #[cfg(target_os = "macos")]
    Xcode,
    /// For programs that don't support any special "open at" semantics
    #[allow(dead_code)]
    Default,
}

impl CliArgumentSupplier {
    /// Add argument(s) to `cmd` to open the file on the specific line, or error if it's not
    /// supported.
    fn open_at_line<'a>(
        &self,
        cmd: &'a mut Command,
        path: &Path,
        line_nr: i32,
    ) -> anyhow::Result<&'a mut Command> {
        match self {
            Self::VSCodeLike => cmd.arg("--goto").arg(self.path_with_line_nr(path, line_nr)),
            Self::Zed => cmd.arg(self.path_with_line_nr(path, line_nr)),
            Self::Sublime => cmd.arg(self.path_with_line_nr(path, line_nr)),
            #[cfg(target_os = "macos")]
            Self::Xcode => cmd.arg("--line").arg(line_nr.to_string()).arg(path),
            Self::Neovim => cmd.arg(format!("+{line_nr}")).arg(path),
            Self::Custom(open_at_line) => open_at_line.open_at_line(cmd, path, line_nr),
            Self::Default => cmd.arg(path),
        };

        Ok(cmd)
    }

    fn path_with_line_nr(&self, path: &Path, line_nr: i32) -> OsString {
        let mut arg = path.as_os_str().to_owned();
        arg.push(":");
        arg.push(line_nr.to_string());
        arg
    }
}

#[derive(Clone)]
struct CustomCliArgumentSupplier {
    /// Arguments to pass to the executable when invoked to open a file.
    ///
    /// Recognized placeholders:
    ///
    /// * [`FILEPATH_PLACEHOLDER`] is substituted for the filepath
    ///
    /// TODO should not assume utf8 for args
    open_args: Vec<String>,
    /// Arguments to pass to the executable when invoked to open at a specific line.
    ///
    /// Recognized placeholders:
    ///
    /// * [`FILEPATH_PLACEHOLDER`] is substituted for the filepath
    /// * [`LINE_NUMBER_PLACEHOLDER`] is substituted for the line number
    ///
    /// TODO should not assume utf8 for args
    open_at_line_args: Vec<String>,
}

impl CustomCliArgumentSupplier {
    fn open_at_line<'a>(&self, cmd: &'a mut Command, path: &Path, line_nr: i32) -> &'a mut Command {
        for arg in &self.open_at_line_args {
            // TODO should not assume utf8 for path
            cmd.arg(
                arg.replace(FILEPATH_PLACEHOLDER, &path.to_string_lossy())
                    .replace(LINE_NUMBER_PLACEHOLDER, &line_nr.to_string()),
            );
        }
        cmd
    }

    fn open<'a>(&self, cmd: &'a mut Command, path: &Path) -> &'a mut Command {
        for arg in &self.open_args {
            // TODO should not assume utf8 for path
            cmd.arg(arg.replace(FILEPATH_PLACEHOLDER, &path.to_string_lossy()));
        }
        cmd
    }
}

pub(crate) static PROGRAMS: LazyLock<Vec<ProgramSpec>> = LazyLock::new(|| {
    Vec::from([
        ProgramSpec {
            id: "nvim".into(),
            name: "Neovim".into(),
            cli_arg_supplier: CliArgumentSupplier::Neovim,
            executable: ExecutableProgram::ShellExecutable(ShellExecutable {
                name_or_path: "nvim".into(),
                requires_tty: true,
            }),
            kind: ProgramType::Editor,
        },
        ProgramSpec {
            id: "cursor".into(),
            name: "Cursor".into(),
            cli_arg_supplier: CliArgumentSupplier::VSCodeLike,
            #[cfg(not(target_os = "macos"))]
            executable: ExecutableProgram::ShellExecutable(ShellExecutable {
                #[cfg(target_os = "linux")]
                name_or_path: "cursor".into(),
                #[cfg(target_os = "windows")]
                name_or_path: "Cursor.exe".into(),
                requires_tty: false,
            }),
            #[cfg(target_os = "macos")]
            executable: ExecutableProgram::MacosApplication(MacosApplication {
                // This looks insane but it's actually the correct bundle ID, see https://forum.cursor.com/t/cursor-bundle-identifier/779
                bundle_identifier: "com.todesktop.230313mzl4w4u92".into(),
                cli_wrapper_path: Some("Contents/Resources/app/bin/cursor".into()),
            }),
            kind: ProgramType::Editor,
        },
        ProgramSpec {
            id: "sublime".into(),
            name: "Sublime Text".into(),
            cli_arg_supplier: CliArgumentSupplier::Sublime,
            #[cfg(not(target_os = "macos"))]
            executable: ExecutableProgram::ShellExecutable(ShellExecutable {
                #[cfg(target_os = "linux")]
                name_or_path: "subl".into(),
                #[cfg(target_os = "windows")]
                name_or_path: "subl.exe".into(),
                requires_tty: false,
            }),
            #[cfg(target_os = "macos")]
            executable: ExecutableProgram::MacosApplication(MacosApplication {
                bundle_identifier: "com.sublimetext.4".into(),
                cli_wrapper_path: Some("Contents/SharedSupport/bin/subl".into()),
            }),
            kind: ProgramType::Editor,
        },
        ProgramSpec {
            id: "vscode".into(),
            name: "VS Code".into(),
            cli_arg_supplier: CliArgumentSupplier::VSCodeLike,
            #[cfg(not(target_os = "macos"))]
            executable: ExecutableProgram::ShellExecutable(ShellExecutable {
                #[cfg(target_os = "linux")]
                name_or_path: "code".into(),
                #[cfg(target_os = "windows")]
                name_or_path: "code.exe".into(),
                requires_tty: false,
            }),
            #[cfg(target_os = "macos")]
            executable: ExecutableProgram::MacosApplication(MacosApplication {
                bundle_identifier: "com.microsoft.VSCode".into(),
                cli_wrapper_path: Some("Contents/Resources/app/bin/code".into()),
            }),
            kind: ProgramType::Editor,
        },
        #[cfg(target_os = "macos")]
        ProgramSpec {
            id: "xcode".into(),
            name: "Xcode".into(),
            cli_arg_supplier: CliArgumentSupplier::Xcode,
            executable: ExecutableProgram::MacosApplication(MacosApplication {
                bundle_identifier: "com.apple.dt.Xcode".into(),
                cli_wrapper_path: Some("Contents/Developer/usr/bin/xed".into()),
            }),
            kind: ProgramType::Editor,
        },
        ProgramSpec {
            id: "zed".into(),
            name: "Zed".into(),
            cli_arg_supplier: CliArgumentSupplier::Zed,
            #[cfg(not(target_os = "macos"))]
            executable: ExecutableProgram::ShellExecutable(ShellExecutable {
                #[cfg(target_os = "linux")]
                name_or_path: "zed".into(),
                #[cfg(target_os = "windows")]
                name_or_path: "zed.exe".into(),
                requires_tty: false,
            }),
            #[cfg(target_os = "macos")]
            executable: ExecutableProgram::MacosApplication(MacosApplication {
                bundle_identifier: "dev.zed.Zed".into(),
                cli_wrapper_path: Some("Contents/MacOS/cli".into()),
            }),
            kind: ProgramType::Editor,
        },
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        ProgramSpec {
            id: "echo".into(),
            name: "echo".into(),
            cli_arg_supplier: CliArgumentSupplier::Custom(CustomCliArgumentSupplier {
                open_at_line_args: vec![
                    "filepath='{{filepath}}'".into(),
                    "line_number='{{line_number}}'".into(),
                ],
                open_args: vec!["filepath='{{filepath}}'".into()],
            }),
            executable: ExecutableProgram::ShellExecutable(ShellExecutable {
                name_or_path: "echo".into(),
                requires_tty: true,
            }),
            kind: ProgramType::Test,
        },
        #[cfg(target_os = "linux")]
        ProgramSpec {
            id: "thunar".into(),
            name: "Thunar".into(),
            cli_arg_supplier: CliArgumentSupplier::Default,
            executable: ExecutableProgram::ShellExecutable(ShellExecutable {
                name_or_path: "thunar".into(),
                requires_tty: false,
            }),
            kind: ProgramType::Other,
        },
        #[cfg(unix)]
        ProgramSpec {
            id: "nvim-remote".into(),
            name: "Neovim Remote".into(),
            cli_arg_supplier: CliArgumentSupplier::Custom(CustomCliArgumentSupplier {
                open_at_line_args: vec![
                    "--server".into(),
                    "/tmp/nvim-server.pipe".into(),
                    "--remote-expr".into(),
                    "execute('edit +{{line_number}} ' . fnameescape('{{filepath}}'))".into(),
                ],
                open_args: vec![
                    "--server".into(),
                    "/tmp/nvim-server.pipe".into(),
                    "--remote-expr".into(),
                    "execute('edit ' . fnameescape('{{filepath}}'))".into(),
                ],
            }),
            executable: ExecutableProgram::ShellExecutable(ShellExecutable {
                name_or_path: "nvim".into(),
                requires_tty: false,
            }),
            kind: ProgramType::Editor,
        },
    ])
});

/// Low-level API to open a `path` with a specified `program`.
///
/// # WARNING
/// It is up to the caller to assure that the `path` is safe to open and that the `program` is safe
/// to use. Therefore, this should never be exposed to an untrusted context, such as the GUI
/// renderer.
pub fn open_in_program_unchecked(
    program: &ProgramSpec,
    path: &Path,
    line_nr: Option<i32>,
) -> anyhow::Result<()> {
    match &program.executable {
        ExecutableProgram::ShellExecutable(shell_executable) => {
            open_in_shell_executable(shell_executable, &program.cli_arg_supplier, path, line_nr)
        }
        #[cfg(target_os = "macos")]
        ExecutableProgram::MacosApplication(macos_application) => {
            open_in_macos_application(macos_application, &program.cli_arg_supplier, path, line_nr)
        }
    }
}

fn open_in_shell_executable(
    shell_executable: &ShellExecutable,
    cli_arg_supplier: &CliArgumentSupplier,
    path: &Path,
    line_nr: Option<i32>,
) -> anyhow::Result<()> {
    let mut cmd = Command::new(&shell_executable.name_or_path);

    if let Some(line_nr) = line_nr {
        cli_arg_supplier.open_at_line(&mut cmd, path, line_nr)?
    } else if let CliArgumentSupplier::Custom(custom_cli_arg_supplier) = cli_arg_supplier {
        custom_cli_arg_supplier.open(&mut cmd, path)
    } else {
        cmd.arg(path)
    };

    if shell_executable.requires_tty {
        cmd.stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .stdin(Stdio::inherit())
            .status()?;
    } else {
        cmd.stdout(Stdio::null()).stderr(Stdio::null());
        spawn_and_reap(cmd, &shell_executable.name_or_path, &path.to_string_lossy())?;
    }

    Ok(())
}

/// A canonically installed macOS application with a bundle ID and an optional CLI wrapper.
#[cfg(target_os = "macos")]
#[derive(Clone)]
pub struct MacosApplication {
    /// macOS bundle identifier for the application.
    pub bundle_identifier: String,
    /// Location of the CLI wrapper inside the application bundle, if it exists.
    pub cli_wrapper_path: Option<String>,
}

#[cfg(target_os = "macos")]
impl MacosApplication {
    #[cfg(target_os = "macos")]
    fn resolve_cli_wrapper_abspath(&self) -> anyhow::Result<PathBuf> {
        let app_dir_path = self.find_app_directory()?;
        let cli_wrapper_path = self.cli_wrapper_path.as_deref().ok_or_else(|| {
            anyhow::anyhow!("No CLI wrapper configured for {}", self.bundle_identifier)
        })?;
        Ok(app_dir_path.join(cli_wrapper_path))
    }

    #[cfg(target_os = "macos")]
    fn find_app_directory(&self) -> anyhow::Result<PathBuf> {
        use objc2_app_kit::NSWorkspace;
        use objc2_foundation::NSString;

        let workspace = NSWorkspace::sharedWorkspace();
        let bundle_identifier = NSString::from_str(&self.bundle_identifier);
        let app_url = workspace
            .URLForApplicationWithBundleIdentifier(&bundle_identifier)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Could not find application for '{}'",
                    self.bundle_identifier
                )
            })?;

        app_url.to_file_path().ok_or_else(|| {
            anyhow::anyhow!(
                "Could not resolve application path for '{}'",
                self.bundle_identifier
            )
        })
    }
}

#[cfg(target_os = "macos")]
fn open_in_macos_application(
    app: &MacosApplication,
    cli_arg_supplier: &CliArgumentSupplier,
    path: &Path,
    line_nr: Option<i32>,
) -> anyhow::Result<()> {
    if let Some(line_nr) = line_nr {
        let cli_abspath = app.resolve_cli_wrapper_abspath()?;
        let mut cmd = Command::new(cli_abspath);
        cli_arg_supplier.open_at_line(&mut cmd, path, line_nr)?;
        cmd.stdout(Stdio::null()).stderr(Stdio::null());
        spawn_and_reap(cmd, &app.bundle_identifier, &path.to_string_lossy())?;
    } else {
        let mut cmd = Command::new("/usr/bin/open");
        let status = cmd
            .arg("-b")
            .arg(&app.bundle_identifier)
            .arg(path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;

        if !status.success() {
            anyhow::bail!(
                "failed to open {path:?} with app bundle identifier '{}'",
                app.bundle_identifier
            );
        }
    }

    Ok(())
}
