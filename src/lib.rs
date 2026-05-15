#![cfg_attr(not(feature = "std"), no_std)]

//! Developer SDK contracts for Alani.
//!
//! `alani-sdk` owns the public boundary for developer CLI commands, repository
//! templates, sysroot management, code generation, local build helpers, and
//! compatibility checks. The crate is dependency-free and `no_std` compatible
//! while config, protocol, and docs repositories stabilize their APIs.

pub mod cli;
pub mod codegen;
pub mod sysroot;
pub mod templates;

pub use cli::{
    CliArgument, CliCommandKind, CliDescriptor, CliInvocation, CliPlan, CliRegistry, CliStatus,
    CLI_SCHEMA_VERSION, MAX_CLI_ARGUMENTS, MAX_CLI_ARGUMENT_LEN, MAX_CLI_NAME_LEN,
    MAX_CLI_WORKDIR_LEN, MAX_COMMANDS,
};
pub use codegen::{
    CodegenArtifact, CodegenDescriptor, CodegenJob, CodegenRegistry, CodegenSchemaKind,
    CodegenStatus, CodegenTarget, CODEGEN_SCHEMA_VERSION, MAX_ARTIFACT_PATH_LEN,
    MAX_CODEGEN_INPUT_LEN, MAX_CODEGEN_JOBS, MAX_CODEGEN_LABEL_LEN, MAX_SCHEMA_NAME_LEN,
};
pub use sysroot::{
    CompatibilityCheck, CompatibilityStatus, SdkHostTriple, SysrootComponent, SysrootDescriptor,
    SysrootLayout, SysrootPlan, SysrootRegistry, SysrootState, MAX_COMPONENTS,
    MAX_COMPONENT_LABEL_LEN, MAX_SYSROOT_PATH_LEN, MAX_TARGET_TRIPLE_LEN, SYSROOT_SCHEMA_VERSION,
};
pub use templates::{
    builtin_repository_template, RenderPlan, TemplateCatalog, TemplateDescriptor, TemplateKind,
    TemplateRecord, TemplateStatus, MAX_TEMPLATES, MAX_TEMPLATE_CONTENT_LEN, MAX_TEMPLATE_FILES,
    MAX_TEMPLATE_LABEL_LEN, MAX_TEMPLATE_PATH_LEN, TEMPLATES_SCHEMA_VERSION,
};

/// Repository name.
pub const REPOSITORY: &str = "alani-sdk";

/// Crate version.
pub const VERSION: &str = "0.1.0";

/// Public module names exposed by this crate.
pub const MODULES: &[&str] = &["cli", "codegen", "templates", "sysroot"];

/// Feature bit for CLI command planning.
pub const SDK_FEATURE_CLI: u64 = 1 << 0;
/// Feature bit for code generation jobs.
pub const SDK_FEATURE_CODEGEN: u64 = 1 << 1;
/// Feature bit for repository templates.
pub const SDK_FEATURE_TEMPLATES: u64 = 1 << 2;
/// Feature bit for sysroot layout and component plans.
pub const SDK_FEATURE_SYSROOT: u64 = 1 << 3;
/// Feature bit for local build-helper contracts.
pub const SDK_FEATURE_BUILD_HELPERS: u64 = 1 << 4;
/// Feature bit for compatibility checks.
pub const SDK_FEATURE_COMPATIBILITY: u64 = 1 << 5;
/// Feature bit for config/protocol/docs schema integration contracts.
pub const SDK_FEATURE_SCHEMA_INTEGRATION: u64 = 1 << 6;
/// Feature bit for data classification and redaction validation.
pub const SDK_FEATURE_REDACTION: u64 = 1 << 7;

/// All SDK feature bits known by this crate version.
pub const SDK_KNOWN_FEATURES: u64 = SDK_FEATURE_CLI
    | SDK_FEATURE_CODEGEN
    | SDK_FEATURE_TEMPLATES
    | SDK_FEATURE_SYSROOT
    | SDK_FEATURE_BUILD_HELPERS
    | SDK_FEATURE_COMPATIBILITY
    | SDK_FEATURE_SCHEMA_INTEGRATION
    | SDK_FEATURE_REDACTION;

