// Evita que se abra una consola extra en Windows en las versiones de lanzamiento.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    mcgod_lib::run();
}
