use wit_bindgen::generate;

generate!({
    world: "smart-agent",
    path: "../../../wit",
    skip: ["tool"],
    generate_all,
});

struct Component;

impl exports::brio::core::agent_runner::Guest for Component {
    fn run(context: exports::brio::core::agent_runner::TaskContext) -> Result<String, String> {
        let _ = context;
        Ok("Coder Agent: Ready to generate code.".to_string())
    }
}

impl exports::brio::core::event_handler::Guest for Component {
    fn handle_event(topic: String, data: exports::brio::core::event_handler::Payload) {
        let _ = topic;
        let _ = data;
    }
}

export!(Component);