/// Caller may inspect SDK metadata.
pub const SDK_RIGHT_READ: u64 = 1 << 0;
/// Caller may plan or execute SDK CLI commands.
pub const SDK_RIGHT_RUN_CLI: u64 = 1 << 1;
/// Caller may run code generation.
pub const SDK_RIGHT_CODEGEN: u64 = 1 << 2;
/// Caller may read templates.
pub const SDK_RIGHT_TEMPLATE_READ: u64 = 1 << 3;
/// Caller may create or update templates.
pub const SDK_RIGHT_TEMPLATE_WRITE: u64 = 1 << 4;
/// Caller may inspect sysroot metadata.
pub const SDK_RIGHT_SYSROOT_READ: u64 = 1 << 5;
/// Caller may modify sysroot layout or components.
pub const SDK_RIGHT_SYSROOT_WRITE: u64 = 1 << 6;
/// Caller may run local build helper plans.
pub const SDK_RIGHT_BUILD: u64 = 1 << 7;
/// Caller may run compatibility checks.
pub const SDK_RIGHT_COMPAT_CHECK: u64 = 1 << 8;
/// Caller may emit or preserve audit evidence.
pub const SDK_RIGHT_AUDIT: u64 = 1 << 9;
/// Caller has administrative SDK authority.
pub const SDK_RIGHT_ADMIN: u64 = 1 << 10;

/// All SDK rights known by this crate version.
pub const SDK_KNOWN_RIGHTS: u64 = SDK_RIGHT_READ
    | SDK_RIGHT_RUN_CLI
    | SDK_RIGHT_CODEGEN
    | SDK_RIGHT_TEMPLATE_READ
    | SDK_RIGHT_TEMPLATE_WRITE
    | SDK_RIGHT_SYSROOT_READ
    | SDK_RIGHT_SYSROOT_WRITE
    | SDK_RIGHT_BUILD
    | SDK_RIGHT_COMPAT_CHECK
    | SDK_RIGHT_AUDIT
    | SDK_RIGHT_ADMIN;

/// Trace flag indicating the SDK event was sampled.
pub const TRACE_FLAG_SAMPLED: u32 = 1 << 0;
/// Trace flag indicating debug metadata may be attached by a trusted sink.
pub const TRACE_FLAG_DEBUG: u32 = 1 << 1;
/// Trace flag indicating a developer-tooling boundary was crossed.
pub const TRACE_FLAG_TOOLING_BOUNDARY: u32 = 1 << 2;
/// Trace flag indicating audit evidence must be preserved.
pub const TRACE_FLAG_AUDIT_REQUIRED: u32 = 1 << 3;

/// Trace flags known by this crate version.
pub const TRACE_KNOWN_FLAGS: u32 =
    TRACE_FLAG_SAMPLED | TRACE_FLAG_DEBUG | TRACE_FLAG_TOOLING_BOUNDARY | TRACE_FLAG_AUDIT_REQUIRED;

/// Result alias for SDK validation and host-mode operations.
pub type SdkResult<T> = Result<T, SdkError>;

/// Error taxonomy for CLI, codegen, template, sysroot, and compatibility contracts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SdkError {
    /// A required field was empty or omitted.
    MissingField,
    /// A bounded field exceeded its documented maximum length.
    FieldTooLong,
    /// A stable label contained a disallowed character.
    InvalidLabel,
    /// Unknown feature, flag, right, or option bits were supplied.
    ReservedBits,
    /// CLI descriptor, argument, or invocation metadata failed validation.
    InvalidCli,
    /// Requested CLI command was not registered.
    CommandNotFound,
    /// Code generation descriptor, job, or artifact metadata failed validation.
    InvalidCodegen,
    /// Requested code generation job was not registered.
    CodegenNotFound,
    /// Template descriptor, record, or render plan metadata failed validation.
    InvalidTemplate,
    /// Requested template was not registered.
    TemplateNotFound,
    /// Sysroot descriptor, component, or plan metadata failed validation.
    InvalidSysroot,
    /// Requested sysroot component was not registered.
    ComponentNotFound,
    /// Compatibility check metadata failed validation or compatibility failed.
    Incompatible,
    /// Caller lacks required SDK authority.
    AccessDenied,
    /// Audit evidence is required but not authorized or available.
    AuditRequired,
    /// Operation attempted to mutate a sealed registry or catalog.
    Sealed,
    /// Fixed-capacity collection is full.
    CapacityExceeded,
    /// Duplicate entry was supplied.
    Duplicate,
    /// State machine transition is not allowed.
    InvalidState,
    /// Trace context was malformed.
    InvalidTrace,
    /// Redaction state is incompatible with data classification.
    InvalidRedaction,
    /// Sensitive content would be exposed without redaction.
    SensitiveData,
    /// Internal invariant failed.
    Internal,
}

