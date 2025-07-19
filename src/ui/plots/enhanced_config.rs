#![allow(dead_code)]

//! Enhanced plot configuration system
//! Based on frog-viz best practices with improved structure and validation

use datafusion::arrow::datatypes::DataType;
use egui::{Color32, Pos2, Vec2};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum ColorScheme {
    Viridis,
    Plasma,
    Inferno,
    Magma,
    Cividis,
    Turbo,
    Rainbow,
    Spectral,
    RdYlBu,
    RdYlGn,
    RdBu,
    RdGy,
    PuOr,
    BrBG,
    PiYG,
    PRGn,
    Pastel1,
    Pastel2,
    Set1,
    Set2,
    Set3,
    Tab10,
    Tab20,
    Tab20b,
    Tab20c,
} 