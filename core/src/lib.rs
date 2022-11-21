use std::collections::HashMap;

// TODO find fixed precision numeric library
type Weight = f32;
// TODO make id wrapper for different id?

#[derive(Debug)]
struct Person {
	id: usize,
	name: String,
}

#[derive(Debug)]
struct Election {
	id: usize,
	title: String,
}

#[derive(Debug)]
struct Candidacy {
	election_id: usize,
	candidate_id: usize,
	stabilization_bucket: Option<Weight>,
}

impl Candidacy {
	pub fn key(&self) -> (usize, usize) {
		(self.election_id, self.candidate_id)
	}
}


#[derive(Debug)]
enum AllocationType {
	For,
	Against,
}

#[derive(Debug)]
struct Allocation {
	voter_id: usize,
	election_id: usize,
	candidate_id: usize,

	weight: Weight,
	allocation_type: AllocationType,
}

impl Allocation {
	pub fn key(&self) -> (usize, usize) {
		(self.election_id, self.candidate_id)
	}

	fn actual_vote(&self) -> Weight {
		let direction = match self.allocation_type {
			AllocationType::For => 1.0,
			AllocationType::Against => -1.0,
		};

		direction * self.weight.sqrt()
	}
}


// #[derive(Debug)]
// struct Analysis {
// 	duplicate_person_vec: ,
// 	duplicate_election_vec: ,
// 	duplicate_candidacy_vec: ,
// 	duplicate_allocation_vec: ,

// 	invalid_voter_allocation_vec: ,
// 	candidacy_total_vote: ,
// }

// fn analyze_all(
// 	person_vec: Vec<Person>,
// 	election_vec: Vec<Election>,
// 	candidacy_vec: Vec<Candidacy>,
// 	allocation_vec: Vec<Allocation>,
// ) -> RetType {
// 	// lookup map for person
// 	// lookup map for election
// }



fn compute_total_votes(candidacy_vec: Vec<Candidacy>, allocation_vec: Vec<Allocation>) -> HashMap<(usize, usize), Weight> {
	// let invalid_allocation_vec = vec![];
	// let duplicate_candidacy_vec = vec![];

	let mut candidacy_vote_map = HashMap::new();
	for candidacy in candidacy_vec {
		if let Some(_) = candidacy_vote_map.insert(candidacy.key(), 0.0) {
			// duplicate_candidacy_vec.push(candidacy);
			eprintln!("duplicate candidacy: {:?}", candidacy);
		}
	}

	for allocation in allocation_vec {
		match candidacy_vote_map.get_mut(&allocation.key()) {
			Some(candidacy_vote) => {
				*candidacy_vote += allocation.actual_vote();
			},
			None => {
			// invalid_allocation_vec.push(candidacy);
				eprintln!("invalid allocation: {:?}", allocation);
			},
		}
	}

	candidacy_vote_map
}

// fn compute_next_candidacy_values(arg: Type) -> RetType {
// 	unimplemented!()
// }



#[cfg(test)]
mod tests {
	use super::*;
	use AllocationType::*;

	fn allo(voter_id: usize, (election_id, candidate_id): (usize, usize), weight: Weight, allocation_type: AllocationType) -> Allocation {
		Allocation { voter_id, election_id, candidate_id, weight, allocation_type }
	}

	fn cand((election_id, candidate_id): (usize, usize), stabilization_bucket: Option<Weight>) -> Candidacy {
		Candidacy { election_id, candidate_id, stabilization_bucket }
	}

	#[test]
	fn test_actual_vote() {
		let mut allocation = Allocation {
			voter_id: 0, election_id: 0, candidate_id: 0,
			weight: 4.0,
			allocation_type: For,
		};

		assert_eq!(allocation.actual_vote(), 2.0);

		allocation.weight = 1.0;
		assert_eq!(allocation.actual_vote(), 1.0);

		allocation.allocation_type = Against;
		assert_eq!(allocation.actual_vote(), -1.0);
	}


	#[test]
	fn test_compute_total_votes() {
		let candidacy_vec = vec![cand((1, 1), Some(0.0))];
		let candidacy_map: HashMap<(usize, usize), Weight> = candidacy_vec.iter()
			.map(|c| (c.key(), c.stabilization_bucket.unwrap_or(0.0)))
			.collect();
		dbg!(candidacy_map);




		let candidacy_vec = vec![
			cand((1, 1), None),
		];
		let allocation_vec = vec![
			allo(0, (1, 1), 4.0, For),
			allo(0, (0, 0), 4.0, For),
		];

		assert_eq!(compute_total_votes(candidacy_vec, allocation_vec), HashMap::from([
			((1, 1), 2.0),
			// ((1, 2), 0.7),
			// ((2, 1), 1.0),
			// ((2, 2), 1.0),
		]));
	}
}