impl SdkError {
    /// Stable reason label for diagnostics and tests.
    pub const fn reason(self) -> &'static str {
        match self {
            Self::MissingField => "missing_field",
            Self::FieldTooLong => "field_too_long",
            Self::InvalidLabel => "invalid_label",
            Self::ReservedBits => "reserved_bits",
            Self::InvalidCli => "invalid_cli",
            Self::CommandNotFound => "command_not_found",
            Self::InvalidCodegen => "invalid_codegen",
            Self::CodegenNotFound => "codegen_not_found",
            Self::InvalidTemplate => "invalid_template",
            Self::TemplateNotFound => "template_not_found",
            Self::InvalidSysroot => "invalid_sysroot",
            Self::ComponentNotFound => "component_not_found",
            Self::Incompatible => "incompatible",
            Self::AccessDenied => "access_denied",
            Self::AuditRequired => "audit_required",
            Self::Sealed => "sealed",
            Self::CapacityExceeded => "capacity_exceeded",
            Self::Duplicate => "duplicate",
            Self::InvalidState => "invalid_state",
            Self::InvalidTrace => "invalid_trace",
            Self::InvalidRedaction => "invalid_redaction",
            Self::SensitiveData => "sensitive_data",
            Self::Internal => "internal",
        }
    }

    /// Returns `true` when this error represents a fail-closed trust boundary.
    pub const fn is_security_relevant(self) -> bool {
        matches!(
            self,
            Self::ReservedBits
                | Self::InvalidCli
                | Self::InvalidCodegen
                | Self::InvalidTemplate
                | Self::InvalidSysroot
                | Self::Incompatible
                | Self::AccessDenied
                | Self::AuditRequired
                | Self::Sealed
                | Self::InvalidTrace
                | Self::InvalidRedaction
                | Self::SensitiveData
        )
    }
}

/// Data sensitivity classification for SDK manifests, templates, and output.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DataClass {
    /// Public metadata or content.
    Public = 0,
    /// Operational metadata suitable for trusted developers.
    Operational = 1,
    /// Sensitive data requiring redaction before broad export.
    Sensitive = 2,
    /// Secret data that must not be exported raw.
    Secret = 3,
}

impl DataClass {
    /// Returns `true` when data with this class must be redacted before export.
    pub const fn requires_redaction(self) -> bool {
        matches!(self, Self::Sensitive | Self::Secret)
    }

    /// Stable data class label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Operational => "operational",
            Self::Sensitive => "sensitive",
            Self::Secret => "secret",
        }
    }
}

/// Redaction state applied to SDK metadata, generated artifacts, and diagnostics.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RedactionState {
    /// Public fields only.
    Public = 0,
    /// Operational metadata only.
    Operational = 1,
    /// Sensitive fields were redacted.
    SensitiveRedacted = 2,
    /// Secret fields were redacted.
    SecretRedacted = 3,
    /// Sensitive fields are present and must not be exported broadly.
    UnredactedSensitive = 4,
}

impl RedactionState {
    /// Stable redaction label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Operational => "operational",
            Self::SensitiveRedacted => "sensitive_redacted",
            Self::SecretRedacted => "secret_redacted",
            Self::UnredactedSensitive => "unredacted_sensitive",
        }
    }
}

/// Stable trace context copied from config, protocol, docs, or release layers.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TraceContext {
    /// Trace identifier shared across component boundaries.
    pub trace_id: u64,
    /// Current span identifier.
    pub span_id: u64,
    /// Parent span identifier.
    pub parent_span_id: u64,
    /// Trace flags.
    pub flags: u32,
}

impl TraceContext {
    /// Empty trace context used when no trace is available.
    pub const EMPTY: Self = Self {
        trace_id: 0,
        span_id: 0,
        parent_span_id: 0,
        flags: 0,
    };

    /// Creates a root trace context for an SDK operation.
    pub const fn root(trace_id: u64, span_id: u64) -> Self {
        Self {
            trace_id,
            span_id,
            parent_span_id: 0,
            flags: TRACE_FLAG_SAMPLED | TRACE_FLAG_TOOLING_BOUNDARY,
        }
    }

