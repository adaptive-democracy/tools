What does this frontend need to be capable of?

There's really only one kind of user: a voter. A voter can be a candidate.
There aren't really admins from the perspective of the frontend. An admin is just a person who has the power to change the server or the database or the frontend hosting.

- A voter must be able to look at the current constitutions/elections, including their current results
  - this would involve the current instantiated constitution tree, and the computed results of all users' current active allocations
  - each individual constitution/election must be separately viewable, with the current results

- A voter must be able to look at their current allocations, change them, and see how changing them would affect election results if no one else changed anything.
  - this would involve their most recent allocation
  - this suggests being able to trigger a change to allocations from an election page, probably jumping to a particular route with some context set

- A voter must be able to enter an election as a candidate.
  - this would just insert a candidacy

- A voter must be able to draft a constitution and enter it into a constitution election.
  - this is the most complicated. the ability to start from an existing constitution is necessary, making changes from there
  - we need the ability to compute a diff between any two constitutions

The server is the thing responsible for all the ongoing update logic, so all the frontend should be capable of is showing the current snapshot.

<!-- https://medium.com/@krutie/building-a-dynamic-tree-diagram-with-svg-and-vue-js-a5df28e300cd -->
