//! Native system dialogs for opening and saving files.
//!

use js_sys::Array;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::utils::ArrayIterator;
#[derive(Debug, Clone, Copy, Hash, Serialize)]
struct DialogFilter<'a> {
    extensions: &'a [&'a str],
    name: &'a str,
}

/// The FileResponse (from the [Tauri v2 API](https://docs.rs/tauri-plugin-dialog/latest/tauri_plugin_dialog/struct.FileResponse.html))
///
/// Constructs a FileResponse object that is returned when the FileDialog returns a file
#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub struct FileResponse {
    pub base64_data: Option<String>,
    pub duration: Option<u64>,
    pub height: Option<usize>,
    pub width: Option<usize>,
    pub mime_type: Option<String>,
    pub modified_at: Option<u64>,
    pub name: Option<String>,
    pub path: PathBuf,
    pub size: u64,
}

/// The file dialog builder.
///
/// Constructs file picker dialogs that can select single/multiple files or directories.
#[derive(Debug, Default, Clone, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDialogBuilder<'a> {
    default_path: Option<&'a Path>,
    filters: Vec<DialogFilter<'a>>,
    title: Option<&'a str>,
    directory: bool,
    multiple: bool,
    recursive: bool,
}

impl<'a> FileDialogBuilder<'a> {
    /// Gets the default file dialog builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set starting file name or directory of the dialog.
    pub fn set_default_path(&mut self, default_path: &'a Path) -> &mut Self {
        self.default_path = Some(default_path);
        self
    }

    /// If directory is true, indicates that it will be read recursively later.
    /// Defines whether subdirectories will be allowed on the scope or not.
    ///
    /// # Example
    ///
    /// ```rust
    /// use tauri_wasm::plugin::dialog::FileDialogBuilder;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let _builder = FileDialogBuilder::new().set_recursive(true);
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_recursive(&mut self, recursive: bool) -> &mut Self {
        self.recursive = recursive;
        self
    }

    /// Set the title of the dialog.
    ///
    /// # Example
    ///
    /// ```rust
    /// use tauri_wasm::plugin::dialog::FileDialogBuilder;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let _builder = FileDialogBuilder::new().set_title("Test Title");
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_title(&mut self, title: &'a str) -> &mut Self {
        self.title = Some(title);
        self
    }

    /// Add file extension filter. Takes in the name of the filter, and list of extensions
    ///
    /// # Example
    ///
    /// ```rust
    /// use tauri_wasm::plugin::dialog::FileDialogBuilder;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let _builder = FileDialogBuilder::new().add_filter("Image", &["png", "jpeg"]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn add_filter(&mut self, name: &'a str, extensions: &'a [&'a str]) -> &mut Self {
        self.filters.push(DialogFilter { name, extensions });
        self
    }

    /// Add many file extension filters.
    ///
    /// # Example
    ///
    /// ```rust
    /// use tauri_wasm::plugin::dialog::FileDialogBuilder;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let _builder = FileDialogBuilder::new().add_filters(&[("Image", &["png", "jpeg"]),("Video", &["mp4"])]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn add_filters(
        &mut self,
        filters: impl IntoIterator<Item = (&'a str, &'a [&'a str])>,
    ) -> &mut Self {
        for (name, extensions) in filters.into_iter() {
            self.filters.push(DialogFilter {
                name: name.as_ref(),
                extensions,
            });
        }
        self
    }

    /// Shows the dialog to select a single file.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use tauri_wasm::plugin::dialog::FileDialogBuilder;
    ///
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let file = FileDialogBuilder::new().pick_file().await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    pub async fn pick_file(&self) -> crate::Result<Option<FileResponse>> {
        let raw = inner::open(serde_wasm_bindgen::to_value(&self)?).await?;
        // Deserialize into FileData
        let file_data: FileResponse = serde_wasm_bindgen::from_value(raw)?;
        // Return the file data wrapped in Some
        Ok(Some(file_data))
    }

    /// Shows the dialog to select multiple files.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use tauri_wasm::plugin::dialog::FileDialogBuilder;
    ///
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let files = FileDialogBuilder::new().pick_files().await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    pub async fn pick_files(&mut self) -> crate::Result<Option<impl Iterator<Item = FileResponse>>> {
        self.multiple = true;
    
        let raw = inner::open(serde_wasm_bindgen::to_value(&self)?).await?;
    
        if let Ok(files) = Array::try_from(raw) {
            let files = ArrayIterator::new(files)
                .map(|raw| serde_wasm_bindgen::from_value::<FileResponse>(raw).unwrap());
    
            Ok(Some(files))
        } else {
            Ok(None)
        }
    }

    /// Shows the dialog to select a single folder.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use tauri_wasm::plugin::dialog::FileDialogBuilder;
    ///
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let files = FileDialogBuilder::new().pick_folder().await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    pub async fn pick_folder(&mut self) -> crate::Result<Option<PathBuf>> {
        self.directory = true;

        let raw = inner::open(serde_wasm_bindgen::to_value(&self)?).await?;

        Ok(serde_wasm_bindgen::from_value(raw)?)
    }

    /// Shows the dialog to select multiple folders.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use tauri_wasm::plugin::dialog::FileDialogBuilder;
    ///
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let files = FileDialogBuilder::new().pick_folders().await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    pub async fn pick_folders(&mut self) -> crate::Result<Option<impl Iterator<Item = PathBuf>>> {
        self.directory = true;
        self.multiple = true;

        let raw = inner::open(serde_wasm_bindgen::to_value(&self)?).await?;

        if let Ok(files) = Array::try_from(raw) {
            let files =
                ArrayIterator::new(files).map(|raw| serde_wasm_bindgen::from_value(raw).unwrap());

            Ok(Some(files))
        } else {
            Ok(None)
        }
    }

    /// Open a file/directory save dialog.
    ///
    /// The selected path is added to the filesystem and asset protocol allowlist scopes.
    /// When security is more important than the easy of use of this API, prefer writing a dedicated command instead.
    ///
    /// Note that the allowlist scope change is not persisted, so the values are cleared when the application is restarted.
    /// You can save it to the filesystem using tauri-plugin-persisted-scope.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use tauri_wasm::plugin::dialog::FileDialogBuilder;
    ///
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let file = FileDialogBuilder::new().save().await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    pub async fn save(&self) -> crate::Result<Option<PathBuf>> {
        let raw = inner::save(serde_wasm_bindgen::to_value(&self)?).await?;

        Ok(serde_wasm_bindgen::from_value(raw)?)
    }
}

