use crate::util;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Profile {
    Debug,
    Release,
}

impl Profile {
    pub fn is_debug(self) -> bool {
        self == Profile::Debug
    }

    pub fn is_release(self) -> bool {
        self == Profile::Release
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Profile::Debug => "debug",
            Profile::Release => "release",
        }
    }
}

pub trait TargetTrait<'a>: Sized {
    fn all() -> &'a BTreeMap<&'a str, Self>;

    fn for_name(name: &str) -> Option<&'a Self> {
        Self::all().get(name)
    }

    fn for_arch(arch: &str) -> Option<&'a Self> {
        Self::all().values().find(|target| target.arch() == arch)
    }

    fn triple(&'a self) -> &'a str;
    fn arch(&'a self) -> &'a str;

    fn rustup_add(&'a self) {
        util::rustup_add(self.triple()).expect("Failed to add target via rustup");
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrElse {
    GiveUp,
    TryAll,
}

impl Default for OrElse {
    fn default() -> Self {
        OrElse::GiveUp
    }
}

#[derive(Default)]
pub struct FallbackBehavior<'a, T: TargetTrait<'a>> {
    // we use `dyn` so the type doesn't need to be known when this is `None`
    pub get_target: Option<&'a dyn Fn() -> Option<&'a T>>,
    pub or_else: OrElse,
}

impl<'a, T: TargetTrait<'a>> FallbackBehavior<'a, T> {
    pub fn get_target(f: &'a dyn Fn() -> Option<&'a T>, or_else: OrElse) -> Self {
        FallbackBehavior {
            get_target: Some(f),
            or_else,
        }
    }

    pub fn all_targets() -> Self {
        FallbackBehavior {
            get_target: None,
            or_else: OrElse::TryAll,
        }
    }
}

pub fn get_targets<'a, Iter, I, T>(
    targets: Option<Iter>,
    fallback: FallbackBehavior<'a, T>,
) -> Option<Vec<&'a T>>
where
    Iter: ExactSizeIterator<Item = &'a I>,
    I: AsRef<str> + 'a,
    T: TargetTrait<'a>,
{
    let targets_empty = targets
        .as_ref()
        .map(|targets| targets.len() == 0)
        .unwrap_or(true);
    if !targets_empty {
        Some(
            targets
                .unwrap()
                .map(|name| T::for_name(name.as_ref()).expect("Invalid target"))
                .collect(),
        )
    } else {
        fallback
            .get_target
            .and_then(|get_target| get_target())
            .map(|target| vec![target])
            .or_else(|| match fallback.or_else {
                OrElse::GiveUp => None,
                OrElse::TryAll => Some(T::all().values().collect()),
            })
    }
}

pub fn call_for_targets<'a, Iter, I, T, F>(
    targets: Option<Iter>,
    fallback: FallbackBehavior<'a, T>,
    f: F,
) where
    Iter: ExactSizeIterator<Item = &'a I>,
    I: AsRef<str> + 'a,
    T: TargetTrait<'a>,
    F: Fn(&T),
{
    let targets = get_targets(targets, fallback).expect("No valid targets specified");
    for target in targets {
        f(target);
    }
}
