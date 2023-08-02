use std::collections::{HashSet, HashMap};


#[derive(Debug)]
enum Allocation {
	Resource(ResourceVote),
	Quadratic(QuadraticVote),
	ResourceScore(ResourceScoreVote),
	QuadraticScore(QuadraticScoreVote),
	ResourceApproval(ResourceApprovalVote),
	QuadraticApproval(QuadraticApprovalVote),
}

trait Vote {
	fn total_weight(&self) -> f64;
	fn calculate_vote(&self) -> f64;
	fn selection_method() -> SelectionMethod;
}
fn aggregate_votes<T: Vote>(votes: &Vec<T>) -> HashMap<usize, f64> {
	let mut vote_aggregation = HashMap::new();

	for vote in votes {
		let actual_vote = vote.calculate_vote();
		vote_aggregation
			.entry(vote.candicacy_id)
			.or_insert(actual_vote)
			.and_modify(|t| *t += actual_vote);
	}

	vote_aggregation
}

impl Vote for ResourceVote {
	fn total_weight(&self) -> f64 { self.weight }
	fn calculate_vote(&self) -> f64 { self.weight }
	fn selection_method() -> SelectionMethod { SelectionMethod::Resource }
}

impl Vote for QuadraticVote {
	fn total_weight(&self) -> f64 { self.weight }
	fn calculate_vote(&self) -> f64 { quadratic_vote(self.weight) }
	fn selection_method() -> SelectionMethod { SelectionMethod::Quadratic }
}
fn quadratic_vote(weight: f64) -> f64 {
	weight.signum() * weight.abs().sqrt()
}


impl Vote for ResourceScoreVote {
	fn total_weight(&self) -> f64 { self.approve_weight + self.disapprove_weight }
	fn calculate_vote(&self) -> f64 { resource_score_vote(self.score, self.approve_weight, self.disapprove_weight) }
	fn selection_method() -> SelectionMethod { SelectionMethod::ResourceScore }
}
fn resource_score_vote(score: f64, approve_weight: f64, disapprove_weight: f64) -> f64 {
	score * (if score < 0 { disapprove_weight } else { approve_weight })
}

impl Vote for ResourceScoreVote {
	fn total_weight(&self) -> f64 { self.approve_weight + self.disapprove_weight }
	fn calculate_vote(&self) -> f64 { quadratic_score_vote(self.score, self.approve_weight, self.disapprove_weight) }
	fn selection_method() -> SelectionMethod { SelectionMethod::QuadraticScore }
}
fn quadratic_score_vote(score: f64, approve_weight: f64, disapprove_weight: f64) -> f64 {
	score * quadratic_vote(if score < 0 { disapprove_weight } else { approve_weight })
}


impl Vote for ResourceScoreVote {
	fn total_weight(&self) -> f64 { self.approve_weight + self.disapprove_weight }
	fn calculate_vote(&self) -> f64 { quadratic_score_vote(self.score, self.approve_weight, self.disapprove_weight) }
	fn selection_method() -> SelectionMethod { SelectionMethod::QuadraticScore }
}


#[derive(Debug)]
struct QuadraticApprovalVote {
	weight: f64,
	approvals: HashMap<usize, bool>,
}
// TODO an individual voter can't give more than one score to the same candidacy in a single allocation
fn aggregate_election_quadratic_approval_votes(votes: &Vec<QuadraticApprovalVote>) -> HashMap<usize, f64> {
	let mut vote_aggregation = HashMap::new();

	for vote in votes {
		// TODO consider just doing both possible multiplications
		let scaled_weight = quadratic_vote(vote.weight);
		for (candicacy_id, approval) in vote.approvals {
			let approval = if approval { 1 } else { 0 };
			let candidate_approval = scaled_weight * approval;
			vote_aggregation
				.entry(candicacy_id)
				.or_insert(approval)
				.and_modify(|t| *t += approval)
		}
	}

	vote_aggregation
}


// fn find_aggregate_winner(vote_aggregation: &HashMap<usize, f64>) -> Option<usize> {
// 	let mut maximum = 0;
// 	let mut current_winners = Vec::new();

// 	for (candicacy_id, total_vote) in vote_aggregation {
// 		if *total_vote > maximum {
// 			maximum = *total_vote;
// 			current_winners.clear();
// 			current_winners.push(candicacy_id);
// 		}
// 		else if *total_vote > 0 && *total_vote == maximum {
// 			current_winners.push(candicacy_id);
// 		}

// 	}
// 	// if there is a tie, no one wins
// 	// if no one receives a strictly positive vote, no one wins
// 	if current_winners.len() == 1 {
// 		Some(*current_winners[0])
// 	}
// 	// TODO possibly consider returning some kind of error object containing all the tied winners
// 	// else if current_winners.len() > 1 {
// 	else {
// 		None
// 	}
// }

