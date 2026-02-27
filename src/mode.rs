use std::ffi::CString;
use std::iter::ExactSizeIterator;

use anyhow::{Context, Result};
use either::Either;
use raqote::Image;

use crate::input_parser::InputValue;
use crate::DesktopEntry;

mod apps;
mod bins;
mod dialog;

macro_rules! delegate {
    (pub fn $name:ident ( &mut self ) -> $ret:ty $(, wrap_with ($wrap:path))?) => {
        delegate!(pub fn $name ( & [mut] self, ) -> $ret $(, wrap_with ($wrap))?);
    };
    (pub fn $name:ident ( &mut self, $($ident:ident : $tp:ty),* ) -> $ret:ty $(, wrap_with ($wrap:path))?) => {
        delegate!(pub fn $name ( & [mut] self, $($ident : $tp),* ) -> $ret $(, wrap_with ($wrap))?);
    };
    (pub fn $name:ident ( & $([$m:ident])? self ) -> $ret:ty $(, wrap_with ($wrap:path))?) => {
        delegate!(pub fn $name ( & $([$m])? self, ) -> $ret $(, wrap_with ($wrap))?);
    };
    (pub fn $name:ident ( & $([$m:ident])? self, $($ident:ident : $tp:ty),* ) -> $ret:ty $(, wrap_with ($wrap:path))?) => {
        pub fn $name ( & $($m)? self, $($ident : $tp),* ) -> $ret {
            match self {
                Mode::Apps(mode) => $($wrap)?(mode.$name($($ident),*)),
                Mode::BinApps(mode) => $($wrap)?(mode.$name($($ident),*)),
                Mode::Dialog(mode) => $($wrap)?(mode.$name($($ident),*)),
            }
        }
    }
}

pub struct EvalInfo<'a> {
    pub index: Option<usize>,
    pub subindex: usize,
    pub input_value: &'a InputValue<'a>,
}

impl<'a> std::ops::Deref for EvalInfo<'a> {
    type Target = InputValue<'a>;

    fn deref(&self) -> &Self::Target {
        self.input_value
    }
}

pub enum Mode {
    Apps(apps::AppsMode),
    BinApps(bins::BinsMode),
    Dialog(dialog::DialogMode),
}

pub struct Entry<'a> {
    pub name: &'a str,
    pub subname: Option<&'a str>,
    pub icon: Option<Image<'a>>,
}

impl Mode {
    pub fn apps(entries: Vec<DesktopEntry>, term: Vec<CString>) -> Self {
        Self::Apps(apps::AppsMode::new(entries, term))
    }

    pub fn bins(term: Vec<CString>) -> Self {
        Self::BinApps(bins::BinsMode::new(term))
    }

    pub fn dialog() -> Result<Self> {
        dialog::DialogMode::new().map(Self::Dialog)
    }

    pub fn dialog_from_lines(lines: Vec<String>) -> Self {
        Self::Dialog(dialog::DialogMode::from_lines(lines))
    }

    pub fn fork_eval(&mut self, info: EvalInfo<'_>) -> Result<()> {
        // Safety:
        // - no need for signal-safety as we single-thread everywhere;
        // - all file descriptors are closed;
        let pid = unsafe { nix::unistd::fork() }.context("fork() error")?;

        if pid.is_child() {
            use std::os::fd::AsRawFd;
            // Just in case, not sure it will break anything.
            let _ = nix::unistd::close(std::io::stdin().as_raw_fd());
            let _ = nix::unistd::close(std::io::stdout().as_raw_fd());
            let _ = nix::unistd::close(std::io::stderr().as_raw_fd());

            self.eval(info)?;
        }

        Ok(())
    }

    delegate!(pub fn eval(&mut self, info: EvalInfo<'_>) -> Result<std::convert::Infallible>);
    delegate!(pub fn entries_len(&self) -> usize);
    delegate!(pub fn subentries_len(&self, idx: usize) -> usize);
    delegate!(pub fn entry(&self, idx: usize, subidx: usize) -> Entry<'_>);

    pub fn text_entries(&self) -> impl ExactSizeIterator<Item = &str> + '_ {
        match self {
            Mode::Apps(mode) => Either::Left(Either::Right(mode.text_entries())),
            Mode::BinApps(mode) => Either::Left(Either::Left(mode.text_entries())),
            Mode::Dialog(mode) => Either::Right(mode.text_entries()),
        }
    }
}
