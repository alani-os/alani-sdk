//! Repository template catalog boundary.
//!
//! Template contracts describe files and render plans without writing to disk.
//! They are intentionally borrowed and bounded so callers can validate template
//! content before integrating real filesystem behavior.

use crate::{
    validate_redaction, validate_sdk_label, DataClass, RedactionState, SdkError, SdkResult,
    SdkRights, TraceContext,
};

/// Template descriptor schema version.
pub const TEMPLATES_SCHEMA_VERSION: &str = "alani-sdk.templates.v1";
/// Maximum template label length.
pub const MAX_TEMPLATE_LABEL_LEN: usize = 96;
/// Maximum template path length.
pub const MAX_TEMPLATE_PATH_LEN: usize = 192;
/// Maximum template content length.
pub const MAX_TEMPLATE_CONTENT_LEN: usize = 8192;
/// Maximum files in a render plan.
pub const MAX_TEMPLATE_FILES: usize = 64;
/// Default template catalog capacity.
pub const MAX_TEMPLATES: usize = 64;

/// Template family.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TemplateKind {
    /// Repository skeleton template.
    Repository,
    /// Rust crate template.
    RustCrate,
    /// Continuous-integration workflow template.
    CiWorkflow,
    /// Documentation template.
    Documentation,
    /// Configuration template.
    Config,
}

/// Template lifecycle status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TemplateStatus {
    /// Template was declared.
    Draft,
    /// Template is ready for host-mode rendering.
    Ready,
    /// Template was rendered in a side-effect-free plan.
    Rendered,
    /// Template is deprecated.
    Deprecated,
}

/// Template descriptor metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TemplateDescriptor<'a> {
    /// Stable template name.
    pub name: &'a str,
    /// Template schema version.
    pub schema: &'static str,
    /// Template family.
    pub kind: TemplateKind,
    /// Template version label.
    pub version: &'a str,
    /// Rights required to read or render the template.
    pub required_rights: SdkRights,
    /// Whether rendering must preserve audit evidence.
    pub requires_audit: bool,
    /// Data class for template metadata and content.
    pub data_class: DataClass,
    /// Redaction state for template metadata and content.
    pub redaction: RedactionState,
    /// Trace context for this template.
    pub trace: TraceContext,
}

impl<'a> TemplateDescriptor<'a> {
    /// Creates a template descriptor.
    pub const fn new(name: &'a str, kind: TemplateKind, version: &'a str) -> Self {
        Self {
            name,
            schema: TEMPLATES_SCHEMA_VERSION,
            kind,
            version,
            required_rights: SdkRights::TEMPLATE_READ,
            requires_audit: false,
            data_class: DataClass::Operational,
            redaction: RedactionState::Operational,
            trace: TraceContext::EMPTY,
        }
    }

    /// Overrides required rights.
    pub const fn with_rights(mut self, rights: SdkRights) -> Self {
        self.required_rights = rights;
        self
    }

    /// Marks rendering as audit-required.
    pub const fn with_audit(mut self) -> Self {
        self.requires_audit = true;
        self
    }

    /// Overrides classification and redaction metadata.
    pub const fn with_data(mut self, data_class: DataClass, redaction: RedactionState) -> Self {
        self.data_class = data_class;
        self.redaction = redaction;
        self
    }

    /// Attaches trace metadata.
    pub const fn with_trace(mut self, trace: TraceContext) -> Self {
        self.trace = trace;
        self
    }

    /// Validates template descriptor metadata.
    pub fn validate(self) -> SdkResult<()> {
        validate_sdk_label(self.name, MAX_TEMPLATE_LABEL_LEN)
            .map_err(|_| SdkError::InvalidTemplate)?;
        validate_sdk_label(self.version, MAX_TEMPLATE_LABEL_LEN)
            .map_err(|_| SdkError::InvalidTemplate)?;
        if self.schema.is_empty() {
            return Err(SdkError::InvalidTemplate);
        }
        self.required_rights.validate()?;
        if self.requires_audit && !self.required_rights.contains(SdkRights::AUDIT) {
            return Err(SdkError::AuditRequired);
        }
        if self.data_class.requires_redaction()
            && matches!(self.redaction, RedactionState::UnredactedSensitive)
        {
            return Err(SdkError::SensitiveData);
        }
        validate_redaction(self.data_class, self.redaction)?;
        self.trace.validate()
    }
}

/// Borrowed template file record.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TemplateRecord<'a> {
    /// Template descriptor.
    pub descriptor: TemplateDescriptor<'a>,
    /// Relative output path.
    pub path: &'a str,
    /// Borrowed template content.
    pub content: &'a str,
    /// Template status.
    pub status: TemplateStatus,
}

impl<'a> TemplateRecord<'a> {
    /// Creates a template record.
    pub const fn new(
        descriptor: TemplateDescriptor<'a>,
        path: &'a str,
        content: &'a str,
        status: TemplateStatus,
    ) -> Self {
        Self {
            descriptor,
            path,
            content,
            status,
        }
    }

