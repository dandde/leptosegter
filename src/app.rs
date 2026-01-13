use crate::backend::pli_segmenter::process_text;
use crate::components::input_ui::InputUI;
use crate::components::result_ui::ResultUI;
use leptos::prelude::*; // Use prelude for Leptos 0.7+

#[component]
pub fn App() -> impl IntoView {
    // 1. State: Raw Input
    // create_signal remains the standard way to get (ReadSignal, WriteSignal)
    let (text, set_text) = signal("1. Tena samayena buddho bhagavā verañjāyaṃ viharati naḷerupucimandamūle mahatā bhikkhusaṅghena saddhiṃ pañcamattehi bhikkhusatehi. Assosi kho verañjo brāhmaṇo – ‘‘samaṇo khalu, bho, gotamo sakyaputto sakyakulā pabbajito verañjāyaṃ viharati naḷerupucimandamūle mahatā bhikkhusaṅghena saddhiṃ pañcamattehi bhikkhusatehi. Taṃ kho pana bhavantaṃ gotamaṃ evaṃ kalyāṇo kittisaddo abbhuggato – ‘itipi so bhagavā arahaṃ sammāsambuddho vijjācaraṇasampanno sugato lokavidū anuttaro purisadammasārathi satthā devamanussānaṃ buddho bhagavā [bhagavāti (syā.), dī. ni. 1.157, abbhuggatākārena pana sameti]. So imaṃ lokaṃ sadevakaṃ samārakaṃ sabrahmakaṃ sassamaṇabrāhmaṇiṃ pajaṃ sadevamanussaṃ sayaṃ abhiññā sacchikatvā pavedeti. So dhammaṃ deseti ādikalyāṇaṃ majjhekalyāṇaṃ pariyosānakalyāṇaṃ sātthaṃ sabyañjanaṃ; kevalaparipuṇṇaṃ parisuddhaṃ brahmacariyaṃ pakāseti; sādhu kho pana tathārūpānaṃ arahataṃ dassanaṃ hotī’’’ti.".to_string());

    // 2. State: Derived Data (Memoized)
    // FIX: Replaced create_memo(...) with Memo::new(...)
    let processing_result = Memo::new(move |_| process_text(&text.get()).to_owned_data());

    // 3. View Layout
    view! {
        <main style="max-width: 800px; margin: 0 auto; padding: 2rem; font-family: sans-serif;">
            <h1 style="text-align: center; color: #333;">"Wasm Auto-Segmenter"</h1>

            // Component: Input
            <InputUI set_text=set_text />

            // Component: Output
            // .into() converts the Memo<SegmentationResult> into a Signal<SegmentationResult>
            <ResultUI data=processing_result.into() />
        </main>
    }
}
