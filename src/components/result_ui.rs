use crate::backend::types::{SegResult, TokenKind};
use leptos::prelude::*; // Updated import

#[component]
pub fn ResultUI(
    /// The processed data passed down from the parent
    data: Signal<SegResult<'static>>,
) -> impl IntoView {
    // Predefined palette of background colors (pastel/vibrant)
    let colors = vec![
        "#ffd90079", // LightGold
        "#ffb6c17a", // LightPink
        "#87cefaa9", // LightSkyBlue
        "#90ee908b", // LightGreen
        "#f4c6f4cb", // LightPlum
        "#ffb99da2", // LightSalmon
        "#20b2ab68", // LightSeaGreen
        "#f0e68c68", // LightKhaki
    ];

    view! {
        <div class="results-container" style="background-color: #ffffff; padding: 2rem; border-radius: 12px; box-shadow: 0 4px 6px rgba(0,0,0,0.05); border: 1px solid #f0f0f0;">

            // List of Segments
            <div>
                <span style="color: #888; font-size: 0.85em; text-transform: uppercase; letter-spacing: 0.05em; font-weight: 600; display: block; margin-bottom: 1rem;">"Analysis Result"</span>
                <ul style="list-style-type: none; padding: 0; margin: 0;">
                    <For
                        each=move || data.get().sentences
                        key=|sent| sent.text.clone()
                        children=move |sentence| {
                            view! {
                                <li style="margin-bottom: 1.5rem; padding: 1.5rem; border-radius: 10px; background-color: #fcfcfc; border: 1px solid #f0f0f0;">
                                    <div style="display: flex; flex-wrap: wrap; gap: 8px; line-height: 1.6;">
                                        {sentence.tokens.into_iter().enumerate().map(|(i, token)| {
                                            let base_style = "padding: 4px 8px; border-radius: 6px; font-size: 0.95em; transition: transform 0.1s;";

                                            let (specific_style, kind_label) = match token.kind {
                                                TokenKind::Word => {
                                                    // Use index to cycle through colors
                                                    let color = colors[i % colors.len()];
                                                    (format!("background-color: {}; color: #333; font-weight: 500;", color), "Word")
                                                },
                                                TokenKind::Number => (
                                                    "background-color: #e9ecef; color: #495057; font-weight: bold; border: 1px solid #ced4da;".to_string(),
                                                    "Number"
                                                ),
                                                TokenKind::Punctuation => (
                                                    "background-color: #f0f0f0ff; color: #333; border-radius: 100%; padding: 4px 8px; font-weight: bold;".to_string(),
                                                    "Punctuation"
                                                ),
                                                TokenKind::Merged => (
                                                    "background-color: #fff3cd; color: #856404; border: 1px solid #ffeeba;".to_string(),
                                                    "Merged Segment"
                                                ),
                                                TokenKind::Other => (
                                                    "color: #6c757d;".to_string(),
                                                    "Other"
                                                ),
                                            };

                                            let tooltip = format!("ID: {}, Offset: {}, Kind: {:?}", token.id, token.offset, kind_label);

                                            view! {
                                                <span
                                                    style=format!("{} {}", base_style, specific_style)
                                                    title=tooltip
                                                >
                                                    {token.text}
                                                </span>
                                            }
                                        }).collect_view()}
                                    </div>
                                </li>
                            }
                        }
                    />
                </ul>
            </div>
        </div>
    }
}
