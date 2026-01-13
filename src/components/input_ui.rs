use leptos::prelude::*; // <--- IMPORT THIS

#[component]
pub fn InputUI(set_text: WriteSignal<String>) -> impl IntoView {
    view! {
        <div class="input-container" style="margin-bottom: 2rem; background: #fff; padding: 1.5rem; border-radius: 12px; box-shadow: 0 4px 6px rgba(0,0,0,0.05); border: 1px solid #f0f0f0;">
            <label
                for="text-input"
                style="display: block; margin-bottom: 0.8rem; font-weight: 600; color: #333; font-size: 1.1em;"
            >
                "Enter Text to Analyze"
            </label>
            <style>
                "
                #text-input:focus {
                    border-color: #87CEFA !important;
                    box-shadow: 0 0 0 3px rgba(135, 206, 250, 0.2) !important;
                    outline: none;
                }
                "
            </style>
            <textarea
                id="text-input"
                rows=6
                on:input=move |ev| set_text.set(event_target_value(&ev))
                placeholder="Type or paste text here (English, Pali, etc.)..."
                style="width: 100%; box-sizing: border-box; padding: 1rem; border-radius: 8px; border: 1px solid #e0e0e0; font-family: 'Segoe UI', sans-serif; font-size: 1rem; line-height: 1.5; resize: vertical; transition: border-color 0.2s, box-shadow 0.2s;"
            />
        </div>
    }
}
