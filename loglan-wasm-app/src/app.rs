use camxes_rs::grammars::LOGLAN_GRAMMAR;
use camxes_rs::peg::grammar::Peg;
use gloo_timers::future::TimeoutFuture;
use log::{error, info};
use std::cell::RefCell;
use wasm_bindgen_futures::spawn_local;
use web_sys::{self, HtmlSelectElement}; // Removed unused HtmlTextAreaElement
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
    let parse_result = use_state(|| AttrValue::from("Output will appear here...")); // Use AttrValue for efficiency
    let error_message = use_state(|| Option::<String>::None); // State for initialization error
    let output_format = use_state(|| OutputFormat::Json); // State for output format
    let copy_button_text = use_state(|| AttrValue::from("Copy")); // State for copy button text

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
                        OutputFormat::Json => match peg.parse_to_json(&input) {
                            Ok(json) => json,
                            Err(e) => {
                                error!("Failed to serialize result to JSON: {}", e);
                                format!("Error serializing to JSON: {}", e)
                            }
                        },
                    };
                    info!("Parse result generated.");
                    parse_result_handle.set(AttrValue::from(result_str)); // Set AttrValue
                    error_message_handle.set(None); // Clear any previous init error
                }
                Err(e) => {
                    error!("Cannot parse, PEG initialization failed: {}", e);
                    parse_result_handle.set(AttrValue::from("Error: Parser not initialized."));
                    error_message_handle.set(Some(e.clone())); // Ensure error message is shown
                }
            }
        })
    };

    let on_copy_click = {
        let parse_result_handle = parse_result.clone();
        let copy_button_text_handle = copy_button_text.clone();
        Callback::from(move |_| {
            let result_text = (*parse_result_handle).clone(); // Clone AttrValue
            let button_text_handle = copy_button_text_handle.clone();
            if let Some(clipboard) = web_sys::window()
                .and_then(|win| Some(win.navigator().clipboard()))
            {
                let promise = clipboard.write_text(&result_text);
                spawn_local(async move {
                    match wasm_bindgen_futures::JsFuture::from(promise).await {
                        Ok(_) => {
                            info!("Text copied to clipboard.");
                            button_text_handle.set(AttrValue::from("Copied!"));
                            // Reset button text after a delay
                            TimeoutFuture::new(1500).await; // Wait 1.5 seconds
                            button_text_handle.set(AttrValue::from("Copy"));
                        }
                        Err(e) => {
                            error!("Failed to copy text: {:?}", e);
                            button_text_handle.set(AttrValue::from("Error"));
                            // Reset button text after a delay
                            TimeoutFuture::new(1500).await;
                            button_text_handle.set(AttrValue::from("Copy"));
                        }
                    }
                });
            } else {
                error!("Clipboard API not available.");
                button_text_handle.set(AttrValue::from("No API"));
                 // Reset button text after a delay
                let button_text_handle_clone = button_text_handle.clone();
                spawn_local(async move {
                    TimeoutFuture::new(1500).await;
                    button_text_handle_clone.set(AttrValue::from("Copy"));
                });
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

                // Display Parse Result Area
                <div class="relative"> // Relative container for positioning the button
                    <h2 class="text-xl font-semibold text-gray-700 mb-3">{ "Parse Result:" }</h2>
                    <textarea
                        readonly=true
                        rows="10" // Adjust rows as needed
                        class="w-full p-3 border border-gray-300 rounded-md bg-gray-50 font-mono text-sm text-gray-700 focus:outline-none focus:ring-1 focus:ring-blue-300" // Added focus style
                        value={(*parse_result).clone()} // Use AttrValue directly
                    />
                    <button
                        onclick={on_copy_click}
                        class="absolute top-10 right-2 bg-gray-200 hover:bg-gray-300 text-gray-700 text-xs font-semibold py-1 px-2 rounded transition duration-150 ease-in-out focus:outline-none focus:ring-1 focus:ring-blue-500"
                        title="Copy result to clipboard"
                    >
                        { (*copy_button_text).clone() } // Use button text state
                    </button>
                </div>
            </div>
        </div>
    }
}
