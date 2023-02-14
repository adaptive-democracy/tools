use uuid::Uuid;
use serde::Deserialize;
use sycamore::prelude::*;

use crate::Weight;

#[derive(Debug, Deserialize)]
struct Election {
	id: Uuid,
	title: String,
	description: String,
	current_winner: Option<WinningCandidate>,
	candidates: Vec<RunningCandidate>,
	// in a quadratic range election the *election* has an allocation rather than the candidates
	// allocated_for_weight: Option<Weight>,
	// allocated_against_weight: Option<Weight>,
}

#[derive(Debug, Deserialize)]
struct Person {
	id: Uuid,
	name: String,
}

#[derive(Debug, Deserialize)]
struct WinningCandidate {
	person: Person,
	current_weight: Weight,
	my_allocation: Option<Allocation>,
	// my_score: Option<(Weight, PreferenceDirection)>,
}

#[derive(Debug, Deserialize)]
struct RunningCandidate {
	person: Person,
	stabilization_bucket: Weight,
	current_weight: Weight,
	my_allocation: Option<Allocation>,
}

#[derive(Debug, Deserialize)]
struct Allocation {
	// candidate_id: Uuid,
	weight: Weight,
	preference_direction: PreferenceDirection,
}

#[derive(Debug, Deserialize)]
pub enum PreferenceDirection {
	For,
	Against,
}

impl PreferenceDirection {
	fn to_str(&self) -> &'static str {
		match self {
			PreferenceDirection::For => "for",
			PreferenceDirection::Against => "against",
		}
	}
}

async fn fetch_election(_id: Uuid) -> Result<Election, gloo_net::Error> {
	gloo_timers::future::TimeoutFuture::new(500).await;
	unimplemented!()
}

// async fn fetch_election(id: Uuid) -> Result<Election, gloo_net::Error> {
// 	use gloo_net::http::Request;
// 	use crate::API_BASE_URL;
// 	let url = format!("{API_BASE_URL}/election/{id}");
// 	let body =
// 		Request::get(&url).send().await?
// 		.json::<Election>().await?;
// 	Ok(body)
// }


#[component(inline_props)]
pub fn ElectionView<G: Html>(cx: Scope, id: Uuid) -> View<G> {
	let election = crate::utils::create_async_signal(cx, fetch_election(id));

	view!{cx,
		(match create_ref(cx, election.get()).as_ref() {
			None => view!{cx, h1 { "loading" } },

			// TODO better error display
			Some(Err(err)) => view!{cx, h1 { "An error occurred!" } div { (format!("{:?}", err)) } },

			Some(Ok(election)) => view!{cx,
				h1 { (election.title) }
				p { (election.description) }

				(if let Some(current_winner) = &election.current_winner {
					view!{cx,
						h2 { "Current winner is :" PersonLink(person=&current_winner.person) " with " (current_winner.current_weight) }
						MyAllocation(my_allocation=&current_winner.my_allocation)
					}
				}
				else { view!{cx, p { "No current winner!" } } })

				h2 { "candidates" }
				(View::new_fragment(election.candidates.iter().map(|candidate| view!{cx,
					h3 { PersonLink(person=&candidate.person) }
					div { "Stabilization bucket: " (candidate.stabilization_bucket) }
					div { "Current weight: " (candidate.current_weight) }
					MyAllocation(my_allocation=&candidate.my_allocation)
				}).collect()))
			},
		})
	}
}

#[component(inline_props)]
fn MyAllocation<'s, G: Html>(cx: Scope<'s>, my_allocation: &'s Option<Allocation>) -> View<G> {
	if let Some(allocation) = my_allocation { view!{cx,
		p { "You have voted " (allocation.preference_direction.to_str()) " this candidate with " (allocation.weight) "weight." }
	}}
	else { view!{cx,} }
}

#[component(inline_props)]
fn PersonLink<'s, G: Html>(cx: Scope<'s>, person: &'s Person) -> View<G> {
	view!{cx, a(href=format!("/person/{}", person.id)) { (person.name) } }
}
