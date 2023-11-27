use std::{collections::HashMap, path::Path};

use anyhow::{anyhow, Result};

use super::csharpier_process::CSharpierProcess;

#[derive(Debug)]
pub struct ProcessProvider {
    processes: HashMap<String, CSharpierProcess>,
}

impl ProcessProvider {
    pub fn new() -> Self {
        Self {
            processes: HashMap::new(),
        }
    }

    pub async fn get_process(&mut self, file_name: &str) -> Result<&mut CSharpierProcess> {
        let path = Path::new(file_name);

        if path.is_dir() {
            return Err(anyhow!("argument file_name is not a file"));
        }

        let dir = path
            .parent()
            .ok_or(anyhow!("Could not find directory"))?
            .to_str()
            .ok_or(anyhow!("Could not convert directory name into string"))?;

        if !self.processes.contains_key(dir) {
            let new_process = CSharpierProcess::spawn(dir).await?;
            self.processes.insert(dir.to_string(), new_process);
        }

        Ok(self.processes.get_mut(dir).unwrap())
    }
}
