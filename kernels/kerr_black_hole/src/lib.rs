// kernels/kerr_black_hole/src/lib.rs

// Kerr Black Hole Ray Tracing Physics Core
// 
// This library implements null geodesic integration in Kerr spacetime.
// All computations use f64 for maximum precision near critical regions.

// Module declarations
pub mod types;
pub mod coordinates;
pub mod geodesic;
pub mod integration;
pub mod transfer_maps;
pub mod disc_model;
pub mod validation;
pub mod render;

// Re-export main types and functions for convenience
pub use types::{
    OrbitDirection,
    BlackHoleType,
    BlackHole,
    Camera,
    RenderConfig,
    Ray,
};

pub use geodesic::{
    PhotonState,
    GeodesicResult,
};

pub use coordinates::{
    cartesian_to_bl,
    bl_to_cartesian,
    sigma,
    delta,
    a_squared,
};

pub use integration::integrate_geodesic;

pub use transfer_maps::{TransferMaps, Manifest, HighPrecisionData, PositionData, DataStatistics};

pub use disc_model::{
    novikov_thorne_emissivity,
    generate_flux_lut,
    peak_emissivity,
};

pub use validation::{
    check_null_invariant,
    ValidationStats,
};

pub use render::render_transfer_maps;
