use camxes_rs::grammars::LOGLAN_GRAMMAR;
use camxes_rs::peg::grammar::Peg;
use log::info;
use std::cell::RefCell;
use web_sys::{self, HtmlSelectElement}; // Import HtmlSelectElement
use yew::prelude::*;

// Define the output format options
#[derive(Clone, PartialEq, Debug)]
enum OutputFormat {
    Debug,
    Json,
}

// Use a simpler RefCell for the Peg instance, initialization handled in the component
thread_local! {
    static LOGLAN_PEG: RefCell<Option<Result<Peg, String>>> = RefCell::new(None);
}

// Function to get or initialize the PEG parser
fn get_or_init_peg() -> Result<Peg, String> {
    LOGLAN_PEG.with(|peg_cell| {
        let mut peg_opt_res = peg_cell.borrow_mut();
        if peg_opt_res.is_none() {
            info!("Initializing Loglan PEG parser...");
            let (start, grammar) = LOGLAN_GRAMMAR;
            let result = Peg::new(start, grammar).map_err(|e| {
                let err_msg = format!("Failed to initialize PEG parser: {}", e);
                log::error!("{}", err_msg);
                err_msg
            });
            if result.is_ok() {
                info!("Loglan PEG parser initialized successfully.");
            }
            *peg_opt_res = Some(result);
        }

        // Clone the result inside the Option
        match peg_opt_res.as_ref().unwrap() {
            Ok(peg) => Ok(peg.clone()), // Clone the Peg if Ok
            Err(e) => Err(e.clone()),   // Clone the error string if Err
        }
    })
}

#[function_component(App)]
pub fn app() -> Html {
    let input_text = use_state(String::new);
    let parse_result = use_state(String::new);
    let error_message = use_state(|| Option::<String>::None); // State for initialization error
    let output_format = use_state(|| OutputFormat::Json); // State for output format

    // Attempt to initialize PEG on first render and handle potential errors
    use_effect_with((), {
        let error_message = error_message.clone();
        move |_| {
            if let Err(e) = get_or_init_peg() {
                error_message.set(Some(e));
            }
            || () // Cleanup function (no-op here)
        }
    });

    let oninput = {
        let input_text_handle = input_text.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            input_text_handle.set(input.value());
        })
    };

    let onclick = {
        let input_text_handle = input_text.clone();
        let parse_result_handle = parse_result.clone();
        let error_message_handle = error_message.clone();
        let output_format_handle = output_format.clone(); // Clone output format handle
        Callback::from(move |_| {
            let input = (*input_text_handle).clone();
            match get_or_init_peg() {
                Ok(peg) => {
                    info!("Parsing input: '{}' with format: {:?}", input, *output_format_handle);
                    let result_str = match *output_format_handle {
                        OutputFormat::Debug => {
                            let result = peg.parse(&input);
                            format!("{:#?}", result)
                        }
                        OutputFormat::Json => {
                             match peg.parse_to_json(&input) {
                                Ok(json) => json,
                                Err(e) => {
                                    log::error!("Failed to serialize result to JSON: {}", e);
                                    format!("Error serializing to JSON: {}", e)
                                }
                            }
                        }
                    };
                    info!("Parse result: {}", result_str);
                    parse_result_handle.set(result_str);
                    error_message_handle.set(None); // Clear any previous init error
                }
                Err(e) => {
                    log::error!("Cannot parse, PEG initialization failed: {}", e);
                    parse_result_handle.set("Error: Parser not initialized.".to_string());
                    error_message_handle.set(Some(e.clone())); // Ensure error message is shown
                }
            }
        })
    };

    let on_format_change = {
        let output_format_handle = output_format.clone();
        Callback::from(move |e: Event| {
            let target = e.target_unchecked_into::<HtmlSelectElement>();
            let value = target.value();
            info!("Output format changed to: {}", value);
            match value.as_str() {
                "json" => output_format_handle.set(OutputFormat::Json),
                _ => output_format_handle.set(OutputFormat::Debug), // Default to Debug
            }
        })
    };

    // Apply Tailwind classes
    html! {
        <div class="flex flex-col items-center p-4 md:p-8 min-h-screen bg-gray-100">
            <div class="w-full max-w-3xl bg-white shadow-md rounded-lg p-6 md:p-8">
                <h1 class="text-2xl md:text-3xl font-bold text-center text-gray-800 mb-4">{ "Loglan Parser (camxes-rs WASM)" }</h1>
                <p class="text-center text-gray-600 mb-6">{ "Enter Loglan text below and click Parse." }</p>

                <textarea
                    rows="6"
                    class="w-full p-3 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent font-mono text-sm mb-4"
                    value={(*input_text).clone()}
                    {oninput}
                    placeholder="Enter Loglan text here... e.g., mi cluva"
                />

                <button
                    {onclick}
                    class="w-full bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded-md transition duration-150 ease-in-out focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-opacity-50 mb-6"
                >
                    { "Parse" }
                </button>

                <div class="mb-6 text-center">
                    <label for="output-format" class="mr-2 text-gray-700">{"Output Format:"}</label>
                    <select
                        id="output-format"
                        class="border border-gray-300 rounded-md p-1 focus:outline-none focus:ring-1 focus:ring-blue-500"
                        onchange={on_format_change}
                    >
                        <option value="json" selected={*output_format == OutputFormat::Json}>{"JSON"}</option>
                        <option value="debug" selected={*output_format == OutputFormat::Debug}>{"Debug"}</option>
                    </select>
                </div>

                // Display Initialization Error if present
                if let Some(err) = &*error_message {
                     <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded relative mb-6" role="alert">
                        <strong class="font-bold">{ "Initialization Error: " }</strong>
                        <span class="block sm:inline">{ "The parser could not be loaded." }</span>
                        <pre class="mt-2 text-xs bg-red-50 p-2 rounded overflow-x-auto">{ err }</pre>
                     </div>
                }

                // Display Parse Result
                <h2 class="text-xl font-semibold text-gray-700 mb-3">{ "Parse Result:" }</h2>
                <pre class="bg-gray-50 border border-gray-200 rounded-md p-4 text-sm text-gray-700 whitespace-pre-wrap break-words overflow-x-auto">
                    { if (*parse_result).is_empty() { "Output will appear here..." } else { &*parse_result } }
                </pre>
            </div>
        </div>
    }
}
