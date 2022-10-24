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

-- create table candidacy (
-- 	candidate_id int references person(id) not null,
-- 	election_id int references election(id) not null,
-- 	unique (candidate_id, election_id)
-- );

create type allocation_type as enum('FOR', 'AGAINST');

create table allocation (
	voter_id int references person(id) not null,
	candidate_id int references person(id) not null,
	election_id int references election(id) not null,
	-- candidate_id int references candidacy(candidate_id) not null,
	-- election_id int references candidacy(election_id) not null,

	weight int not null check (weight > 0),
	"type" allocation_type not null default 'FOR',

	-- -- every allocation must reference a valid candidacy
	-- foreign key (candidate_id, election_id) references candidacy (candidate_id, election_id),

	-- each voter can only allocate to each candidacy once
	primary key (voter_id, candidate_id, election_id)
);


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


insert into person (id, "name") values (1, 'han'), (2, 'luke'), (3, 'leia'), (4, 'vader');
insert into election (id, title) values (1, 'leader');
-- insert into candidacy (candidate_id, election_id) values (3, 1), (4, 1);


-- valid allocations
insert into allocation (voter_id, candidate_id, election_id, weight, "type") values (1, 3, 1, 50, 'FOR');
insert into allocation (voter_id, candidate_id, election_id, weight, "type") values (2, 3, 1, 50, 'FOR');

insert into allocation (voter_id, candidate_id, election_id, weight, "type") values (1, 4, 1, 50, 'AGAINST');
insert into allocation (voter_id, candidate_id, election_id, weight, "type") values (2, 4, 1, 50, 'AGAINST');

insert into allocation (voter_id, candidate_id, election_id, weight, "type") values (4, 3, 1, 100, 'AGAINST');
-- -- invalid allocations
insert into allocation (voter_id, candidate_id, election_id, weight, "type") values (4, 4, 1, 500, 'FOR');
insert into allocation (voter_id, candidate_id, election_id, weight, "type") values (4, 4, 1, 1, 'FOR');



-- voter totals
select
	voter_id, "name" as voter_name,
	sum(weight) as weight_allocated
from
	allocation
	join person on person.id = allocation.voter_id
group by voter_id, voter_name
;


-- candidacy totals
select
	candidate_id, person.name as candidate_name, election_id, election.title as election_title,
	sum((case when allocation."type" = 'FOR' then 1 else -1 end) * sqrt(weight)) as total_vote

from
	allocation
	join person on allocation.candidate_id = person.id
	join election on allocation.election_id = election.id
group by candidate_id, candidate_name, election_id, election_title
;
