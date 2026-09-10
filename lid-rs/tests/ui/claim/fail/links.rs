//! The parts that must be links: every pattern names its trigger by a link in
//! the opener's clause — or, for a ubiquitous claim, in its subject — and a
//! verb whose signature is a shape names its response object by one.
//!
//! `return` is a shape verb, so the third claim's template promises a return
//! type the sentence then never names.
//!
//! The last claim is the one that says *in the subject* rather than *anywhere*:
//! it holds a link, but the link stands after the modal, where it is the
//! response object and not the trigger. Check 13 asks for the trigger before it
//! asks for the object, so what the message names is the subject — an
//! implementation taking the first link of the whole sentence would compile
//! this claim and record `Report` as its trigger.

fn main() {}

/// When the store is unreachable, the run shall stop.
#[derive(lid_rs::Spec)]
struct ATriggerClauseWithoutALink;

/// The run shall stop.
#[derive(lid_rs::Spec)]
struct AUbiquitousSubjectWithoutALink;

/// When the [`Turn`] ends, the run shall return.
#[derive(lid_rs::Spec)]
struct AShapeVerbWithoutAnObject;

/// The run shall return a [`Report`].
#[derive(lid_rs::Spec)]
struct AUbiquitousSubjectLinkedOnlyAfterTheModal;
