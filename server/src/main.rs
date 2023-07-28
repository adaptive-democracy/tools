use std::collections::{HashSet, HashMap};
use core::hash::{Hash, Hasher};
use core::borrow::Borrow;
// use chrono::{DateTime as ChronoDateTime, Utc}

// type DateTime = chrono::DateTime<chrono::Utc>;
type DateTime = i64;


#[derive(Debug, Clone, Copy)]
enum ElectionKind {
	Document,
	Office,
}

#[derive(Debug, PartialEq)]
enum SelectionMethod {
	Resource,
	Quadratic,
	ResourceScore,
	QuadraticScore,
	ResourceApproval,
	QuadraticApproval,
}

trait Vote {
	fn total_weight(&self) -> f64;
	fn calculate_vote(&self) -> f64;
	fn selection_method() -> SelectionMethod;
}

#[derive(Debug)]
struct ResourceVote {
	candicacy_id: usize,
	weight: f64,
}
impl Vote for ResourceVote {
	fn total_weight(&self) -> f64 { self.weight }
	fn calculate_vote(&self) -> f64 { self.weight }
	fn selection_method() -> SelectionMethod { SelectionMethod::Resource }
}
fn aggregate_election_resource_votes(votes: &Vec<ResourceVote>) -> HashMap<usize, f64> {
	let mut vote_aggregation = HashMap::new();

	for vote in votes {
		vote_aggregation
			.entry(vote.candicacy_id)
			.or_insert(vote.weight)
			.and_modify(|t| *t += vote.weight);
	}

	vote_aggregation
}


#[derive(Debug)]
struct QuadraticVote {
	candicacy_id: usize,
	weight: f64,
}
impl Vote for QuadraticVote {
	fn total_weight(&self) -> f64 { self.weight }
	fn calculate_vote(&self) -> f64 { quadratic_vote(self.weight) }
	fn selection_method() -> SelectionMethod { SelectionMethod::Quadratic }
}
fn quadratic_vote(weight: f64) -> f64 {
	self.weight.signum() * self.weight.abs().sqrt()
}
fn aggregate_election_quadratic_votes(votes: &Vec<QuadraticVote>) -> HashMap<usize, f64> {
	let mut vote_aggregation = HashMap::new();

	for vote in votes {
		let weight = quadratic_vote(vote.weight);
		vote_aggregation
			.entry(vote.candicacy_id)
			.or_insert(weight)
			.and_modify(|t| *t += weight);
	}

	vote_aggregation
}


#[derive(Debug)]
struct ResourceScoreVote {
	weight: f64,
	scores: HashMap<usize, f64>,
}
// TODO an individual voter can't give more than one score to the same candidacy in a single allocation
// TODO validate that all scores have the same sign, which should already be true by this point
fn aggregate_election_resource_score_votes(votes: &Vec<ResourceScoreVote>) -> HashMap<usize, f64> {
	let mut vote_aggregation = HashMap::new();

	for vote in votes {
		for (candicacy_id, score) in vote.scores {
			let candidate_score = vote.weight * score;
			vote_aggregation
				.entry(candicacy_id)
				.or_insert(candidate_score)
				.and_modify(|t| *t += candidate_score);
		}
	}

	vote_aggregation
}

#[derive(Debug)]
struct QuadraticScoreVote {
	weight: f64,
	scores: HashMap<usize, f64>,
}
// TODO an individual voter can't give more than one score to the same candidacy in a single allocation
// TODO validate that all scores have the same sign, which should already be true by this point
fn aggregate_election_quadratic_score_votes(votes: &Vec<QuadraticScoreVote>) -> HashMap<usize, f64> {
	let mut vote_aggregation = HashMap::new();

	for vote in votes {
		let scaled_weight = quadratic_vote(vote.weight);
		for (candicacy_id, score) in vote.scores {
			let candidate_score = scaled_weight * score;
			vote_aggregation
				.entry(candicacy_id)
				.or_insert(candidate_score)
				.and_modify(|t| *t += candidate_score);
		}
	}

	vote_aggregation
}


