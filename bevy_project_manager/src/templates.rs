use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::sync::OnceLock;

pub struct Template<'a> {
    pub file_templates: Vec<FileTemplate<'a>>,
}

impl Template<'_> {
    pub fn get_standard_template() -> Template<'static> {
        let file_templates = vec![
            FileTemplate {
                relative_path: PathBuf::from("src/main.rs"),
                contents: include_bytes!("../../src/main.rs"),
            },
            FileTemplate {
                relative_path: PathBuf::from("src/lib.rs"),
                contents: include_bytes!("../../src/lib.rs"),
            },
            FileTemplate {
                relative_path: PathBuf::from(".cargo/config.toml"),
                contents: include_bytes!("../../.cargo/config.toml"),
            },
            FileTemplate {
                relative_path: PathBuf::from("Cargo.toml"),
                contents: include_bytes!("../../Cargo.toml"),
            },
            FileTemplate {
                relative_path: PathBuf::from("Cargo.lock"),
                contents: include_bytes!("../../Cargo.lock"),
            },
        ];
        Template { file_templates }
    }
    pub fn build_template(&self, path: PathBuf) -> io::Result<()> {
        for template in &self.file_templates {
            let mut path_buf = path.clone();
            path_buf.push(template.relative_path.clone());
            let mut dir_path = path_buf.clone();
            dir_path.pop();
            std::fs::create_dir_all(dir_path)?;
            let mut file = std::fs::OpenOptions::new().create(true)
                .write(true)
                .read(true)
                .open(path_buf)?;
            file.write_all(template.contents)?;
        }
        Ok(())
    }
}

pub struct FileTemplate<'a> {
    pub relative_path: PathBuf,
    pub contents: &'a [u8],
}
