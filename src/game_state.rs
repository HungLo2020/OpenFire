use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GameScreen {
    #[default]
    MainMenu,
    InGame,
    Paused,
}
