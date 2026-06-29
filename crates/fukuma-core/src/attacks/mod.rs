pub mod non_sliding;
pub mod sliding;

pub use non_sliding::{king_attacks, knight_attacks, pawn_attacks, pawn_pushes};
pub use sliding::{bishop_attacks, queen_attacks, rook_attacks};
