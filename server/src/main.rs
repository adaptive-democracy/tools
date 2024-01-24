// use actix_web::{get, post, web::{self, Json, Path}, Responder, HttpResponse};

// use tokio::sync::RwLock;
// use std::collections::{hash_map, HashMap};
// use std::sync::atomic::AtomicU64;
// type AppState = RwLock<HashMap<u64, String>>;

// #[get("/get_voter/{voter_id}")]
// async fn get_voter(voter_id: Path<u64>, state: web::Data<AppState>) -> impl Responder {
// 	let voters = state.read().await;
// 	match voters.get(&voter_id.into_inner()) {
// 		Some(voter) => HttpResponse::Ok().json(voter),
// 		None => HttpResponse::NotFound().finish(),
// 	}
// }

// #[post("/add_voter")]
// async fn add_voter(voter: Json<String>, state: web::Data<AppState>) -> impl Responder {
// 	let mut voters = state.write().await;
// 	match voters.entry(voter_id.into_inner()) {
// 		// do nothing in the occupied case?
// 		hash_map::Entry::Occupied(_) => {},
// 		hash_map::Entry::Vacant(e) => { e.insert(voter.into_inner()); },
// 	}
// 	// this is one of those security things, do we say whether we did anything?
// 	HttpResponse::NoContent().finish()
// }


// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
// 	let voters: HashMap<u64, String> = HashMap::new();
// 	let state = web::Data::new(RwLock::new(voters));

// 	actix_web::HttpServer::new(move ||
// 		actix_web::App::new()
// 			.app_data(state.clone())
// 			.service(get_voter)
// 			.service(add_voter)
// 	)
// 		.bind("127.0.0.1:5050")?
// 		.run()
// 		.await
// }


// // we have electorates
// // we have elections
// // we have voters
// // we have candidates, who have to be voters

// // when a voter updates their weights, they're ultimately mutating the state of their current allocation
// // we can of course treat this mutation non-destructively by keeping a full log, or even some kind of diff, but conceptually we want to update in place


// // #[derive(Debug)]
// // struct Voter {
// // 	id: usize,
// // 	// name: String,
// // }


// // #[derive(Debug)]
// // enum Ballot {
// // 	Variant1,
// // 	Variant2,
// // }

// use std::collections::HashMap;

// #[derive(Debug)]
// struct ResourceVoteAllocation {
// 	// election_id: usize,
// 	candidate_id: usize,
// 	allocated_weights: usize,
// 	is_negative: bool,
// }

// fn main() {
// 	let mut voters = HashMap::new();
// 	voters.insert(1, "A");
// 	voters.insert(2, "B");
// 	voters.insert(3, "C");
// 	let voters = dbg!(voters);

// 	// each voter has 100 weights to allocate
// 	const ALLOWED_WEIGHTS = 100;

// 	// let's say these voters are choosing between three options, using a pure resource vote
// 	let mut candidates = HashMap::new();
// 	candidates.insert(10, "Red");
// 	candidates.insert(20, "Green");
// 	candidates.insert(30, "Blue");
// 	let candidates = dbg!(candidates);

// 	let updates_by_day: Vec<Vec<(usize, usize, isize)>> = vec![
// 		vec![
// 			// voter_id, candidate_id, weights
// 			// weights can be negative to indicate a negative vote,
// 			(1, 10, 90), (1, 20, -10),
// 			(2, 30, 100),
// 		],
// 		vec![
// 			(2, 10, -10), (2, 30, 80),
// 		],
// 		vec![
// 			(1, 10, 90), (1, 20, -10),
// 			(3, 20, 80),
// 		],
// 	];

// 	let mut weight_by_voter = HashMap::new();
// 	let mut weights_by_candidate = HashMap::new();
// 	for updates in updates_by_day {
// 		for voter_id in voters.keys() {
// 			weights_by_voter.set(voter_id, 0);
// 		}
// 		for candidate_id in candidates.keys() {
// 			weights_by_candidate.set(candidate_id, 0);
// 		}

// 		for (voter_id, candidate_id, weights) in updates {
// 			match weight_by_voter.get_mut(voter_id) {
// 				// none means this voter doesn't exist, ignore this update
// 				// TODO put update in an error log
// 				None => { continue },
// 				Some(voter_weights) => {
// 					let used_weights = weights.abs();
// 					// check if the voter has exceeded their weights, and don't perform this update if they have
// 					// TODO put update in an error log
// 					if (*voter_weights + used_weights) > ALLOWED_WEIGHTS {
// 						continue
// 					}
// 					*voter_weights += used_weights;
// 				},
// 			}

// 			match weights_by_candidate.get_mut(candidate_id) {
// 				// none means the voter somehow voted for a non-existent candidate, ignore this update
// 				// TODO put update in an error log
// 				None => { continue },
// 				Some(candidate_weights) => {
// 					*candidate_weights += weights;
// 				},
// 			}

