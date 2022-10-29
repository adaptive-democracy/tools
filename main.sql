drop schema public cascade;
create schema public;
grant all on schema public to public;
comment on schema public is 'standard public schema';


create table person (
	id serial primary key,
	"name" text not null
);

-- create table document (
-- 	id serial primary key,
-- 	"text" text not null
-- );

create table election (
	id serial primary key,
	title text not null
);

create table candidacy (
	election_id int references election(id) not null,
	candidate_id int references person(id) not null,
	stabilization_bucket numeric default 0 check (stabilization_bucket >= 0),

	primary key (election_id, candidate_id),

	-- this exclusion constraint is equivalent to partial unique index, except it's deferrable
	-- create unique index unique_winner on candidacy(election_id) where stabilization_bucket is null
	-- https://dba.stackexchange.com/questions/166082/deferrable-unique-index-in-postgres
	constraint unique_winner
		exclude (election_id with =)
		where (stabilization_bucket is null)
		deferrable initially deferred
);

create type allocation_type as enum('FOR', 'AGAINST');
create table allocation (
	voter_id int references person(id) not null,
	election_id int not null,
	candidate_id int not null,

	weight int not null check (weight > 0),
	"type" allocation_type not null default 'FOR',

	-- every allocation must reference a valid candidacy
	foreign key (election_id, candidate_id) references candidacy(election_id, candidate_id),

	-- each voter can only allocate to each candidacy once
	primary key (voter_id, election_id, candidate_id)
);

create index index_allocation_voter_id on allocation(voter_id);

-- TODO make this concurrency safe https://www.cybertec-postgresql.com/en/triggers-to-enforce-constraints/
create or replace function allocation_weight_valid() returns trigger as
$$
begin
	if (new.weight + (select coalesce(sum(weight), 0) from allocation where voter_id = new.voter_id)) <= 100 then
		return new;
	else
		raise exception 'allocated too much weight';
	end if;
end;
$$ language plpgsql;

create trigger check_allocation_weight_valid
before insert or update on allocation
for each row
execute procedure allocation_weight_valid();


create view candidacy_votes as
select
	election_id, candidate_id, stabilization_bucket,
	coalesce(sum((case when allocation."type" = 'FOR' then 1 else -1 end) * sqrt(weight)), 0) as total_vote
from
	candidacy
	left join allocation using (election_id, candidate_id)
group by election_id, candidate_id, stabilization_bucket;

create view current_winner as
select election_id, candidate_id, total_vote
from candidacy_votes
where stabilization_bucket is null;

