//! Sysroot layout, component, and compatibility-check boundary.
//!
//! Sysroot contracts describe target triples, SDK-managed components, and
//! compatibility checks without installing files. All mutating plans are
//! authority-checked and audit-aware.

use crate::{
    validate_redaction, validate_sdk_label, DataClass, RedactionState, SdkError, SdkResult,
    SdkRights, TraceContext,
};

/// Sysroot descriptor schema version.
pub const SYSROOT_SCHEMA_VERSION: &str = "alani-sdk.sysroot.v1";
/// Maximum target triple length.
pub const MAX_TARGET_TRIPLE_LEN: usize = 96;
/// Maximum sysroot path length.
pub const MAX_SYSROOT_PATH_LEN: usize = 192;
/// Maximum sysroot component label length.
pub const MAX_COMPONENT_LABEL_LEN: usize = 96;
/// Default component registry capacity.
pub const MAX_COMPONENTS: usize = 64;

/// Known SDK host triple family.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SdkHostTriple {
    /// x86_64 Linux host.
    X86_64UnknownLinuxGnu,
    /// aarch64 Linux host.
    Aarch64UnknownLinuxGnu,
    /// Host is intentionally abstract or simulated.
    HostSimulated,
}

impl SdkHostTriple {
    /// Stable host triple label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::X86_64UnknownLinuxGnu => "x86_64-unknown-linux-gnu",
            Self::Aarch64UnknownLinuxGnu => "aarch64-unknown-linux-gnu",
            Self::HostSimulated => "host-simulated",
        }
    }

    /// Parses a stable host triple label.
    pub const fn from_label(label: &str) -> Option<Self> {
        match label.as_bytes() {
            b"x86_64-unknown-linux-gnu" => Some(Self::X86_64UnknownLinuxGnu),
            b"aarch64-unknown-linux-gnu" => Some(Self::Aarch64UnknownLinuxGnu),
            b"host-simulated" => Some(Self::HostSimulated),
            _ => None,
        }
    }
}

/// Sysroot lifecycle state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SysrootState {
    /// Sysroot is declared but not prepared.
    Declared,
    /// Sysroot layout is planned.
    Planned,
    /// Sysroot is ready for use.
    Ready,
    /// Sysroot validation failed.
    Failed,
}

impl SysrootState {
    /// Stable state label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Declared => "declared",
            Self::Planned => "planned",
            Self::Ready => "ready",
            Self::Failed => "failed",
        }
    }
}

/// Compatibility check status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompatibilityStatus {
    /// Compatibility has not been evaluated.
    Unknown,
    /// Component versions are compatible.
    Compatible,
    /// Component versions are incompatible.
    Incompatible,
    /// Compatibility evidence is missing.
    MissingEvidence,
}

impl CompatibilityStatus {
    /// Stable compatibility status label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Compatible => "compatible",
            Self::Incompatible => "incompatible",
            Self::MissingEvidence => "missing.evidence",
        }
    }
}

/// Sysroot layout descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SysrootLayout<'a> {
    /// Root path label.
    pub root: &'a str,
    /// Library path label.
    pub lib: &'a str,
    /// Include path label.
    pub include: &'a str,
    /// Tool path label.
    pub tools: &'a str,
}

impl<'a> SysrootLayout<'a> {
    /// Creates a sysroot layout descriptor.
    pub const fn new(root: &'a str, lib: &'a str, include: &'a str, tools: &'a str) -> Self {
        Self {
            root,
            lib,
            include,
            tools,
        }
    }

    /// Validates layout path labels.
    pub fn validate(self) -> SdkResult<()> {
        validate_sdk_label(self.root, MAX_SYSROOT_PATH_LEN)
            .map_err(|_| SdkError::InvalidSysroot)?;
        validate_sdk_label(self.lib, MAX_SYSROOT_PATH_LEN).map_err(|_| SdkError::InvalidSysroot)?;
        validate_sdk_label(self.include, MAX_SYSROOT_PATH_LEN)
            .map_err(|_| SdkError::InvalidSysroot)?;
        validate_sdk_label(self.tools, MAX_SYSROOT_PATH_LEN)
            .map_err(|_| SdkError::InvalidSysroot)?;
        Ok(())
    }
}

