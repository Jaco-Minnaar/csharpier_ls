use std::process::Stdio;

use anyhow::{anyhow, Result};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, ChildStdout, Command},
    sync::mpsc::{self, Receiver, Sender},
};

async fn read_stdout(stdout: ChildStdout, sender: Sender<Result<String>>) -> Result<()> {
    let reader = BufReader::new(stdout);
    let mut segments = reader.split('\u{0003}' as u8);

    while let Some(segment) = segments.next_segment().await? {
        sender
            .send(String::from_utf8(segment).or_else(|err| Err(anyhow::anyhow!(err))))
            .await?;
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

    pub async fn format_file(&mut self, content: &str, file_path: &str) -> Result<Option<String>> {
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
                let len = output.len();
                if len == 0 || len == content.len() {
                    log::debug!("No changes to apply");
                    Ok(None)
                } else {
                    Ok(Some(output.to_string()))
                }
            }
            Some(Err(err)) => Err(anyhow!("Could not format content. Err: {}", err)),
            None => Err(anyhow!("Could not format content. Empty response")),
        }
    }
}
