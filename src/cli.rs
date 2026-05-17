//! SDK CLI command planning boundary.
//!
//! CLI contracts describe developer-tooling commands without invoking the host
//! shell. Each invocation carries required rights, audit requirements, trace
//! metadata, and classified argument data.

use crate::{
    validate_redaction, validate_sdk_label, DataClass, RedactionState, SdkError, SdkResult,
    SdkRights, TraceContext,
};

/// CLI descriptor schema version.
pub const CLI_SCHEMA_VERSION: &str = "alani-sdk.cli.v1";
/// Local build-helper schema version.
pub const BUILD_HELPER_SCHEMA_VERSION: &str = "alani-sdk.build-helper.v1";
/// Maximum CLI command label length.
pub const MAX_CLI_NAME_LEN: usize = 64;
/// Maximum CLI argument value length.
pub const MAX_CLI_ARGUMENT_LEN: usize = 256;
/// Maximum working-directory label length.
pub const MAX_CLI_WORKDIR_LEN: usize = 192;
/// Maximum arguments accepted by a CLI invocation.
pub const MAX_CLI_ARGUMENTS: usize = 24;
/// Default CLI command registry capacity.
pub const MAX_COMMANDS: usize = 64;
/// Maximum build-helper label length.
pub const MAX_BUILD_HELPER_LABEL_LEN: usize = 64;
/// Default build-helper plan capacity.
pub const MAX_BUILD_HELPERS: usize = 16;

/// SDK CLI command family.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CliCommandKind {
    /// Print SDK version and catalog metadata.
    Version,
    /// Instantiate a repository template.
    InitRepository,
    /// Run code generation.
    Generate,
    /// List or inspect templates.
    TemplateList,
    /// Plan sysroot installation or update.
    SysrootPlan,
    /// Run local build helpers.
    Build,
    /// Run compatibility checks.
    CompatCheck,
    /// Print help metadata.
    Help,
}

impl CliCommandKind {
    /// Stable command-kind label for manifests and CLI metadata.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Version => "version",
            Self::InitRepository => "init.repository",
            Self::Generate => "generate",
            Self::TemplateList => "template.list",
            Self::SysrootPlan => "sysroot.plan",
            Self::Build => "build",
            Self::CompatCheck => "compat.check",
            Self::Help => "help",
        }
    }

    /// Parses a stable command-kind label.
    pub const fn from_label(label: &str) -> Option<Self> {
        match label.as_bytes() {
            b"version" => Some(Self::Version),
            b"init.repository" => Some(Self::InitRepository),
            b"generate" => Some(Self::Generate),
            b"template.list" => Some(Self::TemplateList),
            b"sysroot.plan" => Some(Self::SysrootPlan),
            b"build" => Some(Self::Build),
            b"compat.check" => Some(Self::CompatCheck),
            b"help" => Some(Self::Help),
            _ => None,
        }
    }
}

/// CLI invocation status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CliStatus {
    /// Invocation was accepted but not run.
    Planned,
    /// Invocation completed.
    Completed,
    /// Invocation was denied by policy.
    Denied,
    /// Invocation requires audit readiness before it can proceed.
    NeedsAudit,
    /// Invocation failed validation or planning.
    Failed,
}

impl CliStatus {
    /// Stable invocation status label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Completed => "completed",
            Self::Denied => "denied",
            Self::NeedsAudit => "needs.audit",
            Self::Failed => "failed",
        }
    }
}

/// Local build-helper family.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuildHelperKind {
    /// Verify formatting without rewriting sources.
    FormatCheck,
    /// Run Rust compiler checks.
    Check,
    /// Run tests.
    Test,
    /// Run lint checks.
    Lint,
    /// Build generated documentation.
    Docs,
    /// Validate checked-in example manifests or schemas.
    ValidateExamples,
}

impl BuildHelperKind {
    /// Stable build-helper label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::FormatCheck => "format.check",
            Self::Check => "check",
            Self::Test => "test",
            Self::Lint => "lint",
            Self::Docs => "docs",
            Self::ValidateExamples => "validate.examples",
        }
    }

    /// Parses a stable build-helper label.
    pub const fn from_label(label: &str) -> Option<Self> {
        match label.as_bytes() {
            b"format.check" => Some(Self::FormatCheck),
            b"check" => Some(Self::Check),
            b"test" => Some(Self::Test),
            b"lint" => Some(Self::Lint),
            b"docs" => Some(Self::Docs),
            b"validate.examples" => Some(Self::ValidateExamples),
            _ => None,
        }
    }
}

