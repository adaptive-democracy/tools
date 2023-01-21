use sycamore::prelude::*;

#[component]
fn App<G: Html>(cx: Scope) -> View<G> {
	let names = create_signal(cx, vec![]);

	let push_name = || {
		names.clone().set_fn_mut(|v| v.push("yo".to_string()));
	};

	let age = create_signal(cx, 0);

	view! { cx,
		button(on:click=|_| age.set_fn(|v| v + 1)) { "increment" }
		button(on:click=|_| age.set_fn(|v| v - 1)) { "decrement" }

		MyComponent(age=&age)
	}
}

#[component(inline_props)]
fn MyComponent<'a, G: Html>(cx: Scope<'a>, age: &'a ReadSignal<i32>) -> View<G> {
	view! { cx,
		div {
			"age: " (age.get())
		}
	}
}


fn main() {
	sycamore::render(|cx| view! { cx, App{} });
}
