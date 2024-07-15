#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use reqwest::Client;
use tauri::{
    api::notification::Notification, AppHandle, CustomMenuItem, GlobalShortcutManager, Manager,
    SystemTray, SystemTrayEvent, SystemTrayMenu,
};
use tauri_plugin_clipboard::ClipboardManager;

fn main() {
    let tray_menu = SystemTrayMenu::new().add_item(CustomMenuItem::new("quit".to_string(), "Quit"));
    let tray = SystemTray::new().with_menu(tray_menu);

    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard::init())
        .system_tray(tray)
        .on_system_tray_event(on_system_tray_event)
        .setup(|app| {
            // Register the global shortcut here
            let app_handle = app.handle();
            app.global_shortcut_manager()
                .register("Control+Shift+P", move || {
                    handle_shortcut(app_handle.clone())
                })
                .expect("Failed to register global shortcut");
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app_handle, event| match event {
            tauri::RunEvent::ExitRequested { api, .. } => {
                api.prevent_exit();
            }
            _ => {}
        });
}

fn on_system_tray_event(app: &AppHandle, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
            "quit" => app.exit(0),
            _ => {}
        },
        _ => {}
    }
}

async fn make_post_request(
    input_text: String,
    app_handle: tauri::AppHandle,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let url = "http://localhost:11434/v1/chat/completions";
    let payload = serde_json::json!({
        "model": "deepseek-coder-v2:16b-lite-instruct-q8_0", // TOO: set as an external config
        "temperature": 0.0,
        "messages" : [
            {
                "role": "system",
                "content": "As a proofreading assistant, your task is to review user-provided text and make only spelling corrections. Follow these guidelines:
1. Correct misspelled words to their proper spelling.
2. Do not alter grammar, punctuation, capitalization, or sentence structure.
3. Preserve the original formatting, including line breaks and paragraphs.
4. The input text may range from a single word to multiple paragraphs. Apply the same rules regardless of length.
5. Return only the corrected text, without any additional comments or explanations.

IMPORTANT: Return only the corrected text. IMPORTANT:Return only the corrected text. ONLY THAT! Nothing else."
            },
            {
                "role": "user",
                "content": input_text
            }
        ]
    });

    let res = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await?;

    if res.status().is_success() {
        //TODO: tidy up this extraction
        let res = res.text().await?;
        let parsed = serde_json::from_str::<serde_json::Value>(&res)?;
        let choices = parsed["choices"]
            .as_array()
            .ok_or("No choices in response")?;
        let first_choice = choices.get(0).ok_or("No first choice in response")?;
        let message = first_choice["message"]["content"]
            .as_str()
            .ok_or("No message content in response")?;

        let trimmed_text = message.trim_start();

        // Update clipboard with corrected text
        let clipboard: tauri::State<ClipboardManager> = app_handle.state();
        clipboard.write_text(trimmed_text.to_string())?;

        Notification::new(app_handle.package_info().name.clone())
            .title("Proofreading Complete")
            .show()?;
    } else {
        println!("Request failed with status: {}", res.status());
    }

    Ok(())
}

fn handle_shortcut(app: AppHandle) {
    let clipboard: tauri::State<ClipboardManager> = app.state();
    // Gaurd against empty clipboard
    if clipboard.has_text().unwrap_or(false) {
        let text = clipboard.read_text().unwrap_or_default();
        let app_handle = app.clone();

        tauri::async_runtime::spawn(async move {
            if let Err(e) = make_post_request(text, app_handle).await {
                eprintln!("Error making post request: {}", e);
            }
        });
    } else {
        println!("Clipboard is empty, skipping the post request");
    }
}
