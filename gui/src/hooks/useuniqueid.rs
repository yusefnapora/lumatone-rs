use dioxus::prelude::*;

#[derive(Clone, Copy)]
struct IdCounter(u64);

/// Shared state provider for the [use_unique_id] hook. 
/// Call at the top of your component tree to enable `use_unique_id`.
pub fn use_unique_id_provider(cx: &ScopeState) {
  use_shared_state_provider(cx, || IdCounter(0));
}

/// A hook that provides a unique component id, prefixed with the given string.
/// Note that you must call [use_unique_id_provider] at a higher level in the component
/// tree first, e.g. in your App component.
pub fn use_unique_id<'a>(cx: &'a ScopeState, prefix: &str) -> &'a UseState<String> {
  let id_context = use_shared_state::<IdCounter>(cx)
    .expect("No unique id provider found! Call unique_id_provider in a top-level component before calling use_unique_id.");
  
  let id = id_context.read().0;
  let id_string = use_state(cx, || format!("{prefix}-{}", id));
  
  // update the counter without forcing a re-render
  id_context.write_silent().0 = id + 1;
  id_string
}