use dioxus::prelude::*;

/// A definition for a single tab, to be rendered in a [TabContainer].
/// Note that the `id` prop must be unique within a TabContainer.
/// The `content` prop should be rendered using `cx.render`, e.g.:
/// 
/// ```no_run
/// use dioxus::prelude::*;
/// use crate::components::tabs::TabItem;
/// 
/// fn app(cx: Scope<'_>) -> Element {
///   let foo_tab = TabItem {
///     id: "foo",
///     title: "Foo",
///     content: cx.render(rsx! {
///       div { "Foo content" }
///     })
///   }
/// }
/// ```
#[derive(Props)]
pub struct TabItem<'a> {
  pub title: &'a str,
  pub id: &'a str,
  pub content: Element<'a>,
}

#[derive(Props)]
pub struct TabsProps<'a> {
  pub tabs: Vec<TabItem<'a>>
}

/// A container component that renders a tab header, allowing the user to
/// click to change which tab is displayed.
/// 
/// The tabs are defined in a Vec of [TabItem]s, which consist of a (unique) id,
/// title, and content Element. The content element must be rendered with `cx.render`
/// to have the correct type.
/// 
/// Example:
/// 
/// ```no_run
/// use dioxus::prelude::*;
/// use crate::components::tabs::{TabContainer, TabItem};
/// 
/// fn app(cx: Scope<'_>) -> Element {
///   cx.render(rsx! {
///     TabContainer {
///       tabs: vec![
///         TabItem {
///           id: "foo",
///           title: "Foo",
///           content: cx.render(rsx! {
///             div { "Foo stuff!" }
///           })
///         },
/// 
///         TabItem {
///           id: "bar",
///           title: "Bar",
///           content: cx.render(rsx! {
///             div { "Bar stuff!" }
///           })
///         },       
///      ]
///     }
///   })
/// }
/// ```
pub fn TabContainer<'a>(cx: Scope<'a, TabsProps<'a>>) -> Element<'a> {
  let first_id = cx.props.tabs.first()
    .map(|t| String::from(t.id))
    .unwrap_or_default();

  let active_tab = use_state(cx, || first_id);

  let nav_items = cx.props.tabs.iter().map(|t| {
    rsx! {
      TabNavItem {
        key: "{t.id}",
        id: t.id,
        title: t.title,
        active_tab: "{active_tab}",
        onclick: move |_e| { active_tab.set(String::from(t.id)) },
      }
    }
  });

  let content = cx.props.tabs.iter().map(|t| {
    rsx! {
      TabContent {
        key: "{t.id}",
        id: t.id,
        active_tab: "{active_tab}",
        &t.content
      }
    }
  });

  cx.render(rsx!{
    div { 
      style { include_str!("./style.css") }
    
      div {
        class: "tab-container",

        ul {
          class: "tab-header",
          
          nav_items
        }
        content
      }
    }
  })
}

#[derive(Props)]
struct TabNavItemProps<'a> {
  id: &'a str,
  title: &'a str,
  active_tab: &'a str,

  onclick: EventHandler<'a, MouseEvent>, 
}

fn TabNavItem<'a>(cx: Scope<'a, TabNavItemProps<'a>>) -> Element<'a> {
  let TabNavItemProps { id, title, active_tab, .. } = cx.props;
  let class = if *id == *active_tab { "active" } else { "inactive" };
  let title = *title;
  cx.render(rsx!{
    li {
      class: class,
      onclick: move |evt| cx.props.onclick.call(evt),

      title
    }
  })
}

#[derive(Props)]
struct TabContentProps<'a> {
  id: &'a str,
  active_tab: &'a str,
  children: Element<'a>,
}

fn TabContent<'a>(cx: Scope<'a, TabContentProps<'a>>) -> Element<'a> {
  let TabContentProps { id, active_tab, children } = cx.props;
  let active = *id == *active_tab;
  cx.render(rsx! {
    if active {
      children
    }
  })
}