    /// Creates a child trace context preserving trace flags.
    pub const fn child(self, span_id: u64) -> Self {
        Self {
            trace_id: self.trace_id,
            span_id,
            parent_span_id: self.span_id,
            flags: self.flags,
        }
    }

    /// Sets trace flags.
    pub const fn with_flags(mut self, flags: u32) -> Self {
        self.flags = flags;
        self
    }

    /// Returns `true` when both trace and span identifiers are present.
    pub const fn is_present(self) -> bool {
        self.trace_id != 0 && self.span_id != 0
    }

    /// Validates trace metadata.
    pub const fn validate(self) -> SdkResult<()> {
        if self.flags & !TRACE_KNOWN_FLAGS != 0 {
            return Err(SdkError::ReservedBits);
        }
        if self.trace_id == 0 && self.span_id == 0 && self.parent_span_id == 0 {
            return Ok(());
        }
        if self.trace_id == 0 || self.span_id == 0 {
            return Err(SdkError::InvalidTrace);
        }
        if self.parent_span_id != 0 && self.parent_span_id == self.span_id {
            return Err(SdkError::InvalidTrace);
        }
        Ok(())
    }
}

/// SDK authority bitmap used for fail-closed developer tooling gates.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SdkRights(pub u64);

impl SdkRights {
    /// No authority.
    pub const NONE: Self = Self(0);
    /// Inspect SDK metadata.
    pub const READ: Self = Self(SDK_RIGHT_READ);
    /// Plan or execute CLI commands.
    pub const RUN_CLI: Self = Self(SDK_RIGHT_RUN_CLI);
    /// Run code generation.
    pub const CODEGEN: Self = Self(SDK_RIGHT_CODEGEN);
    /// Read templates.
    pub const TEMPLATE_READ: Self = Self(SDK_RIGHT_TEMPLATE_READ);
    /// Create or update templates.
    pub const TEMPLATE_WRITE: Self = Self(SDK_RIGHT_TEMPLATE_WRITE);
    /// Read sysroot metadata.
    pub const SYSROOT_READ: Self = Self(SDK_RIGHT_SYSROOT_READ);
    /// Modify sysroot layout or components.
    pub const SYSROOT_WRITE: Self = Self(SDK_RIGHT_SYSROOT_WRITE);
    /// Run build helper plans.
    pub const BUILD: Self = Self(SDK_RIGHT_BUILD);
    /// Run compatibility checks.
    pub const COMPAT_CHECK: Self = Self(SDK_RIGHT_COMPAT_CHECK);
    /// Emit or preserve audit evidence.
    pub const AUDIT: Self = Self(SDK_RIGHT_AUDIT);
    /// Administrative SDK authority.
    pub const ADMIN: Self = Self(SDK_RIGHT_ADMIN);
    /// Full authority for host-mode administrative tests.
    pub const ADMINISTRATOR: Self = Self(SDK_KNOWN_RIGHTS);

    /// Creates rights from raw bits after rejecting unknown bits.
    pub const fn from_bits(bits: u64) -> SdkResult<Self> {
        if bits & !SDK_KNOWN_RIGHTS != 0 {
            Err(SdkError::ReservedBits)
        } else {
            Ok(Self(bits))
        }
    }

    /// Returns raw rights bits.
    pub const fn bits(self) -> u64 {
        self.0
    }

    /// Returns `true` when all required rights are present.
    pub const fn contains(self, required: Self) -> bool {
        self.0 & required.0 == required.0
    }

    /// Combines two rights sets.
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Validates reserved bits.
    pub const fn validate(self) -> SdkResult<()> {
        if self.0 & !SDK_KNOWN_RIGHTS != 0 {
            Err(SdkError::ReservedBits)
        } else {
            Ok(())
        }
    }

    /// Fails closed when required rights are absent.
    pub const fn require(self, required: Self) -> SdkResult<()> {
        if self.0 & !SDK_KNOWN_RIGHTS != 0 || required.0 & !SDK_KNOWN_RIGHTS != 0 {
            return Err(SdkError::ReservedBits);
        }
        if self.contains(required) {
            Ok(())
        } else {
            Err(SdkError::AccessDenied)
        }
    }
}