create view candidacy_updated as
select
	c.election_id, c.candidate_id, c.total_vote,

	case
		-- current candidate is the winner, will be elsewhere updated to 0 if overtaken
		when c.stabilization_bucket is null then null
		-- current candidate isn't the winner
		else greatest(
			-- if the candidate has never overtaken the winner, their total_vote will be 0 and the greatest op keeps them there
			-- if the candidate has overtaken the winner either now or previously, then this difference op makes sense
			-- (will be positive when candidate is ahead, will be negative if they've fallen behind)
			c.stabilization_bucket + (c.total_vote - current_winner.total_vote),
			-- NOTE the fill amount should be designed to make sense when a raw difference is taken
			0
		)
	end as stabilization_bucket
from
	candidacy_votes as c
	left join current_winner using (election_id);

create view next_candidacy_values as
with
election_maxes as (
	select
		election_id,
		-- if all candidates are negative or zero, then:
		-- - if there isn't a current winner the new current winner must have non-negative votes
		-- - if there is a current winner who is negative and someone is less negative then the less negative should fill
		-- - if there is a current winner who is zero and everyone is negative or zero then no one should fill
		max(total_vote) as max_votes,
		-- TODO arbitrary fill requirement for now
		max(case when stabilization_bucket is not null and stabilization_bucket >= 10 then stabilization_bucket else null end) as max_bucket

	from candidacy_updated
	group by election_id
),

max_filled as (
	select c.election_id, stabilization_bucket, count(candidate_id) as num_candidates
	from
		candidacy_updated as c
		join election_maxes as m on c.election_id = m.election_id and c.stabilization_bucket = m.max_bucket
	group by c.election_id, stabilization_bucket
),

max_votes as (
	select c.election_id, total_vote, count(candidate_id) as num_candidates
	from
		candidacy_updated as c
		join election_maxes as m on c.election_id = m.election_id and c.total_vote = m.max_votes
	group by c.election_id, total_vote
)

select
	c.election_id, c.candidate_id,
	case
		-- there's no current winner
		when current_winner.total_vote is null then
			case
				-- this row uniquely has max non-negative votes
				when max_votes.num_candidates = 1 and max_votes.total_vote >= 0 and c.total_vote = max_votes.total_vote then null
				-- there's a tie, so we simply "do nothing", making no one the winner yet and keeping all the buckets at 0
				else 0
			end

		-- there's a new unique winner
		when max_filled.num_candidates = 1 and max_filled.stabilization_bucket is not null then
			case
				-- this row is the new winner
				when c.stabilization_bucket = max_filled.stabilization_bucket then null
				-- this row is not the new winner
				else 0
			end

		-- otherwise there's a tie for max filled bucket or no one has filled the bucket
		else c.stabilization_bucket

	end as stabilization_bucket

from
	candidacy_updated as c
	left join current_winner using (election_id)
	left join max_votes using (election_id)
	left join max_filled using (election_id);


create procedure perform_vote_update()
language sql as $$
update candidacy as c
set stabilization_bucket = n.stabilization_bucket
from next_candidacy_values as n
where c.election_id = n.election_id and c.candidate_id = n.candidate_id
$$;





insert into person (id, "name") values (1, 'han'), (2, 'luke'), (3, 'leia'), (4, 'vader'), (5, 'palpatine'), (6, 'lando'), (7, 'jabba');
insert into election (id, title) values (1, 'chancellor'), (2, 'hutt');

-- for chancellor: leia, vader, palpatine
insert into candidacy (election_id, candidate_id) values (1, 3), (1, 4), (1, 5);
-- for hutt: han, lando, jabba
insert into candidacy (election_id, candidate_id) values (2, 1), (2, 6), (2, 7);

-- X no winner to new winner (chancellor now leia)
-- X no winner to no winner because max vote tie (hutt)
truncate allocation;
-- luke votes for leia and against vader and palpatine
insert into allocation (voter_id, election_id, candidate_id, weight, "type") values (1, 1, 3, 50, 'FOR');
insert into allocation (voter_id, election_id, candidate_id, weight, "type") values (1, 1, 4, 25, 'AGAINST');
insert into allocation (voter_id, election_id, candidate_id, weight, "type") values (1, 1, 5, 25, 'AGAINST');

-- han does same, also votes for himself and against other two
insert into allocation (voter_id, election_id, candidate_id, weight, "type") values (2, 1, 3, 10, 'FOR');
insert into allocation (voter_id, election_id, candidate_id, weight, "type") values (2, 1, 4, 10, 'AGAINST');
insert into allocation (voter_id, election_id, candidate_id, weight, "type") values (2, 1, 5, 10, 'AGAINST');

insert into allocation (voter_id, election_id, candidate_id, weight, "type") values (2, 2, 1, 40, 'FOR');
insert into allocation (voter_id, election_id, candidate_id, weight, "type") values (2, 2, 6, 15, 'AGAINST');
insert into allocation (voter_id, election_id, candidate_id, weight, "type") values (2, 2, 7, 15, 'AGAINST');

-- vader merely votes against leia
insert into allocation (voter_id, election_id, candidate_id, weight, "type") values (4, 1, 3, 100, 'AGAINST');

-- lando votes to perfectly balance out han
insert into allocation (voter_id, election_id, candidate_id, weight, "type") values (6, 2, 1, 40, 'AGAINST');
insert into allocation (voter_id, election_id, candidate_id, weight, "type") values (6, 2, 6, 15, 'FOR');
insert into allocation (voter_id, election_id, candidate_id, weight, "type") values (6, 2, 7, 15, 'FOR');

select * from candidacy_updated order by election_id, candidate_id;
call perform_vote_update();
select * from candidacy order by election_id, candidate_id;


-- X current winner to same winner because no filled (chancellor still leia)
-- X current winner to same winner because filled tie (hutt)
truncate allocation;
-- arbitrarily make jabba hutt winner for test
update candidacy set stabilization_bucket = null where election_id = 2 and candidate_id = 7;

-- palpatine slightly overtakes leia but not enough to fill, because luke and han drop those votes
insert into allocation (voter_id, election_id, candidate_id, weight, "type") values (3, 1, 3, 100, 'FOR');

insert into allocation (voter_id, election_id, candidate_id, weight, "type") values (4, 1, 5, 50, 'FOR');
insert into allocation (voter_id, election_id, candidate_id, weight, "type") values (5, 1, 5, 50, 'FOR');

-- both han and lando overtake jabba enough to fill, but they're tied
insert into allocation (voter_id, election_id, candidate_id, weight, "type") values (1, 2, 1, 100, 'FOR');
insert into allocation (voter_id, election_id, candidate_id, weight, "type") values (6, 2, 6, 100, 'FOR');

select * from candidacy_updated order by election_id, candidate_id;
call perform_vote_update();
select * from candidacy order by election_id, candidate_id;


-- X current winner to new winner because filled (hutt now han)
truncate allocation;
-- han has still filled, lando doesn't keep up
insert into allocation (voter_id, election_id, candidate_id, weight, "type") values (1, 2, 1, 100, 'FOR');

select * from candidacy_updated order by election_id, candidate_id;
call perform_vote_update();
select * from candidacy order by election_id, candidate_id;