/// Sysroot descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SysrootDescriptor<'a> {
    /// Stable sysroot name.
    pub name: &'a str,
    /// Sysroot schema version.
    pub schema: &'static str,
    /// Target triple.
    pub target_triple: &'a str,
    /// Host triple family.
    pub host: SdkHostTriple,
    /// Sysroot layout.
    pub layout: SysrootLayout<'a>,
    /// Sysroot lifecycle state.
    pub state: SysrootState,
    /// Rights required to modify this sysroot.
    pub required_rights: SdkRights,
    /// Whether updates must preserve audit evidence.
    pub requires_audit: bool,
    /// Data class for sysroot metadata.
    pub data_class: DataClass,
    /// Redaction state for sysroot metadata.
    pub redaction: RedactionState,
    /// Trace context attached to the descriptor.
    pub trace: TraceContext,
}

impl<'a> SysrootDescriptor<'a> {
    /// Creates a sysroot descriptor.
    pub const fn new(
        name: &'a str,
        target_triple: &'a str,
        host: SdkHostTriple,
        layout: SysrootLayout<'a>,
    ) -> Self {
        Self {
            name,
            schema: SYSROOT_SCHEMA_VERSION,
            target_triple,
            host,
            layout,
            state: SysrootState::Declared,
            required_rights: SdkRights::SYSROOT_WRITE,
            requires_audit: false,
            data_class: DataClass::Operational,
            redaction: RedactionState::Operational,
            trace: TraceContext::EMPTY,
        }
    }

    /// Overrides lifecycle state.
    pub const fn with_state(mut self, state: SysrootState) -> Self {
        self.state = state;
        self
    }

    /// Overrides required rights.
    pub const fn with_rights(mut self, rights: SdkRights) -> Self {
        self.required_rights = rights;
        self
    }

    /// Marks updates as audit-required.
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
        validate_sdk_label(self.name, MAX_COMPONENT_LABEL_LEN)
            .map_err(|_| SdkError::InvalidSysroot)?;
        validate_sdk_label(self.target_triple, MAX_TARGET_TRIPLE_LEN)
            .map_err(|_| SdkError::InvalidSysroot)?;
        if self.schema.is_empty() {
            return Err(SdkError::InvalidSysroot);
        }
        self.layout.validate()?;
        self.required_rights.validate()?;
        if self.requires_audit && !self.required_rights.contains(SdkRights::AUDIT) {
            return Err(SdkError::AuditRequired);
        }
        validate_redaction(self.data_class, self.redaction)?;
        self.trace.validate()
    }
}

/// Sysroot component metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SysrootComponent<'a> {
    /// Stable component name.
    pub name: &'a str,
    /// Component version.
    pub version: &'a str,
    /// Relative install path.
    pub path: &'a str,
    /// Whether this component is required.
    pub required: bool,
    /// Data class for component metadata.
    pub data_class: DataClass,
    /// Redaction state for component metadata.
    pub redaction: RedactionState,
}

impl<'a> SysrootComponent<'a> {
    /// Creates sysroot component metadata.
    pub const fn new(name: &'a str, version: &'a str, path: &'a str, required: bool) -> Self {
        Self {
            name,
            version,
            path,
            required,
            data_class: DataClass::Operational,
            redaction: RedactionState::Operational,
        }
    }

    /// Overrides classification and redaction metadata.
    pub const fn with_data(mut self, data_class: DataClass, redaction: RedactionState) -> Self {
        self.data_class = data_class;
        self.redaction = redaction;
        self
    }

    /// Validates component metadata.
    pub fn validate(self) -> SdkResult<()> {
        validate_sdk_label(self.name, MAX_COMPONENT_LABEL_LEN)
            .map_err(|_| SdkError::InvalidSysroot)?;
        validate_sdk_label(self.version, MAX_COMPONENT_LABEL_LEN)
            .map_err(|_| SdkError::InvalidSysroot)?;
        validate_sdk_label(self.path, MAX_SYSROOT_PATH_LEN)
            .map_err(|_| SdkError::InvalidSysroot)?;
        validate_redaction(self.data_class, self.redaction)
    }
}

/// Sysroot compatibility check record.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CompatibilityCheck<'a> {
    /// Component name.
    pub component: &'a str,
    /// Required version.
    pub required_version: &'a str,
    /// Observed version.
    pub observed_version: &'a str,
    /// Compatibility status.
    pub status: CompatibilityStatus,
    /// Trace context for the check.
    pub trace: TraceContext,
}