#[derive(Debug)]
struct ResourceApprovalVote {
	weight: f64,
	approvals: HashMap<usize, bool>,
}
// TODO an individual voter can't give more than one score to the same candidacy in a single allocation
fn aggregate_election_resource_approval_votes(votes: &Vec<ResourceApprovalVote>) -> HashMap<usize, f64> {
	let mut vote_aggregation = HashMap::new();

	for vote in votes {
		// TODO consider just doing both possible multiplications
		for (candicacy_id, approval) in vote.approvals {
			let approval = if approval { 1 } else { 0 };
			let candidate_approval = vote.weight * approval;
			vote_aggregation
				.entry(candicacy_id)
				.or_insert(approval)
				.and_modify(|t| *t += approval);
		}
	}

	vote_aggregation
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



#[derive(Debug)]
struct PolityActionEntry {
	occurred_at: DateTime,
	change: PolityAction,
}

#[derive(Debug)]
enum PolityAction {
	EnterPerson{ id: usize },
	SetAllocations{ voter_id: usize, allocations: Vec<Allocation> },
	ExitPerson{ id: usize },

	EnterCandidacy{ id: usize, owner_id: usize, election_id: usize, pitch: String, content: CandidacyContent },
	ExitCandidacy{ id: usize },

	Recalculate,
}

#[derive(Debug)]
struct Allocation {
	candidacy_id: usize,
	weight: f64,
	vote
}

#[derive(Debug)]
enum CandidacyContent {
	Document{ body: String, sub_elections: Vec<ElectionDefinition> },
	Office,
}

#[derive(Debug)]
struct ElectionDefinition {
	id: usize,
	title: String,
	description: String,
	kind: ElectionKind,
	selection_method: SelectionMethod,

	negative_buckets: NegativeBucketsKind,
	nomination_requirement_method: NominationRequirementMethod,
	fill_requirement_method: FillRequirementMethod,
	update_frequency: chrono::Duration,
}

#[derive(Debug)]
enum NegativeBucketsKind {
	None,
	WithoutRemoval,
	WithRemoval,
}

#[derive(Debug)]
enum NominationRequirementMethod {
	Constant(f64),
	NoiseAdaptive,
}

#[derive(Debug)]
enum FillRequirementMethod {
	Constant(f64),
	OnlyElectorateSize,
	ElectorateSizeWithWideness,
}



#[derive(Debug)]
struct PolityState {
	person_table: HashSet<Person>,
	election_table: HashSet<Election>,
	candidacy_table: HashSet<Candidacy>,
}

// separating changes into a low level makes it possible to use any other persistence layer, as long as we can somehow serialize to that layer
impl PolityState {
	fn apply_changes(&mut self, changes: Vec<PolityStateChange>) {
		for change in changes.into_iter() {
			self.apply_change(change);
		}
	}

	fn apply_change(&mut self, change: PolityStateChange) {
		match expr {
			InsertPerson{ person_id } => {
				insert_or_conflict(errors, self.person_table, person_id, Person{ id: person_id, ... });
			},
			SetAllocations{ voter_id, allocations } => {
				if let Some(person) = require_present(errors, self.person_table, voter_id) {
					person.allocations = allocations;
				}
			},
			RemovePerson{ person_id } => {
				require_remove(errors, self.person_table, person_id);
			},

			InsertElection{ election } => {
				insert_or_conflict(errors, self.election_table, election.id, election);
			},
			RemoveElection{ election_id } => {
				require_remove(errors, self.election_table, election_id);
			},

			InsertCandidacy{ candidacy } => {
				insert_or_conflict(errors, self.candidacy_table, candidacy.id, candidacy);
			},
			SetCandidacyStatus{ candidacy_id, status } => {
				if let Some(candidacy) = require_present(errors, self.candidacy_table, voter_id) {
					candidacy.status = status;
				}
			},
			RemoveCandidacy{ candidacy_id } => {
				require_remove(errors, self.candidacy_table, candidacy_id);
			},
		}
	}
}

#[derive(Debug)]
enum PolityStateChange {
	InsertPerson{ person_id: usize },
	SetAllocations{ voter_id: usize, allocations: Vec<Allocation> },
	RemovePerson{ person_id: usize },

	InsertElection{ election: Election },
	RemoveElection{ election_id: usize },

	InsertCandidacy{ candidacy: Candidacy },
	SetCandidacyStatus{ candidacy_id: usize, status: CandidacyStatus },
	RemoveCandidacy{ candidacy_id: usize },
}


// trait IdAble {
// 	fn get_id(&self) -> usize;
// }

#[derive(Debug)]
enum TableKind {
	Person,
	Election,
	Candidacy,
}

trait TableKindAble { fn table_kind() -> TableKind; }

macro_rules! impl_id_traits {
	($structname: ident) => {
		// impl IdAble for $structname { fn get_id(&self) -> usize { self.id } }

		impl PartialEq for $structname {
			fn eq(&self, other: &Self) -> bool {
				self.id == other.id
			}
		}
		impl Eq for $structname {}
		impl Hash for $structname {
			fn hash<H: Hasher>(&self, state: &mut H) {
				self.id.hash(state);
			}
		}
		impl Borrow<usize> for $structname {
			fn borrow(&self) -> &usize {
				&self.id
			}
		}
		impl TableKindAble for $structname {
			fn table_kind() -> TableKind { TableKind::$structname }
		}
	};
}



#[derive(Debug)]
struct Person {
	id: usize,
	name: String,
	allocations: Vec<Allocation>,
}
impl_id_traits!(Person);

#[derive(Debug)]
struct Election {
	id: usize,
	title: String,
	description: String,
	nomination_fill_requirement: f64,
	fill_requirement: f64,
	kind: ElectionKind,
	selection_method: SelectionMethod,
	defining_document_id: Option<usize>,
}
impl_id_traits!(Election);

#[derive(Debug)]
struct Candidacy {
	id: usize,
	owner_id: usize,
	election_id: usize,
	content: CandidacyContent,
	status: CandidacyStatus,
}
impl_id_traits!(Candidacy);


#[derive(Debug)]
enum CandidacyStatus {
	Nomination(f64),
	Election(f64),
	Winner,
}


#[derive(Debug)]
enum PolityActionError {
	IdConflict{ id: usize, table_kind: TableKind },
	NotFound{ id: usize, table_kind: TableKind },
	OverAllowedWeight{ voter_id: usize, found_weight: f64, allowed_weight: f64 },
	MismatchedKind{ candidacy_id: usize, expected_kind: ElectionKind },
	WinningDocumentExit{ candidacy_id: usize },
}


fn require_not_present<T: Borrow<usize> + TableKindAble>(errors: &mut Vec<PolityActionError>, table: &HashSet<T>, id: &usize) -> Option<()> {
	if table.contains(id) {
		let table_kind = T::table_kind();
		errors.push(PolityActionError::IdConflict{ id: *id, table_kind });
		return None;
	}
	Some(())
}
fn require_present<T: Borrow<usize> + TableKindAble>(errors: &mut Vec<PolityActionError>, table: &HashSet<T>, id: &usize) -> Option<&T> {
	match table.get(id) {
		None => {
			let table_kind = T::table_kind();
			errors.push(PolityActionError::NotFound{ id: *id, table_kind });
			None
		},
		item => item,
	}
}

fn validate_allocations(errors: &mut Vec<PolityActionError>, allocations: &Vec<Allocation>, person: &Person) -> Option<()> {
	let mut have_errors = false;
	let mut found_weight = 0.0;
	for allocation in allocations {
		found_weight += allocation.total_weight();
		match require_present(errors, state.candidacy_table, allocation.candidacy_id) {
			None => { have_errors = true; },
			Some(candidacy) => {
				// TODO have to get the election selection_method and ensure this allocation fits it
				validate_method_matches_allocation()
				election.selection_method

				// this is probably overkill
				// the candidacy existing in this table should imply it's valid
				// match require_present(errors, state.election_table, candidacy.election_id) {
				// 	Some(election) => {
				// 		require_kinds_match(errors, candidacy.content, election.kind);
				// 	},
				// 	None => { have_errors = true; },
				// }
			},
		}
	}

	let allowed_weight = person.allowed_weight;
	if found_weight > allowed_weight {
		errors.push(PolityActionError::OverAllowedWeight{ voter_id: person.id, found_weight, allowed_weight });
		have_errors = true;
	}

	if have_errors { None } else { Some(()) }
}

fn require_candidacy_matches_election_kind(
	errors: &mut Vec<PolityActionError>,
	content: &CandidacyContent,
	election_kind: &ElectionKind,
	candidacy_id: &usize,
) -> Option<()> {
	match (content, election_kind) {
		(CandidacyContent::Document{..}, ElectionKind::Document)
		| (CandidacyContent::Office, ElectionKind::Office) => {
			Some(())
		},

		(_, _) => {
			errors.push(PolityActionError::MismatchedKind{ candidacy_id: *candidacy_id, expected_kind: *election_kind });
			None
		},
	}
}

fn require_not_winning_document(
	errors: &mut Vec<PolityActionError>,
	status: &CandidacyStatus,
	content: &CandidacyContent,
	candicacy_id: &usize,
) -> Option<()> {
	match (status, content) {
		(CandidacyStatus::Winner, CandidacyContent::Document{..}) => {
			errors.push(PolityActionError::WinningDocumentExit{ candicacy_id: *candicacy_id });
			None
		},
		_ => Some(())
	}
}

fn calculate_polity_action(
	state: &mut PolityState,
	errors: &mut Vec<PolityActionError>,
	changes: &mut Vec<PolityStateChange>,
	action: PolityAction,
) -> Option<()> {
	match action {
		PolityAction::EnterPerson{ id } => {
			require_not_present(errors, state.person_table, &id)?;
			changes.push(PolityStateChange::InsertPerson{ person });
		},
		PolityAction::SetAllocations{ voter_id, allocations } => {
			let person = require_present(errors, state.person_table, &voter_id)?;
			validate_allocations(errors, &allocations, person)?;
			changes.push(PolityStateChange::SetAllocations{ voter_id, allocations });
		},
		PolityAction::ExitPerson{ id } => {
			require_present(errors, state.person_table, &id)?;
			changes.push(PolityStateChange::RemovePerson{ id });
		},

		PolityAction::EnterCandidacy{ id, owner_id, election_id, pitch, content } => {
			require_not_present(errors, state.candidacy_table, &id)?;
			require_present(errors, state.person_table, &owner_id)?;
			let election = require_present(errors, state.election_table, &election_id)?;
			require_candidacy_matches_election_kind(errors, &content, &election.kind)?;

			let status = match election.nomination_requirement_method {
				NominationRequirementMethod::Constant(_) => { CandidacyStatus::Nomination(0.0) },
				NominationRequirementMethod::None => { CandidacyStatus::Election(0.0) },
			};
			let candidacy = Candidacy{ id, owner_id, election_id, pitch, content, status };
			changes.push(PolityStateChange::InsertCandidacy{ candicacy });
		},
		PolityAction::ExitCandidacy{ candicacy_id } => {
			let candidacy = require_present(errors, state.candidacy_table, &candicacy_id)?;
			require_not_winning_document(errors, &candidacy.status, &candidacy.content, &candicacy_id)?;

			// no need to issue election deletions, this isn't allowed to be a document winner
			// similarly no need to delete allocations, we should just ignore allocations to non-existent candidacies
			changes.push(PolityStateChange::RemoveCandidacy{ candicacy_id });
		},

		PolityAction::Recalculate => {
			// let grouped_candidacies = group_candidacies_by_election_id(state.candidacy_table);
			let mut grouped_candidacies: HashMap<usize, HashSet<&Candidacy>> = HashMap::new();
			for candidacy in state.candidacy_table {
				grouped_candidacies
					.entry(candidacy.election_id)
					.or_insert_default()
					.and_modify(|v| v.add(&candicacy));
			}
			let grouped_candidacies = grouped_candidacies;

			// find all allocations and group them by election_id
			let mut allocations_by_election_id = HashMap::new::<usize, Vec<&RawAllocation>>();
			for person in state.person_table {
				for allocation in person.allocations {
					allocations_by_election_id
						.entry(allocation.election_id)
						.or_insert_default()
						.and_modify(|v| v.push(allocation.raw));
				}
			}
			let allocations_by_election_id = allocations_by_election_id;

			for (election_id, candidacy_set) in grouped_candidacies {
				let election = match require_present(errors, state.election_table, &election_id) {
					Some(election) => election,
					None => { continue; },
				};
				let unparsed_allocations = match allocations_by_election_id.get(&election_id) {
					Some(unparsed_allocations) => unparsed_allocations,
					None => { continue; },
				};
				let aggregated = match election.selection_method {
					SelectionMethod::Resource => {
						aggregate_election_resource_votes(parse_votes(errors, unparsed_allocations))
					},
					SelectionMethod::Quadratic => {
						aggregate_election_quadratic_votes(parse_votes(errors, unparsed_allocations))
					},
					SelectionMethod::ResourceScore => {
						aggregate_election_resource_score_votes(parse_votes(errors, unparsed_allocations))
					},
					SelectionMethod::QuadraticScore => {
						aggregate_election_quadratic_score_votes(parse_votes(errors, unparsed_allocations))
					},
					SelectionMethod::ResourceApproval => {
						aggregate_election_resource_approval_votes(parse_votes(errors, unparsed_allocations))
					},
					SelectionMethod::QuadraticApproval => {
						aggregate_election_quadratic_approval_votes(parse_votes(errors, unparsed_allocations))
					},
				}

				// simply ignore (or mark) allocations that point to candidacies that no longer exist, since that's probably not the fault of the voter
				// we just need to notify them to switch their weights, which they can do whenever they want

				// TODO separate them by status
				// calculate their new statii
				// issue candidacy updates for all that changed
				// issue election deletions for elections that are no longer live

			}

			find_current_winner
			calculate_next_stabilization_buckets
		},
	}

	Some(())
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
