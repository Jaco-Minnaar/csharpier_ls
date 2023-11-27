use anyhow::{anyhow, Result};
use std::collections::HashMap;
use tower_lsp::lsp_types::TextDocumentContentChangeEvent;

#[derive(Debug)]
pub struct Buffers {
    buffers: HashMap<String, String>,
}

impl Buffers {
    pub fn new() -> Self {
        Self {
            buffers: HashMap::new(),
        }
    }

    pub fn update_buffer(
        &mut self,
        file_name: &str,
        changes: &[TextDocumentContentChangeEvent],
    ) -> Result<()> {
        match changes {
            [c] if c.range.is_none() && c.range_length.is_none() => {
                self.buffers.insert(file_name.to_string(), c.text.clone());
                return Ok(());
            }
            _ => return Err(anyhow!("Could not update buffer {}", file_name)),
        }

        // let lines: Vec<&str> = buffer.split_inclusive('\n').collect();
        // let mut new_buffer: String = buffer.to_string();
        // let mut offset: isize = 0;
        //
        // for change in changes {
        //     let range = change.range.unwrap();
        //     let (start_line, end_line) = (range.start.line as usize, range.end.line as usize);
        //     let (start_char, end_char) =
        //         (range.start.character as usize, range.end.character as usize);
        //
        //     let start_idx = lines[..start_line]
        //         .iter()
        //         .map(|line| line.len())
        //         .sum::<usize>()
        //         + start_char;
        //
        //     let start_idx = if offset > 0 {
        //         start_idx + offset as usize
        //     } else {
        //         start_idx - offset.unsigned_abs()
        //     };
        //
        //     let affected_lines = &lines[start_line..=end_line];
        //     let length = affected_lines.len();
        //
        //     let mut matching_str_len = 0;
        //     for (idx, line) in affected_lines.iter().enumerate() {
        //         let val = if idx == 0 {
        //             line[start_char..].len()
        //         } else if idx == length - 1 {
        //             line[..end_char].len()
        //         } else {
        //             line.len()
        //         };
        //
        //         matching_str_len += val;
        //     }
        //
        //     let end_idx = start_idx + matching_str_len;
        //
        //     new_buffer =
        //         new_buffer[..start_idx].to_string() + change.text.as_str() + &new_buffer[end_idx..];
        //
        //     let diff = matching_str_len as isize - change.text.len() as isize;
        //     offset += diff;
        // }

        // dbg!(&new_buffer);
        // self.buffers.insert(file_name.to_string(), new_buffer);
        //
        // Ok(())
    }

    pub fn create_buffer(&mut self, file_name: String, content: String) {
        self.buffers.insert(file_name, content);
    }

    pub fn get_buffer(&self, file_name: &str) -> Option<&str> {
        self.buffers.get(file_name).map(|x| x.as_str())
    }
}
