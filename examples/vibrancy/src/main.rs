#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

use millennium::Manager;
use millennium_plugin_vibrancy::{apply_effect, VibrancyEffect};

#[millennium::command]
fn apply(window: millennium::Window, effect: String) -> Result<(), String> {
	apply_effect(
		&window,
		match effect.as_str() {
			"mica" => VibrancyEffect::Mica,
			"fluent-acrylic" => VibrancyEffect::FluentAcrylic,
			"unified-acrylic" => VibrancyEffect::UnifiedAcrylic(None),
			"blurbehind" => VibrancyEffect::Blurbehind(None),
			_ => VibrancyEffect::None
		}
	)
	.map_err(|e| e.to_string())
}

#[millennium::command]
fn clear(window: millennium::Window) -> Result<(), String> {
	apply_effect(&window, VibrancyEffect::None).map_err(|e| e.to_string())
}

fn main() {
	millennium::Builder::default()
		.invoke_handler(millennium::generate_handler![apply, clear])
		.run(millennium::generate_context!())
		.expect("error while running application");
}