/// Implementation maturity marker for generated repository metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComponentStatus {
    /// API is present as a draft skeleton.
    Draft,
    /// API is implemented enough for host-mode experimentation.
    Experimental,
    /// API is compatible and stable.
    Stable,
}

/// Stable component identity record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComponentInfo {
    /// Repository name.
    pub repository: &'static str,
    /// Crate version.
    pub version: &'static str,
    /// Current implementation status.
    pub status: ComponentStatus,
}

/// Returns stable component identity metadata.
pub const fn component_info() -> ComponentInfo {
    ComponentInfo {
        repository: REPOSITORY,
        version: VERSION,
        status: ComponentStatus::Experimental,
    }
}

/// Returns the repository name.
pub const fn repository_name() -> &'static str {
    REPOSITORY
}

/// Returns public module names.
pub fn module_names() -> &'static [&'static str] {
    MODULES
}

/// Compact root view of the SDK crate contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SdkCatalog {
    /// Repository name.
    pub repository: &'static str,
    /// Crate version.
    pub version: &'static str,
    /// Feature bitmap.
    pub features: u64,
    /// Rights bitmap recognized by this crate version.
    pub rights: u64,
    /// CLI schema version.
    pub cli_schema: &'static str,
    /// Codegen schema version.
    pub codegen_schema: &'static str,
    /// Template schema version.
    pub templates_schema: &'static str,
    /// Sysroot schema version.
    pub sysroot_schema: &'static str,
}

impl SdkCatalog {
    /// Current SDK catalog.
    pub const CURRENT: Self = Self {
        repository: REPOSITORY,
        version: VERSION,
        features: SDK_KNOWN_FEATURES,
        rights: SDK_KNOWN_RIGHTS,
        cli_schema: CLI_SCHEMA_VERSION,
        codegen_schema: CODEGEN_SCHEMA_VERSION,
        templates_schema: TEMPLATES_SCHEMA_VERSION,
        sysroot_schema: SYSROOT_SCHEMA_VERSION,
    };

    /// Validates catalog metadata.
    pub const fn validate(self) -> SdkResult<()> {
        if self.repository.is_empty()
            || self.version.is_empty()
            || self.cli_schema.is_empty()
            || self.codegen_schema.is_empty()
            || self.templates_schema.is_empty()
            || self.sysroot_schema.is_empty()
        {
            return Err(SdkError::MissingField);
        }
        if self.features & !SDK_KNOWN_FEATURES != 0 || self.rights & !SDK_KNOWN_RIGHTS != 0 {
            return Err(SdkError::ReservedBits);
        }
        Ok(())
    }
}

/// Current SDK catalog.
pub const SDK_CATALOG: SdkCatalog = SdkCatalog::CURRENT;

/// Returns the current SDK catalog.
pub const fn sdk_catalog() -> SdkCatalog {
    SdkCatalog::CURRENT
}

/// Validates redaction state for a data class.
pub const fn validate_redaction(data_class: DataClass, redaction: RedactionState) -> SdkResult<()> {
    match data_class {
        DataClass::Public => {
            if matches!(redaction, RedactionState::Public) {
                Ok(())
            } else {
                Err(SdkError::InvalidRedaction)
            }
        }
        DataClass::Operational => {
            if matches!(redaction, RedactionState::Operational) {
                Ok(())
            } else {
                Err(SdkError::InvalidRedaction)
            }
        }
        DataClass::Sensitive => {
            if matches!(
                redaction,
                RedactionState::SensitiveRedacted | RedactionState::SecretRedacted
            ) {
                Ok(())
            } else {
                Err(SdkError::InvalidRedaction)
            }
        }
        DataClass::Secret => {
            if matches!(redaction, RedactionState::SecretRedacted) {
                Ok(())
            } else {
                Err(SdkError::InvalidRedaction)
            }
        }
    }
}

/// Validates a stable SDK label.
pub fn validate_sdk_label(label: &str, max_len: usize) -> SdkResult<()> {
    if label.is_empty() {
        return Err(SdkError::MissingField);
    }
    if label.len() > max_len {
        return Err(SdkError::FieldTooLong);
    }
    if !label.bytes().all(|byte| {
        byte.is_ascii_alphanumeric() || matches!(byte, b':' | b'_' | b'-' | b'.' | b'/' | b'@')
    }) {
        return Err(SdkError::InvalidLabel);
    }
    Ok(())
}