impl<'a> CompatibilityCheck<'a> {
    /// Creates a compatibility check record.
    pub const fn new(
        component: &'a str,
        required_version: &'a str,
        observed_version: &'a str,
        status: CompatibilityStatus,
        trace: TraceContext,
    ) -> Self {
        Self {
            component,
            required_version,
            observed_version,
            status,
            trace,
        }
    }

    /// Validates compatibility metadata.
    pub fn validate(self) -> SdkResult<()> {
        validate_sdk_label(self.component, MAX_COMPONENT_LABEL_LEN)?;
        validate_sdk_label(self.required_version, MAX_COMPONENT_LABEL_LEN)?;
        validate_sdk_label(self.observed_version, MAX_COMPONENT_LABEL_LEN)?;
        self.trace.validate()?;
        if matches!(self.status, CompatibilityStatus::Incompatible) {
            return Err(SdkError::Incompatible);
        }
        Ok(())
    }
}

/// Fixed-capacity sysroot plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SysrootPlan<'a, const N: usize> {
    /// Sysroot descriptor.
    pub descriptor: SysrootDescriptor<'a>,
    components: [Option<SysrootComponent<'a>>; N],
    len: usize,
}

impl<'a, const N: usize> SysrootPlan<'a, N> {
    /// Creates an empty sysroot plan.
    pub fn new(descriptor: SysrootDescriptor<'a>) -> SdkResult<Self> {
        descriptor.validate()?;
        Ok(Self {
            descriptor,
            components: [None; N],
            len: 0,
        })
    }

    /// Returns the number of components in the plan.
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` when no components are planned.
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns a component by index.
    pub fn component(&self, index: usize) -> Option<SysrootComponent<'a>> {
        if index >= self.len {
            None
        } else {
            self.components[index]
        }
    }

    /// Adds a component to the plan.
    pub fn push_component(&mut self, component: SysrootComponent<'a>) -> SdkResult<()> {
        if self.len >= N {
            return Err(SdkError::CapacityExceeded);
        }
        component.validate()?;
        self.components[self.len] = Some(component);
        self.len += 1;
        Ok(())
    }

    /// Checks whether a caller may apply this plan.
    pub fn authorize(&self, caller: SdkRights, audit_ready: bool) -> SdkResult<()> {
        caller.validate()?;
        caller.require(self.descriptor.required_rights)?;
        if self.descriptor.requires_audit && (!audit_ready || !caller.contains(SdkRights::AUDIT)) {
            return Err(SdkError::AuditRequired);
        }
        Ok(())
    }
}

/// Fixed-capacity sysroot component registry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SysrootRegistry<'a, const N: usize> {
    entries: [Option<SysrootComponent<'a>>; N],
    len: usize,
    sealed: bool,
}

impl<'a, const N: usize> SysrootRegistry<'a, N> {
    /// Creates an empty component registry.
    pub const fn new() -> Self {
        Self {
            entries: [None; N],
            len: 0,
            sealed: false,
        }
    }

    /// Returns the number of registered components.
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` when no components are registered.
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Prevents further registrations.
    pub fn seal(&mut self) {
        self.sealed = true;
    }

    /// Registers a component.
    pub fn register(&mut self, component: SysrootComponent<'a>) -> SdkResult<()> {
        if self.sealed {
            return Err(SdkError::Sealed);
        }
        component.validate()?;
        if self.find(component.name).is_ok() {
            return Err(SdkError::Duplicate);
        }
        if self.len >= N {
            return Err(SdkError::CapacityExceeded);
        }
        self.entries[self.len] = Some(component);
        self.len += 1;
        Ok(())
    }

    /// Finds a component by name.
    pub fn find(&self, name: &str) -> SdkResult<SysrootComponent<'a>> {
        validate_sdk_label(name, MAX_COMPONENT_LABEL_LEN)?;
        for component in self.entries.iter().take(self.len).flatten() {
            if component.name == name {
                return Ok(*component);
            }
        }
        Err(SdkError::ComponentNotFound)
    }
}

impl<'a, const N: usize> Default for SysrootRegistry<'a, N> {
    fn default() -> Self {
        Self::new()
    }
}
