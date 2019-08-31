use crate::cmd::{Binary, Command, Runnable};
use std::path::PathBuf;
use crate::tools::{binary_path, Tool, CARGO_INSTALL_UPDATE};
use crate::{Toolchain, Workspace};
use failure::Error;

pub(crate) enum CrateName {
    Registry(&'static str),
    Git(&'static str),
}

pub(crate) struct BinaryCrate {
    pub(super) crate_name: CrateName,
    pub(super) binary: &'static str,
    pub(super) cargo_subcommand: Option<&'static str>,
}

impl BinaryCrate {
    pub(crate) fn path(&self, workspace: &Workspace) -> PathBuf {
        binary_path(workspace, self.binary)
    }
}

impl Runnable for BinaryCrate {
    fn name(&self) -> Binary {
        Binary::ManagedByRustwide(if self.cargo_subcommand.is_some() {
            "cargo".into()
        } else {
            self.binary.into()
        })
    }

    fn prepare_command<'w, 'pl>(&self, mut cmd: Command<'w, 'pl>) -> Command<'w, 'pl> {
        if let Some(subcommand) = self.cargo_subcommand {
            cmd = cmd.args(&[subcommand]);
        }
        cmd
    }
}

impl Tool for BinaryCrate {
    fn name(&self) -> &'static str {
        self.binary
    }

    fn is_installed(&self, workspace: &Workspace) -> Result<bool, Error> {
        let path = self.path(workspace);
        if !path.is_file() {
            return Ok(false);
        }

        Ok(crate::native::is_executable(path)?)
    }

    fn install(&self, workspace: &Workspace, fast_install: bool) -> Result<(), Error> {
        let mut cmd = Command::new(workspace, &Toolchain::MAIN.cargo())
            .timeout(None)
            .args(&["install"]);
        cmd = match &self.crate_name {
            CrateName::Registry(name) => cmd.args(&[name]),
            CrateName::Git(url) => cmd.args(&["--git", url]),
        };
        if fast_install {
            cmd = cmd.args(&["--debug"]);
        }
        cmd.run()?;
        Ok(())
    }

    fn update(&self, workspace: &Workspace, _fast_install: bool) -> Result<(), Error> {
        if let CrateName::Registry(crate_name) = &self.crate_name {
            Command::new(workspace, &CARGO_INSTALL_UPDATE)
                .args(&[crate_name])
                .timeout(None)
                .run()?;
        }
        Ok(())
    }
}
