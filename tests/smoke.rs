use alani_sdk::*;

#[test]
fn repository_catalog_is_stable() {
    let catalog = sdk_catalog();
    assert_eq!(repository_name(), "alani-sdk");
    assert_eq!(module_names(), ["cli", "codegen", "templates", "sysroot"]);
    assert_eq!(catalog.cli_schema, CLI_SCHEMA_VERSION);
    assert_eq!(catalog.codegen_schema, CODEGEN_SCHEMA_VERSION);
    assert!(catalog.validate().is_ok());
    assert_eq!(component_info().status, ComponentStatus::Experimental);
}

#[test]
fn cli_registry_plans_commands_and_fails_closed() {
    let trace = TraceContext::root(1, 10);
    let generate = CliDescriptor::new(
        "generate",
        CliCommandKind::Generate,
        SdkRights::RUN_CLI.union(SdkRights::CODEGEN),
    )
    .with_summary("Generate SDK artifacts.")
    .with_trace(trace);
    let sysroot = CliDescriptor::new(
        "sysroot.plan",
        CliCommandKind::SysrootPlan,
        SdkRights::RUN_CLI
            .union(SdkRights::SYSROOT_WRITE)
            .union(SdkRights::AUDIT),
    )
    .with_audit()
    .with_trace(trace);

    let mut registry: CliRegistry<4> = CliRegistry::new();
    registry.register(generate).unwrap();
    assert_eq!(
        registry.register(generate).unwrap_err(),
        SdkError::Duplicate
    );
    registry.register(sysroot).unwrap();

    let args = [CliArgument::named("schema", "protocol")];
    let invocation = CliInvocation::new(generate, &args, "/work/alani", trace);
    let caller = SdkRights::RUN_CLI.union(SdkRights::CODEGEN);
    let plan = registry.plan(caller, invocation, false).unwrap();
    assert_eq!(plan.command, "generate");
    assert_eq!(plan.status, CliStatus::Planned);

    let invocation = CliInvocation::new(sysroot, &[], "/work/alani", trace);
    let caller = SdkRights::RUN_CLI
        .union(SdkRights::SYSROOT_WRITE)
        .union(SdkRights::AUDIT);
    assert_eq!(
        registry.plan(caller, invocation, false).unwrap_err(),
        SdkError::AuditRequired
    );
}

#[test]
fn codegen_registry_declares_artifacts_and_rejects_missing_rights() {
    let trace = TraceContext::root(2, 20);
    let descriptor = CodegenDescriptor::new(
        "protocol.rust",
        CodegenSchemaKind::Protocol,
        CodegenTarget::Rust,
    )
    .with_trace(trace);
    let mut registry: CodegenRegistry<2> = CodegenRegistry::new();
    registry.register(descriptor).unwrap();

    let job = CodegenJob::new(
        descriptor,
        "alani.protocol.message",
        "message.schema.v1",
        "generated/protocol.rs",
        trace,
    );
    assert_eq!(
        registry
            .generate_declared(SdkRights::READ, job, false)
            .unwrap_err(),
        SdkError::AccessDenied
    );

    let artifact = registry
        .generate_declared(SdkRights::CODEGEN, job, false)
        .unwrap();
    assert_eq!(artifact.job, "protocol.rust");
    assert_eq!(artifact.path, "generated/protocol.rs");
    assert_eq!(artifact.status, CodegenStatus::Generated);
}

#[test]
fn templates_build_render_plans_and_block_sensitive_content() {
    let trace = TraceContext::root(3, 30);
    let record = builtin_repository_template(trace);
    let mut catalog: TemplateCatalog<4> = TemplateCatalog::new();
    catalog.register(record).unwrap();
    assert_eq!(
        catalog.find("repo.rust", "missing").unwrap_err(),
        SdkError::TemplateNotFound
    );

    let mut plan: RenderPlan<4> =
        RenderPlan::new(record.descriptor, "/tmp/alani-sdk", trace).unwrap();
    plan.push_file(record).unwrap();
    assert_eq!(plan.len(), 1);
    assert!(plan.authorize(SdkRights::TEMPLATE_READ, false).is_ok());

    let sensitive = TemplateDescriptor::new("secret.template", TemplateKind::Config, "0.1.0")
        .with_data(DataClass::Sensitive, RedactionState::UnredactedSensitive);
    let record = TemplateRecord::new(
        sensitive,
        "secret.toml",
        "token = \"secret\"",
        TemplateStatus::Ready,
    );
    assert_eq!(record.validate().unwrap_err(), SdkError::SensitiveData);
}

#[test]
fn sysroot_plans_components_and_checks_compatibility() {
    let trace = TraceContext::root(4, 40);
    let layout = SysrootLayout::new(
        "/sdk/sysroot",
        "/sdk/sysroot/lib",
        "/sdk/sysroot/include",
        "/sdk/bin",
    );
    let descriptor = SysrootDescriptor::new(
        "host.sysroot",
        "x86_64-unknown-linux-gnu",
        SdkHostTriple::X86_64UnknownLinuxGnu,
        layout,
    )
    .with_rights(SdkRights::SYSROOT_WRITE.union(SdkRights::AUDIT))
    .with_audit()
    .with_trace(trace);
    let mut plan: SysrootPlan<4> = SysrootPlan::new(descriptor).unwrap();
    let component = SysrootComponent::new("alani-lib", "0.1.0", "lib/libalani.rlib", true);
    plan.push_component(component).unwrap();
    assert_eq!(plan.len(), 1);
    assert_eq!(
        plan.authorize(SdkRights::SYSROOT_WRITE.union(SdkRights::AUDIT), false)
            .unwrap_err(),
        SdkError::AuditRequired
    );
    assert!(plan
        .authorize(SdkRights::SYSROOT_WRITE.union(SdkRights::AUDIT), true)
        .is_ok());

    let compatible = CompatibilityCheck::new(
        "alani-lib",
        "0.1.0",
        "0.1.0",
        CompatibilityStatus::Compatible,
        trace,
    );
    assert!(compatible.validate().is_ok());
    let incompatible = CompatibilityCheck::new(
        "alani-lib",
        "0.1.0",
        "0.2.0",
        CompatibilityStatus::Incompatible,
        trace,
    );
    assert_eq!(incompatible.validate().unwrap_err(), SdkError::Incompatible);
}

#[test]
fn reserved_bits_and_invalid_traces_are_rejected() {
    assert_eq!(
        SdkRights::from_bits(1 << 63).unwrap_err(),
        SdkError::ReservedBits
    );

    let invalid_trace = TraceContext {
        trace_id: 0,
        span_id: 99,
        parent_span_id: 0,
        flags: 0,
    };
    assert_eq!(
        invalid_trace.validate().unwrap_err(),
        SdkError::InvalidTrace
    );
}