// 			// now we can figure out the standing of candidates
// 			weights_by_candidate
// 		}

// 		println!("{:?}", day);
// 	}
// }


// // adaptive democracy is ultimately implemented by a long-running server
// // such a server needs to support these things:
// // - creating new elections, and in the future determining the set of elections based on adaptive constitutions
// // - adding or removing candidates to elections, using whatever nomination rules exist for that election
// // - adding or removing voters
// // - accepting voter weight allocation updates
// // - making the current published state available, considering whatever update schedule exists

// // so the server has this global state that needs to be managed, which I'll do with async RwLocks:
// // - list of voters
// // - list of elections, and their candidates
// // - list of historic updates, those updates that have already been applied in a previous published state
// // - list of live updates that will be applied to the published state on the next update cycle
// // - current published state, so for each live election who is the current winning candidate and what is the weight/bucket state

// // so we need these functions, which will become routes:
// // - add/remove voter, needs write access to voters
// // - add/remove election/candidate, needs write access to elections
// // - update vote allocation for some voter, this needs read access to voters and elections, write access to live updates. only needs read access to voters and elections because those updates are performed immediately when given. this immediacy underscores the need for some anti-noise mechanisms in nomination
// // - perform update, needs read access to voters/elections, write access to historic/live updates. updates are checked for still valid voters/candidates, which might have been removed since an update was sent in. any updates referring to old things should be ignored for tabulation but the voter in question should be notified
// // - access current published state. the function is trivial since we're assuming the current state is mutated in place every update, so the route just returns the current object

// fn add_voter(voters: &mut IdentitySet<Voter>, new_voter: Voter) -> Result<(), AddVoterErr> {
// 	voters.insert_if_new(new_voter)
// 		.map_err()
// }

// fn remove_voter(voters: &mut IdentitySet<Voter>, voter_id: usize) -> Result<(), RemoveVoterErr> {
// 	voters.remove_by_id(voter_id)
// 		.into_err()
// }

// fn update_voter_metadata(voters: &mut IdentitySet<Voter>, voter_id: usize, metadata: VoterMetadata) -> Result<(), NotFoundErr> {
// 	voters.entry_by_id(voter_id)
// 		.and_modify(|voter| { voter.metadata = metadata })
// 		.into_err()
// }

// // similar functions for elections and candidates

// fn check_allocation_valid(
// 	voters: &IdentitySet<Voter>,
// 	elections: &IdentitySet<Election>,
// 	allocation: Allocation,
// ) -> Result<Allocation, InvalidAllocationErr> {
// 	asdf
// }

// fn update_voter_allocation(
// 	voters: &IdentitySet<Voter>,
// 	elections: &IdentitySet<Election>,
// 	// we could store these live allocations in a map by voter id instead, but whatever
// 	live_allocations: &mut Vec<Allocation>,
// 	new_allocation: Allocation,
// ) -> Result<(), InvalidAllocationErr> {
// 	// check if the allocation is valid

// 	// if it is, put it in
// 	live_allocations.push(new_allocation);
// 	Ok(())
// }

// fn do_update_tick(
// 	voters: &IdentitySet<Voter>,
// 	elections: &IdentitySet<Election>,
// 	applied_allocations: &mut Vec<Allocation>,
// 	live_allocations: &mut Vec<Allocation>,
// 	published_state: &mut PublishedState,
// ) -> Vec<InvalidAllocationErr> {
// 	let mut errors = vec![];
// 	let mut valid_allocations = vec![];
// 	for allocation in live_allocations {
// 		match check_allocation_valid(voters, elections, allocation) {
// 			Ok(valid_allocation) => { valid_allocations.push(valid_allocation); },
// 			Err(invalid_allocation_err) => { errors.push(invalid_allocation_err); },
// 		}
// 	}

// 	// TODO if the structure of live_allocations allows there to be more than one update for one voter, we need to make sure we ignore all but the latest, and just immediately put all old ones in applied_allocations

// 	unimplemented!()
// 	// with no stabilization buckets, the new published_state is merely a product of the existing live_allocations (which I'm realizing should definitely be a map from voters, since we need to keep around the latest allocation for each voter even if they haven't made an update this tick. old_allocations should only be pushed when a voter updates their allocation)

// 	applied_allocations.append(valid_allocations);
// 	errors
// }

// // this type is just a wrapper around a hashmap, but it accepts a function that calculates the lookup key
// // it's just used so we can store records that contain something like an id that should be used as their unique lookup key
// #[derive(Debug)]
// struct IdentitySet<T> {
// 	internal_map: HashMap<usize, T>,
// 	id_func: (T) -> usize,
// }