/// Types of message, ask and confirm dialogs.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub enum MessageDialogKind {
    #[default]
    #[serde(rename = "info")]
    Info,
    #[serde(rename = "warning")]
    Warning,
    #[serde(rename = "error")]
    Error,
}

/// A builder for message dialogs.
#[derive(Debug, Default, Clone, Copy, Hash, Serialize)]
pub struct MessageDialogBuilder<'a> {
    title: Option<&'a str>,
    #[serde(rename = "type")]
    kind: MessageDialogKind,
}

impl<'a> MessageDialogBuilder<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the title of the dialog.
    ///
    /// # Example
    ///
    /// ```rust
    /// use tauri_wasm::plugin::dialog::MessageDialogBuilder;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let _builder = MessageDialogBuilder::new().set_title("Test Title");
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_title(&mut self, title: &'a str) -> &mut Self {
        self.title = Some(title);
        self
    }

    /// Set the type of the dialog.
    ///
    /// # Example
    ///
    /// ```rust
    /// use tauri_wasm::plugin::dialog::{MessageDialogBuilder,MessageDialogKind};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let _builder = MessageDialogBuilder::new().set_kind(MessageDialogKind::Error);
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_kind(&mut self, kind: MessageDialogKind) -> &mut Self {
        self.kind = kind;
        self
    }

    /// Shows a message dialog with an `Ok` button.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use tauri_wasm::plugin::dialog::MessageDialogBuilder;
    ///
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let file = MessageDialogBuilder::new().message("Tauri is awesome").await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    pub async fn message(&self, message: &str) -> crate::Result<()> {
        Ok(inner::message(message, serde_wasm_bindgen::to_value(&self)?).await?)
    }

    /// Shows a question dialog with `Yes` and `No` buttons.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use tauri_wasm::plugin::dialog::MessageDialogBuilder;
    ///
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let confirmation = MessageDialogBuilder::new().ask("Are you sure?").await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    pub async fn ask(&self, message: &str) -> crate::Result<bool> {
        let raw = inner::ask(message, serde_wasm_bindgen::to_value(&self)?).await?;

        Ok(serde_wasm_bindgen::from_value(raw)?)
    }

    /// Shows a question dialog with `Ok` and `Cancel` buttons.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use tauri_wasm::plugin::dialog::MessageDialogBuilder;
    ///
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let confirmation = MessageDialogBuilder::new().confirm("Are you sure?").await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    pub async fn confirm(&self, message: &str) -> crate::Result<bool> {
        let raw = inner::confirm(message, serde_wasm_bindgen::to_value(&self)?).await?;

        Ok(serde_wasm_bindgen::from_value(raw)?)
    }
}

mod inner {
    use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

    #[wasm_bindgen(module = "/src/scripts/plugins/dialog.js")]
    extern "C" {
        #[wasm_bindgen(catch)]
        pub async fn ask(message: &str, options: JsValue) -> Result<JsValue, JsValue>;
        #[wasm_bindgen(catch)]
        pub async fn confirm(message: &str, options: JsValue) -> Result<JsValue, JsValue>;
        #[wasm_bindgen(catch)]
        pub async fn open(options: JsValue) -> Result<JsValue, JsValue>;
        #[wasm_bindgen(catch)]
        pub async fn message(message: &str, option: JsValue) -> Result<(), JsValue>;
        #[wasm_bindgen(catch)]
        pub async fn save(options: JsValue) -> Result<JsValue, JsValue>;
    }
}