/// Local build-helper lifecycle status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuildHelperStatus {
    /// Build helper was declared.
    Declared,
    /// Build helper was added to a side-effect-free plan.
    Planned,
    /// Build helper completed in a host executor.
    Completed,
    /// Build helper was denied by rights or audit readiness.
    Denied,
    /// Build helper failed validation or execution.
    Failed,
}

impl BuildHelperStatus {
    /// Stable build-helper status label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Declared => "declared",
            Self::Planned => "planned",
            Self::Completed => "completed",
            Self::Denied => "denied",
            Self::Failed => "failed",
        }
    }
}

/// CLI argument metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CliArgument<'a> {
    /// Argument name. Positional arguments use an empty name.
    pub name: &'a str,
    /// Argument value.
    pub value: &'a str,
    /// Data class for the value.
    pub data_class: DataClass,
    /// Redaction state for the value.
    pub redaction: RedactionState,
}

impl<'a> CliArgument<'a> {
    /// Creates a named CLI argument.
    pub const fn named(name: &'a str, value: &'a str) -> Self {
        Self {
            name,
            value,
            data_class: DataClass::Public,
            redaction: RedactionState::Public,
        }
    }

    /// Creates a positional CLI argument.
    pub const fn positional(value: &'a str) -> Self {
        Self {
            name: "",
            value,
            data_class: DataClass::Public,
            redaction: RedactionState::Public,
        }
    }

    /// Overrides classification and redaction metadata.
    pub const fn with_data(mut self, data_class: DataClass, redaction: RedactionState) -> Self {
        self.data_class = data_class;
        self.redaction = redaction;
        self
    }

    /// Validates argument metadata.
    pub fn validate(self) -> SdkResult<()> {
        if !self.name.is_empty() {
            validate_sdk_label(self.name, MAX_CLI_NAME_LEN).map_err(|_| SdkError::InvalidCli)?;
        }
        if self.value.len() > MAX_CLI_ARGUMENT_LEN {
            return Err(SdkError::FieldTooLong);
        }
        if self.data_class.requires_redaction()
            && matches!(self.redaction, RedactionState::UnredactedSensitive)
        {
            return Err(SdkError::SensitiveData);
        }
        validate_redaction(self.data_class, self.redaction)
    }
}

/// Stable CLI command descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CliDescriptor<'a> {
    /// Stable command name.
    pub name: &'a str,
    /// CLI schema version.
    pub schema: &'static str,
    /// Command family.
    pub kind: CliCommandKind,
    /// Short summary for help output.
    pub summary: &'a str,
    /// Rights required to plan or execute the command.
    pub required_rights: SdkRights,
    /// Whether the command must preserve audit evidence.
    pub requires_audit: bool,
    /// Data class for command metadata.
    pub data_class: DataClass,
    /// Redaction state for command metadata.
    pub redaction: RedactionState,
    /// Trace context attached to the descriptor.
    pub trace: TraceContext,
}

impl<'a> CliDescriptor<'a> {
    /// Creates a CLI command descriptor.
    pub const fn new(name: &'a str, kind: CliCommandKind, required_rights: SdkRights) -> Self {
        Self {
            name,
            schema: CLI_SCHEMA_VERSION,
            kind,
            summary: "",
            required_rights,
            requires_audit: false,
            data_class: DataClass::Operational,
            redaction: RedactionState::Operational,
            trace: TraceContext::EMPTY,
        }
    }

    /// Adds a short summary.
    pub const fn with_summary(mut self, summary: &'a str) -> Self {
        self.summary = summary;
        self
    }

    /// Marks the command as audit-required.
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

    /// Validates command descriptor metadata.
    pub fn validate(self) -> SdkResult<()> {
        validate_sdk_label(self.name, MAX_CLI_NAME_LEN).map_err(|_| SdkError::InvalidCli)?;
        if self.schema.is_empty() {
            return Err(SdkError::InvalidCli);
        }
        if self.summary.len() > MAX_CLI_ARGUMENT_LEN {
            return Err(SdkError::FieldTooLong);
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

/// CLI invocation request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CliInvocation<'a> {
    /// Command descriptor to invoke.
    pub descriptor: CliDescriptor<'a>,
    /// Command arguments.
    pub arguments: &'a [CliArgument<'a>],
    /// Working directory label or path.
    pub working_dir: &'a str,
    /// Trace context for this invocation.
    pub trace: TraceContext,
}

impl<'a> CliInvocation<'a> {
    /// Creates a CLI invocation.
    pub const fn new(
        descriptor: CliDescriptor<'a>,
        arguments: &'a [CliArgument<'a>],
        working_dir: &'a str,
        trace: TraceContext,
    ) -> Self {
        Self {
            descriptor,
            arguments,
            working_dir,
            trace,
        }
    }