    /// Validates template record metadata and content bounds.
    pub fn validate(self) -> SdkResult<()> {
        self.descriptor.validate()?;
        validate_sdk_label(self.path, MAX_TEMPLATE_PATH_LEN)
            .map_err(|_| SdkError::InvalidTemplate)?;
        if self.content.is_empty() {
            return Err(SdkError::MissingField);
        }
        if self.content.len() > MAX_TEMPLATE_CONTENT_LEN {
            return Err(SdkError::FieldTooLong);
        }
        if self.descriptor.data_class.requires_redaction()
            && matches!(
                self.descriptor.redaction,
                RedactionState::UnredactedSensitive
            )
        {
            return Err(SdkError::SensitiveData);
        }
        Ok(())
    }
}

/// Side-effect-free render plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderPlan<'a, const N: usize> {
    /// Template descriptor.
    pub descriptor: TemplateDescriptor<'a>,
    /// Destination root label.
    pub destination_root: &'a str,
    files: [Option<TemplateRecord<'a>>; N],
    len: usize,
    /// Trace context for rendering.
    pub trace: TraceContext,
}

impl<'a, const N: usize> RenderPlan<'a, N> {
    /// Creates an empty render plan.
    pub fn new(
        descriptor: TemplateDescriptor<'a>,
        destination_root: &'a str,
        trace: TraceContext,
    ) -> SdkResult<Self> {
        descriptor.validate()?;
        validate_sdk_label(destination_root, MAX_TEMPLATE_PATH_LEN)
            .map_err(|_| SdkError::InvalidTemplate)?;
        trace.validate()?;
        Ok(Self {
            descriptor,
            destination_root,
            files: [None; N],
            len: 0,
            trace,
        })
    }

    /// Returns the number of files in the render plan.
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` when the render plan has no files.
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns a file by index.
    pub fn file(&self, index: usize) -> Option<TemplateRecord<'a>> {
        if index >= self.len {
            None
        } else {
            self.files[index]
        }
    }

    /// Adds a template file to the plan.
    pub fn push_file(&mut self, record: TemplateRecord<'a>) -> SdkResult<()> {
        if self.len >= N {
            return Err(SdkError::CapacityExceeded);
        }
        record.validate()?;
        self.files[self.len] = Some(record);
        self.len += 1;
        Ok(())
    }

    /// Checks whether a caller may render the plan.
    pub fn authorize(&self, caller: SdkRights, audit_ready: bool) -> SdkResult<()> {
        caller.validate()?;
        caller.require(self.descriptor.required_rights)?;
        if self.descriptor.requires_audit && (!audit_ready || !caller.contains(SdkRights::AUDIT)) {
            return Err(SdkError::AuditRequired);
        }
        Ok(())
    }
}

/// Fixed-capacity template catalog.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemplateCatalog<'a, const N: usize> {
    entries: [Option<TemplateRecord<'a>>; N],
    len: usize,
    sealed: bool,
}

impl<'a, const N: usize> TemplateCatalog<'a, N> {
    /// Creates an empty template catalog.
    pub const fn new() -> Self {
        Self {
            entries: [None; N],
            len: 0,
            sealed: false,
        }
    }

    /// Returns the number of registered template files.
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` when no template files are registered.
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Prevents further registrations.
    pub fn seal(&mut self) {
        self.sealed = true;
    }

    /// Registers a template record.
    pub fn register(&mut self, record: TemplateRecord<'a>) -> SdkResult<()> {
        if self.sealed {
            return Err(SdkError::Sealed);
        }
        record.validate()?;
        if self.find(record.descriptor.name, record.path).is_ok() {
            return Err(SdkError::Duplicate);
        }
        if self.len >= N {
            return Err(SdkError::CapacityExceeded);
        }
        self.entries[self.len] = Some(record);
        self.len += 1;
        Ok(())
    }

    /// Finds a template file by template name and path.
    pub fn find(&self, name: &str, path: &str) -> SdkResult<TemplateRecord<'a>> {
        validate_sdk_label(name, MAX_TEMPLATE_LABEL_LEN)?;
        validate_sdk_label(path, MAX_TEMPLATE_PATH_LEN)?;
        for record in self.entries.iter().take(self.len).flatten() {
            if record.descriptor.name == name && record.path == path {
                return Ok(*record);
            }
        }
        Err(SdkError::TemplateNotFound)
    }
}

impl<'a, const N: usize> Default for TemplateCatalog<'a, N> {
    fn default() -> Self {
        Self::new()
    }
}

/// Built-in repository template record used by smoke tests.
pub const fn builtin_repository_template<'a>(trace: TraceContext) -> TemplateRecord<'a> {
    let descriptor =
        TemplateDescriptor::new("repo.rust", TemplateKind::RustCrate, "0.1.0").with_trace(trace);
    TemplateRecord::new(
        descriptor,
        "Cargo.toml",
        "[package]\nname = \"alani-example\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        TemplateStatus::Ready,
    )
}
