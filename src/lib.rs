// Разрешаем dead_code для библиотечных элементов,
// которые могут не использоваться в bin, но публичны для внешнего API
#![allow(dead_code)]

pub mod config;
pub mod anonymizer;
pub mod api;
pub mod mcp;
pub mod middleware;
pub mod models;
