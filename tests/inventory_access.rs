#[path = "../examples/support/inventory_access_fixture.rs"]
mod fixture;

#[test]
fn full_extraction_retains_the_old_stack_and_notifies_twice_without_recursive_notifications() {
    let value = fixture::Fixture::run("E 0 64 0 1 1 64 0 0 8 0 1 64 0 0 7 64 0 0");
    assert!(value.starts_with("1:0/1/64/0|0:-1/0/64/0|"), "{value}");
    assert_eq!(value.matches("ON:").count(), 2);
    assert_eq!(value.matches("SAVE:").count(), 2);
    assert!(!value.contains("COPY:"));
}

#[test]
fn partial_extraction_shrinks_the_retained_inventory_stack_and_returns_a_copy() {
    let value = fixture::Fixture::run("E 0 2 0 1 1 64 0 0 0 0 5 64 0 0 7 64 0 0");
    assert!(value.starts_with("3:0/2/64/0|1:0/3/64/0|"), "{value}");
    assert_eq!(value.matches("ON:").count(), 1);
    assert!(
        value.ends_with("EXTRACT:0:2;COPY:1;COUNT:1:3;ON:0:1:0/3/64/0;SAVE:0;COUNT:3:2;"),
        "{value}"
    );
}

#[test]
fn insertion_rereads_the_callers_stack_after_callbacks_replace_the_slot_and_mutate_the_argument() {
    let value = fixture::Fixture::run("I 0 0 0 1 1 64 0 0 3 0 63 64 0 0 7 64 0 0");
    assert!(
        value.starts_with("5:0/9/64/0|4:2/7/64/2|2:0/10/64/0|1:0/63/64/0|"),
        "{value}"
    );
}

#[test]
fn a_full_slot_refuses_before_expensive_component_comparison() {
    let value = fixture::Fixture::run("I 0 0 0 1 1 64 0 0 0 0 64 64 0 0 7 64 1 0");
    assert!(value.starts_with("2:0/7/64/1|"), "{value}");
    assert!(!value.contains("SAME:"), "{value}");
    assert!(value.ends_with("VALID:0:2;GET:0;LIMIT:0;"), "{value}");
}

#[test]
fn wrapper_refuses_internal_wrapped_stacks_before_filters_and_has_its_own_invalid_slot_boundary() {
    let wrapped = fixture::Fixture::run("W 0 1 0 1 1 64 0 0 0 3 5 64 0 0 7 64 0 0");
    assert!(wrapped.starts_with("0:-1/0/64/0|"), "{wrapped}");
    assert!(wrapped.ends_with("GET:0;"), "{wrapped}");
    let invalid = fixture::Fixture::run("W -1 1 0 1 1 64 0 0 0 0 5 64 0 0 7 64 0 0");
    assert!(invalid.starts_with("SlotRead|"), "{invalid}");
    let direct = fixture::Fixture::run("E -1 1 0 1 1 64 0 0 0 0 5 64 0 0 7 64 0 0");
    assert!(direct.starts_with("SlotCheck|"), "{direct}");
    assert!(direct.ends_with('|'), "{direct}");
}