#[derive(Debug)]
struct CandidacyEntry {
	stabilization_bucket: f64,
	total_vote: f64,
}

// fill requirements are best if calculated based on the size of the electorate exposed to an election, along with some for now undetermined "splitting" concept based on how many elections there are that this electorate is exposed to
// the fill_requirement should be the amount where the election would change immediately if the entire electorate all voted for the same candidate
// again, not sure if that means

fn find_current_winner(candidacy_vec: &Vec<Candidacy>) -> Result<Option<&Candidacy>, (&Candidacy, &Candidacy)> {
	let mut current_winner = None;
	for candidacy in candidacy_vec {
		match (current_winner, candidacy.stabilization_bucket) {
			// this candidacy isn't the winner, do nothing
			(_, Some(_)) => {},
			// this candidacy is the winner and isn't conflicting with a previous find, set it
			(None, None) => { current_winner = Some(candidacy); }
			// inconsistency
			(Some(a), Some(b)) => { return Err((a, b)); },
		}
	}

	Ok(current_winner)
}


fn calculate_next_stabilization_buckets(
	fill_requirement: f64,
	current_winner: Option<(usize, f64)>,
	candicacy_entries: &HashMap<usize, CandidacyEntry>,
) -> HashMap<usize, Option<f64>> {

	let (current_winner_id, current_winner_total_vote) = winner.unwrap_or_default((0, 0.0));
	let current_winner_id = if current_winner_id == 0 { None } else { Some(current_winner_id) };
	let mut candicacy_new_buckets = HashMap::new();

	let mut positive_filled_maximum = 0.0;
	let mut current_possible_winners = Vec::new();
	for (candicacy_id, CandidacyEntry{stabilization_bucket, total_vote}) in candicacy_entries {
		// TODO consider allowing buckets to *go negative* if total_vote is negative, and even possibly *removing* a candidate if they reach *negative* fill_requirement
		let candidacy_new_bucket = std::cmp::max(
			stabilization_bucket + (total_vote - current_winner_total_vote),
			0,
		);
		candicacy_new_buckets.insert(candicacy_id, Some(candidacy_new_bucket));

		// it isn't sound to declare the mere highest candidate the new winner when there isn't a current winner
		// doing so would be vulnerable, where a highly approved current winner resigns, allowing a weak challenger to immediately take the stabilized spot
		// it makes sense to *always* require a bucket fill even in situations where there isn't a current winner
		// the alternative would be to simply change fill_requirement to 0 if there isn't a current winner

		// if this candidacy has reached the requirement then it has the chance to be the *unique* winner
		if total_vote <= 0.0 || candidacy_new_bucket < fill_requirement { continue; }

		if total_vote == positive_filled_maximum {
			current_possible_winners.push(candicacy_id);
		}
		else if total_vote > positive_filled_maximum {
			positive_filled_maximum = total_vote;
			current_possible_winners.clear();
			current_possible_winners.push(candicacy_id);
		}
	}

	// there's a new unique winner
	if current_possible_winners.len() == 1 {
		let new_winner_id = *current_possible_winners[0];
		candicacy_new_buckets.insert(new_winner_id, None)
	}
	// there's a tie or no one met the requirements
	else {
		// the current winner (if there is one) remains the current winner
		if let Some(winner_id) = current_winner_id {
			candicacy_new_buckets.insert(winner_id, None);
		}
	}

	candicacy_new_buckets
}



fn main() {
	let candidacy_vec = vec![
		Candidacy{owner_id: 0, election_id: 0, stabilization_bucket: None},
		Candidacy{owner_id: 1, election_id: 0, stabilization_bucket: Some(0.0)},
		Candidacy{owner_id: 2, election_id: 0, stabilization_bucket: Some(0.0)},
		Candidacy{owner_id: 3, election_id: 0, stabilization_bucket: Some(0.0)},
	];

	let resource_votes = vec![
		ResourceVote{candicacy_id: 0, weight: 200},
		ResourceVote{candicacy_id: 0, weight: 400},
		ResourceVote{candicacy_id: 0, weight: -200},

		ResourceVote{candicacy_id: 1, weight: 200},
		ResourceVote{candicacy_id: 1, weight: 300},
	];

	let agg = aggregate_resource_votes(&resource_votes);
	println!("{:?}", agg);
	let winner = find_aggregate_winner(&agg);
	println!("{:?}", winner);


	let next_stabilization_buckets = calculate_next_stabilization_buckets(
		10.0,
		Some((0, )),
		HashMap::from([
			(0, CandidacyEntry{stabilization_bucket, total_vote}),
		]);
	)
}


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


// // persistent democracy is ultimately implemented by a long-running server
// // such a server needs to support these things:
// // - creating new elections, and in the future determining the set of elections based on persistent constitutions
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
