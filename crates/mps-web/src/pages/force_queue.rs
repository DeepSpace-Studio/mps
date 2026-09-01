use dioxus::prelude::*;
use dioxus_i18n::{prelude::*, t};

/// Force Queue Tutorial page — zero-copy shared-memory force application.
/// Live-worked example: Java enqueues forces → native consumes in world_step.
///
/// Code samples are stored as Rust raw string literals instead of in the FTL
/// catalogues, because Fluent 0.16 does not allow literal `{` / `}` characters
/// in pattern values (they always start an inline expression). Code blocks
/// that legitimately contain braces — C structs, Rust `fn { ... }`, Java
/// `if (...) { ... }` — must therefore live outside i18n.
const FFI_SAMPLE: &str = r#"
// Header struct (64-byte aligned, cbindgen-exported)
typedef struct ForceQueueHeader {
    uint64_t capacity;
    uint64_t head;
    uint64_t tail;
    uint64_t generation;
    uint32_t stride;
    uint32_t flags;
    // bitmap + payload follow in memory
} ForceQueueHeader;

// Consumer — called once per frame from world_step or directly
uint32_t rigid_body_consume_force_queue(void* world, ForceQueueHeader* queue);"#;

const JNI_SAMPLE: &str = r#"// Allocate queue (capacity must be power of 2)
ForceQueue queue = ForceQueue.allocate(capacity, 6); // stride 6 = force only

// Enqueue a force for body_id
int slot = queue.tryEnqueue();
if (slot >= 0) {
    queue.writeForce(slot, bodyId, fx, fy, fz);
    queue.commit(slot); // sets bitmap bit + releases head
}

// Once per frame: native consumes
RapierNative.rigidBodyConsumeForceQueue(worldHandle, queue.address());"#;

const FFM_SAMPLE: &str = r#"// Map native queue memory as MemorySegment
MemorySegment segment = (MemorySegment) ForceQueueFFM.mapQueue(capacity, 6);
ForceQueueHeader header = ForceQueueHeader.of(segment);

// Enqueue
int slot = ForceQueueFFM.tryEnqueue(header);
if (slot >= 0) {
    ForceQueueFFM.writeForce(header, slot, bodyId, fx, fy, fz);
    ForceQueueFFM.commit(header, slot);
}

// Consume via downcall
Linker linker = Linker.nativeLinker();
MethodHandle consume = linker.downcallHandle(
    SymbolLookup.loaderLookup().find("rigid_body_consume_force_queue").get(),
    FunctionDescriptor.of(ValueLayout.JAVA_INT,
        ValueLayout.ADDRESS, ValueLayout.ADDRESS)
);
consume.invokeExact(worldAddress, segment.address());"#;

const TEST_SAMPLE: &str = r#"#[test]
fn force_queue_integration_full_cycle() {
    let world = world_create();
    let body = rigid_body_create_dynamic(world, 0, 0, 0);
    let queue = allocate_queue(1024, 6);

    // Java-style: write force into slot 0, set bit, advance head
    write_force(queue, 0, body, 10.0, 0.0, 0.0);
    set_bit(queue, 0);
    atomic_store_release(&queue.head, 1);

    // Native consume
    rigid_body_consume_force_queue(world, queue);

    // Verify force applied
    let force = rigid_body_get_force(world, body);
    assert!((force.x - 10.0).abs() < 1e-9);
}"#;

#[component]
pub fn ForceQueue() -> Element {
    rsx! {
        section { id: "sec-force-queue", class: "doc-section",
            h2 { class: "section-title", { t!("force-queue-tag") } }
            h2 { class: "section-title", { t!("force-queue-title") } }
            p { class: "lead", { t!("force-queue-desc") } }

            // Overview
            h3 { class: "subsection-title", { t!("force-queue-overview-title") } }
            p { class: "body-text", { t!("force-queue-overview-lead") } }
            p { class: "body-text", { t!("force-queue-overview-body") } }

            // Memory Layout
            h3 { class: "subsection-title", { t!("force-queue-layout-title") } }
            p { class: "body-text", { t!("force-queue-layout-desc") } }

            pre { class: "code-block", { t!("force-queue-layout-diagram") } }

            div { class: "callout callout-info",
                p { class: "callout-title", { t!("force-queue-layout-note") } }
            }

            // Synchronization Model
            h3 { class: "subsection-title", { t!("force-queue-sync-title") } }
            p { class: "body-text", { t!("force-queue-sync-lead") } }
            ul { class: "bullet-list",
                li { { t!("force-queue-sync-li-1") } }
                li { { t!("force-queue-sync-li-2") } }
                li { { t!("force-queue-sync-li-3") } }
                li { { t!("force-queue-sync-li-4") } }
                li { { t!("force-queue-sync-li-5") } }
                li { { t!("force-queue-sync-li-6") } }
            }

            // Stride Modes
            h3 { class: "subsection-title", { t!("force-queue-stride-title") } }
            p { class: "body-text", { t!("force-queue-stride-desc") } }
            div { class: "table-wrap",
                table { class: "doc-table",
                    thead { tr { th { { t!("force-queue-stride-col-mode") } } th { { t!("force-queue-stride-col-desc") } } th { { t!("force-queue-stride-col-use") } } } }
                    tbody {
                        tr { td { code { "6" } } td { { t!("force-queue-stride-6-desc") } } td { { t!("force-queue-stride-6-use") } } }
                        tr { td { code { "7" } } td { { t!("force-queue-stride-7-desc") } } td { { t!("force-queue-stride-7-use") } } }
                    }
                }
            }

            // FFI Surface
            h3 { class: "subsection-title", { t!("force-queue-ffi-title") } }
            p { class: "body-text", { t!("force-queue-ffi-desc") } }
            pre { class: "code-block", { FFI_SAMPLE } }

            // Java Producer Example (JNI)
            h3 { class: "subsection-title", { t!("force-queue-jni-title") } }
            p { class: "body-text", { t!("force-queue-jni-desc") } }
            pre { class: "code-block", { JNI_SAMPLE } }

            // Java Producer Example (FFM)
            h3 { class: "subsection-title", { t!("force-queue-ffm-title") } }
            p { class: "body-text", { t!("force-queue-ffm-desc") } }
            pre { class: "code-block", { FFM_SAMPLE } }

            // Integration Test Reference
            h3 { class: "subsection-title", { t!("force-queue-test-title") } }
            p { class: "body-text", { t!("force-queue-test-desc") } }
            pre { class: "code-block", { TEST_SAMPLE } }

            // Performance Notes
            h3 { class: "subsection-title", { t!("force-queue-perf-title") } }
            ul { class: "bullet-list",
                li { { t!("force-queue-perf-li-1") } }
                li { { t!("force-queue-perf-li-2") } }
                li { { t!("force-queue-perf-li-3") } }
                li { { t!("force-queue-perf-li-4") } }
            }
        }
    }
}
