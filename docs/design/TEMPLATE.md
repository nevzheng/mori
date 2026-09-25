# Title: what this design does, in plain English

<!--
Copy this file to docs/design/<area>/<what-it-does>.md. See README.md in this directory.
Keep it short: a reader should get the idea from Q1-Q8 alone. Delete guidance comments as you go.
-->

|             |                                                                                  |
| ----------- | -------------------------------------------------------------------------------- |
| **Author**  | @github-handle                                                                   |
| **Status**  | draft <!-- draft or review in a PR; accepted or superseded by <link> on main --> |
| **Area**    | cli <!-- the directory this doc lives in -->                                     |
| **Issue**   | #123 <!-- the tracking issue, or "none" -->                                      |
| **PR**      | #124 <!-- the PR that proposes this doc -->                                      |
| **Created** | YYYY-MM-DD                                                                       |
| **Updated** | YYYY-MM-DD                                                                       |

## Q1. What are you trying to do?

<!-- Your objectives, in plain words with no jargon. Two or three sentences. -->

## Q2. What problems is this not trying to solve?

<!-- The non-goals. Name the nearby problems a reader might assume are in scope. -->

## Q3. How is it done today, and what are the limits?

<!-- How users (or agents) do this now, with or without mori, and where that falls short. -->

## Q4. What is new in your approach, and why will it work?

<!-- The core idea in a paragraph. Why this, rather than the obvious alternative? -->

## Q5. Who cares? If it works, what difference does it make?

<!-- Which users or journeys (CUJs) benefit, and what they can do afterwards that they can't now. -->

## Q6. What are the risks?

<!-- What could go wrong: lost work, confusing UX, lock-in to a bad model, cost to maintain. -->

## Q7. How long will it take?

<!-- A rough size and the order of the PRs. mori is a hobby project, so no dates: count CLs. -->

## Q8. How will we know it worked?

<!--
The mid-term check (first CL merged, first scenario passing) and the final one (the journey
works end to end). Name the spec scenarios that must pass.
-->

## Appendix A. API (proto) changes

<!--
Changes to proto/mori/v1alpha1: new or changed messages, RPCs, fields and error reasons
(UPPER_SNAKE_CASE). Say what breaks for existing callers, if anything. "None" is a fine answer.
-->

## Appendix B. Design sketch

<!-- How it works: components, data flow, state, the commands and their output. A diagram helps. -->

## Appendix C. Rejected designs

<!-- The alternatives you considered and why you didn't pick them. Saves the next person the trip. -->

## Appendix D. Failure modes and security

<!--
What happens on partial failure, interruption, concurrent runs or bad input. Can it lose work?
What does it trust (paths, repo contents, remote data, tokens), and what if that input is hostile?
-->

## Appendix E. Test plan

<!--
The CUJs this touches and the Gherkin scenarios for spec/cuj/, written first. Then the unit and
integration tests, following the test pyramid.
-->

```gherkin
Scenario: <the behavior, in the user's words>
  Given <a starting state>
  When I run "mori <command>"
  Then <what the user sees>
```

## Appendix F. Migration

<!--
What changes for existing users: config, state database, on-disk layout, command names.
How they move over, and whether it can be undone. "None" is a fine answer.
-->
