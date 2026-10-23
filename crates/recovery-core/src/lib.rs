//! Dependency-free contracts for future source-backed game recovery.
//!
//! This crate intentionally contains no ESP Ra.De. Psi gameplay implementation.

/// Current evidence status of the game implementation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GameStatus {
    /// No recovered game routine is integrated into the game-facing core.
    Unknown,
}

/// Stable logical controls accepted by the future recovered core.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Control {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
    Confirm = 4,
    Cancel = 5,
    Start = 6,
}

/// A normalized digital and signed-axis snapshot for one logical frame.
///
/// Axes use the full i16 range, where -32768 and 32767 represent the extremes.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct InputFrame {
    buttons: u16,
    pub axis_x: i16,
    pub axis_y: i16,
}

impl InputFrame {
    pub const fn new(buttons: u16, axis_x: i16, axis_y: i16) -> Self {
        Self {
            buttons,
            axis_x,
            axis_y,
        }
    }

    pub const fn is_down(self, control: Control) -> bool {
        self.buttons & (1 << control as u8) != 0
    }

    pub const fn with_down(mut self, control: Control, down: bool) -> Self {
        let bit = 1 << control as u8;
        if down {
            self.buttons |= bit;
        } else {
            self.buttons &= !bit;
        }
        self
    }
}

/// Button state transitions between two successive logical frames.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ButtonEdges {
    pub pressed: u16,
    pub released: u16,
}

impl ButtonEdges {
    pub const fn between(previous: InputFrame, current: InputFrame) -> Self {
        Self {
            pressed: current.buttons & !previous.buttons,
            released: previous.buttons & !current.buttons,
        }
    }

    pub const fn just_pressed(self, control: Control) -> bool {
        self.pressed & (1 << control as u8) != 0
    }

    pub const fn just_released(self, control: Control) -> bool {
        self.released & (1 << control as u8) != 0
    }

    /// Accumulates transitions observed before the next fixed update.
    pub const fn merge(self, later: Self) -> Self {
        Self {
            pressed: self.pressed | later.pressed,
            released: self.released | later.released,
        }
    }
}

/// The explicit integration seam for source-backed game state updates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TickError {
    NotRecovered,
}

/// Stub for the future recovered game-state implementation.
#[derive(Default)]
pub struct RecoveredCore;

impl RecoveredCore {
    /// No game update is implemented yet; input contract is not game behavior.
    pub const fn tick(&mut self, _input: InputFrame, _edges: ButtonEdges) -> Result<(), TickError> {
        Err(TickError::NotRecovered)
    }
}

/// Honest scaffold status; no synthetic or placeholder behavior is exposed as game logic.
pub const fn game_status() -> GameStatus {
    GameStatus::Unknown
}