    /// Validates invocation metadata and argument bounds.
    pub fn validate(self) -> SdkResult<()> {
        self.descriptor.validate()?;
        if self.arguments.len() > MAX_CLI_ARGUMENTS {
            return Err(SdkError::CapacityExceeded);
        }
        validate_sdk_label(self.working_dir, MAX_CLI_WORKDIR_LEN)
            .map_err(|_| SdkError::InvalidCli)?;
        for argument in self.arguments {
            argument.validate()?;
        }
        self.trace.validate()
    }
}

/// Side-effect-free CLI execution plan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CliPlan<'a> {
    /// Command name.
    pub command: &'a str,
    /// Planned status.
    pub status: CliStatus,
    /// Summary message.
    pub message: &'a str,
    /// Required rights.
    pub required_rights: SdkRights,
    /// Whether audit evidence is required.
    pub audit_required: bool,
    /// Trace context for the plan.
    pub trace: TraceContext,
}

impl<'a> CliPlan<'a> {
    /// Creates a CLI plan.
    pub const fn new(
        command: &'a str,
        status: CliStatus,
        message: &'a str,
        required_rights: SdkRights,
        audit_required: bool,
        trace: TraceContext,
    ) -> Self {
        Self {
            command,
            status,
            message,
            required_rights,
            audit_required,
            trace,
        }
    }

    /// Validates a CLI plan.
    pub fn validate(self) -> SdkResult<()> {
        validate_sdk_label(self.command, MAX_CLI_NAME_LEN)?;
        if self.message.len() > MAX_CLI_ARGUMENT_LEN {
            return Err(SdkError::FieldTooLong);
        }
        self.required_rights.validate()?;
        if self.audit_required && !self.required_rights.contains(SdkRights::AUDIT) {
            return Err(SdkError::AuditRequired);
        }
        self.trace.validate()
    }

    /// Checks whether a caller may perform this plan.
    pub fn authorize(self, caller: SdkRights, audit_ready: bool) -> SdkResult<()> {
        self.validate()?;
        caller.validate()?;
        caller.require(self.required_rights)?;
        if self.audit_required && (!audit_ready || !caller.contains(SdkRights::AUDIT)) {
            return Err(SdkError::AuditRequired);
        }
        Ok(())
    }
}

/// Stable local build-helper descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BuildHelperDescriptor<'a> {
    /// Stable helper name.
    pub name: &'a str,
    /// Build-helper schema version.
    pub schema: &'static str,
    /// Build-helper family.
    pub kind: BuildHelperKind,
    /// Workspace label or path.
    pub workspace: &'a str,
    /// Rights required to plan or execute the helper.
    pub required_rights: SdkRights,
    /// Whether the helper must preserve audit evidence.
    pub requires_audit: bool,
    /// Data class for helper metadata.
    pub data_class: DataClass,
    /// Redaction state for helper metadata.
    pub redaction: RedactionState,
    /// Trace context attached to the helper.
    pub trace: TraceContext,
}

