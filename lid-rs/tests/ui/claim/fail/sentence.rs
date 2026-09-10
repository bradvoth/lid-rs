//! The one-sentence and one-modal rules of the controlled language: a claim
//! ends with a period, holds no second terminator outside backticks, and says
//! `shall` exactly once.
//!
//! Each struct here breaks one of those rules and nothing else, so the
//! `.stderr` beside this file holds one message per rule. A terminator is a
//! period, a question mark, or an exclamation mark, so the third and fourth
//! structs differ from the second only in which of the three stands in the
//! middle of the sentence — an implementation that scanned for `.` alone would
//! let them through.
//!
//! The last struct's claim spans two doc lines, and what its message quotes is
//! the joined sentence: the lines trimmed and joined by single spaces. Joined
//! by anything else — the empty string, a newline, the lines' own leading
//! spaces — the quoted text would differ, so this is where the joining rule is
//! observed rather than only relied on. Its quoted text is byte-identical to
//! the first struct's, whose claim is the same sentence on one line.

fn main() {}

/// When the [`Turn`] ends, the run shall stop
#[derive(lid_rs::Spec)]
struct NoTerminator;

/// When the [`Turn`] ends, the run shall stop. Then nothing follows.
#[derive(lid_rs::Spec)]
struct ASecondTerminator;

/// When the [`Turn`] ends, the run shall stop? The log closes.
#[derive(lid_rs::Spec)]
struct AQuestionMarkTerminator;

/// When the [`Turn`] ends, the run shall stop! The log closes.
#[derive(lid_rs::Spec)]
struct AnExclamationMarkTerminator;

/// When the [`Turn`] ends, the run shall stop and the log shall close.
#[derive(lid_rs::Spec)]
struct TwoModals;

/// When the [`Turn`] ends, the run stops.
#[derive(lid_rs::Spec)]
struct NoModal;

/// When the [`Turn`] ends,
/// the run shall stop
#[derive(lid_rs::Spec)]
struct JoinedLinesInTheMessage;
