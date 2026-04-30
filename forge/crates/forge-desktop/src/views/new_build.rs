use std::collections::BTreeSet;

use dioxus::prelude::*;

use forge_core::domain::{
    Architecture, BaseImage, BuildSpec, ComplianceProfile, HardeningOptions, Runtime,
};

use crate::services::orchestration;
use crate::state::use_app_state;
use crate::views::Route;

#[component]
pub fn NewBuild(route: Signal<Route>) -> Element {
    let state = use_app_state();

    let mut name = use_signal(String::new);
    let mut runtime = use_signal(|| RuntimeChoice::Java);
    let mut base = use_signal(|| BaseChoice::Alpine);
    let mut amd64 = use_signal(|| true);
    let mut arm64 = use_signal(|| false);
    let mut sign = use_signal(|| true);
    let mut sbom = use_signal(|| true);
    let mut cis = use_signal(|| true);
    let mut hipaa = use_signal(|| false);
    let mut soc2 = use_signal(|| false);
    let mut error = use_signal(String::new);

    let mut submit = move || {
        let n = name.read().trim().to_string();
        if n.is_empty() {
            error.set("Name is required".into());
            return;
        }
        let mut archs = BTreeSet::new();
        if *amd64.read() {
            archs.insert(Architecture::Amd64);
        }
        if *arm64.read() {
            archs.insert(Architecture::Arm64);
        }
        if archs.is_empty() {
            error.set("Pick at least one architecture".into());
            return;
        }
        let mut compliance = BTreeSet::new();
        if *cis.read() {
            compliance.insert(ComplianceProfile::Cis);
        }
        if *hipaa.read() {
            compliance.insert(ComplianceProfile::Hipaa);
        }
        if *soc2.read() {
            compliance.insert(ComplianceProfile::Soc2);
        }

        let spec = BuildSpec {
            name: n,
            runtime: (*runtime.read()).into(),
            base_image: (*base.read()).into(),
            architectures: archs,
            compliance,
            hardening: HardeningOptions::strict(),
            generate_sbom: *sbom.read(),
            sign: *sign.read(),
        };
        orchestration::start_build(&state, spec);
        error.set(String::new());
        route.set(Route::Builds);
    };

    rsx! {
        section {
            class: "view",
            header { class: "view-header", h1 { "New build" } }
            form {
                class: "panel form",
                onsubmit: move |_| { submit(); },

                div { class: "form-row",
                    label { "Name" }
                    input {
                        r#type: "text",
                        placeholder: "my-secure-app",
                        value: "{name.read()}",
                        oninput: move |evt| name.set(evt.value()),
                    }
                }

                div { class: "form-row",
                    label { "Runtime" }
                    select {
                        oninput: move |evt| runtime.set(RuntimeChoice::parse(&evt.value())),
                        option { value: "java",   selected: matches!(*runtime.read(), RuntimeChoice::Java),   "Java" }
                        option { value: "dotnet", selected: matches!(*runtime.read(), RuntimeChoice::Dotnet), ".NET" }
                        option { value: "go",     selected: matches!(*runtime.read(), RuntimeChoice::Go),     "Go" }
                        option { value: "node",   selected: matches!(*runtime.read(), RuntimeChoice::Node),   "Node" }
                        option { value: "python", selected: matches!(*runtime.read(), RuntimeChoice::Python), "Python" }
                    }
                }

                div { class: "form-row",
                    label { "Base image" }
                    select {
                        oninput: move |evt| base.set(BaseChoice::parse(&evt.value())),
                        option { value: "alpine",     selected: matches!(*base.read(), BaseChoice::Alpine),     "Alpine" }
                        option { value: "debian",     selected: matches!(*base.read(), BaseChoice::Debian),     "Debian" }
                        option { value: "distroless", selected: matches!(*base.read(), BaseChoice::Distroless), "Distroless" }
                    }
                }

                fieldset { class: "form-row",
                    legend { "Architectures" }
                    label { class: "checkbox",
                        input { r#type: "checkbox", checked: *amd64.read(),
                            oninput: move |evt| amd64.set(evt.value().parse().unwrap_or(false)) }
                        "linux/amd64"
                    }
                    label { class: "checkbox",
                        input { r#type: "checkbox", checked: *arm64.read(),
                            oninput: move |evt| arm64.set(evt.value().parse().unwrap_or(false)) }
                        "linux/arm64"
                    }
                }

                fieldset { class: "form-row",
                    legend { "Compliance" }
                    label { class: "checkbox",
                        input { r#type: "checkbox", checked: *cis.read(),
                            oninput: move |evt| cis.set(evt.value().parse().unwrap_or(false)) }
                        "CIS"
                    }
                    label { class: "checkbox",
                        input { r#type: "checkbox", checked: *hipaa.read(),
                            oninput: move |evt| hipaa.set(evt.value().parse().unwrap_or(false)) }
                        "HIPAA"
                    }
                    label { class: "checkbox",
                        input { r#type: "checkbox", checked: *soc2.read(),
                            oninput: move |evt| soc2.set(evt.value().parse().unwrap_or(false)) }
                        "SOC2"
                    }
                }

                fieldset { class: "form-row",
                    legend { "Artifacts" }
                    label { class: "checkbox",
                        input { r#type: "checkbox", checked: *sbom.read(),
                            oninput: move |evt| sbom.set(evt.value().parse().unwrap_or(false)) }
                        "Generate SBOM"
                    }
                    label { class: "checkbox",
                        input { r#type: "checkbox", checked: *sign.read(),
                            oninput: move |evt| sign.set(evt.value().parse().unwrap_or(false)) }
                        "Sign image"
                    }
                }

                if !error.read().is_empty() {
                    p { class: "form-error", "{error.read()}" }
                }

                div { class: "form-actions",
                    button { class: "btn btn-ghost", r#type: "button",
                        onclick: move |_| route.set(Route::Builds), "Cancel" }
                    button { class: "btn btn-primary", r#type: "submit", "Start build" }
                }
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum RuntimeChoice {
    Java,
    Dotnet,
    Go,
    Node,
    Python,
}

impl RuntimeChoice {
    fn parse(s: &str) -> Self {
        match s {
            "dotnet" => Self::Dotnet,
            "go" => Self::Go,
            "node" => Self::Node,
            "python" => Self::Python,
            _ => Self::Java,
        }
    }
}

impl From<RuntimeChoice> for Runtime {
    fn from(c: RuntimeChoice) -> Self {
        match c {
            RuntimeChoice::Java => Runtime::Java,
            RuntimeChoice::Dotnet => Runtime::Dotnet,
            RuntimeChoice::Go => Runtime::Go,
            RuntimeChoice::Node => Runtime::Node,
            RuntimeChoice::Python => Runtime::Python,
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum BaseChoice {
    Alpine,
    Debian,
    Distroless,
}

impl BaseChoice {
    fn parse(s: &str) -> Self {
        match s {
            "debian" => Self::Debian,
            "distroless" => Self::Distroless,
            _ => Self::Alpine,
        }
    }
}

impl From<BaseChoice> for BaseImage {
    fn from(c: BaseChoice) -> Self {
        match c {
            BaseChoice::Alpine => BaseImage::Alpine,
            BaseChoice::Debian => BaseImage::Debian,
            BaseChoice::Distroless => BaseImage::Distroless,
        }
    }
}
