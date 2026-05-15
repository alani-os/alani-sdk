//! Code generation job boundary.
//!
//! Codegen contracts describe schema inputs, target outputs, generated
//! artifacts, and compatibility requirements without reading or writing files.

use crate::{
    validate_redaction, validate_sdk_label, DataClass, RedactionState, SdkError, SdkResult,
    SdkRights, TraceContext,
};

/// Codegen descriptor schema version.
pub const CODEGEN_SCHEMA_VERSION: &str = "alani-sdk.codegen.v1";
/// Maximum codegen label length.
pub const MAX_CODEGEN_LABEL_LEN: usize = 64;
/// Maximum source schema name length.
pub const MAX_SCHEMA_NAME_LEN: usize = 128;
/// Maximum generated artifact path length.
pub const MAX_ARTIFACT_PATH_LEN: usize = 192;
/// Maximum source input length accepted by the skeleton.
pub const MAX_CODEGEN_INPUT_LEN: usize = 8192;
/// Default codegen registry capacity.
pub const MAX_CODEGEN_JOBS: usize = 64;

/// Schema family consumed by code generation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodegenSchemaKind {
    /// Alani ABI schema.
    Abi,
    /// Protocol schema.
    Protocol,
    /// Configuration schema.
    Config,
    /// Documentation metadata schema.
    Docs,
    /// Repository-template manifest schema.
    Repository,
    /// CLI contract schema.
    Cli,
}

/// Code generation target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodegenTarget {
    /// Rust source bindings.
    Rust,
    /// Markdown reference output.
    Markdown,
    /// JSON Schema output.
    JsonSchema,
    /// TOML-like configuration output.
    Toml,
    /// POSIX shell helper output.
    Shell,
}

/// Code generation lifecycle status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodegenStatus {
    /// Job was declared.
    Declared,
    /// Job is ready for host-mode validation.
    Ready,
    /// Job completed.
    Generated,
    /// Job failed validation or generation.
    Failed,
    /// Job was denied before generation.
    Denied,
}

/// Code generation job descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CodegenDescriptor<'a> {
    /// Stable job name.
    pub name: &'a str,
    /// Codegen schema version.
    pub schema: &'static str,
    /// Source schema family.
    pub schema_kind: CodegenSchemaKind,
    /// Output target.
    pub target: CodegenTarget,
    /// Rights required to run the job.
    pub required_rights: SdkRights,
    /// Whether generated artifacts must preserve audit evidence.
    pub requires_audit: bool,
    /// Data class for source and artifact metadata.
    pub data_class: DataClass,
    /// Redaction state for source and artifact metadata.
    pub redaction: RedactionState,
    /// Trace context attached to the job.
    pub trace: TraceContext,
}

impl<'a> CodegenDescriptor<'a> {
    /// Creates a code generation descriptor.
    pub const fn new(name: &'a str, schema_kind: CodegenSchemaKind, target: CodegenTarget) -> Self {
        Self {
            name,
            schema: CODEGEN_SCHEMA_VERSION,
            schema_kind,
            target,
            required_rights: SdkRights::CODEGEN,
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

    /// Marks the job as audit-required.
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

    /// Validates descriptor metadata.
    pub fn validate(self) -> SdkResult<()> {
        validate_sdk_label(self.name, MAX_CODEGEN_LABEL_LEN)
            .map_err(|_| SdkError::InvalidCodegen)?;
        if self.schema.is_empty() {
            return Err(SdkError::InvalidCodegen);
        }
        self.required_rights.validate()?;
        if self.requires_audit && !self.required_rights.contains(SdkRights::AUDIT) {
            return Err(SdkError::AuditRequired);
        }
        validate_redaction(self.data_class, self.redaction)?;
        self.trace.validate()
    }
}

/// Code generation job request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CodegenJob<'a> {
    /// Job descriptor.
    pub descriptor: CodegenDescriptor<'a>,
    /// Source schema name.
    pub source_schema: &'a str,
    /// Source schema text or stable input label.
    pub input: &'a str,
    /// Output path label.
    pub output_path: &'a str,
    /// Trace context for generation.
    pub trace: TraceContext,
}

impl<'a> CodegenJob<'a> {
    /// Creates a code generation job.
    pub const fn new(
        descriptor: CodegenDescriptor<'a>,
        source_schema: &'a str,
        input: &'a str,
        output_path: &'a str,
        trace: TraceContext,
    ) -> Self {
        Self {
            descriptor,
            source_schema,
            input,
            output_path,
            trace,
        }
    }

