# Execution Workflow
This skill operates in two mandatory, sequential phases. Do not generate artifacts until Phase 1 is complete.
## Phase 1: Iterative Interview
Upon receiving the initial idea, do not immediately output the final architecture. Instead, generate exactly 5 targeted questions for the user to answer. These questions must determine:
1. The absolute core mechanical essence of the idea.
2. The exact implementation boundaries (what it is NOT).
3. Edge cases and user-perceived pitfalls.
4. Scalability or state-management assumptions.
5. Success criteria for the implementation.
_Wait for the user's response to these 5 questions before proceeding to Phase 2._
## Phase 2: Artifact Generation
Once the user answers the questions, synthesize the context and generate the following exact files in the `artifacts/idea/<number>/` directory (replace `<number>` with the current sequential issue/idea ID).
Output these files consecutively with maximum technical density. Zero conversational fluff.
### 1. `artifacts/idea/<number>/concept.md`
- **Focus**: The core essence of the idea.
- **Structure**:
    - `## Definition`: 1-2 sentences defining the exact feature.
    - `## Value Proposition`: Why this exists.
    - `## Core Mechanics`: Step-by-step logical flow of the feature in a vacuum.
### 2. `artifacts/idea/<number>/summary.md`
- **Focus**: Synthesis of the Q&A phase.
- **Structure**:
    - `## Initial Proposition`: What was requested.
    - `## Clarifications`: Key takeaways from the user's answers.
    - `## Perceived Pitfalls`: Risks explicitly identified by the user.
### 3. `artifacts/idea/<number>/hypotheses.md`
- **Focus**: Technical assumptions that must be validated against the real codebase later.
- **Structure**:
    - `## H1: [Name]`: Description of the assumption + Validation condition.
    - `## H2: [Name]`: ...
### 4. `artifacts/idea/<number>/constraints.md`
- **Focus**: Vacuum-state limitations.
- **Structure**:
    - `## Logical Conflicts`: Mutual exclusions or paradoxes in the logic.
    - `## Edge Cases`: Unhandled states or unexpected inputs.
    - `## Performance Risks`: Asymptotic complexity warnings or memory leak vectors.
### 5. `artifacts/idea/<number>/prior_art.md`
- **Focus**: Existing industry solutions.
- **Structure**:
    - `## Standard Patterns`: Well-known design patterns applicable here (e.g., Event Sourcing, Saga).
    - `## Alternative Approaches`: Other ways to solve the same problem.
### 6. `artifacts/idea/<number>/context_requirements.md`
- **Focus**: Bridge to Phase 2 (Context-aware implementation).
- **Structure**:
    - `## Required Codebase Context`: A checklist of system parts (e.g., Database schemas, Router logic, specific APIs) the AI must analyze in the next stage to prove the hypotheses.
# Output Constraints for Phase 2
- Use AST-like Markdown.
- No human-oriented explanations.
- English language only.
