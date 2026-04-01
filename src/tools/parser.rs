use serde_json::Value;

#[derive(Debug, Clone)]
pub struct ToolCall {
    pub name: String,
    pub params: Value,
}

pub struct StreamingParser {
    buffer: String,
}

impl StreamingParser {
    pub fn new() -> Self {
        Self { buffer: String::new() }
    }

    /// Appends the new chunk and returns isolated clean text alongside any parsed tool calls.
    pub fn push(&mut self, chunk: &str) -> (String, Vec<ToolCall>) {
        self.buffer.push_str(chunk);
        
        let mut text_output = String::new();
        let mut tools = vec![];
        
        while let Some(start_idx) = self.buffer.find("<tool_call") {
            // Flush ordinary text preceding the tool call
            if start_idx > 0 {
                text_output.push_str(&self.buffer[..start_idx]);
            }
            
            // Wait for completion sequence
            if let Some(end_idx) = self.buffer[start_idx..].find("</tool_call>") {
                let complete_tag = &self.buffer[start_idx..start_idx + end_idx + 12];
                if let Some(tool) = Self::parse_tag(complete_tag) {
                    tools.push(tool);
                }
                self.buffer = self.buffer[start_idx + end_idx + 12..].to_string();
            } else {
                // Partial tag, keep it strictly in buffer and yield current components
                self.buffer = self.buffer[start_idx..].to_string();
                return (text_output, tools);
            }
        }
        
        // If no full tool open tag is present, we must flush until the last possible partial `<`
        if !self.buffer.contains('<') {
            text_output.push_str(&self.buffer);
            self.buffer.clear();
        } else {
            if let Some(last_lt) = self.buffer.rfind('<') {
                text_output.push_str(&self.buffer[..last_lt]);
                self.buffer = self.buffer[last_lt..].to_string();
            }
        }
        
        (text_output, tools)
    }

    fn parse_tag(tag: &str) -> Option<ToolCall> {
        let name_idx = tag.find("name=\"")? + 6;
        let name_end = tag[name_idx..].find('\"')?;
        let name = &tag[name_idx..name_idx + name_end];
        
        let close_brace_idx = tag.find('>')? + 1;
        let json_end = tag.rfind("</tool_call>")?;
        
        if close_brace_idx < json_end {
            let json_str = &tag[close_brace_idx..json_end];
            if let Ok(params) = serde_json::from_str(json_str) {
                return Some(ToolCall { name: name.to_string(), params });
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_parser_partial() {
        let mut parser = StreamingParser::new();
        let (txt, tools) = parser.push("Thinking about it... <too");
        assert_eq!(txt, "Thinking about it... ");
        assert!(tools.is_empty());
        
        let (txt, tools) = parser.push("l_call name=\"run_shell\">{\"command\":\"l");
        assert_eq!(txt, "");
        assert!(tools.is_empty());

        let (txt, tools) = parser.push("s\"}</tool_call> Done.");
        assert_eq!(txt, "");
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "run_shell");
        
        let (txt2, tools2) = parser.push(" Finalizing.");
        assert_eq!(txt2, " Done. Finalizing.");
        assert!(tools2.is_empty());
    }
}