impl<'a> BuildHelperDescriptor<'a> {
    /// Creates a build-helper descriptor.
    pub const fn new(name: &'a str, kind: BuildHelperKind, workspace: &'a str) -> Self {
        Self {
            name,
            schema: BUILD_HELPER_SCHEMA_VERSION,
            kind,
            workspace,
            required_rights: SdkRights::BUILD,
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

    /// Marks the helper as audit-required.
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

    /// Validates build-helper descriptor metadata.
    pub fn validate(self) -> SdkResult<()> {
        validate_sdk_label(self.name, MAX_BUILD_HELPER_LABEL_LEN)
            .map_err(|_| SdkError::InvalidCli)?;
        validate_sdk_label(self.workspace, MAX_CLI_WORKDIR_LEN)
            .map_err(|_| SdkError::InvalidCli)?;
        if self.schema.is_empty() {
            return Err(SdkError::InvalidCli);
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

/// Side-effect-free local build-helper plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuildHelperPlan<'a, const N: usize> {
    /// Workspace label or path shared by this plan.
    pub workspace: &'a str,
    /// Planned lifecycle status.
    pub status: BuildHelperStatus,
    helpers: [Option<BuildHelperDescriptor<'a>>; N],
    len: usize,
    /// Trace context for this build plan.
    pub trace: TraceContext,
}

impl<'a, const N: usize> BuildHelperPlan<'a, N> {
    /// Creates an empty build-helper plan.
    pub fn new(workspace: &'a str, trace: TraceContext) -> SdkResult<Self> {
        validate_sdk_label(workspace, MAX_CLI_WORKDIR_LEN).map_err(|_| SdkError::InvalidCli)?;
        trace.validate()?;
        Ok(Self {
            workspace,
            status: BuildHelperStatus::Declared,
            helpers: [None; N],
            len: 0,
            trace,
        })
    }

    /// Returns the number of helpers in the plan.
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` when no helpers are planned.
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns a helper by index.
    pub fn helper(&self, index: usize) -> Option<BuildHelperDescriptor<'a>> {
        if index >= self.len {
            None
        } else {
            self.helpers[index]
        }
    }

    /// Finds a helper by name.
    pub fn find(&self, name: &str) -> SdkResult<BuildHelperDescriptor<'a>> {
        validate_sdk_label(name, MAX_BUILD_HELPER_LABEL_LEN)?;
        for helper in self.helpers.iter().take(self.len).flatten() {
            if helper.name == name {
                return Ok(*helper);
            }
        }
        Err(SdkError::CommandNotFound)
    }

    /// Adds a helper to the plan.
    pub fn push_helper(&mut self, helper: BuildHelperDescriptor<'a>) -> SdkResult<()> {
        if self.len >= N {
            return Err(SdkError::CapacityExceeded);
        }
        helper.validate()?;
        if helper.workspace != self.workspace {
            return Err(SdkError::InvalidCli);
        }
        if self.find(helper.name).is_ok() {
            return Err(SdkError::Duplicate);
        }
        self.helpers[self.len] = Some(helper);
        self.len += 1;
        self.status = BuildHelperStatus::Planned;
        Ok(())
    }

    /// Validates the complete build-helper plan.
    pub fn validate(&self) -> SdkResult<()> {
        validate_sdk_label(self.workspace, MAX_CLI_WORKDIR_LEN)
            .map_err(|_| SdkError::InvalidCli)?;
        if self.len == 0 {
            return Err(SdkError::MissingField);
        }
        for helper in self.helpers.iter().take(self.len).flatten() {
            helper.validate()?;
            if helper.workspace != self.workspace {
                return Err(SdkError::InvalidCli);
            }
        }
        self.trace.validate()
    }

    /// Checks whether a caller may run every helper in the plan.
    pub fn authorize(&self, caller: SdkRights, audit_ready: bool) -> SdkResult<()> {
        self.validate()?;
        caller.validate()?;
        for helper in self.helpers.iter().take(self.len).flatten() {
            caller.require(helper.required_rights)?;
            if helper.requires_audit && (!audit_ready || !caller.contains(SdkRights::AUDIT)) {
                return Err(SdkError::AuditRequired);
            }
        }
        Ok(())
    }
}

/// Fixed-capacity CLI command registry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CliRegistry<'a, const N: usize> {
    entries: [Option<CliDescriptor<'a>>; N],
    len: usize,
    sealed: bool,
}

impl<'a, const N: usize> CliRegistry<'a, N> {
    /// Creates an empty CLI registry.
    pub const fn new() -> Self {
        Self {
            entries: [None; N],
            len: 0,
            sealed: false,
        }
    }

    /// Returns the number of registered commands.
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` when no commands are registered.
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Prevents further registrations.
    pub fn seal(&mut self) {
        self.sealed = true;
    }

    /// Registers a command descriptor.
    pub fn register(&mut self, descriptor: CliDescriptor<'a>) -> SdkResult<()> {
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

    /// Finds a command descriptor by name.
    pub fn find(&self, name: &str) -> SdkResult<CliDescriptor<'a>> {
        validate_sdk_label(name, MAX_CLI_NAME_LEN)?;
        for descriptor in self.entries.iter().take(self.len).flatten() {
            if descriptor.name == name {
                return Ok(*descriptor);
            }
        }
        Err(SdkError::CommandNotFound)
    }

    /// Creates a side-effect-free plan for a declared invocation.
    pub fn plan(
        &self,
        caller: SdkRights,
        invocation: CliInvocation<'a>,
        audit_ready: bool,
    ) -> SdkResult<CliPlan<'a>> {
        invocation.validate()?;
        let descriptor = self.find(invocation.descriptor.name)?;
        let plan = CliPlan::new(
            descriptor.name,
            CliStatus::Planned,
            descriptor.summary,
            descriptor.required_rights,
            descriptor.requires_audit,
            invocation.trace,
        );
        plan.authorize(caller, audit_ready)?;
        Ok(plan)
    }
}

impl<'a, const N: usize> Default for CliRegistry<'a, N> {
    fn default() -> Self {
        Self::new()
    }
}
