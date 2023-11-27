use std::process::Stdio;

use anyhow::{anyhow, Result};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, ChildStdout, Command},
    sync::mpsc::{self, Receiver, Sender},
};

async fn read_stdout(stdout: ChildStdout, sender: Sender<Result<String>>) -> Result<()> {
    let reader = BufReader::new(stdout);
    // let mut total_buffer = vec![];
    // let mut buffer = vec![];
    let mut segments = reader.split('\u{0003}' as u8);

    while let Some(segment) = segments.next_segment().await? {
        sender
            .send(String::from_utf8(segment).or_else(|err| Err(anyhow::anyhow!(err))))
            .await?;

        // if length == 0 {
        //     break;
        // }
        //
        // if let Some(idx) = buffer.iter().position(|b| *b == '\u{0003}' as u8) {
        //     total_buffer.extend_from_slice(&buffer[..idx]);
        //
        //     sender
        //         .send(String::from_utf8(total_buffer).or_else(|err| Err(anyhow::anyhow!(err))))
        //         .await?;
        //
        //     total_buffer = buffer[idx + 1..length].to_vec();
        // } else {
        //     total_buffer.extend_from_slice(&buffer[..length]);
        // }
        //
        // buffer.clear();
    }

    Ok(())
}

#[derive(Debug)]
pub struct CSharpierProcess {
    process: Child,
    stdin: ChildStdin,
    output: Receiver<Result<String>>,
}

impl CSharpierProcess {
    pub async fn spawn(working_dir: &str) -> Result<Self> {
        let mut process = Command::new("dotnet-csharpier")
            .arg("--pipe-multiple-files")
            .current_dir(working_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .env("DOTNET_NOLOGO", "1")
            .spawn()?;

        let stdout = process.stdout.take().expect("No handle to stdout");
        let (tx, rx) = mpsc::channel(1024 * 1024);
        tokio::spawn(async {
            read_stdout(stdout, tx).await?;
            Result::<(), anyhow::Error>::Ok(())
        });

        let mut csharpier_process = CSharpierProcess {
            stdin: process.stdin.take().expect("No handle to stdin"),
            process,
            output: rx,
        };

        let warmup_text = "public class ClassName { }";

        csharpier_process
            .format_file(warmup_text, "Text.cs")
            .await?;
        csharpier_process
            .format_file(warmup_text, "Text.cs")
            .await?;

        Ok(csharpier_process)
    }

    pub async fn format_file(&mut self, content: &str, file_path: &str) -> Result<String> {
        // log::debug!("Formatting {}", content);

        log::debug!("Format input length: {}", content.len());
        let input = format!("{file_path}\u{0003}{content}\u{0003}");

        self.stdin
            .write(input.as_bytes())
            .await
            .expect("Could not write to Stdin");

        match self.output.recv().await {
            Some(Ok(output)) => {
                log::debug!("Format output length: {}", output.len());
                if output.len() > 0 {
                    Ok(output)
                } else {
                    Ok(content.to_string())
                }
            }
            Some(Err(err)) => Err(anyhow!("Could not format content. Err: {}", err)),
            None => Err(anyhow!("Could not format content. Empty response")),
        }
    }
}