    /// Validates job metadata and bounds.
    pub fn validate(self) -> SdkResult<()> {
        self.descriptor.validate()?;
        validate_sdk_label(self.source_schema, MAX_SCHEMA_NAME_LEN)
            .map_err(|_| SdkError::InvalidCodegen)?;
        validate_sdk_label(self.output_path, MAX_ARTIFACT_PATH_LEN)
            .map_err(|_| SdkError::InvalidCodegen)?;
        if self.input.is_empty() {
            return Err(SdkError::MissingField);
        }
        if self.input.len() > MAX_CODEGEN_INPUT_LEN {
            return Err(SdkError::FieldTooLong);
        }
        self.trace.validate()
    }
}

/// Generated artifact metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CodegenArtifact<'a> {
    /// Job that produced the artifact.
    pub job: &'a str,
    /// Artifact path label.
    pub path: &'a str,
    /// Output target.
    pub target: CodegenTarget,
    /// Artifact status.
    pub status: CodegenStatus,
    /// Data class for generated content.
    pub data_class: DataClass,
    /// Redaction state for generated content.
    pub redaction: RedactionState,
    /// Trace context for the artifact.
    pub trace: TraceContext,
}

impl<'a> CodegenArtifact<'a> {
    /// Creates artifact metadata.
    pub const fn new(
        job: &'a str,
        path: &'a str,
        target: CodegenTarget,
        status: CodegenStatus,
        data_class: DataClass,
        redaction: RedactionState,
        trace: TraceContext,
    ) -> Self {
        Self {
            job,
            path,
            target,
            status,
            data_class,
            redaction,
            trace,
        }
    }

    /// Validates artifact metadata.
    pub fn validate(self) -> SdkResult<()> {
        validate_sdk_label(self.job, MAX_CODEGEN_LABEL_LEN)?;
        validate_sdk_label(self.path, MAX_ARTIFACT_PATH_LEN)?;
        if self.data_class.requires_redaction()
            && matches!(self.redaction, RedactionState::UnredactedSensitive)
        {
            return Err(SdkError::SensitiveData);
        }
        validate_redaction(self.data_class, self.redaction)?;
        self.trace.validate()
    }
}

/// Fixed-capacity code generation registry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodegenRegistry<'a, const N: usize> {
    entries: [Option<CodegenDescriptor<'a>>; N],
    len: usize,
    sealed: bool,
}

impl<'a, const N: usize> CodegenRegistry<'a, N> {
    /// Creates an empty code generation registry.
    pub const fn new() -> Self {
        Self {
            entries: [None; N],
            len: 0,
            sealed: false,
        }
    }

    /// Returns the number of registered jobs.
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` when no jobs are registered.
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Prevents further registrations.
    pub fn seal(&mut self) {
        self.sealed = true;
    }

    /// Registers a codegen descriptor.
    pub fn register(&mut self, descriptor: CodegenDescriptor<'a>) -> SdkResult<()> {
        if self.sealed {
            return Err(SdkError::Sealed);
        }
        descriptor.validate()?;
        if self.find(descriptor.name).is_ok() {
            return Err(SdkError::Duplicate);
        }
        if self.len >= N {
            return Err(SdkError::CapacityExceeded);
        }
        self.entries[self.len] = Some(descriptor);
        self.len += 1;
        Ok(())
    }

    /// Finds a codegen descriptor by name.
    pub fn find(&self, name: &str) -> SdkResult<CodegenDescriptor<'a>> {
        validate_sdk_label(name, MAX_CODEGEN_LABEL_LEN)?;
        for descriptor in self.entries.iter().take(self.len).flatten() {
            if descriptor.name == name {
                return Ok(*descriptor);
            }
        }
        Err(SdkError::CodegenNotFound)
    }

    /// Validates a declared job and returns generated artifact metadata.
    pub fn generate_declared(
        &self,
        caller: SdkRights,
        job: CodegenJob<'a>,
        audit_ready: bool,
    ) -> SdkResult<CodegenArtifact<'a>> {
        job.validate()?;
        let descriptor = self.find(job.descriptor.name)?;
        caller.validate()?;
        caller.require(descriptor.required_rights)?;
        if descriptor.requires_audit && (!audit_ready || !caller.contains(SdkRights::AUDIT)) {
            return Err(SdkError::AuditRequired);
        }
        let artifact = CodegenArtifact::new(
            descriptor.name,
            job.output_path,
            descriptor.target,
            CodegenStatus::Generated,
            descriptor.data_class,
            descriptor.redaction,
            job.trace,
        );
        artifact.validate()?;
        Ok(artifact)
    }
}

impl<'a, const N: usize> Default for CodegenRegistry<'a, N> {
    fn default() -> Self {
        Self::new()
    }
}
