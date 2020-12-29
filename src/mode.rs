use std::ffi::CString;
use std::iter::ExactSizeIterator;

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
                Mode::AppsMode(mode) => $($wrap)?(mode.$name($($ident),*)),
                Mode::BinAppsMode(mode) => $($wrap)?(mode.$name($($ident),*)),
                Mode::DialogMode(mode) => $($wrap)?(mode.$name($($ident),*)),
            }
        }
    }
}

pub struct EvalInfo<'a> {
    pub index: Option<usize>,
    pub input_value: &'a InputValue<'a>,
}

impl<'a> std::ops::Deref for EvalInfo<'a> {
    type Target = InputValue<'a>;

    fn deref(&self) -> &Self::Target {
        &*self.input_value
    }
}

pub enum Mode {
    AppsMode(apps::AppsMode),
    BinAppsMode(bins::BinsMode),
    DialogMode(dialog::DialogMode),
}

pub struct Entry<'a> {
    pub name: &'a str,
    pub icon: Option<Image<'a>>,
}

impl Mode {
    pub fn apps(entries: Vec<DesktopEntry>, term: Vec<CString>) -> Self {
        Self::AppsMode(apps::AppsMode::new(entries, term))
    }

    pub fn bins(term: Vec<CString>) -> Self {
        Self::BinAppsMode(bins::BinsMode::new(term))
    }

    pub fn dialog() -> Self {
        Self::DialogMode(dialog::DialogMode::new())
    }

    delegate!(pub fn eval(&mut self, info: EvalInfo<'_>) -> std::convert::Infallible);
    delegate!(pub fn entries_len(&self) -> usize);
    delegate!(pub fn entry(&self, idx: usize) -> Entry<'_>);

    pub fn text_entries(&self) -> impl Iterator<Item = &str> + ExactSizeIterator + '_ {
        match self {
            Mode::AppsMode(mode) => Either::Left(Either::Right(mode.text_entries())),
            Mode::BinAppsMode(mode) => Either::Left(Either::Left(mode.text_entries())),
            Mode::DialogMode(mode) => Either::Right(mode.text_entries()),
        }
        .into_iter()
    }
}
